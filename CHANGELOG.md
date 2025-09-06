# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- GitHub Actions CI/CD pipeline
- Comprehensive documentation
- Contributing guidelines
- Security audit workflow

## [0.1.0] - 2024-01-XX

### Added
- **Hybrid Search System**: Combines Tantivy (text search) with LanceDB (vector search)
- **Environment-Based Configuration**: Separate dev/prod configurations with automated validation
- **Production Optimization**: Optimized for 100GB+ text corpora with 25M+ vectors
- **Development Optimization**: 300x faster search complexity for dev iteration
- **Comprehensive Testing**: Unit, integration, and regression test suites
- **Technical Documentation Focus**: Optimized for general technical knowledge (machinery, construction, engineering)

### Configuration Features
- **Development Config** (`config.dev.toml`):
  - 64 partitions for fast iteration
  - 4 search probes (6% coverage)
  - 10x refine factor for fast re-ranking
  - Debug logging enabled
  - Fast indexing mode

- **Production Config** (`config.prod.toml`):
  - 6,144 partitions for 25M+ vectors
  - 300 search probes (5% coverage)
  - 40x refine factor for accuracy
  - Monitoring and profiling enabled
  - Optimized for large-scale deployment

### Search Features
- **PQ Over-retrieval Strategy**: Retrieve top candidates using compressed vectors
- **Flat Re-ranking**: Exact distance computation for final results
- **Vector Compression**: 16x compression (1536 → 96 bytes per vector)
- **Storage Optimization**: 98.4% storage reduction (153.6GB → 2.4GB)

### Embedding Support
- **Primary Model**: `text-embedding-3-small` (OpenAI)
- **Dimensions**: 1536
- **Optimized For**: General technical knowledge
- **Alternative Models**: BGE-M3, E5, CodeBERT (configurable)

### Testing Framework
- **Unit Tests**: Individual function testing in `src/config.rs`
- **Integration Tests**: Component interaction testing in `tests/`
- **Regression Tests**: Performance and memory validation
- **Configuration Tests**: Environment-specific validation
- **Test Utilities**: Comprehensive test suite in `src/lib.rs`

### Development Tools
- **Workflow Script**: `dev_workflow.sh` for easy development
- **Test Runner**: Comprehensive test execution
- **Configuration Comparison**: Dev vs prod parameter comparison
- **Environment Detection**: Automatic environment-based loading

### Performance Characteristics
- **Development**: 300x faster search complexity than production
- **Production**: Optimized for 25M+ vectors with high accuracy
- **Memory Usage**: Efficient memory management with PQ compression
- **Scalability**: Linear scaling with corpus size

### Documentation
- **README.md**: Comprehensive project documentation
- **CONFIG_README.md**: Detailed configuration guide
- **CONTRIBUTING.md**: Contribution guidelines
- **API Documentation**: Rustdoc-generated documentation

### Project Structure
```
src/
├── lib.rs                    # Library crate with test utilities
├── config.rs                 # Configuration module with unit tests
├── main.rs                   # Main binary
├── facet_mapping.rs          # Facet mapping binary
├── lancedb_demo.rs           # LanceDB demo binary
├── apps/localdb-cli               # CLI binaries (index/search)
└── search_tests.rs           # Search tests binary

tests/
├── config_integration_tests.rs  # Integration tests
└── config_test_runner.rs        # Test runner integration test

config.dev.toml               # Development configuration
config.prod.toml              # Production configuration
config.toml                   # Default configuration
dev_workflow.sh               # Development workflow script
```

### Dependencies
- **Tantivy**: 0.24 - Fast full-text search engine
- **LanceDB**: Vector database for AI applications
- **Serde**: 1.0 - Serialization framework
- **Anyhow**: 1.0 - Error handling
- **Walkdir**: 2.5 - Directory traversal
- **Twox-hash**: 1.6 - Fast hashing

### License
- **MIT License**: Open source with permissive licensing

### Repository
- **GitHub**: https://github.com/yourusername/local-db-engine
- **Documentation**: https://docs.rs/local-db-engine
- **Issues**: GitHub Issues for bug reports and feature requests
- **Discussions**: GitHub Discussions for community interaction

---

## Version History

- **0.1.0**: Initial release with hybrid search system and environment-based configuration
