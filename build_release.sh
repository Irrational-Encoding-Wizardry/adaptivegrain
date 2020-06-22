#!/bin/sh

# Set rustflags without target-cpu to build for legacy hosts.
# I only build this for windows because Linux users on old machines can just compile their own binary
# --emit=asm forces rustc to compile the crate with only one thread, which can help the optimizer (1-2% faster on my machine)
RUSTFLAGS="--emit asm" cargo build --release --target=x86_64-pc-windows-gnu --locked
mv target/x86_64-pc-windows-gnu/release/adaptivegrain_rs.dll ./adaptivegrain_rs-no-fma.dll
RUSTFLAGS="-C target-cpu=haswell --emit asm" cargo build --release --locked
RUSTFLAGS="-C target-cpu=haswell --emit asm" cargo build --release --target=x86_64-pc-windows-gnu --locked
mv target/x86_64-pc-windows-gnu/release/adaptivegrain_rs.dll ./
mv target/release/libadaptivegrain_rs.so ./
strip libadaptivegrain_rs.so
strip adaptivegrain_rs.dll
strip adaptivegrain_rs-no-fma.dll
