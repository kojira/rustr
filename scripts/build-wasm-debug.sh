#!/bin/bash
set -e

echo "🔧 Building WASM (Debug Test Mode)..."

cd ui

# デバッグテスト機能を有効にしてビルド
CC=/opt/homebrew/opt/llvm/bin/clang \
AR=/opt/homebrew/opt/llvm/bin/llvm-ar \
wasm-pack build \
  --target web \
  --out-dir pkg \
  --features debug-test \
  -- --features debug-test

# 静的ファイルをコピー
cp index.html pkg/
cp app.js pkg/

echo "✅ WASM build (debug-test) completed!"
echo ""
echo "📝 デバッグテストモードで起動するには:"
echo "   http://localhost:8080/?debug_test=1"

