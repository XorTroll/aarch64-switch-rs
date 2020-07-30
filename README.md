# aarch64-switch-rs

This project is just a simple project playing aroung with aarch64 cross-compile and Nintendo Switch homebrew.

It contains a homebrew library (`nx` crate) and a sample project (`sample`) using it.

This project started as some testing of making a Nintendo Switch homebrew project in Rust as simple as possible.

## Requirements

- **[Rust](https://rustup.rs)**: `rustup`, `cargo`, etc.

- **xargo**, used to cross-compile: `cargo install xargo`

- **rust-src**, needed by xargo: `rustup component add rust-src`

- **[linkle](https://github.com/MegatonHammer/linkle)**, used by the bash build scripts to generate a homebrew NRO binary file from the compiled ELF (`cargo install --features=binaries linkle`)

## Status / TODO

Many things are not still supported (this is mainly a work-in-progress Rust port of [libbio](https://github.com/biosphere-switch/libbio), a C++ homebrew library)

Currently has very little support (SVCs and some stuff), but it still lacks some essential elements like IPC support.

## Credits

- Other main homebrew libraries (libnx and libtransistor) since libbio (the C++ base of this project's library) was made thanks to all the work made on these two libraries.