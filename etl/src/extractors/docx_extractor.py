"""
DOCX document extractor using unstructured library.

This module provides specialized text extraction for Microsoft Word documents (.docx files)
using the unstructured library. It extracts text content while preserving document
structure and formatting information.

Key Features:
- Text extraction from DOCX files
- Preservation of document structure (paragraphs, headings, lists)
- Metadata extraction (file info, element counts)
- Error handling and logging
- Structured element access

Example:
    >>> from etl.src.extractors.docx_extractor import DOCXExtractor
    >>> 
    >>> # Initialize extractor
    >>> extractor = DOCXExtractor()
    >>> 
    >>> # Extract from DOCX
    >>> result = extractor.extract(Path("document.docx"))
    >>> print(f"Extracted {len(result['content'])} characters")
    >>> print(f"Elements: {result['metadata']['num_elements']}")
"""

import logging
from pathlib import Path
from typing import List, Dict, Any
from unstructured.partition.docx import partition_docx
from unstructured.staging.base import elements_to_json

logger = logging.getLogger(__name__)


class DOCXExtractor:
    """Extract text and metadata from DOCX documents."""
    
    def extract(self, file_path: Path) -> Dict[str, Any]:
        """
        Extract text and metadata from DOCX document.
        
        This method uses the unstructured library to extract text content and metadata
        from Microsoft Word documents. It preserves document structure and provides
        access to individual elements for advanced processing.
        
        Args:
            file_path: Path to the DOCX file to extract
            
        Returns:
            Dictionary containing:
                - content: Extracted text content as string
                - metadata: Dictionary with file information and extraction details
                    - file_path: Original file path
                    - file_name: File name
                    - file_size: File size in bytes
                    - num_elements: Number of extracted elements
                    - elements: Raw extracted elements (if successful)
                    - error: Error message (if extraction failed)
        """
        try:
            logger.info(f"Extracting DOCX: {file_path}")
            
            # Use unstructured to partition the DOCX into elements
            # This preserves document structure (paragraphs, headings, lists, etc.)
            elements = partition_docx(filename=str(file_path))
            
            # Convert elements to JSON format for easier processing
            # This provides structured access to text, formatting, and layout information
            elements_json = elements_to_json(elements)
            
            # Extract and concatenate text content from all elements
            # Join with double newlines to preserve paragraph structure
            text_content = "\n\n".join([elem.get("text", "") for elem in elements_json])
            
            # Build comprehensive metadata dictionary
            metadata = {
                "file_path": str(file_path),
                "file_name": file_path.name,
                "file_size": file_path.stat().st_size,
                "num_elements": len(elements_json),  # Count of extracted elements
                "elements": elements_json,  # Raw elements for advanced processing
            }
            
            return {
                "content": text_content,
                "metadata": metadata,
            }
            
        except Exception as e:
            # Log error and return empty result with error information
            logger.error(f"Failed to extract DOCX {file_path}: {e}")
            return {
                "content": "",
                "metadata": {
                    "file_path": str(file_path),
                    "file_name": file_path.name,
                    "error": str(e),
                }
            }
