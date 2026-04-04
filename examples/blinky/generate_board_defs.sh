#!/bin/bash
set -e

cd "$(dirname "$0")"

echo "Generating board definitions from netlistquery JSON..."

# Run the host tool with cleared RUSTFLAGS to avoid cross-compilation linking errors
# targeting riscv32imc in the workspace
cd bsp_gen
RUSTFLAGS="" cargo run -- ../src/board_defs/board-a.json ../src/board_defs/board_a.rs
cd ../src/board_defs
cargo fmt

echo "Done!"
