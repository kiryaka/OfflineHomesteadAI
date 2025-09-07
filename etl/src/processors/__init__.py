"""
Text processors for cleaning, chunking, and tokenizing.

This module provides text processing components for the ETL pipeline,
including text chunking, tokenization, and embedding generation.

Available Processors:
- TextChunker: Split text into manageable chunks for vector search
- Tokenizer: Tokenize text using tiktoken for accurate token counting
- Embedder: Generate vector embeddings using sentence-transformers

Example:
    >>> from etl.src.processors import TextChunker, Tokenizer, Embedder
    >>> 
    >>> # Initialize processors
    >>> chunker = TextChunker(chunk_size=1000, chunk_overlap=200)
    >>> tokenizer = Tokenizer(model_name="text-embedding-3-small")
    >>> embedder = Embedder(model_name="all-MiniLM-L6-v2")
    >>> 
    >>> # Process text
    >>> chunks = chunker.chunk_document(content, metadata)
    >>> chunks = tokenizer.add_token_counts(chunks)
    >>> chunks = embedder.embed_chunks(chunks)
"""

from .chunker import TextChunker
from .tokenizer import Tokenizer
from .embedder import Embedder

__all__ = ["TextChunker", "Tokenizer", "Embedder"]
