"""Generate embeddings using sentence-transformers."""

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
        """Generate embedding for a single text.
        
        Args:
            text: Text to embed
            
        Returns:
            Embedding vector as list of floats
        """
        try:
            embedding = self.model.encode(text, convert_to_tensor=False)
            return embedding.tolist()
        except Exception as e:
            logger.error(f"Failed to embed text: {e}")
            return []
    
    def embed_chunks(self, chunks: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """Generate embeddings for multiple chunks.
        
        Args:
            chunks: List of chunk dictionaries
            
        Returns:
            Chunks with added embeddings
        """
        logger.info(f"Generating embeddings for {len(chunks)} chunks")
        
        for i, chunk in enumerate(chunks):
            content = chunk.get("content", "")
            if content:
                embedding = self.embed_text(content)
                chunk["embedding"] = embedding
                chunk["metadata"]["embedding_model"] = self.model_name
                chunk["metadata"]["embedding_dimension"] = len(embedding)
            else:
                logger.warning(f"Empty content for chunk {i}")
                chunk["embedding"] = []
        
        return chunks
