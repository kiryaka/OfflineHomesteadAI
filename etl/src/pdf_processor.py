"""PDF processing utilities for text extraction and cleaning."""

import re
import logging
from pathlib import Path
from typing import List, Optional, Tuple
import fitz  # PyMuPDF
import pdfplumber
import PyPDF2

logger = logging.getLogger(__name__)


class PDFProcessor:
    """PDF text extraction and cleaning processor."""
    
    def __init__(self, config):
        """Initialize PDF processor with configuration.
        
        Args:
            config: Configuration object
        """
        self.config = config
        self.extractor = config.pdf_extractor
        
    def extract_text(self, pdf_path: Path) -> str:
        """Extract text from PDF using configured method.
        
        Args:
            pdf_path: Path to PDF file
            
        Returns:
            Extracted text content
        """
        try:
            if self.extractor == "pymupdf":
                return self._extract_with_pymupdf(pdf_path)
            elif self.extractor == "pdfplumber":
                return self._extract_with_pdfplumber(pdf_path)
            elif self.extractor == "pypdf2":
                return self._extract_with_pypdf2(pdf_path)
            else:
                raise ValueError(f"Unknown PDF extractor: {self.extractor}")
        except Exception as e:
            logger.error(f"Failed to extract text from {pdf_path}: {e}")
            return ""
    
    def _extract_with_pymupdf(self, pdf_path: Path) -> str:
        """Extract text using PyMuPDF (fitz)."""
        text = ""
        with fitz.open(pdf_path) as doc:
            for page in doc:
                text += page.get_text()
        return text
    
    def _extract_with_pdfplumber(self, pdf_path: Path) -> str:
        """Extract text using pdfplumber."""
        text = ""
        with pdfplumber.open(pdf_path) as pdf:
            for page in pdf.pages:
                page_text = page.extract_text()
                if page_text:
                    text += page_text + "\n"
        return text
    
    def _extract_with_pypdf2(self, pdf_path: Path) -> str:
        """Extract text using PyPDF2."""
        text = ""
        with open(pdf_path, 'rb') as file:
            reader = PyPDF2.PdfReader(file)
            for page in reader.pages:
                text += page.extract_text() + "\n"
        return text
    
    def clean_text(self, text: str) -> str:
        """Clean and normalize extracted text.
        
        Args:
            text: Raw extracted text
            
        Returns:
            Cleaned text with preserved paragraph structure
        """
        if not text:
            return ""
        
        # First, clean up common PDF extraction artifacts
        text = self._clean_pdf_artifacts(text)
        
        # Split into lines for processing
        lines = text.split('\n')
        cleaned_lines = []
        
        for line in lines:
            # Skip empty lines if configured
            if self.config.remove_empty_lines and not line.strip():
                continue
            
            # Skip whitespace-only lines if configured
            if self.config.remove_whitespace_only and not line.strip():
                continue
            
            # Skip very short lines if configured
            if len(line.strip()) < self.config.min_line_length:
                continue
            
            # Skip lines that are mostly punctuation or special characters
            if self._is_mostly_punctuation(line):
                continue
            
            # Normalize whitespace if configured
            if self.config.normalize_whitespace:
                line = self._normalize_whitespace(line)
            
            # Skip lines that are too long (likely formatting artifacts)
            if len(line) > self.config.max_line_length:
                # Split long lines at natural break points
                split_lines = self._split_long_line(line)
                cleaned_lines.extend(split_lines)
            else:
                cleaned_lines.append(line)
        
        # Join lines and preserve paragraph structure
        if self.config.preserve_paragraphs:
            return self._preserve_paragraphs(cleaned_lines)
        else:
            return '\n'.join(cleaned_lines)
    
    def _clean_pdf_artifacts(self, text: str) -> str:
        """Clean common PDF extraction artifacts."""
        import re
        
        # Remove excessive dots/periods (common in PDFs)
        text = re.sub(r'\.{3,}', '...', text)
        
        # Remove excessive spaces
        text = re.sub(r' +', ' ', text)
        
        # Remove excessive newlines
        text = re.sub(r'\n+', '\n', text)
        
        # Remove lines that are mostly special characters
        lines = text.split('\n')
        cleaned_lines = []
        
        for line in lines:
            # Skip lines that are mostly dots, dashes, or other special chars
            if len(line.strip()) > 0:
                special_char_ratio = sum(1 for c in line if c in '.-_=+*#@$%^&()[]{}|\\/:;"\'<>?~`') / len(line)
                if special_char_ratio < 0.7:  # Keep lines that aren't mostly special chars
                    cleaned_lines.append(line)
        
        return '\n'.join(cleaned_lines)
    
    def _is_mostly_punctuation(self, line: str) -> bool:
        """Check if a line is mostly punctuation or special characters."""
        if not line.strip():
            return True
        
        # Count alphanumeric characters
        alnum_count = sum(1 for c in line if c.isalnum())
        total_count = len(line.strip())
        
        # If less than 30% alphanumeric, consider it mostly punctuation
        return alnum_count / total_count < 0.3
    
    def _normalize_whitespace(self, line: str) -> str:
        """Normalize whitespace in a line."""
        # Replace multiple spaces with single space
        line = re.sub(r' +', ' ', line)
        # Remove leading/trailing whitespace
        line = line.strip()
        return line
    
    def _split_long_line(self, line: str) -> List[str]:
        """Split very long lines at natural break points."""
        if len(line) <= self.config.max_line_length:
            return [line]
        
        # Try to split at sentence boundaries first
        sentences = re.split(r'(?<=[.!?])\s+', line)
        if len(sentences) > 1:
            result = []
            current_line = ""
            for sentence in sentences:
                if len(current_line + sentence) <= self.config.max_line_length:
                    current_line += sentence + " "
                else:
                    if current_line:
                        result.append(current_line.strip())
                    current_line = sentence + " "
            if current_line:
                result.append(current_line.strip())
            return result
        
        # If no sentence boundaries, split at word boundaries
        words = line.split()
        result = []
        current_line = ""
        for word in words:
            if len(current_line + " " + word) <= self.config.max_line_length:
                current_line += " " + word if current_line else word
            else:
                if current_line:
                    result.append(current_line)
                current_line = word
        if current_line:
            result.append(current_line)
        return result
    
    def _preserve_paragraphs(self, lines: List[str]) -> str:
        """Preserve paragraph structure by detecting paragraph breaks."""
        if not lines:
            return ""
        
        # First, join all lines and then split by common paragraph indicators
        full_text = ' '.join(line.strip() for line in lines if line.strip())
        
        # Split by double spaces, periods followed by capital letters, or other indicators
        # This is a more aggressive approach to create readable paragraphs
        paragraphs = []
        
        # Split by common paragraph break patterns
        import re
        
        # Split by double spaces (common in PDF extraction)
        if '  ' in full_text:
            parts = full_text.split('  ')
            for part in parts:
                part = part.strip()
                if part and len(part) > 20:  # Only keep substantial parts
                    paragraphs.append(part)
        else:
            # If no double spaces, try to split by sentence patterns
            # Look for sentence endings followed by capital letters
            sentences = re.split(r'(?<=[.!?])\s+(?=[A-Z])', full_text)
            
            current_paragraph = []
            for sentence in sentences:
                sentence = sentence.strip()
                if not sentence:
                    continue
                    
                current_paragraph.append(sentence)
                
                # If this sentence is short and the next might be a new paragraph
                if (len(sentence) < 100 and 
                    sentence.endswith(('.', '!', '?')) and
                    len(current_paragraph) > 1):
                    # This might be a paragraph break
                    paragraph_text = ' '.join(current_paragraph)
                    if len(paragraph_text) > 50:  # Only keep substantial paragraphs
                        paragraphs.append(paragraph_text)
                    current_paragraph = []
            
            # Add remaining sentences as a paragraph
            if current_paragraph:
                paragraph_text = ' '.join(current_paragraph)
                if len(paragraph_text) > 50:
                    paragraphs.append(paragraph_text)
        
        # If we still don't have good paragraphs, create them by length
        if len(paragraphs) < 2:
            # Split the text into chunks of reasonable length
            words = full_text.split()
            chunk_size = 150  # words per paragraph
            paragraphs = []
            
            for i in range(0, len(words), chunk_size):
                chunk = ' '.join(words[i:i + chunk_size])
                if len(chunk) > 50:
                    paragraphs.append(chunk)
        
        return '\n\n'.join(paragraphs)
    
    def process_pdf(self, pdf_path: Path) -> str:
        """Process a PDF file: extract and clean text.
        
        Args:
            pdf_path: Path to PDF file
            
        Returns:
            Cleaned text content
        """
        logger.info(f"Processing PDF: {pdf_path}")
        
        # Extract text
        raw_text = self.extract_text(pdf_path)
        if not raw_text:
            logger.warning(f"No text extracted from {pdf_path}")
            return ""
        
        # Check minimum text length
        if len(raw_text.strip()) < self.config.min_text_length:
            logger.warning(f"Text too short in {pdf_path}: {len(raw_text)} chars")
            return ""
        
        # Clean text
        cleaned_text = self.clean_text(raw_text)
        
        logger.info(f"Processed {pdf_path}: {len(raw_text)} -> {len(cleaned_text)} chars")
        return cleaned_text
