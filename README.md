# rust-procmap
2D map editor for creating beautiful and stylized fantasy maps for games and / or pen and paper adventures. It also serves as a living example of the EditAll editor framework (unstable, work in progress).

## Table of contents
- [rust-procmap](#rust-procmap)
  - [Table of contents](#table-of-contents)
  - [General Information](#general-information)
  - [Setup](#setup)
  - [Test](#test)
    - [Coverage](#coverage)

## General Information
Put a more general information about your project

## Setup
Development:

Make sure you have the latest stable version of rust installed as well as trunk 
(https://crates.io/crates/trunk). 
To start your development server execute the following:
```
trunk serve --watch "rust_editor/src" "rust_internal/src" "rust_macro/src"  
```
## Test
### Coverage
1. We use [grcov](https://github.com/mozilla/grcov) as the code coverage tool which needs the nightly
toolchain to work. Activate it with:
```sh
rustup default nightly
```
2. Also set the following variables in your terminal session / build process:
```sh
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
export RUSTDOCFLAGS="-Cpanic=abort"
```
3. Build and run the test of your project like this
```
cargo build && cargo test
```
4. Generate the html report with
```sh
grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/
```
You'll find the report here: ```target/debug/coverage/index.html```

Production: 
Comming Soon