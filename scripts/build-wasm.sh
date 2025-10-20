#!/bin/bash
set -e

echo "Building WASM module..."

cd ui

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack is not installed. Installing..."
    cargo install wasm-pack
fi

# Build WASM with Homebrew LLVM
export CC=/opt/homebrew/opt/llvm/bin/clang
export AR=/opt/homebrew/opt/llvm/bin/llvm-ar

wasm-pack build --target web --out-dir pkg

# Copy static files
cp index.html pkg/
cp app.js pkg/

echo "WASM build complete!"
echo "Package generated at: ui/pkg"

