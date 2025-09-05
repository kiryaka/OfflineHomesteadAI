"""Export data to Tantivy text search index."""

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
        """Export chunks to JSON format for Tantivy.
        
        Args:
            chunks: List of processed chunks
            
        Returns:
            Path to exported file
        """
        try:
            logger.info(f"Exporting {len(chunks)} chunks to Tantivy format")
            
            # Prepare data for Tantivy
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
            
            # Save to JSON
            output_file = self.output_dir / "tantivy_data.json"
            with open(output_file, "w", encoding="utf-8") as f:
                json.dump(tantivy_data, f, indent=2, ensure_ascii=False)
            
            logger.info(f"Exported to {output_file}")
            return output_file
            
        except Exception as e:
            logger.error(f"Failed to export to Tantivy: {e}")
            raise
