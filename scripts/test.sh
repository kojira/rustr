#!/bin/bash
set -e

echo "🧪 Running Rustr Tests..."
echo ""

# Core tests (ネイティブターゲットのみ)
echo "📦 Testing core module..."
cd core
cargo test --lib --target aarch64-apple-darwin
cargo test --test integration_test --target aarch64-apple-darwin
cd ..

echo ""
echo "✅ All tests passed!"

