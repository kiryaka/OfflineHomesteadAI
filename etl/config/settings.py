"""Configuration settings for the ELT pipeline."""

from pathlib import Path
from typing import List, Dict, Any
import yaml


class ELTConfig:
    """Configuration for the ELT pipeline."""
    
    def __init__(self, config_file: Path = None):
        """Initialize configuration.
        
        Args:
            config_file: Path to YAML configuration file
        """
        self.config_file = config_file or Path("config/etl_config.yaml")
        self.config = self._load_config()
    
    def _load_config(self) -> Dict[str, Any]:
        """Load configuration from file."""
        if self.config_file.exists():
            with open(self.config_file, 'r') as f:
                return yaml.safe_load(f)
        else:
            return self._default_config()
    
    def _default_config(self) -> Dict[str, Any]:
        """Default configuration."""
        return {
            "extraction": {
                "pdf": {
                    "strategy": "auto",
                    "ocr_languages": ["eng"]
                },
                "image": {
                    "ocr_languages": ["eng"]
                },
                "supported_formats": ["pdf", "docx", "jpg", "jpeg", "png", "tiff"]
            },
            "chunking": {
                "chunk_size": 1000,
                "chunk_overlap": 200,
                "separators": ["\n\n", "\n", " ", ""]
            },
            "embedding": {
                "model": "text-embedding-3-small",
                "batch_size": 32
            },
            "output": {
                "tantivy_format": "json",
                "lancedb_format": "json"
            }
        }
    
    def get_extraction_config(self) -> Dict[str, Any]:
        """Get extraction configuration."""
        return self.config.get("extraction", {})
    
    def get_chunking_config(self) -> Dict[str, Any]:
        """Get chunking configuration."""
        return self.config.get("chunking", {})
    
    def get_embedding_config(self) -> Dict[str, Any]:
        """Get embedding configuration."""
        return self.config.get("embedding", {})
    
    def get_output_config(self) -> Dict[str, Any]:
        """Get output configuration."""
        return self.config.get("output", {})
