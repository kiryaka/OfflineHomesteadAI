"""
Text tokenization using tiktoken library.

This module provides text tokenization functionality using OpenAI's tiktoken library,
which is the same tokenizer used by GPT models. It supports various OpenAI models
and provides accurate token counting for text processing.

Key Features:
- Text tokenization using tiktoken
- Support for various OpenAI models
- Accurate token counting
- Batch processing for multiple text chunks
- Metadata tracking for token counts

Supported Models:
- text-embedding-3-small: OpenAI's small embedding model
- gpt-3.5-turbo: GPT-3.5 model
- gpt-4: GPT-4 model
- Custom models with cl100k_base encoding fallback

Example:
    >>> from etl.src.processors.tokenizer import Tokenizer
    >>> 
    >>> # Initialize tokenizer
    >>> tokenizer = Tokenizer(model_name="text-embedding-3-small")
    >>> 
    >>> # Tokenize text
    >>> tokens = tokenizer.tokenize("This is a sample text.")
    >>> print(f"Tokens: {tokens}")
    >>> 
    >>> # Count tokens
    >>> count = tokenizer.count_tokens("This is a sample text.")
    >>> print(f"Token count: {count}")
    >>> 
    >>> # Add token counts to chunks
    >>> chunks = [{"content": "Text 1", "metadata": {}}, {"content": "Text 2", "metadata": {}}]
    >>> chunks_with_tokens = tokenizer.add_token_counts(chunks)
"""

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
        """
        Tokenize text into token IDs.
        
        This method converts a text string into a list of token IDs using the
        tiktoken encoding. The token IDs correspond to the vocabulary used by
        the specified OpenAI model.
        
        Args:
            text: Text string to tokenize
            
        Returns:
            List of integers representing token IDs
            
        Note:
            The tokenization is deterministic and consistent with OpenAI's models.
            Different models may use different tokenizers, but this method
            uses the appropriate tokenizer for the specified model.
        """
        return self.encoding.encode(text)
    
    def count_tokens(self, text: str) -> int:
        """
        Count tokens in text.
        
        This method provides an accurate count of tokens in a text string,
        which is essential for managing text length limits in language models
        and calculating processing costs.
        
        Args:
            text: Text string to count tokens for
            
        Returns:
            Integer representing the number of tokens
            
        Note:
            Token count is more accurate than character count for determining
            text length in language models, as tokens can represent subwords,
            words, or punctuation marks depending on the text content.
        """
        return len(self.tokenize(text))
    
    def add_token_counts(self, chunks: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
        """
        Add token counts to text chunks.
        
        This method processes a list of text chunks and adds token count information
        to each chunk's metadata. This is useful for tracking text length and
        managing processing limits in downstream applications.
        
        Args:
            chunks: List of chunk dictionaries, each containing:
                - content: Text content to count tokens for
                - metadata: Dictionary for storing metadata
                
        Returns:
            List of chunk dictionaries with added token count information:
                - metadata.token_count: Number of tokens in the content
                
        Note:
            This method modifies the chunks in-place and also returns them
            for convenience. The token count is added to the metadata dictionary
            of each chunk.
        """
        for chunk in chunks:
            content = chunk.get("content", "")
            token_count = self.count_tokens(content)
            chunk["metadata"]["token_count"] = token_count
        
        return chunks
