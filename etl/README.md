# Universe ETL Pipeline

A comprehensive Extract, Transform, Load (ETL) pipeline for processing documents of various formats and preparing them for search and analysis. This pipeline supports text extraction, cleaning, chunking, embedding generation, and export to multiple search systems.

## 🎯 Current Status

**✅ Production Ready**: The ETL pipeline is fully functional with comprehensive text cleaning, format support, and quality validation.

### Format Support Status
- **✅ Supported (6 formats)**: PDF (text), TXT, Markdown, HTML, DOCX, HTM
- **⏳ Pending (5 formats)**: RTF (needs pandoc), EML (email partitioner issue), EPUB, MSG, DOC
- **🔍 OCR Pending (8 formats)**: PDF (image), JPG, JPEG, PNG, TIFF, TIF, BMP, GIF

### Key Achievements
- Universal text cleaning with format-aware optimizations
- Comprehensive test suite with 19 format validations
- Unicode corruption detection and repair
- Hyphenation fixing across line breaks
- Quality validation with detailed reporting
- Support for both Tantivy (full-text) and LanceDB (vector) search

## 🏗️ Architecture & Design

The ETL pipeline follows a modular architecture with clear separation of concerns:

```
Raw Documents → Extractors → Processors → Loaders → Search Indexes
     ↓              ↓           ↓          ↓           ↓
  File I/O    Text Extraction  Cleaning   Export    Search Ready
```

### Core Components

#### 1. **Extractors** (`src/extractors/`)
- **`pdf_extractor.py`**: PDF text extraction with multiple strategies (PyMuPDF, pdfplumber, PyPDF2, unstructured)
- **`docx_extractor.py`**: Microsoft Word document processing using unstructured
- **`image_extractor.py`**: OCR text extraction from images using Tesseract

