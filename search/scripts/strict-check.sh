#!/bin/bash
# Strict compilation script
# This script enforces zero warnings and strict linting

set -e

echo "🔍 Running strict compilation checks..."

echo "📦 Checking with cargo check..."
cargo check --lib

echo "🧹 Running clippy with strict settings..."
cargo clippy --lib -- -D warnings -D clippy::all

echo "🎨 Running rustfmt..."
cargo fmt --check

echo "✅ All strict compilation checks passed!"
