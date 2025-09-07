"""
Configuration management for ETL pipeline.

This module provides a centralized configuration system that supports multiple environments
(dev, prod, test) and allows for environment variable overrides. It loads YAML configuration
files and provides easy access to configuration values through dot-notation keys.

Key Features:
- Environment-specific configuration files (dev.yaml, prod.yaml, test.yaml)
- Environment variable overrides for sensitive or dynamic values
- Dot-notation key access (e.g., 'data.raw_dir')
- Type-safe property accessors for common configuration values
- Automatic path resolution for directory configurations

Configuration Structure:
    data:
        raw_dir: "path/to/raw/files"
        txt_dir: "path/to/processed/text"
    pdf:
        extractor: "pymupdf"  # or "pdfplumber", "pypdf2", "unstructured"
        min_text_length: 50
        max_line_length: 1000
    text_cleaning:
        remove_empty_lines: true
        remove_whitespace_only: true
        min_line_length: 10
        preserve_paragraphs: true
        normalize_whitespace: true
    logging:
        level: "INFO"

Environment Variables:
    ETL_ENV: Environment name (dev, prod, test)
    ETL_RAW_DIR: Override raw directory path
    ETL_TXT_DIR: Override text directory path

Example:
    >>> from etl.config.settings import Config
    >>> 
    >>> # Load development configuration
    >>> config = Config()
    >>> 
    >>> # Access configuration values
    >>> raw_dir = config.raw_dir
    >>> pdf_extractor = config.pdf_extractor
    >>> 
    >>> # Use dot notation for nested values
    >>> min_length = config.get('pdf.min_text_length', 50)
"""

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
        """
        Get configuration value by dot-separated key.
        
        This method allows accessing nested configuration values using dot notation.
        For example, 'data.raw_dir' will access config['data']['raw_dir'].
        
        Args:
            key: Dot-separated key path (e.g., 'data.raw_dir', 'pdf.extractor')
            default: Default value to return if key is not found
            
        Returns:
            Configuration value or default if key not found
            
        Example:
            >>> config.get('data.raw_dir', '/default/path')
            '/path/to/raw/files'
            >>> config.get('nonexistent.key', 'default')
            'default'
        """
        keys = key.split('.')
        value = self.config
        
        try:
            # Navigate through nested dictionary structure
            for k in keys:
                value = value[k]
            return value
        except (KeyError, TypeError):
            # Return default if any key in the path doesn't exist
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