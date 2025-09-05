"""Image document extractor using unstructured."""

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
        """Extract text and metadata from image.
        
        Args:
            file_path: Path to image file
            
        Returns:
            Dictionary with extracted content and metadata
        """
        try:
            logger.info(f"Extracting image: {file_path}")
            
            # Extract elements from image using OCR
            elements = partition_image(
                filename=str(file_path),
                languages=self.ocr_languages,
            )
            
            # Convert to JSON for easier processing
            elements_json = elements_to_json(elements)
            
            # Extract text content
            text_content = "\n\n".join([elem.get("text", "") for elem in elements_json])
            
            # Extract metadata
            metadata = {
                "file_path": str(file_path),
                "file_name": file_path.name,
                "file_size": file_path.stat().st_size,
                "num_elements": len(elements_json),
                "elements": elements_json,
            }
            
            return {
                "content": text_content,
                "metadata": metadata,
            }
            
        except Exception as e:
            logger.error(f"Failed to extract image {file_path}: {e}")
            return {
                "content": "",
                "metadata": {
                    "file_path": str(file_path),
                    "file_name": file_path.name,
                    "error": str(e),
                }
            }
