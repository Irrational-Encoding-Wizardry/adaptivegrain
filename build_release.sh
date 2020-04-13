#!/bin/sh

# set rustflags as empty to make sure we’re not building for march=native,
# as these are public release builds
RUSTFLAGS="" cargo build --release --locked
RUSTFLAGS="" cargo build --release --target=x86_64-pc-windows-gnu --locked
mv target/release/libadaptivegrain_rs.so ./
mv target/x86_64-pc-windows-gnu/release/adaptivegrain_rs.dll ./
strip libadaptivegrain_rs.so
strip adaptivegrain_rs.dll
