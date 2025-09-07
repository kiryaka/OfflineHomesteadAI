"""
Image document extractor using unstructured library with OCR.

This module provides text extraction from image files using Optical Character Recognition (OCR)
through the unstructured library. It supports multiple image formats and languages.

Key Features:
- OCR text extraction from images
- Multi-language OCR support
- Support for common image formats (JPG, PNG, TIFF, BMP, GIF)
- Metadata extraction and error handling
- Structured element access

Supported Image Formats:
- JPG/JPEG: JPEG images
- PNG: Portable Network Graphics
- TIFF/TIF: Tagged Image File Format
- BMP: Bitmap images
- GIF: Graphics Interchange Format

Requirements:
- Tesseract OCR engine must be installed on the system
- Python packages: unstructured, pytesseract

Example:
    >>> from etl.src.extractors.image_extractor import ImageExtractor
    >>> 
    >>> # Initialize extractor with English OCR
    >>> extractor = ImageExtractor(ocr_languages=["eng"])
    >>> 
    >>> # Extract from image
    >>> result = extractor.extract(Path("document.png"))
    >>> print(f"Extracted {len(result['content'])} characters")
    >>> print(f"Elements: {result['metadata']['num_elements']}")
"""

import logging
from pathlib import Path
from typing import List, Dict, Any
from unstructured.partition.image import partition_image
from unstructured.staging.base import elements_to_json

logger = logging.getLogger(__name__)


class ImageExtractor:
    """Extract text and metadata from image documents using OCR."""
    
    def __init__(self, ocr_languages: List[str] = None):
        """Initialize image extractor.
        
        Args:
            ocr_languages: Languages for OCR (e.g., ['eng', 'spa'])
        """
        self.ocr_languages = ocr_languages or ["eng"]
    
    def extract(self, file_path: Path) -> Dict[str, Any]:
        """
        Extract text and metadata from image using OCR.
        
        This method uses the unstructured library with OCR capabilities to extract
        text content from image files. It requires Tesseract OCR to be installed
        on the system and configured for the specified languages.
        
        Args:
            file_path: Path to the image file to extract
            
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
                    
        Note:
            This method requires Tesseract OCR to be installed and accessible.
            Common error: "tesseract is not installed or it's not in your PATH"
        """
        try:
            logger.info(f"Extracting image: {file_path}")
            
            # Use unstructured to partition the image using OCR
            # This extracts text from images using Tesseract OCR engine
            elements = partition_image(
                filename=str(file_path),
                languages=self.ocr_languages,  # OCR language settings
            )
            
            # Convert elements to JSON format for easier processing
            # This provides structured access to extracted text and layout information
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
            logger.error(f"Failed to extract image {file_path}: {e}")
            return {
                "content": "",
                "metadata": {
                    "file_path": str(file_path),
                    "file_name": file_path.name,
                    "error": str(e),
                }
            }
