#!/bin/bash

# Run the complete ELT pipeline

set -e

echo "🔄 Running ELT pipeline..."

# Check if input directory exists
if [ ! -d "data/raw" ]; then
    echo "❌ Input directory data/raw not found"
    echo "Please add your documents to data/raw/ first"
    exit 1
fi

# Check if Python environment is set up
if [ ! -d "etl" ]; then
    echo "❌ ELT directory not found"
    exit 1
fi

cd etl

# Run the complete pipeline
echo "📄 Extracting documents..."
python -m src.cli pipeline \
    --input ../data/raw \
    --output ../data \
    --chunk-size 1000 \
    --chunk-overlap 200 \
    --model text-embedding-3-small

cd ..

echo "✅ ELT pipeline completed!"
echo "📊 Results:"
echo "  - Processed documents: data/processed/"
echo "  - Text chunks: data/chunks/"
echo "  - Embeddings: data/embeddings/"
echo "  - Search indexes: data/indexes/"
