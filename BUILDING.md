# How to build
1. Download and install Rust from https://www.rust-lang.org/
   * Windows users will need Windows C++ build tools https://visualstudio.microsoft.com/visual-cpp-build-tools/
     * Select `Desktop development with C++`
     * This is a Rust requirement for Windows
2. Git clone the repo
3. Navigate to cloned repo.
4. Execute `cargo build` to build debug library. `cargo build --release` to build release version
   * Navigate to examples directory and run `cargo build --release` to build the example files
   * `unifiedlog_parser` can also parse a `logarchive` if passed as an arguement

# Running test suite
1. Follow steps above
2. Download `tests.zip` from Github releases
3. Copy/move `tests.zip` to clone repo
4. Decompress `tests.zip`
5. Execute `cargo test --release` to run tests
   * You can also just use `cargo test` to run tests but it will be slower


# Running benchmarks
1. Download `tests.zip` from Github releases
2. Copy/move `tests.zip` to clone repo
3. Decompress `tests.zip`
4. Run `cargo bench`  
or  
4. Install criterion, `cargo install cargo-criterion`
5. Run `cargo criterion`

