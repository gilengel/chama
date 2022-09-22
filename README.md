<div align="center">
  <img height="200" src="editall.svg">

  <h1>Chama</h1>

  <p>
    <strong>Rust / Wasm editor framework</strong>
  </p>

  <p>  
    <a href="https://github.com/gilengel/chama/actions"><img src="https://img.shields.io/github/workflow/status/gilengel/chama/Code%20Coverage?style=for-the-badge" alt="build" ></a>
    <a href="https://app.codecov.io/gh/gilengel/chama" rel="nofollow"><img src="https://img.shields.io/codecov/c/github/gilengel/chama?style=for-the-badge" alt="coverage"></a>
    <a href="https://github.com/gilengel/chama/blob/main/LICENSE"><img src="https://img.shields.io/github/license/gilengel/chama?style=for-the-badge" alt="license"></a>
  </p>
</div>

# About
<b>Chama</b> is a modern framework for creating graphical editors that run everywhere.

* Features an easy to use plugin system to allow the development of any type of editor, small or complex.
* Comes with predefined components and an unique design language to ease usage of end users
* Can be deployed as as WASM application directly on a webserver or embedded into [tauri](https://tauri.app/) / [electron](https://www.electronjs.org/) to offer a native application

# Releases
🙁 Chama is in its early developing stage and so far not released yet. Please take at the issues the roadmap for the next release.

In the meantime you can always use the <b>main</b> branch version containing all the latest improvements. Define <b>Chama</b> in your Cargo.toml file like this:
```
[dependencies]
chama = { git = "https://github.com/gilengel/chama", branch = "main" }
```

# Contributing
Main development is currently done by a small team of developers. We have big goals that can only achieve with the community in a joined effort. Therefore we really encourage you to contribute regardless of your knowledge or skillset. 
Developing Chama shall be a pleasent experience for everyone and we strive for a happy, helping and respectful community. So please read our [Code of Conduct](CODE_OF_CONDUCT.md) to learn behavior will not be tolerated.

🥳 You are new to <b>Chama</b>? Take a look at our Wiki to learn the first steps.

❔ Something is unclear, not good documented or missing? Please let us know by opening a new issue. We might switch to another form of communication (e.g. Discord) in the future.

🐞 Found a bug? Please open an issue. We are currently working on a corresponding template.



## Development
1. Install Rust. You can use the following script to do so:
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
2. We use trunk as a live reloading server for development and higly recommend it. With a live reloading server you get a program that automatically triggers the compiler after you changed some
code and reloads the live preview in your browser making development way easier and fun. You find more information about trunk and how to install it on their [official website](https://trunkrs.dev/#install).
3. Clone this repo:
```sh
git clone https://github.com/gilengel/chama
```
3. Switch to the cloned directory
```sh
cd chama
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
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests"
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