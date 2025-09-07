"""
Export data to LanceDB vector database format.

This module provides functionality to export processed text chunks with embeddings
to LanceDB format. LanceDB is a vector database that can be used for semantic
search and similarity matching.

Key Features:
- Export text chunks with embeddings to LanceDB format
- Preserve metadata and chunk relationships
- JSON format for easy integration
- Error handling and logging

LanceDB Format:
- id: Unique chunk identifier
- content: Text content of the chunk
- embedding: Vector embedding for semantic search
- file_name: Source file name
- file_path: Source file path
- chunk_index: Index of chunk within document
- total_chunks: Total number of chunks in document
- token_count: Number of tokens in chunk
- embedding_model: Model used for embedding generation
- embedding_dimension: Dimension of embedding vector

Example:
    >>> from etl.src.loaders.lancedb_loader import LanceDBLoader
    >>> 
    >>> # Initialize loader
    >>> loader = LanceDBLoader(Path("output/lancedb"))
    >>> 
    >>> # Export chunks
    >>> chunks = [{"chunk_id": "doc_0", "content": "Text", "embedding": [0.1, 0.2], "metadata": {...}}]
    >>> output_file = loader.export_chunks(chunks)
    >>> print(f"Exported to {output_file}")
"""

import logging
import json
from pathlib import Path
from typing import List, Dict, Any

logger = logging.getLogger(__name__)


class LanceDBLoader:
    """Export processed data to LanceDB format."""
    
    def __init__(self, output_dir: Path):
        """Initialize LanceDB loader.
        
        Args:
            output_dir: Directory to save LanceDB data
        """
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)
    
    def export_chunks(self, chunks: List[Dict[str, Any]]) -> Path:
        """
        Export chunks to JSON format for LanceDB vector database.
        
        This method takes processed text chunks with embeddings and converts them
        to LanceDB format. The output is a JSON file that can be imported into
        LanceDB for vector search and similarity matching.
        
        Args:
            chunks: List of processed chunks, each containing:
                - chunk_id: Unique identifier
                - content: Text content
                - embedding: Vector embedding
                - metadata: Dictionary with additional information
                
        Returns:
            Path to the exported JSON file
            
        Raises:
            Exception: If export fails for any reason
            
        Note:
            The exported JSON file contains all necessary information for LanceDB
            including embeddings, metadata, and chunk relationships. The file
            is saved with UTF-8 encoding to support international characters.
        """
        try:
            logger.info(f"Exporting {len(chunks)} chunks to LanceDB format")
            
            # Transform chunks to LanceDB format
            # This flattens the metadata structure for easier database import
            lancedb_data = []
            for chunk in chunks:
                lancedb_doc = {
                    "id": chunk.get("chunk_id", ""),
                    "content": chunk.get("content", ""),
                    "embedding": chunk.get("embedding", []),
                    "file_name": chunk.get("metadata", {}).get("file_name", ""),
                    "file_path": chunk.get("metadata", {}).get("file_path", ""),
                    "chunk_index": chunk.get("metadata", {}).get("chunk_index", 0),
                    "total_chunks": chunk.get("metadata", {}).get("total_chunks", 0),
                    "token_count": chunk.get("metadata", {}).get("token_count", 0),
                    "embedding_model": chunk.get("metadata", {}).get("embedding_model", ""),
                    "embedding_dimension": chunk.get("metadata", {}).get("embedding_dimension", 0),
                }
                lancedb_data.append(lancedb_doc)
            
            # Save to JSON file with proper formatting
            output_file = self.output_dir / "lancedb_data.json"
            with open(output_file, "w", encoding="utf-8") as f:
                json.dump(lancedb_data, f, indent=2, ensure_ascii=False)
            
            logger.info(f"Exported to {output_file}")
            return output_file
            
        except Exception as e:
            logger.error(f"Failed to export to LanceDB: {e}")
            raise
