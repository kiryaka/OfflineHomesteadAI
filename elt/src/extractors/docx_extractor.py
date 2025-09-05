"""DOCX document extractor using unstructured."""

import logging
from pathlib import Path
from typing import List, Dict, Any
from unstructured.partition.docx import partition_docx
from unstructured.staging.base import elements_to_json

logger = logging.getLogger(__name__)


class DOCXExtractor:
    """Extract text and metadata from DOCX documents."""
    
    def extract(self, file_path: Path) -> Dict[str, Any]:
        """Extract text and metadata from DOCX.
        
        Args:
            file_path: Path to DOCX file
            
        Returns:
            Dictionary with extracted content and metadata
        """
        try:
            logger.info(f"Extracting DOCX: {file_path}")
            
            # Extract elements from DOCX
            elements = partition_docx(filename=str(file_path))
            
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
            logger.error(f"Failed to extract DOCX {file_path}: {e}")
            return {
                "content": "",
                "metadata": {
                    "file_path": str(file_path),
                    "file_name": file_path.name,
                    "error": str(e),
                }
            }
