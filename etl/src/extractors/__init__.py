"""
Document extractors for various formats.

This module provides specialized extractors for different document formats,
each optimized for the specific characteristics of that format.

Available Extractors:
- PDFExtractor: PDF documents with multiple extraction strategies
- DOCXExtractor: Microsoft Word documents
- ImageExtractor: Image files with OCR capabilities

Example:
    >>> from etl.src.extractors import PDFExtractor, DOCXExtractor, ImageExtractor
    >>> 
    >>> # Initialize extractors
    >>> pdf_extractor = PDFExtractor(strategy="auto")
    >>> docx_extractor = DOCXExtractor()
    >>> image_extractor = ImageExtractor(ocr_languages=["eng"])
    >>> 
    >>> # Extract from different formats
    >>> pdf_result = pdf_extractor.extract(Path("document.pdf"))
    >>> docx_result = docx_extractor.extract(Path("document.docx"))
    >>> image_result = image_extractor.extract(Path("document.png"))
"""

from .pdf_extractor import PDFExtractor
from .docx_extractor import DOCXExtractor
from .image_extractor import ImageExtractor

__all__ = ["PDFExtractor", "DOCXExtractor", "ImageExtractor"]
