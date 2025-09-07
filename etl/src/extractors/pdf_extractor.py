"""
PDF document extractor using unstructured library.

This module provides a specialized PDF text extraction service using the unstructured
library. It supports multiple extraction strategies and can handle both text-based
and image-based PDFs with OCR capabilities.

Key Features:
- Multiple extraction strategies (auto, fast, hi_res, ocr_only)
- OCR support for image-based PDFs
- Multi-language OCR support
- Structured element extraction (text, tables, images)
- Comprehensive metadata extraction
- Error handling and logging

Extraction Strategies:
- auto: Automatically chooses the best strategy
- fast: Quick extraction for text-based PDFs
- hi_res: High-resolution extraction for complex layouts
- ocr_only: Force OCR for image-based PDFs

Example:
    >>> from etl.src.extractors.pdf_extractor import PDFExtractor
    >>> 
    >>> # Initialize extractor
    >>> extractor = PDFExtractor(strategy="auto", ocr_languages=["eng"])
    >>> 
    >>> # Extract from PDF
    >>> result = extractor.extract(Path("document.pdf"))
    >>> print(f"Extracted {len(result['content'])} characters")
    >>> print(f"Pages: {result['metadata']['num_pages']}")
"""

import logging
from pathlib import Path
from typing import List, Dict, Any
from unstructured.partition.pdf import partition_pdf
from unstructured.staging.base import elements_to_json

logger = logging.getLogger(__name__)


class PDFExtractor:
    """Extract text and metadata from PDF documents."""
    
    def __init__(self, strategy: str = "auto", ocr_languages: List[str] = None):
        """Initialize PDF extractor.
        
        Args:
            strategy: Extraction strategy ('auto', 'fast', 'hi_res', 'ocr_only')
            ocr_languages: Languages for OCR (e.g., ['eng', 'spa'])
        """
        self.strategy = strategy
        self.ocr_languages = ocr_languages or ["eng"]
    
    def extract(self, file_path: Path) -> Dict[str, Any]:
        """
        Extract text and metadata from PDF document.
        
        This method uses the unstructured library to extract text content and metadata
        from PDF files. It supports multiple extraction strategies and can handle
        both text-based and image-based PDFs.
        
        Args:
            file_path: Path to the PDF file to extract
            
        Returns:
            Dictionary containing:
                - content: Extracted text content as string
                - metadata: Dictionary with file information and extraction details
                    - file_path: Original file path
                    - file_name: File name
                    - file_size: File size in bytes
                    - num_pages: Number of pages in the PDF
                    - elements: Raw extracted elements (if successful)
                    - error: Error message (if extraction failed)
        """
        try:
            logger.info(f"Extracting PDF: {file_path}")
            
            # Use unstructured to partition the PDF into elements
            # This handles both text extraction and OCR for image-based PDFs
            elements = partition_pdf(
                filename=str(file_path),
                strategy=self.strategy,  # Use configured extraction strategy
                languages=self.ocr_languages,  # OCR language settings
                include_page_breaks=True,  # Include page break markers
            )
            
            # Convert elements to JSON format for easier processing
            # This provides structured access to text, tables, images, etc.
            elements_json = elements_to_json(elements)
            
            # Extract and concatenate text content from all elements
            # Join with double newlines to preserve paragraph structure
            text_content = "\n\n".join([elem.get("text", "") for elem in elements_json])
            
            # Build comprehensive metadata dictionary
            metadata = {
                "file_path": str(file_path),
                "file_name": file_path.name,
                "file_size": file_path.stat().st_size,
                # Count pages by counting PageBreak elements + 1
                "num_pages": len([e for e in elements_json if e.get("type") == "PageBreak"]) + 1,
                "elements": elements_json,  # Raw elements for advanced processing
            }
            
            return {
                "content": text_content,
                "metadata": metadata,
            }
            
        except Exception as e:
            # Log error and return empty result with error information
            logger.error(f"Failed to extract PDF {file_path}: {e}")
            return {
                "content": "",
                "metadata": {
                    "file_path": str(file_path),
                    "file_name": file_path.name,
                    "error": str(e),
                }
            }
