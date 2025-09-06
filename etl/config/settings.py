"""Configuration management for ETL pipeline."""

import os
from pathlib import Path
from typing import Dict, Any
import yaml
from dotenv import load_dotenv

# Load environment variables
load_dotenv()


class Config:
    """Configuration manager with environment support."""
    
    def __init__(self, config_file: str = None):
        """Initialize configuration.
        
        Args:
            config_file: Path to YAML configuration file
        """
        self.environment = os.getenv("ETL_ENV", "dev")
        self.config_file = config_file or f"config/{self.environment}.yaml"
        self.config = self._load_config()
    
    def _load_config(self) -> Dict[str, Any]:
        """Load configuration from file."""
        config_path = Path(self.config_file)
        if not config_path.exists():
            raise FileNotFoundError(f"Configuration file not found: {config_path}")
        
        with open(config_path, 'r', encoding='utf-8') as f:
            config = yaml.safe_load(f)
        
        # Override with environment variables
        self._apply_env_overrides(config)
        return config
    
    def _apply_env_overrides(self, config: Dict[str, Any]) -> None:
        """Apply environment variable overrides."""
        # Override data paths if set in environment
        if os.getenv("ETL_RAW_DIR"):
            config["data"]["raw_dir"] = os.getenv("ETL_RAW_DIR")
        if os.getenv("ETL_TXT_DIR"):
            config["data"]["txt_dir"] = os.getenv("ETL_TXT_DIR")
    
    def get(self, key: str, default: Any = None) -> Any:
        """Get configuration value by dot-separated key.
        
        Args:
            key: Dot-separated key (e.g., 'data.raw_dir')
            default: Default value if key not found
            
        Returns:
            Configuration value
        """
        keys = key.split('.')
        value = self.config
        
        try:
            for k in keys:
                value = value[k]
            return value
        except (KeyError, TypeError):
            return default
    
    @property
    def raw_dir(self) -> Path:
        """Get raw data directory path."""
        return Path(self.get("data.raw_dir")).resolve()
    
    @property
    def txt_dir(self) -> Path:
        """Get processed text directory path."""
        return Path(self.get("data.txt_dir")).resolve()
    
    @property
    def pdf_extractor(self) -> str:
        """Get PDF extractor method."""
        return self.get("pdf.extractor", "pymupdf")
    
    @property
    def min_text_length(self) -> int:
        """Get minimum text length for keeping content."""
        return self.get("pdf.min_text_length", 50)
    
    @property
    def max_line_length(self) -> int:
        """Get maximum line length before splitting."""
        return self.get("pdf.max_line_length", 1000)
    
    @property
    def remove_empty_lines(self) -> bool:
        """Get whether to remove empty lines."""
        return self.get("text_cleaning.remove_empty_lines", True)
    
    @property
    def remove_whitespace_only(self) -> bool:
        """Get whether to remove whitespace-only lines."""
        return self.get("text_cleaning.remove_whitespace_only", True)
    
    @property
    def min_line_length(self) -> int:
        """Get minimum line length to keep."""
        return self.get("text_cleaning.min_line_length", 10)
    
    @property
    def preserve_paragraphs(self) -> bool:
        """Get whether to preserve paragraph breaks."""
        return self.get("text_cleaning.preserve_paragraphs", True)
    
    @property
    def normalize_whitespace(self) -> bool:
        """Get whether to normalize whitespace."""
        return self.get("text_cleaning.normalize_whitespace", True)
    
    @property
    def log_level(self) -> str:
        """Get logging level."""
        return self.get("logging.level", "INFO")
    
    def __str__(self) -> str:
        """String representation of configuration."""
        return f"Config(env={self.environment}, file={self.config_file})"