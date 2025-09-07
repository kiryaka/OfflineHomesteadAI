"""
Configuration module for ETL pipeline.

This module provides configuration management for the ETL pipeline,
supporting multiple environments and environment variable overrides.

Key Features:
- Environment-specific configuration files (dev, prod, test)
- Environment variable overrides
- Dot-notation key access
- Type-safe property accessors
- Automatic path resolution

Example:
    >>> from etl.config.settings import Config
    >>> 
    >>> # Load development configuration
    >>> config = Config()
    >>> 
    >>> # Access configuration values
    >>> raw_dir = config.raw_dir
    >>> pdf_extractor = config.pdf_extractor
"""
