# Example binaries
Precompiled binaries are available in GitHub releases, they can also be built following the steps under `BUILDING.md`.  
`unifiedlog_parser` and `unifiedlog_parser_json` can run on a live macOS system or a `logarchive`.  To run on a live system execute `unifiedlog_parser` or `unifiedlog_parser_json` with the arguements `-l true`.  
To run on a `logarchive` provide the full path to the `logarchive` as an arguement to `unifiedlog_parser` or `unifiedlog_parser_json`.  
- Ex: `unifiedlog_parser -i <path/to/file.logarchive>`  

Full exmample: 
```
./unifiedlog_parser -i system_logs.logarchive
Starting Unified Log parser...
Parsing: system_logs.logarchive/Persist/0000000000000462.tracev3
Parsing: system_logs.logarchive/Persist/0000000000000454.tracev3
...
```

A very simple help menu is provided via the `-h` option for both `unifiedlog_parser`
```
./unifiedlog_parser -h
unifiedlog_parser 0.1.0

USAGE:
    unifiedlog_parser [OPTIONS]

OPTIONS:
    -h, --help             Print help information
    -i, --input <INPUT>    Path to logarchive formatted directory [default: ]
    -V, --version          Print version information
```


To create an `logarchive`, execute `sudo log collect`. If you cannot execute the `log` command, you can manually create a `logarchive`.  
The example binary `parse_tracev3` can parse a single `tracev3`.  
- Ex: `parse_tracev3 <path/to/file.tracev3>`  

## Manually create logarchive
1. Create a directory. Ex: `mkdir output`
2. Copy all contents from `/private/var/db/uuidtext` to created directory
3. Copy all contents from `/private/var/db/diagnostics` to created directory
4. Execute `unifiedlog_parser` or `unifiedlog_parser_json` with path to created directory
- Ex: `unifiedlog_parser -i <path/to/output>`

* Breakdown of warnings
  * `[WARN] Failed to get message string from alternative UUIDText file: "8151CEAA69AF3C059474AAE3403C91A7"` 
     * The parser failed to extract the base log message string from the designated UUIDText file (UUID file).  
       macOS `log` command would report the error as `error: ~~> Invalid image <8151CEAA-69AF-3C05-9474-AAE3403C91A7>`
  * `[WARN] Failed to get string: Utf8Error { valid_up_to: 0, error_len: Some(1) }`
     * The parser failed to extract string metadata from a log message. This is commonly happens with log files in the `Special` directory. The parser currently attempts to extract strings associated with metdata on the log entry. Sometimes the metadata cannot be represented as a string

`<Missing message data>` in output. Sometimes log data may get deleted or not recorded, if the parser fails to find all the data associated with the log entries it will use `<Missing message data>` when attempting to build the logs.  
macOS `log` command would report the missing data as `<decode: missing data>`   
This sometimes occurs when a `tracev3` file references data in a deleted `tracev3` file. 

## Reviewing Unified Logs
The logs typically retain 30 days worth of information.  
Some possible starting points when reviewing log data:  
https://github.com/jamf/jamfprotect/tree/main/unified_log_filters
