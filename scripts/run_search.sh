#!/bin/bash

# Run the search system

set -e

echo "🔍 Starting search system..."

# Check if search directory exists
if [ ! -d "search" ]; then
    echo "❌ Search directory not found"
    exit 1
fi

cd search

# Check if indexes exist
if [ ! -d "../data/indexes/tantivy" ] || [ ! -d "../data/indexes/lancedb" ]; then
    echo "❌ Search indexes not found"
    echo "Please run the ELT pipeline first: ./scripts/run_elt_pipeline.sh"
    exit 1
fi

# Run the search system
echo "🚀 Starting search server..."
cargo run --release --bin lancedb_production_example

cd ..
