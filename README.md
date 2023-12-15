# Reveaal

A model checking engine for ECDAR (Environment for Compositional Design and Analysis of Real Time Systems) written in Rust. 

#### Libraries
- ECDAR DBM (Difference Bound Matrix) Library for operations on zones of time (https://www.github.com/ECDAR/EDBM).
- Protobuf Rust library (https://github.com/Ecdar/Ecdar-ProtoBuf-rs).

## Prerequisites 
- Rust compiler (https://www.rust-lang.org/learn/get-started)
  - **Windows**: It is recommended installing and using the default `x86_64-pc-windows-msvc` Rust targets. If you instead (not recommended) are using `x86_64-pc-windows-gnu` targets on Windows you need to install mingw and add it to your PATH environment variable to build.
- Protobuf compiler
  - **Debian based (Ubuntu, mint etc.)**: `apt install protobuf-compiler`
  - **Arch based (Endeavour etc.)**: `yay protobuf-c`
  - **Windows**: Download protobuf (https://github.com/protocolbuffers/protobuf/releases/) and add the bin folder to your PATH environment variable (https://www.computerhope.com/issues/ch000549.htm)

## Compiling and running
- run `git submodule update --init --recursive` to fetch protobuf submodules
- Build the project: `cargo build` / `cargo build --release`
- Run the tests: `cargo test`
- Example twin-Query: `cargo run -- query -e "consistency: (G17 && G6); refinement: (G17 && G6) <= G17" -i samples/xml/ConsTests.xml`
- Example run as server: `run -- serve 127.0.0.1:7000`

#### Cross compiling
The project is pure Rust so one should be able to crosscompile to any platform with a rust target.

**Debian -> windows** Make sure you have mingw installed `sudo apt-get install mingw-w64` and the rustc windows target is installed with `rustup target add x86_64-pc-windows-gnu` and build with cargo: `cargo build --target x86_64-pc-windows-gnu`

#### Update GitHub cargo packages
`cargo update ecdar-protobuf` and `cargo update edbm`
