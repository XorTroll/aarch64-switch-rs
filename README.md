
# aarch64-switch-rs

## **IMPORTANT!** this project is no longer continued. For the ongoing work on Rust support for 64-bit Nintendo Switch homebrew, check [here](https://github.com/aarch64-switch-rs).

This repository is the home of some native and cross-platform work regarding Nintendo Switch homebrew:

- **[nx](nx)**: completely native homebrew library made in and for Rust

- **[tests](tests)**: simple tests made as a PoC of what can be done with **nx**, or as a way help with **nx**'s development.

## Requirements

- **[Rust](https://rustup.rs)**: `rustup`, `cargo`, etc.

- **xargo**, used to cross-compile: `cargo install xargo`

- **rust-src**, needed by xargo: `rustup component add rust-src`

- **[linkle](https://github.com/MegatonHammer/linkle)**, used by the bash build scripts to generate a homebrew NRO executable from the compiled ELF (`cargo install --features=binaries linkle`)

## Status / TODO

Many things still have to be implemented (**nx** is basically a work-in-progress Rust port of [libbio](https://github.com/biosphere-switch/libbio), a C++ homebrew library)

### TODO list

- Split wrapper/optional modules into crate features ("gpu", "input")

- Figure out how to implement the module name (link_section attribute doesn't seem to work fine...?)

- Finish bits of CRT0 (implement all hbl ABI parsing, etc.)

- Secondary crate/lib for UI, some 2D framework

- Thread-local variable support?

- Decide how to properly handle early Result assertions (before main() gets called)

- FileSystem wrapper (maybe "fs" module)?

- RomFs support

- Documentation

## Information

### Results

- Result module: `430` (`2430-****`)

- Result submodules (can be found as consts named `nx::results::lib::<module_name>::RESULT_SUBMODULE`):

  - Dynamic: `1` (`2430-01**`)

  - ELF-related: `2` (`2430-02**`)

  - Util: `3` (`2430-03**`)

  - Assert: `4` (`2430-04**`)

  - GPU: `5` (`2430-08**`)

## Credits

- Other main homebrew libraries (libnx and libtransistor) since libbio (the C++ base of this project's library) was made thanks to all the work made on these two libraries.

- The MegatonHammer Rust community, for helping with small details of this project's development.

## Support / helping

Any help or suggestion will be appreciated - GitHub issues and pull requests are the best way to share your suggestions, problems or improvements for this project.
