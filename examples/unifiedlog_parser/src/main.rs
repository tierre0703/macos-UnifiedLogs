// Copyright 2022 Mandiant, Inc. All Rights Reserved
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed under the License
// is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

// use chrono::{SecondsFormat, TimeZone, Utc};
// use log::LevelFilter;
use macos_unifiedlogs::dsc::SharedCacheStrings;
use macos_unifiedlogs::parser::{
    build_log, collect_shared_strings, collect_shared_strings_system, collect_strings,
    collect_strings_system, collect_timesync, collect_timesync_system, parse_log,
};
use macos_unifiedlogs::timesync::TimesyncBoot;
use macos_unifiedlogs::unified_log::{LogData, UnifiedLogData};
use macos_unifiedlogs::uuidtext::UUIDText;
// use simplelog::{Config, SimpleLogger};
// use std::error::Error;
use std::fs;
// use std::fs::OpenOptions;
use std::path::PathBuf;
use regex::Regex;
use clap::Parser;

use std::time::Instant;
use walkdir::{DirEntry, WalkDir};
use alphanumeric_sort;
use std::cmp::Ordering;
/* use serde::Serialize;


#[derive(Debug, Serialize)]
pub struct MessageData {
    pub subsystem: String,
    pub pid: u64,
    pub message: String,
}
 */
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Run on live system
    #[clap(short, long, default_value = "false")]
    live: String,

    /// Path to logarchive formatted directory
    #[clap(short, long, default_value = "")]
    input: String,
//
//    /// Path to output file. Any directories must already exist
//    #[clap(short, long)]
//    output: String,
}

fn main() {
    // println!("Starting Unified Log parser...");
    // Set logging level to warning
    // SimpleLogger::init(LevelFilter::Warn, Config::default())
    //    .expect("Failed to initialize simple logger");

    let args = Args::parse();
    // Create headers for CSV file
    // output_header().unwrap();

    if args.input != "" {
        parse_log_archive(&args.input);
    } else if args.live != "false" {
        parse_live_system();
    }
}

// Parse a provided directory path. Currently expect the path to follow macOS log collect structure
fn parse_log_archive(path: &str) {
    let mut archive_path = PathBuf::from(path);

    // Parse all UUID files which contain strings and other metadata
    let string_results = collect_strings(&archive_path.display().to_string()).unwrap();

    archive_path.push("dsc");
    // Parse UUID cache files which also contain strings and other metadata
    let shared_strings_results =
        collect_shared_strings(&archive_path.display().to_string()).unwrap();
    archive_path.pop();

    archive_path.push("timesync");
    // Parse all timesync files
    let timesync_data = collect_timesync(&archive_path.display().to_string()).unwrap();
    archive_path.pop();

    // Keep UUID, UUID cache, timesync files in memory while we parse all tracev3 files
    // Allows for faster lookups
    parse_trace_file(
        &string_results,
        &shared_strings_results,
        &timesync_data,
        path,
    );

    // println!("\nFinished parsing Unified Log data. Saved results to: output.csv");
}

// Parse a live macOS system
fn parse_live_system() {
    let strings = collect_strings_system().unwrap();
    let shared_strings = collect_shared_strings_system().unwrap();
    let timesync_data = collect_timesync_system().unwrap();

    parse_trace_file(
        &strings,
        &shared_strings,
        &timesync_data,
        "/private/var/db/diagnostics",
    );

    println!("\nFinished parsing Unified Log data. Saved results to: output.csv");
}

