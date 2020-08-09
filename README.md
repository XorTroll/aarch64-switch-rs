
# aarch64-switch-rs

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

- Implement applet services

- Implement server-side IPC (pretty ambicious TODO)

- Secondary crate/lib for UI, some 2D framework

- Thread-local variable support?

- Decide how to properly handle early Result assertions (before main() gets called)

- FileSystem wrapper (maybe "fs" module)?

- RomFs support

- NSO specific test (the only thing different aside from code would be the build script...)

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

  - Memory: `10` (`2430-10**`)

## Credits

- Other main homebrew libraries (libnx and libtransistor) since libbio (the C++ base of this project's library) was made thanks to all the work made on these two libraries.

- The MegatonHammer Rust community, for helping with small details of this project's development.

## Support / helping

Any help or suggestion will be appreciated - GitHub issues and pull requests are the best way to share your suggestions, problems or improvements for this project.