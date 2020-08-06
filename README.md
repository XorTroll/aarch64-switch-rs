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

> TODO: write a proper TODO :P

## Information

### Results

- Result module: `430` (`2430-****`)

- Result submodules (can be found as consts named `nx::<module_name>::RESULT_SUBMODULE`):

  - Dynamic: `1` (`2430-01**`)

  - ELF-related operations: `2` (`2430-02**`)

  - Util: `3` (`2430-03**`)

  - Common IPC: `4` (`2430-04**`)

  - Client-side IPC: `5` (`2430-05**`)

  - Assert: `6` (`2430-06**`)

  - NV error codes: `7` (`2430-07**`)

  - GPU (binder): `8` (`2430-08**`)

  - GPU (parcel): `9` (`2430-09**`)

## Credits

- Other main homebrew libraries (libnx and libtransistor) since libbio (the C++ base of this project's library) was made thanks to all the work made on these two libraries.