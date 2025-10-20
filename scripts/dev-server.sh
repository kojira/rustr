#!/bin/bash
set -e

echo "Starting development server..."

# WASMをビルド（静的ファイルのコピーも含む）
./scripts/build-wasm.sh

# 開発サーバーを起動
echo ""
echo "🚀 Server starting at http://localhost:8080"
echo "Press Ctrl+C to stop"
echo ""

cd ui/pkg
python3 -m http.server 8080

