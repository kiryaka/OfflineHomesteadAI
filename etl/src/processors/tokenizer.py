"""Text tokenization using tiktoken."""

import logging
from typing import List, Dict, Any
import tiktoken

logger = logging.getLogger(__name__)


class Tokenizer:
    """Tokenize text using tiktoken."""
    
    def __init__(self, model_name: str = "text-embedding-3-small"):
        """Initialize tokenizer.
        
        Args:
            model_name: OpenAI model name for tokenization
        """
        self.model_name = model_name
        try:
            self.encoding = tiktoken.encoding_for_model(model_name)
        except KeyError:
            logger.warning(f"Model {model_name} not found, using cl100k_base encoding")
            self.encoding = tiktoken.get_encoding("cl100k_base")
    
    def tokenize(self, text: str) -> List[int]:
        """Tokenize text into token IDs.
        
        Args:
            text: Text to tokenize
            
        Returns:
            List of token IDs
        """
        return self.encoding.encode(text)
    
    def count_tokens(self, text: str) -> int:
        """Count tokens in text.
        
        Args:
            text: Text to count tokens for
            
        Returns:
            Number of tokens
        """
        return len(self.tokenize(text))
    
    def add_token_counts(self, chunks: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """Add token counts to chunks.
        
        Args:
            chunks: List of chunk dictionaries
            
        Returns:
            Chunks with added token counts
        """
        for chunk in chunks:
            content = chunk.get("content", "")
            token_count = self.count_tokens(content)
            chunk["metadata"]["token_count"] = token_count
        
        return chunks