// Use the provided strings, shared strings, timesync data to parse the Unified Log data at provided path.
// Currently expect the path to follow macOS log collect structure
fn parse_trace_file(
    string_results: &[UUIDText],
    shared_strings_results: &[SharedCacheStrings],
    timesync_data: &[TimesyncBoot],
    path: &str,
) {
    // We need to persist the Oversize log entries (they contain large strings that don't fit in normal log entries)
    // Some log entries have Oversize strings located in different tracev3 files.
    // This is very rare. Seen in ~20 log entries out of ~700,000. Seen in ~700 out of ~18 million
    let mut oversize_strings = UnifiedLogData {
        header: Vec::new(),
        catalog_data: Vec::new(),
        oversize: Vec::new(),
    };
    let start = Instant::now();

    let mut batterhealth_string_offset: u32 = 0;

    
    let mut str_real_offset: u32 = 0;
    let mut ch_offset: u32 = 0;

    for string_entry in string_results {
        let strings = &string_entry.footer_data;
        let footer_data: &[u8] = strings;
        let mut str_buffer = String::new();
        let mut ch: char = char::from(0);

        for entry_index in 0..string_entry.entry_descriptors.len() {
            let entry = string_entry.entry_descriptors.get(entry_index).unwrap();
            ch_offset = str_real_offset;
    
            while ch_offset < str_real_offset + entry.entry_size {
                ch = footer_data[ch_offset as usize] as char;
                if ch == char::from(0) {
                    // end of string
                    // check string
                    if str_buffer.contains("Updated Battery Health") &&
                    str_buffer.contains("MaxCapacity:"){
                        let str_len = str_buffer.len() as u32;
                        batterhealth_string_offset = entry.range_start_offset + ch_offset - str_real_offset - str_len;
                        // println!("{} {} {} {} {} {}", str_buffer, batterhealth_string_offset, str_real_offset, ch_offset, entry.range_start_offset, entry.entry_size);
                        // bh_pos = batteryhealth_offset;
                        break;
                    }
    
                    str_buffer = String::new();
                }else{
                    str_buffer.push(ch);
                }
                ch_offset += 1;
            }
    
            str_real_offset += entry.entry_size;
        }
    
    }


    
    // Exclude missing data from returned output. Keep separate until we parse all oversize entries.
    // Then at end, go through all missing data and check all parsed oversize entries again
    let exclude_missing = true;
    let mut missing_data: Vec<UnifiedLogData> = Vec::new();

    let mut archive_path = PathBuf::from(path);
    let mut log_data_vec: Vec<String> = Vec::new();


    archive_path.push("logdata.LiveData.tracev3");

    // Check if livedata exists. We only have it if 'log collect' was used
    if archive_path.exists() {
        // println!("Parsing: logdata.LiveData.tracev3");
        let mut log_data = parse_log(&archive_path.display().to_string(), batterhealth_string_offset).unwrap();
        log_data.oversize.append(&mut oversize_strings.oversize);
        let (results, missing_logs) = build_log(
            &log_data,
            string_results,
            shared_strings_results,
            timesync_data,
            exclude_missing,
            batterhealth_string_offset,
        );
        // Track missing data
        missing_data.push(missing_logs);

        let mut messages = output(&results);
        if messages.len() > 0 {
            log_data_vec.append(&mut messages);
            //let duration = start.elapsed();
            //println!("Time elapsed in expensive_function() is: {:?}", duration);
        }
        // Track oversize entries
        oversize_strings.oversize = log_data.oversize;
    }


    if log_data_vec.len() == 0 {
        archive_path.pop();
        archive_path.push("Special");

        if archive_path.exists() {
            let paths = fs::read_dir(&archive_path).unwrap();

            // Loop through all tracev3 files in Special directory
            for log_path in paths {
                let data = log_path.unwrap();
                let full_path = data.path().display().to_string();
                //println!("Parsing: {}", full_path);

                let mut log_data = if data.path().exists() {
                    parse_log(&full_path, batterhealth_string_offset).unwrap()
                } else {
                    // println!("File {} no longer on disk", full_path);
                    continue;
                };

                // Append our old Oversize entries in case these logs point to other Oversize entries the previous tracev3 files
                log_data.oversize.append(&mut oversize_strings.oversize);
                let (results, missing_logs) = build_log(
                    &log_data,
                    string_results,
                    shared_strings_results,
                    timesync_data,
                    exclude_missing,
                    batterhealth_string_offset,
                );
                // Track Oversize entries
                oversize_strings.oversize = log_data.oversize;
                // Track missing logs
                missing_data.push(missing_logs);
                let mut messages = output(&results);
                if messages.len() > 0 {
                    log_data_vec.append(&mut messages);
                    //let duration = start.elapsed();
                    //println!("Time elapsed in expensive_function() is: {:?}", duration);
                }
            }
        }
    }


    if log_data_vec.len() == 0 {

        archive_path.pop();
        archive_path.push("Persist");

        // let mut log_count = 0;
        if archive_path.exists() {
        
            // let paths = fs::read_dir(&archive_path).unwrap();
            let mut paths:Vec<DirEntry> = WalkDir::new(&archive_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|entry| entry.file_type().is_file()).collect();

            paths.sort_by(|a, b| alphanumeric_sort::compare_path(b.path().display().to_string(), a.path().display().to_string()));

            // Loop through all tracev3 files in Persist directory
            for log_path in paths {
                let full_path = log_path.path().display().to_string();


                let log_data = if log_path.path().exists() {
                    match parse_log(&full_path, batterhealth_string_offset) {
                        Ok(results) => results,
                        Err(err) => continue
                    }
                } else {
                    //println!("File {} no longer on disk", full_path);
                    continue;
                };

                // Get all constructed logs and any log data that failed to get constrcuted (exclude_missing = true)
                let (results, missing_logs) = build_log(
                    &log_data,
                    string_results,
                    shared_strings_results,
                    timesync_data,
                    exclude_missing,
                    batterhealth_string_offset,
                );

                // Track Oversize entries
                // oversize_strings
                //    .oversize
                //    .append(&mut log_data.oversize.to_owned());

                // Track missing logs
                // missing_data.push(missing_logs);
                // log_count += results.len();
                let mut messages = output(&results);
                if messages.len() > 0 {
                    log_data_vec.append(&mut messages);
                    //let duration = start.elapsed();
                    //println!("Time elapsed in expensive_function() is: {:?}", duration);
                    break
                }

            }
        }
    }


    

    if log_data_vec.len() > 0 {
        println!("{}", log_data_vec[log_data_vec.len() - 1].to_string())
    }
    /*

    archive_path.pop();
    archive_path.push("Special");

    if archive_path.exists() {
        let paths = fs::read_dir(&archive_path).unwrap();

        // Loop through all tracev3 files in Special directory
        for log_path in paths {
            let data = log_path.unwrap();
            let full_path = data.path().display().to_string();
            println!("Parsing: {}", full_path);

            let mut log_data = if data.path().exists() {
                parse_log(&full_path).unwrap()
            } else {
                println!("File {} no longer on disk", full_path);
                continue;
            };

            // Append our old Oversize entries in case these logs point to other Oversize entries the previous tracev3 files
            log_data.oversize.append(&mut oversize_strings.oversize);
            let (results, missing_logs) = build_log(
                &log_data,
                string_results,
                shared_strings_results,
                timesync_data,
                exclude_missing,
            );
            // Track Oversize entries
            oversize_strings.oversize = log_data.oversize;
            // Track missing logs
            missing_data.push(missing_logs);
            log_count += results.len();

            output(&results).unwrap();
        }
    }

    archive_path.pop();
    archive_path.push("Signpost");

    if archive_path.exists() {
        let paths = fs::read_dir(&archive_path).unwrap();

        // Loop through all tracev3 files in Signpost directory
        for log_path in paths {
            let data = log_path.unwrap();
            let full_path = data.path().display().to_string();
            println!("Parsing: {}", full_path);

            let log_data = if data.path().exists() {
                parse_log(&full_path).unwrap()
            } else {
                println!("File {} no longer on disk", full_path);
                continue;
            };

            let (results, missing_logs) = build_log(
                &log_data,
                string_results,
                shared_strings_results,
                timesync_data,
                exclude_missing,
            );

            // Signposts have not been seen with Oversize entries
            missing_data.push(missing_logs);
            log_count += results.len();

            output(&results).unwrap();
        }
    }
    archive_path.pop();
    archive_path.push("HighVolume");

    if archive_path.exists() {
        let paths = fs::read_dir(&archive_path).unwrap();

        // Loop through all tracev3 files in HighVolume directory
        for log_path in paths {
            let data = log_path.unwrap();
            let full_path = data.path().display().to_string();
            println!("Parsing: {}", full_path);

            let log_data = if data.path().exists() {
                parse_log(&full_path).unwrap()
            } else {
                println!("File {} no longer on disk", full_path);
                continue;
            };
            let (results, missing_logs) = build_log(
                &log_data,
                string_results,
                shared_strings_results,
                timesync_data,
                exclude_missing,
            );

            // Oversize entries have not been seen in logs in HighVolume
            missing_data.push(missing_logs);
            log_count += results.len();

            output(&results).unwrap();
        }
    }
    archive_path.pop();
    archive_path.push("logdata.LiveData.tracev3");

    // Check if livedata exists. We only have it if 'log collect' was used
    if archive_path.exists() {
        println!("Parsing: logdata.LiveData.tracev3");
        let mut log_data = parse_log(&archive_path.display().to_string()).unwrap();
        log_data.oversize.append(&mut oversize_strings.oversize);
        let (results, missing_logs) = build_log(
            &log_data,
            string_results,
            shared_strings_results,
            timesync_data,
            exclude_missing,
        );
        // Track missing data
        missing_data.push(missing_logs);
        log_count += results.len();

        output(&results).unwrap();
        // Track oversize entries
        oversize_strings.oversize = log_data.oversize;
        archive_path.pop();
    }

    exclude_missing = false;

    // Since we have all Oversize entries now. Go through any log entries that we were not able to build before
    for mut leftover_data in missing_data {
        // Add all of our previous oversize data to logs for lookups
        leftover_data
            .oversize
            .append(&mut oversize_strings.oversize.to_owned());

        // Exclude_missing = false
        // If we fail to find any missing data its probably due to the logs rolling
        // Ex: tracev3A rolls, tracev3B references Oversize entry in tracev3A will trigger missing data since tracev3A is gone
        let (results, _) = build_log(
            &leftover_data,
            string_results,
            shared_strings_results,
            timesync_data,
            exclude_missing,
        );
        log_count += results.len();

        output(&results).unwrap();
    } 
    println!("Parsed {} log entries", log_count);
    */
}

