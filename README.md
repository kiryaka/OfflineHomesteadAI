# Tantivy + LanceDB Hybrid Search Demo

A demonstration of combining Tantivy (text search) with LanceDB (vector search) for comprehensive document search capabilities.

## 🎯 Features

### Tantivy (Text Search Engine)
- ✅ **Full-text search** with exact matching
- ✅ **Fuzzy search** with edit distance (typo tolerance)
- ✅ **Faceted search** with hierarchical categories
- ✅ **Boolean queries** (AND, OR, NOT)
- ✅ **Context snippets** with highlighted matches
- ✅ **Deterministic facet allocation** based on filename hash

### LanceDB (Vector Search Engine)
- ✅ **Semantic similarity search** (placeholder implementation)
- ✅ **Deterministic embeddings** based on content hash
- ✅ **Same facet mapping** as Tantivy for consistency

## 📊 Data Schema

### Tantivy Index
```json
{
  "id": "000_Harvard_Classics:0",
  "doc_id": "000_Harvard_Classics", 
  "doc_path": "../data/000_Harvard_Classics.txt",
  "text": "Full document content...",
  "category": "/lit/fiction",
  "category_text": "/lit/fiction"
}
```

### LanceDB Table
```json
{
  "id": "000_Harvard_Classics:0",
  "filename": "000_Harvard_Classics.txt",
  "content": "Full document content...",
  "embedding": [0.702, 0.451, 0.836, ...],
  "facet": "lit/fiction"
}
```

## 🔧 Deterministic Facet Allocation

Files are assigned to facets based on filename hash (modulo 4):
- `hash % 4 == 0` → `tech/math`
- `hash % 4 == 1` → `tech/it`
- `hash % 4 == 2` → `lit/fiction`
- `hash % 4 == 3` → `lit/romcom`

This ensures **consistent facet mapping** between Tantivy and LanceDB.

## 🚀 Usage

### Build and Run
```bash
# Build all binaries
cargo build --release

# Index data with Tantivy
cargo run --release

# Search with Tantivy
cargo run --release --bin search_tests -- ./tantivy_index -q "coffee" --fuzzy

# Show facet mapping
cargo run --release --bin facet_mapping

# Show LanceDB demo
cargo run --release --bin lancedb_demo
```

### Search Examples
```bash
# Exact text search
cargo run --release --bin search_tests -- ./tantivy_index -q "coffee house"

# Fuzzy search (typo tolerant)
cargo run --release --bin search_tests -- ./tantivy_index -q "coffe" --fuzzy

# Faceted search
cargo run --release --bin search_tests -- ./tantivy_index -q "coffee" -f tech/math

# Multiple facets
cargo run --release --bin search_tests -- ./tantivy_index -q "coffee" -f tech lit

# Limit results
cargo run --release --bin search_tests -- ./tantivy_index -q "coffee" -n 10
```

## 📈 Performance

### Index Size
- **Tantivy**: ~67MB for 52 Harvard Classics
- **LanceDB**: Would be similar size with real embeddings

### Search Speed
- **Tantivy**: Sub-millisecond for most queries
- **LanceDB**: Depends on vector dimension and dataset size

## 🔗 Hybrid Search Architecture

```rust
async fn hybrid_search(query: &str) -> Vec<SearchResult> {
    // 1. Text search with Tantivy
    let text_results = tantivy_search(query).await?;
    
    // 2. Vector search with LanceDB
    let query_embedding = generate_embedding(query).await?;
    let vector_results = lancedb_search(&query_embedding).await?;
    
    // 3. Merge and rank results
    let combined = merge_results(text_results, vector_results);
    
    combined
}
```

## 🎯 Benefits

### Tantivy Strengths
- ✅ **Exact matching** ("coffee house")
- ✅ **Fuzzy search** ("coffe" → "coffee")
- ✅ **Faceted filtering** (tech/math, lit/fiction)
- ✅ **Boolean queries** (AND, OR, NOT)
- ✅ **Fast indexing** and **small index size**

### LanceDB Strengths
- ✅ **Semantic search** ("caffeine drink" → coffee docs)
- ✅ **Similarity search** (find docs like this one)
- ✅ **Multi-modal search** (text + images)
- ✅ **Recommendation systems**

### Combined Benefits
- 🎯 **Comprehensive coverage**: Exact + semantic matches
- 🚀 **High performance**: Both Rust-based, optimized
- 🔧 **Consistent data**: Same facet mapping
- 📊 **Rich results**: Text snippets + similarity scores

## 📁 Project Structure

```
universe/
├── src/
│   ├── main.rs              # Tantivy indexing and search
│   ├── search_tests.rs      # CLI search tool
│   ├── facet_mapping.rs     # Show facet assignments
│   └── lancedb_demo.rs      # LanceDB integration demo
├── data/                    # Harvard Classics text files
├── tantivy_index/           # Tantivy index directory
├── Cargo.toml              # Dependencies
└── README.md               # This file
```

## 🔮 Next Steps

1. **Real Embeddings**: Replace placeholder embeddings with actual AI-generated vectors
2. **LanceDB Integration**: Add actual LanceDB client code
3. **Result Merging**: Implement intelligent result combination
4. **Scoring**: Develop hybrid scoring algorithms
5. **API**: Create REST API for search endpoints

## 📚 Dependencies

- **tantivy**: Text search engine
- **walkdir**: File system traversal
- **twox_hash**: Fast hashing for deterministic facets
- **anyhow**: Error handling
- **bytemuck**: Byte conversion utilities

## 🎉 Conclusion

This demo shows how Tantivy and LanceDB can work together to provide comprehensive search capabilities. Tantivy handles exact text matching and faceting, while LanceDB provides semantic similarity search. The deterministic facet allocation ensures consistency between both systems, making hybrid search results meaningful and coherent.
