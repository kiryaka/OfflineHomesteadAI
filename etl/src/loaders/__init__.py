"""
Data loaders for exporting to search systems.

This module provides loaders for exporting processed data to different
search systems, enabling both full-text and vector search capabilities.

Available Loaders:
- TantivyLoader: Export to Tantivy for full-text search with BM25 scoring
- LanceDBLoader: Export to LanceDB for vector search and similarity matching

Example:
    >>> from etl.src.loaders import TantivyLoader, LanceDBLoader
    >>> 
    >>> # Initialize loaders
    >>> tantivy_loader = TantivyLoader(Path("output/tantivy"))
    >>> lancedb_loader = LanceDBLoader(Path("output/lancedb"))
    >>> 
    >>> # Export chunks to both systems
    >>> tantivy_loader.export_chunks(chunks)
    >>> lancedb_loader.export_chunks(chunks)
"""

from .tantivy_loader import TantivyLoader
from .lancedb_loader import LanceDBLoader

__all__ = ["TantivyLoader", "LanceDBLoader"]
