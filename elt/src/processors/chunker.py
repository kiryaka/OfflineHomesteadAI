"""Text chunking using langchain."""

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
        """Chunk a document into smaller pieces.
        
        Args:
            content: Text content to chunk
            metadata: Document metadata
            
        Returns:
            List of chunk dictionaries
        """
        try:
            logger.info(f"Chunking document: {metadata.get('file_name', 'unknown')}")
            
            # Create langchain document
            doc = Document(page_content=content, metadata=metadata)
            
            # Split into chunks
            chunks = self.splitter.split_documents([doc])
            
            # Convert to our format
            chunk_list = []
            for i, chunk in enumerate(chunks):
                chunk_dict = {
                    "chunk_id": f"{metadata.get('file_name', 'unknown')}_{i}",
                    "content": chunk.page_content,
                    "metadata": {
                        **chunk.metadata,
                        "chunk_index": i,
                        "total_chunks": len(chunks),
                    }
                }
                chunk_list.append(chunk_dict)
            
            logger.info(f"Created {len(chunk_list)} chunks")
            return chunk_list
            
        except Exception as e:
            logger.error(f"Failed to chunk document: {e}")
            return []
