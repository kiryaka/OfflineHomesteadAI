#!/bin/bash

# Setup script for Universe workspace

set -e

echo "🚀 Setting up Universe workspace..."

# Create data directories
echo "📁 Creating data directories..."
mkdir -p data/{raw,processed,chunks,embeddings,indexes/{tantivy,lancedb}}

# Setup Rust search system
echo "🦀 Setting up Rust search system..."
cd search
if [ -f "Cargo.toml" ]; then
    cargo build --release
    echo "✅ Rust search system built"
else
    echo "❌ Cargo.toml not found in search directory"
fi
cd ..

# Setup Python ELT pipeline
echo "🐍 Setting up Python ELT pipeline..."
cd etl
if [ -f "requirements.txt" ]; then
    pip install -r requirements.txt
    echo "✅ Python ELT pipeline installed"
else
    echo "❌ requirements.txt not found in etl directory"
fi
cd ..

echo "✅ Setup complete!"
echo ""
echo "Next steps:"
echo "1. Add your documents to data/raw/"
echo "2. Run: ./scripts/run_etl_pipeline.sh"
echo "3. Run: ./scripts/run_search.sh"