// Create csv file and create headers
/*    
fn output_header() -> Result<(), Box<dyn Error>> {

 println!("Timestamp, PID, Library, Process, Message");
    let args = Args::parse();

    let csv_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(args.output)?;
    let mut writer = csv::Writer::from_writer(csv_file);

    writer.write_record(&[
        "Timestamp",
        "Event Type",
        "Log Type",
        "Subsystem",
        "Thread ID",
        "PID",
        "EUID",
        "Library",
        "Library UUID",
        "Activity ID",
        "Category",
        "Process",
        "Process UUID",
        "Message",
        //"Raw Message",
        //"Boot UUID",
        //"System Timezone Name",
    ])?;
    Ok(())
}
 */

// Append or create csv file
fn output(results: &Vec<LogData>) -> Vec<String> {
    let mut message_vec: Vec<String> = Vec::new();
    let message_re = Regex::new(
        r"Battery Health:.*MaxCapacity:([0-9]+)"
    ).unwrap();
    for data in results {
        //let date_time = Utc.timestamp_nanos(data.time as i64);
        //if message_re.is_match(&data.message) {
        if data.message != "" {
            let percent = message_re.captures(&data.message).unwrap();

            message_vec.push(percent.get(1).map_or("", |m| m.as_str()).to_string());
            break

        }
            /* println!("{}, {}, {}, {}", 
                date_time.to_rfc3339_opts(SecondsFormat::Millis, true), 
                data.pid.to_string(),
                data.subsystem.to_owned(),
                //data.process.to_owned(),
                data.message.to_owned()
            ); */
        //}
    }

    message_vec

    /*
    let args = Args::parse();

    let csv_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(args.output)?;
    let mut writer = csv::Writer::from_writer(csv_file);

    for data in results {
        let date_time = Utc.timestamp_nanos(data.time as i64);
        writer.write_record(&[
            date_time.to_rfc3339_opts(SecondsFormat::Millis, true),
            data.event_type.to_owned(),
            data.log_type.to_owned(),
            data.subsystem.to_owned(),
            data.thread_id.to_string(),
            data.pid.to_string(),
            data.euid.to_string(),
            data.library.to_owned(),
            data.library_uuid.to_owned(),
            data.activity_id.to_string(),
            data.category.to_owned(),
            data.process.to_owned(),
            data.process_uuid.to_owned(),
            data.message.to_owned(),
            //data.raw_message.to_owned(),
            //data.boot_uuid.to_owned(),
            //data.timezone_name.to_owned(),
        ])?;
    }
     */
}
