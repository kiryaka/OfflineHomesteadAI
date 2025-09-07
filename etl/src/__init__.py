"""
ETL Pipeline - Extract, Transform, Load for document processing.

This package provides a comprehensive ETL pipeline for processing documents
of various formats and preparing them for search and analysis. It supports
text extraction, cleaning, chunking, embedding generation, and export to
multiple search systems.

Key Components:
- Extractors: Text extraction from PDF, DOCX, images, and other formats
- Processors: Text cleaning, chunking, tokenization, and embedding generation
- Loaders: Export to Tantivy (full-text search) and LanceDB (vector search)
- CLI: Command-line interface for pipeline execution

Supported Formats:
- PDF (text and image-based with OCR)
- DOCX, DOC (Microsoft Word documents)
- HTML, HTM (Web pages)
- Markdown, TXT (Plain text)
- RTF, EPUB (Rich text formats)
- MSG, EML (Email formats)
- Images: JPG, PNG, TIFF, BMP, GIF (with OCR)

Example:
    >>> from etl.src.pdf_processor import FileProcessor
    >>> from etl.config.settings import Config
    >>> 
    >>> # Initialize processor
    >>> config = Config('etl/config/dev.yaml')
    >>> processor = FileProcessor(config)
    >>> 
    >>> # Process a document
    >>> cleaned_text = processor.process_file(Path('document.pdf'))
    >>> print(f"Extracted {len(cleaned_text)} characters")
"""

__version__ = "0.1.0"
__author__ = "Kirill Butin"