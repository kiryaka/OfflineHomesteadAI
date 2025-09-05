# Universe Architecture

## Overview

Universe is a hybrid search system that combines text search (Tantivy) and vector search (LanceDB) to provide fast, accurate search across large document collections.

## System Components

### 1. ELT Pipeline (Python)

**Purpose**: Extract, Load, Transform documents into searchable format

**Components**:
- **Extractors**: Parse various document formats (PDF, DOCX, images)
- **Processors**: Clean, chunk, and tokenize text
- **Loaders**: Export data for search systems

**Technologies**:
- `unstructured` - Document parsing
- `langchain` - Text processing
- `tiktoken` - Tokenization
- `sentence-transformers` - Embeddings

### 2. Search System (Rust)

**Purpose**: Fast text and vector search with hybrid results

**Components**:
- **Tantivy**: Full-text search engine
- **LanceDB**: Vector database for semantic search
- **Hybrid Search**: Combine text and vector results

**Technologies**:
- `tantivy` - Text search
- `lancedb` - Vector search
- `tokio` - Async runtime

## Data Flow

```
Raw Documents → ELT Pipeline → Processed Data → Search System
     ↓              ↓              ↓              ↓
  PDF/DOCX    →  Extractors  →  Clean Text  →  Tantivy Index
  Images      →  Processors  →  Chunks      →  LanceDB Index
  Various     →  Loaders     →  Embeddings  →  Hybrid Search
```

## Directory Structure

```
universe/
├── search/           # Rust search system
├── elt/             # Python data processing
├── data/            # Shared data directory
├── scripts/         # Utility scripts
└── docs/            # Documentation
```

## Configuration

### Development vs Production

- **Development**: Fast iteration, smaller indexes
- **Production**: Optimized for 100GB+ corpora

### Environment Variables

- `RUST_ENV=dev` - Use development config
- `RUST_ENV=prod` - Use production config

## Performance Characteristics

### ELT Pipeline
- **Throughput**: ~1000 documents/hour
- **Memory**: 2-4GB for typical workloads
- **Storage**: 16x compression with vector quantization

### Search System
- **Query Latency**: <100ms for typical queries
- **Index Size**: 98.4% reduction with PQ compression
- **Concurrent Users**: 1000+ with proper hardware

## Scalability

### Horizontal Scaling
- **ELT Pipeline**: Can be distributed across multiple workers
- **Search System**: Can be replicated for read scaling

### Vertical Scaling
- **Memory**: 8GB+ recommended for production
- **CPU**: Multi-core for parallel processing
- **Storage**: SSD recommended for index performance

## Security Considerations

- **Data Privacy**: All processing happens locally
- **Access Control**: Implement at application level
- **Encryption**: Use filesystem encryption for sensitive data

## Monitoring and Observability

- **Logging**: Structured logging throughout
- **Metrics**: Performance and usage metrics
- **Health Checks**: System health monitoring
