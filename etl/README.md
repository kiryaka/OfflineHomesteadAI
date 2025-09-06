# Universe ELT Pipeline

Extract, Load, Transform pipeline for processing documents and preparing them for the search system.

## Features

- **Document Extraction**: Support for PDF, DOCX, HTML, images, and more
- **Text Processing**: Smart chunking, tokenization, and cleaning
- **Embedding Generation**: Create vector embeddings for semantic search
- **Data Export**: Output clean data for Tantivy and LanceDB

## Quick Start

```bash
# Install dependencies
pip install -r requirements.txt

# Process documents
python -m etl.cli extract --input data/raw --output data/processed

# Generate embeddings
python -m etl.cli embed --input data/processed --output data/embeddings

# Load into search system
python -m etl.cli load --embeddings data/embeddings --indexes data/indexes
```

## Architecture

```
Raw Documents → Extractors → Processors → Loaders → Search Indexes
```

- **Extractors**: Parse various document formats
- **Processors**: Clean, chunk, and tokenize text
- **Loaders**: Export data for search systems