#### 2. **Processors** (`src/processors/`)
- **`chunker.py`**: Intelligent text chunking with semantic boundary preservation
- **`tokenizer.py`**: Accurate token counting using tiktoken (OpenAI's tokenizer)
- **`embedder.py`**: Vector embedding generation using sentence-transformers

#### 3. **Loaders** (`src/loaders/`)
- **`tantivy_loader.py`**: Export to Tantivy for full-text search with BM25 scoring
- **`lancedb_loader.py`**: Export to LanceDB for vector search and similarity matching

#### 4. **Core Processing** (`src/`)
- **`pdf_processor.py`**: Universal file processor with format-aware text cleaning
- **`cli.py`**: Command-line interface for pipeline execution
- **`load.py`**: Main ETL script for batch processing

#### 5. **Configuration** (`config/`)
- **`settings.py`**: Environment-specific configuration management
- **`dev.yaml`**, **`prod.yaml`**, **`test.yaml`**: Environment configurations

#### 6. **Testing** (`tests/`)
- **`test_text_cleaning.py`**: Comprehensive validation suite for all formats
- **`run_text_cleaning_tests.py`**: Test runner with detailed reporting
- **`FORMAT_STATUS.md`**: Format support status and roadmap

## 📁 File Structure & Purpose

```
etl/
├── src/
│   ├── pdf_processor.py          # Universal file processor with text cleaning
│   ├── cli.py                    # Command-line interface
│   ├── load.py                   # Main ETL script
│   ├── extractors/               # Format-specific text extractors
│   │   ├── pdf_extractor.py      # PDF processing with multiple strategies
│   │   ├── docx_extractor.py     # Word document processing
│   │   └── image_extractor.py    # OCR for image files
│   ├── processors/               # Text processing components
│   │   ├── chunker.py            # Text chunking for vector search
│   │   ├── tokenizer.py          # Token counting and processing
│   │   └── embedder.py           # Vector embedding generation
│   └── loaders/                  # Export to search systems
│       ├── tantivy_loader.py     # Full-text search export
│       └── lancedb_loader.py     # Vector search export
├── config/
│   ├── settings.py               # Configuration management
│   ├── dev.yaml                  # Development environment config
│   ├── prod.yaml                 # Production environment config
│   └── test.yaml                 # Test environment config
├── tests/
│   ├── test_text_cleaning.py     # Format validation test suite
│   ├── run_text_cleaning_tests.py # Test runner with reporting
│   ├── FORMAT_STATUS.md          # Format support status
│   └── README.md                 # Test documentation
├── requirements.txt              # Python dependencies
├── pyproject.toml               # Project configuration
└── README.md                    # This file
```

## 🚀 Quick Start

### Prerequisites
- Python 3.8+
- Virtual environment (`.venv` in workspace root)
- Tesseract OCR (for image processing)
- Pandoc (for RTF/EPUB processing)

### Installation
```bash
# Activate virtual environment
source .venv/bin/activate

# Install dependencies
pip install -r requirements.txt
```

### Basic Usage
```bash
# Process documents from raw to text
python load.py --env dev

# Run complete pipeline
python -m etl.src.cli pipeline --input data/raw --output data/processed

# Test format support
python etl/tests/run_text_cleaning_tests.py
```

## 🔧 Key Design Decisions

### 1. **Universal Text Cleaning Approach**
- **Decision**: Implemented a universal `normalize_for_search()` function with format-aware optimizations
- **Rationale**: Ensures consistent text quality across all formats while allowing format-specific tweaks
- **Implementation**: Uses `unstructured.cleaners.core` with conditional aggressive cleaning for corrupted text

### 2. **Format-Aware Processing**
- **Decision**: Conditional cleaning based on file format and content analysis
- **Rationale**: Different formats have different corruption patterns (PDFs have unicode issues, HTML has artifacts)
- **Implementation**: `_needs_aggressive_cleaning()` detects corruption and applies appropriate cleaning

### 3. **Comprehensive Test Suite**
- **Decision**: Created validation tests for all 19 planned formats with quality metrics
- **Rationale**: Ensures text quality and provides clear roadmap for format implementation
- **Implementation**: `TextValidator` class with universal validation rules and format-specific tests

### 4. **Multiple PDF Extraction Libraries**
- **Decision**: Support for PyMuPDF, pdfplumber, PyPDF2, and unstructured
- **Rationale**: Different libraries excel at different PDF types (text vs. image-based)
- **Implementation**: Configurable PDF extractor with fallback options

### 5. **Dual Search System Export**
- **Decision**: Export to both Tantivy (full-text) and LanceDB (vector) formats
- **Rationale**: Enables both keyword-based and semantic search capabilities
- **Implementation**: Separate loaders for each system with optimized data structures

## ⚠️ Important Considerations

### Dependencies to Remember
1. **Tesseract OCR**: Required for image processing (JPG, PNG, TIFF, etc.)
   - Install: `brew install tesseract` (macOS) or `apt-get install tesseract-ocr` (Ubuntu)
   - Common error: "tesseract is not installed or it's not in your PATH"

2. **Pandoc**: Required for RTF and EPUB processing
   - Install: `brew install pandoc` (macOS) or `apt-get install pandoc` (Ubuntu)
   - Common error: "No pandoc was found"

3. **Virtual Environment**: Always use the workspace `.venv`
   - Activate: `source .venv/bin/activate`
   - Never create new virtual environments

### Configuration Management
- **Environment Variables**: Use `ETL_ENV` to specify environment (dev/prod/test)
- **Path Overrides**: `ETL_RAW_DIR` and `ETL_TXT_DIR` can override default paths
- **Configuration Files**: Each environment has its own YAML config file

### Text Quality Standards
- **Unicode Corruption**: Automatically detected and cleaned
- **Hyphenation**: Fixed across line breaks
- **Whitespace**: Normalized and excessive empty lines removed
- **Format Artifacts**: Removed based on format type

### Performance Considerations
- **PDF Processing**: PyMuPDF is fastest for text-based PDFs
- **Image Processing**: OCR is slow and should be used sparingly
- **Chunking**: Overlap between chunks improves search quality but increases storage
- **Embeddings**: Model choice affects quality vs. speed tradeoff

## 🧪 Testing & Validation

### Running Tests
```bash
# Run all format tests
python etl/tests/run_text_cleaning_tests.py

# Run specific format test
pytest etl/tests/test_text_cleaning.py::TestTextCleaning::test_pdf_cleaning

# Test format support
python scripts/test_format_support.py
```

### Test Results
- **Current Status**: 6/19 formats fully supported (32%)
- **Quality**: All supported formats pass comprehensive validation
- **Coverage**: Tests cover unicode, hyphenation, whitespace, and format-specific issues

## 📈 Future Roadmap

### Phase 1: Document Formats (Next Priority)
1. **RTF**: Install pandoc system dependency
2. **EML**: Fix email partitioner configuration
3. **DOC**: Implement unstructured support
4. **EPUB**: Implement unstructured support
5. **MSG**: Implement unstructured support

### Phase 2: OCR Implementation
1. **Install Tesseract**: System dependency setup
2. **PDF Images**: OCR for image-based PDFs
3. **Image Formats**: JPG, PNG, TIFF, etc.

### Phase 3: Advanced Features
1. **Hybrid Search**: Combine full-text and vector search
2. **Metadata Extraction**: Enhanced document metadata
3. **Batch Processing**: Optimized for large document sets
4. **Monitoring**: Processing metrics and quality tracking

## 🤝 Contributing

When working on the ETL pipeline:

1. **Always use the workspace `.venv`** - never create new virtual environments
2. **Run tests before committing** - ensure all supported formats still pass
3. **Update format status** - modify `FORMAT_STATUS.md` when adding new formats
4. **Document decisions** - update this README with significant changes
5. **Test with real data** - validate with actual documents, not just test files

## 📚 Documentation

- **Code Documentation**: All Python files have comprehensive docstrings
- **Test Documentation**: `etl/tests/README.md` explains the test suite
- **Format Status**: `etl/tests/FORMAT_STATUS.md` tracks format support
- **Configuration**: `CONFIG_README.md` explains configuration options

---

**Last Updated**: December 2024  
**Version**: 0.1.0  
**Status**: Production Ready