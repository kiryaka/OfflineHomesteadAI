"""
Text chunking using langchain library.

This module provides text chunking functionality using langchain's RecursiveCharacterTextSplitter,
which intelligently splits text into smaller chunks while preserving semantic boundaries.
It's designed for vector search applications where text needs to be broken into manageable pieces.

Key Features:
- Intelligent text chunking with semantic boundary preservation
- Configurable chunk size and overlap
- Customizable separators for different text types
- Metadata preservation and chunk tracking
- Error handling and logging

Chunking Strategy:
- Recursive splitting using multiple separators
- Preserves paragraph and sentence boundaries
- Configurable overlap between chunks
- Metadata tracking for chunk relationships

Example:
    >>> from etl.src.processors.chunker import TextChunker
    >>> 
    >>> # Initialize chunker
    >>> chunker = TextChunker(chunk_size=1000, chunk_overlap=200)
    >>> 
    >>> # Chunk a document
    >>> content = "This is a long document with multiple paragraphs..."
    >>> metadata = {"file_name": "document.txt", "source": "test"}
    >>> chunks = chunker.chunk_document(content, metadata)
    >>> print(f"Created {len(chunks)} chunks")
"""

import logging
from typing import List, Dict, Any
from langchain.text_splitter import RecursiveCharacterTextSplitter
from langchain.schema import Document

logger = logging.getLogger(__name__)


class TextChunker:
    """Chunk text documents for vector search."""
    
    def __init__(
        self,
        chunk_size: int = 1000,
        chunk_overlap: int = 200,
        separators: List[str] = None,
    ):
        """Initialize text chunker.
        
        Args:
            chunk_size: Maximum size of each chunk
            chunk_overlap: Overlap between chunks
            separators: List of separators to split on
        """
        self.chunk_size = chunk_size
        self.chunk_overlap = chunk_overlap
        self.separators = separators or ["\n\n", "\n", " ", ""]
        
        self.splitter = RecursiveCharacterTextSplitter(
            chunk_size=chunk_size,
            chunk_overlap=chunk_overlap,
            separators=separators,
        )
    
    def chunk_document(self, content: str, metadata: Dict[str, Any]) -> List[Dict[str, Any]]:
        """
        Chunk a document into smaller pieces for vector search.
        
        This method takes a document's text content and metadata, then splits it
        into smaller, manageable chunks using the configured chunking strategy.
        Each chunk preserves the original metadata and adds chunk-specific information.
        
        Args:
            content: Text content to chunk into smaller pieces
            metadata: Document metadata to preserve in each chunk
            
        Returns:
            List of chunk dictionaries, each containing:
                - chunk_id: Unique identifier for the chunk
                - content: Text content of the chunk
                - metadata: Dictionary with original metadata plus:
                    - chunk_index: Index of this chunk (0-based)
                    - total_chunks: Total number of chunks created
                    
        Note:
            The chunking process uses the configured separators to split text
            at semantic boundaries (paragraphs, sentences, words) to preserve
            meaning and context in each chunk.
        """
        try:
            logger.info(f"Chunking document: {metadata.get('file_name', 'unknown')}")
            
            # Create langchain document object for processing
            doc = Document(page_content=content, metadata=metadata)
            
            # Use the configured splitter to create chunks
            # This preserves semantic boundaries and applies overlap
            chunks = self.splitter.split_documents([doc])
            
            # Convert langchain chunks to our internal format
            chunk_list = []
            for i, chunk in enumerate(chunks):
                chunk_dict = {
                    "chunk_id": f"{metadata.get('file_name', 'unknown')}_{i}",
                    "content": chunk.page_content,
                    "metadata": {
                        **chunk.metadata,  # Preserve original metadata
                        "chunk_index": i,  # Add chunk-specific information
                        "total_chunks": len(chunks),
                    }
                }
                chunk_list.append(chunk_dict)
            
            logger.info(f"Created {len(chunk_list)} chunks")
            return chunk_list
            
        except Exception as e:
            logger.error(f"Failed to chunk document: {e}")
            return []
