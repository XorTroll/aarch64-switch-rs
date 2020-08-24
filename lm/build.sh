RUST_TARGET_PATH=$PWD RUSTFLAGS="-Z macro-backtrace" xargo build --release --target aarch64-none-elf
DIR=`basename "$PWD"`
mkdir -p target/aarch64-none-elf/release/exe
linkle nso target/aarch64-none-elf/release/$DIR.elf target/aarch64-none-elf/release/exe/main
npdmtool npdm.json target/aarch64-none-elf/release/exe/main.npdm
build_pfs0 target/aarch64-none-elf/release/exe target/aarch64-none-elf/release/exefs.nsp