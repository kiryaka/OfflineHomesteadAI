"""PDF document extractor using unstructured."""

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
        """Extract text and metadata from PDF.
        
        Args:
            file_path: Path to PDF file
            
        Returns:
            Dictionary with extracted content and metadata
        """
        try:
            logger.info(f"Extracting PDF: {file_path}")
            
            # Extract elements from PDF
            elements = partition_pdf(
                filename=str(file_path),
                strategy=self.strategy,
                languages=self.ocr_languages,
                include_page_breaks=True,
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
                "num_pages": len([e for e in elements_json if e.get("type") == "PageBreak"]) + 1,
                "elements": elements_json,
            }
            
            return {
                "content": text_content,
                "metadata": metadata,
            }
            
        except Exception as e:
            logger.error(f"Failed to extract PDF {file_path}: {e}")
            return {
                "content": "",
                "metadata": {
                    "file_path": str(file_path),
                    "file_name": file_path.name,
                    "error": str(e),
                }
            }
