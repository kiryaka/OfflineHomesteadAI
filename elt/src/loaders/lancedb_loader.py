"""Export data to LanceDB vector database."""

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
        """Export chunks to JSON format for LanceDB.
        
        Args:
            chunks: List of processed chunks with embeddings
            
        Returns:
            Path to exported file
        """
        try:
            logger.info(f"Exporting {len(chunks)} chunks to LanceDB format")
            
            # Prepare data for LanceDB
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
            
            # Save to JSON
            output_file = self.output_dir / "lancedb_data.json"
            with open(output_file, "w", encoding="utf-8") as f:
                json.dump(lancedb_data, f, indent=2, ensure_ascii=False)
            
            logger.info(f"Exported to {output_file}")
            return output_file
            
        except Exception as e:
            logger.error(f"Failed to export to LanceDB: {e}")
            raise
