# Universe - Hybrid Search System

A powerful hybrid search system combining text search (Tantivy) and vector search (LanceDB) for processing and searching large document collections.

## 🏗️ Architecture

```
Raw Documents → ELT Pipeline → Processed Data → Search System
```

- **ELT Pipeline** (Python): Extract, Load, Transform documents
- **Search System** (Rust): Fast text and vector search

## 📁 Project Structure

```
universe/
├── search/                    # Rust search system
│   ├── src/                   # Source code
│   ├── tests/                 # Tests
│   ├── Cargo.toml            # Rust dependencies
│   └── config*.toml          # Configuration files
│
├── etl/                      # Python data processing pipeline
│   ├── src/                  # Source code
│   │   ├── extractors/       # Document extractors
│   │   ├── processors/       # Text processors
│   │   └── loaders/          # Data loaders
│   ├── config/               # Configuration
│   ├── requirements.txt      # Python dependencies
│   └── pyproject.toml        # Python project config
│
├── data/                     # Shared data directory
│   ├── raw/                  # Raw documents
│   ├── processed/            # Clean text files
│   ├── chunks/               # Chunked text
│   ├── embeddings/           # Vector embeddings
│   └── indexes/              # Search indexes
│
├── scripts/                  # Utility scripts
│   ├── setup.sh             # Initial setup
│   ├── run_etl_pipeline.sh  # Run data processing
│   └── run_search.sh        # Run search system
│
└── docs/                     # Documentation
```

## 🚀 Quick Start

### 1. Setup

```bash
# Clone and setup
git clone <repository>
cd universe
./scripts/setup.sh
```

### 2. Add Documents

```bash
# Add your documents to data/raw/
cp your_documents/* data/raw/
```

### 3. Process Documents

```bash
# Run the complete ELT pipeline
./scripts/run_etl_pipeline.sh
```

### 4. Start Search System

```bash
# Start the search server
./scripts/run_search.sh
```

## 🔧 Configuration

### Search System (Rust)
- `search/config.dev.toml` - Development settings
- `search/config.prod.toml` - Production settings

### ELT Pipeline (Python)
- `etl/config/etl_config.yaml` - Processing settings

## 📊 Features

### ELT Pipeline
- **Document Extraction**: PDF, DOCX, images, and more
- **Text Processing**: Smart chunking and tokenization
- **Embedding Generation**: Vector embeddings for semantic search
- **Data Export**: Clean data for search systems

### Search System
- **Text Search**: Fast full-text search with Tantivy
- **Vector Search**: Semantic search with LanceDB
- **Hybrid Search**: Combine text and vector results
- **Environment Configs**: Separate dev/prod settings

## 🛠️ Development

### Rust Search System
```bash
cd search
cargo test                    # Run tests
cargo run --bin lancedb_production_example  # Run production example
```

### Python ELT Pipeline
```bash
cd etl
pip install -r requirements.txt
python -m src.cli --help      # See available commands
```

## 📈 Performance

- **Development**: Fast iteration with smaller indexes
- **Production**: Optimized for 100GB+ corpora
- **Vector Compression**: 16x compression with 98.4% storage reduction
- **Search Speed**: Sub-second search across large collections

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📄 License

MIT License - see LICENSE file for details.