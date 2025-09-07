"""
Generate embeddings using sentence-transformers library.

This module provides text embedding generation using pre-trained transformer models
from the sentence-transformers library. It supports various embedding models and
can process both individual texts and batches of text chunks.

Key Features:
- Text embedding generation using sentence-transformers
- Support for various pre-trained models
- Batch processing for multiple text chunks
- Metadata tracking for embeddings
- Error handling and logging

Supported Models:
- text-embedding-3-small: OpenAI's small embedding model
- all-MiniLM-L6-v2: Lightweight general-purpose model
- all-mpnet-base-v2: High-quality general-purpose model
- Custom models from Hugging Face Hub

Example:
    >>> from etl.src.processors.embedder import Embedder
    >>> 
    >>> # Initialize embedder
    >>> embedder = Embedder(model_name="all-MiniLM-L6-v2")
    >>> 
    >>> # Generate embedding for single text
    >>> embedding = embedder.embed_text("This is a sample text.")
    >>> print(f"Embedding dimension: {len(embedding)}")
    >>> 
    >>> # Generate embeddings for multiple chunks
    >>> chunks = [{"content": "Text 1", "metadata": {}}, {"content": "Text 2", "metadata": {}}]
    >>> embedded_chunks = embedder.embed_chunks(chunks)
"""

import logging
from typing import List, Dict, Any
import numpy as np
from sentence_transformers import SentenceTransformer

logger = logging.getLogger(__name__)


class Embedder:
    """Generate vector embeddings for text chunks."""
    
    def __init__(self, model_name: str = "text-embedding-3-small"):
        """Initialize embedder.
        
        Args:
            model_name: Model name for embeddings
        """
        self.model_name = model_name
        try:
            self.model = SentenceTransformer(model_name)
            logger.info(f"Loaded embedding model: {model_name}")
        except Exception as e:
            logger.error(f"Failed to load model {model_name}: {e}")
            raise
    
    def embed_text(self, text: str) -> List[float]:
        """
        Generate embedding for a single text.
        
        This method converts a text string into a high-dimensional vector representation
        using the loaded sentence-transformer model. The embedding captures semantic
        meaning and can be used for similarity search and clustering.
        
        Args:
            text: Text string to convert to embedding
            
        Returns:
            List of floats representing the embedding vector, or empty list on error
            
        Note:
            The embedding dimension depends on the model used. Common dimensions:
            - all-MiniLM-L6-v2: 384 dimensions
            - all-mpnet-base-v2: 768 dimensions
            - text-embedding-3-small: 1536 dimensions
        """
        try:
            # Use the sentence-transformer model to encode text
            # convert_to_tensor=False returns numpy array instead of PyTorch tensor
            embedding = self.model.encode(text, convert_to_tensor=False)
            return embedding.tolist()
        except Exception as e:
            logger.error(f"Failed to embed text: {e}")
            return []
    
    def embed_chunks(self, chunks: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """
        Generate embeddings for multiple text chunks.
        
        This method processes a list of text chunks and adds embedding vectors
        to each chunk. It also updates the metadata with embedding information
        for tracking and debugging purposes.
        
        Args:
            chunks: List of chunk dictionaries, each containing:
                - content: Text content to embed
                - metadata: Dictionary for storing metadata
                
        Returns:
            List of chunk dictionaries with added embedding information:
                - embedding: List of floats representing the embedding vector
                - metadata.embedding_model: Name of the model used
                - metadata.embedding_dimension: Dimension of the embedding vector
                
        Note:
            Chunks with empty content will have empty embedding lists and
            will be logged as warnings.
        """
        logger.info(f"Generating embeddings for {len(chunks)} chunks")
        
        for i, chunk in enumerate(chunks):
            content = chunk.get("content", "")
            if content:
                # Generate embedding for the text content
                embedding = self.embed_text(content)
                chunk["embedding"] = embedding
                
                # Add embedding metadata for tracking
                chunk["metadata"]["embedding_model"] = self.model_name
                chunk["metadata"]["embedding_dimension"] = len(embedding)
            else:
                # Handle empty content gracefully
                logger.warning(f"Empty content for chunk {i}")
                chunk["embedding"] = []
        
        return chunks
