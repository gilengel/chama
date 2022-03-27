
[![build](https://img.shields.io/github/workflow/status/gilengel/rust_procmap/Code%20Coverage?style=for-the-badge)](https://github.com/gilengel/rust_procmap/actions)
[![coverage](https://img.shields.io/codecov/c/github/gilengel/rust_procmap?style=for-the-badge)](https://app.codecov.io/gh/gilengel/rust_procmap)
[![license](https://img.shields.io/github/license/gilengel/rust_procmap?style=for-the-badge)](https://github.com/gilengel/rust_procmap/blob/main/LICENSE)
# rust-procmap
2D map editor for creating beautiful and stylized fantasy maps for games and / or pen and paper adventures. It also serves as a living example of the EditAll editor framework (unstable, work in progress).



## Development
1. Install Rust. You can use the following script to do so:
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
2. We use trunk as a live reloading server for development and higly recommend it. With a live reloading server you get a program that automatically triggers the compiler after you changed some
code and reloads the live preview in your browser making development way easier and fun. You find more information about trunk and how to install it on their [official website](https://trunkrs.dev/#install).
3. Clone this repo:
```sh
git clone https://github.com/gilengel/rust_procmap
```
3. Switch to the cloned directory
```sh
cd rust_procmap
```
4. Open your favorite code editing tool to get startet. We use [Visual Studio Code](https://code.visualstudio.com/) and highly recommend it (if you use it make sure to install the [rust-analyzer](https://rust-analyzer.github.io/) extension)
5. Start the live server with
```sh
trunk serve --watch "rust_editor/src" "rust_internal/src" "rust_macro/src"  
```   
6. Open your favorite web browser and navigate to ```localhost:8080```. You now should see the editor :).

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

## Production
Comming Soon

## Release
Comming Soon