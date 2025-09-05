# Tantivy + LanceDB Hybrid Search System

A production-ready hybrid search system combining Tantivy (text search) with LanceDB (vector search) for comprehensive document search capabilities. Features environment-based configuration, automated testing, and optimized parameters for both development and production use.

## 🚀 Features

- **Hybrid Search**: Combines exact text matching (Tantivy) with semantic similarity (LanceDB)
- **Environment-Based Config**: Separate dev/prod configurations with automated validation
- **Production Ready**: Optimized for 100GB+ text corpora with 25M+ vectors
- **Comprehensive Testing**: Unit, integration, and regression tests
- **Fast Development**: 300x faster search complexity for dev iteration
- **Technical Documentation**: Optimized for general technical knowledge (machinery, construction, engineering, etc.)

## 📊 Performance Characteristics

### Development Configuration
- **Partitions**: 64 (fast iteration)
- **Search Probes**: 4 (6% coverage)
- **Refine Factor**: 10x over-retrieval
- **Search Complexity**: 40 (300x faster than prod)

### Production Configuration
- **Partitions**: 6,144 (1,000-4,000 vectors per partition)
- **Search Probes**: 300 (5% coverage)
- **Refine Factor**: 40x over-retrieval
- **Search Complexity**: 12,000 (optimized for 25M+ vectors)

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   User Query    │───▶│  Hybrid Search  │───▶│  Ranked Results │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                              │
                    ┌─────────┴─────────┐
                    │                   │
            ┌───────▼───────┐  ┌───────▼───────┐
            │   Tantivy     │  │    LanceDB    │
            │ (Text Search) │  │ (Vector Search)│
            └───────────────┘  └───────────────┘
```

## 🛠️ Installation

### Prerequisites
- Rust 1.70+
- 8GB+ RAM (for production)
- 100GB+ storage (for large corpora)

### Quick Start
```bash
# Clone the repository
git clone https://github.com/yourusername/tantivy-lancedb-hybrid-search.git
cd tantivy-lancedb-hybrid-search

# Build the project
cargo build --release

# Run development configuration
RUST_ENV=dev cargo run --release --bin lancedb_production_example

# Run production configuration
RUST_ENV=prod cargo run --release --bin lancedb_production_example
```

## ⚙️ Configuration

### Environment-Based Configuration

The system automatically detects the environment using the `RUST_ENV` variable:

```bash
# Development (fast iteration)
RUST_ENV=dev cargo run --bin my_app

# Production (optimized for scale)
RUST_ENV=prod cargo run --bin my_app

# Default (falls back to dev)
cargo run --bin my_app
```

### Configuration Files

- **`config.dev.toml`** - Development configuration (fast iteration)
- **`config.prod.toml`** - Production configuration (100GB+ corpus)
- **`config.toml`** - Default/fallback configuration

### Key Parameters

| Parameter | Development | Production | Purpose |
|-----------|-------------|------------|---------|
| `num_partitions` | 64 | 6,144 | Vector space partitioning |
| `nprobes` | 4 | 300 | Search coverage |
| `refine_factor` | 10x | 40x | Re-ranking accuracy |
| `default_limit` | 3 | 10 | Result count |

## 🧪 Testing

### Run All Tests
```bash
cargo test
```

### Run Specific Test Suites
```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test config_integration_tests
cargo test --test config_test_runner

# Development workflow
./dev_workflow.sh all
```

### Test Coverage
- **Unit Tests**: Individual function testing
- **Integration Tests**: Component interaction testing
- **Regression Tests**: Performance and memory validation
- **Configuration Tests**: Environment-specific validation

## 🚀 Usage

### Basic Search
```rust
use tantivy_demo::config::Config;

// Load configuration
let config = Config::load()?;

// Perform hybrid search
let results = hybrid_search(&query, &config)?;
```

### Environment-Specific Loading
```rust
// Load development configuration
let dev_config = Config::load_dev()?;

// Load production configuration
let prod_config = Config::load_prod()?;

// Load with explicit environment
let config = Config::load_for_env(Some("prod"))?;
```

## 📈 Performance Optimization

### Development
- **Fast Iteration**: 300x faster search complexity
- **Small Partitions**: 64 partitions for quick testing
- **Reduced Probes**: 4 probes for fast queries
- **Debug Logging**: Enabled for development insights

### Production
- **High Accuracy**: 40x refine factor for precision
- **Large Scale**: 6,144 partitions for 25M+ vectors
- **Comprehensive Coverage**: 300 probes for recall
- **Monitoring**: Performance profiling enabled

## 🔧 Development Workflow

### Quick Commands
```bash
# Show current environment and config
./dev_workflow.sh

# Compare dev vs prod configurations
./dev_workflow.sh compare

# Run all tests and validations
./dev_workflow.sh all

# Test specific environment
./dev_workflow.sh dev
./dev_workflow.sh prod
```

### Regression Prevention
- **Automated Validation**: Ensures dev < prod for performance parameters
- **Comprehensive Test Suite**: Catches configuration issues
- **Environment-Specific Validation**: Prevents invalid configurations
- **Performance Regression Tests**: Ensures no performance degradation

## 📚 Technical Details

### Embedding Model
- **Model**: `text-embedding-3-small` (OpenAI)
- **Dimensions**: 1536
- **Optimized For**: General technical knowledge (machinery, construction, engineering, etc.)
- **Alternative Models**: BGE-M3, E5, CodeBERT (configurable)

### Vector Index
- **Type**: IVF_PQ (Inverted File with Product Quantization)
- **Compression**: 16x (1536 → 96 bytes per vector)
- **Storage Reduction**: 98.4% (153.6GB → 2.4GB)
- **Search Strategy**: PQ over-retrieval + flat re-ranking

### Search Process
1. **PQ Search**: Fast initial retrieval using compressed vectors
2. **Flat Re-ranking**: Exact distance computation for top candidates
3. **Final Results**: Return most relevant documents after re-ranking

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Run tests (`./dev_workflow.sh all`)
4. Commit changes (`git commit -m 'Add amazing feature'`)
5. Push to branch (`git push origin feature/amazing-feature`)
6. Open a Pull Request

### Development Guidelines
- Follow Rust best practices
- Add tests for new features
- Update documentation
- Ensure all tests pass
- Use conventional commit messages

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Tantivy](https://github.com/quickwit-oss/tantivy) - Fast full-text search engine
- [LanceDB](https://github.com/lancedb/lancedb) - Vector database for AI applications
- [OpenAI](https://openai.com/) - Embedding models

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/tantivy-lancedb-hybrid-search/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/tantivy-lancedb-hybrid-search/discussions)
- **Documentation**: [Wiki](https://github.com/yourusername/tantivy-lancedb-hybrid-search/wiki)

---

**Version**: 0.01  
**Rust**: 1.70+  
**License**: MIT  
**Status**: Production Ready