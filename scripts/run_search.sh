#!/bin/bash

# Run the search system

set -e

echo "🔍 Starting search system..."

# Run a demo vector search via the new CLI

echo "🔎 Running demo vector search (localdb-cli)..."

# Optional: index first if no indexes exist
if [ ! -d "dev_data/indexes/lancedb" ]; then
  echo "ℹ️ No LanceDB index found under dev_data/indexes/lancedb. Indexing sample data..."
  cargo run -p localdb-cli --bin localdb-indexer || exit 1
fi

# Run a sample query
cargo run -p localdb-cli --bin localdb-vector-search "fire" || exit 1

echo "✅ Done"
