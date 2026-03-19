#!/bin/bash
set -e

cargo test --features web

echo "===================================================================="
# check if tarpaulin is installed, and if so, run coverage
if command -v cargo-tarpaulin &> /dev/null
then
    echo "checking code coverage with cargo-tarpaulin..."
    cargo tarpaulin --features web --lib
else
    echo "cargo-tarpaulin not found, skipping coverage."
    echo "hint: install Tarpaulin by running 'cargo install cargo-tarpaulin'"
fi