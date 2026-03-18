#!/bin/bash

set -e
# check if wasm-pack is installed, if not, install it
if ! command -v wasm-pack &> /dev/null
then
    echo "wasm-pack not found, installing..."
    cargo install wasm-pack
else
    echo "wasm-pack found, skipping installation"
fi
wasm-pack build --dev --no-typescript --target web --features web

echo "Running tests in headless Chrome..."
wasm-pack test --headless --chrome --features web
