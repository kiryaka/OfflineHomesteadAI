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
├── crates/                   # Rust library crates
│   ├── localdb-core          # config + data processing
│   ├── localdb-text          # Tantivy index/search
│   ├── localdb-embed         # Embedding backends (Candle + fake)
│   └── localdb-vector        # LanceDB index/search
│
├── apps/                     # CLI binaries
│   └── localdb-cli           # indexer + search CLIs (config*.toml)
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
├── dev_data/                 # Developer data (indexes, txt)
├── test_data/                # Test data (gitignored)
├── models/                   # Local models (e.g., bge-m3)
│
├── scripts/                  # Utility scripts
│   ├── setup.sh              # Initial setup
│   ├── run_etl_pipeline.sh   # Run data processing
│   └── run_search.sh         # Demo vector search
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

### 2. Development Environment

```bash
# Activate virtual environment (required for Python ETL)
source .venv/bin/activate

# Or use the setup script
./scripts/setup_env.sh
```

### 3. Add Documents

```bash
# Add your documents to dev_data/raw/
cp your_documents/* dev_data/raw/
```

### 4. Process Documents

```bash
# Run the ELT pipeline (requires venv activated)
cd etl && python load.py --env dev
```

### 5. Index and Search (Rust)

```bash
# Build workspace
cargo build

# Index from dev_data/txt into dev_data/indexes/*
cargo run -p localdb-cli --bin localdb-indexer

# Text search (Tantivy)
cargo run -p localdb-cli --bin localdb-tantivy-search 'your query'

# Vector search (LanceDB)
cargo run -p localdb-cli --bin localdb-vector-search 'your query'
```

## 🔧 Configuration

### Search System (Rust)
- `apps/localdb-cli/config.dev.toml` - Development settings
- `apps/localdb-cli/config.prod.toml` - Production settings
- `apps/localdb-cli/config.test.toml` - Test settings

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

### Rust workspace
```bash
# Run full-flow tests per engine
cargo test -p localdb-text -p localdb-vector -- --show-output

# Build and run CLIs
cargo run -p localdb-cli --bin localdb-indexer
cargo run -p localdb-cli --bin localdb-tantivy-search 'query'
cargo run -p localdb-cli --bin localdb-vector-search 'query'
```

### Python ELT Pipeline
```bash
# Activate virtual environment first
source .venv/bin/activate

# Run ETL pipeline
cd etl && python load.py --env dev

# Or use the setup script
./scripts/setup_env.sh
cd etl && python load.py --env dev
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
