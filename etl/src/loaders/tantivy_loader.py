"""
Export data to Tantivy text search index format.

This module provides functionality to export processed text chunks to Tantivy format.
Tantivy is a full-text search engine that provides fast and efficient text search
capabilities with BM25 scoring and faceted search.

Key Features:
- Export text chunks to Tantivy format
- Preserve metadata and chunk relationships
- JSON format for easy integration
- Error handling and logging

Tantivy Format:
- id: Unique chunk identifier
- content: Text content for full-text search
- file_name: Source file name
- file_path: Source file path
- chunk_index: Index of chunk within document
- total_chunks: Total number of chunks in document
- token_count: Number of tokens in chunk

Example:
    >>> from etl.src.loaders.tantivy_loader import TantivyLoader
    >>> 
    >>> # Initialize loader
    >>> loader = TantivyLoader(Path("output/tantivy"))
    >>> 
    >>> # Export chunks
    >>> chunks = [{"chunk_id": "doc_0", "content": "Text", "metadata": {...}}]
    >>> output_file = loader.export_chunks(chunks)
    >>> print(f"Exported to {output_file}")
"""

import logging
import json
from pathlib import Path
from typing import List, Dict, Any

logger = logging.getLogger(__name__)


class TantivyLoader:
    """Export processed data to Tantivy format."""
    
    def __init__(self, output_dir: Path):
        """Initialize Tantivy loader.
        
        Args:
            output_dir: Directory to save Tantivy data
        """
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)
    
    def export_chunks(self, chunks: List[Dict[str, Any]]) -> Path:
        """
        Export chunks to JSON format for Tantivy text search index.
        
        This method takes processed text chunks and converts them to Tantivy format.
        The output is a JSON file that can be imported into Tantivy for full-text
        search with BM25 scoring and faceted search capabilities.
        
        Args:
            chunks: List of processed chunks, each containing:
                - chunk_id: Unique identifier
                - content: Text content for search
                - metadata: Dictionary with additional information
                
        Returns:
            Path to the exported JSON file
            
        Raises:
            Exception: If export fails for any reason
            
        Note:
            The exported JSON file contains text content and metadata optimized
            for full-text search. Unlike LanceDB, this format focuses on text
            search rather than vector similarity search.
        """
        try:
            logger.info(f"Exporting {len(chunks)} chunks to Tantivy format")
            
            # Transform chunks to Tantivy format
            # This focuses on text content and metadata for full-text search
            tantivy_data = []
            for chunk in chunks:
                tantivy_doc = {
                    "id": chunk.get("chunk_id", ""),
                    "content": chunk.get("content", ""),
                    "file_name": chunk.get("metadata", {}).get("file_name", ""),
                    "file_path": chunk.get("metadata", {}).get("file_path", ""),
                    "chunk_index": chunk.get("metadata", {}).get("chunk_index", 0),
                    "total_chunks": chunk.get("metadata", {}).get("total_chunks", 0),
                    "token_count": chunk.get("metadata", {}).get("token_count", 0),
                }
                tantivy_data.append(tantivy_doc)
            
            # Save to JSON file with proper formatting
            output_file = self.output_dir / "tantivy_data.json"
            with open(output_file, "w", encoding="utf-8") as f:
                json.dump(tantivy_data, f, indent=2, ensure_ascii=False)
            
            logger.info(f"Exported to {output_file}")
            return output_file
            
        except Exception as e:
            logger.error(f"Failed to export to Tantivy: {e}")
            raise
