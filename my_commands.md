# This is a file with a set of commands with wich i have started this project, install rust etc. 
1) curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
2) ls ~/.cargo/bin
3) cargo --version
4)  open -e ~/.zshrc + add source "$HOME/.cargo/env" at the end
5) source ~/.zshrc
6) cargo --version
7) rustc --version
8) rustup default stable - install newest rust version
9) rustup update stable
10) rustup component add rustfmt clippy
11) 
```
$ cargo --version
cargo 1.91.0 (ea2d97820 2025-10-10)

$ rustc --version
rustc 1.91.0 (f8297e351 2025-10-28)

$ rustup show
Default host: aarch64-apple-darwin
rustup home:  /Users/piotrjankiewicz/.rustup

installed toolchains
--------------------
stable-aarch64-apple-darwin (active, default)

active toolchain
----------------
name: stable-aarch64-apple-darwin
active because: it's the default toolchain
installed targets:
  aarch64-apple-darwin
```
12) cargo new mac_textpad --bin
13) install cargo-edit package
14) install: cargo add eframe rfd anyhow (libraries for GUI, popups and error handling)




