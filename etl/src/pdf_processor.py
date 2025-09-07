"""File processing utilities for text extraction and cleaning from all supported formats."""

import re
import logging
from pathlib import Path
from typing import List, Optional, Tuple, Dict, Any
import fitz  # PyMuPDF
import pdfplumber
import PyPDF2
from unstructured.partition.auto import partition
from unstructured.cleaners.core import clean, replace_unicode_quotes, clean_non_ascii_chars

logger = logging.getLogger(__name__)


def normalize_for_search(text: str, english_only: bool = True, lowercase: bool = True, 
                        aggressive_cleaning: bool = True) -> str:
    """
    Universal text normalization for all file formats using Unstructured.
    - Fixes EOL hyphenation, bullets, whitespace, stray punctuation.
    - Normalizes quotes/dashes; optionally strips non-ASCII noise for English corpora.
    - Preserves UUIDs, URLs, emails, numbers, ranges by *not* touching internal hyphens.
    - Can apply aggressive cleaning for corrupted text (e.g., from PDFs).
    """
    if not text:
        return ""
    
    # 1) Fix hyphenated words BEFORE other processing
    text = _fix_hyphenated_words(text)
    
    # 2) Normalize quotes; this eliminates the classic â\x80\x99 fiasco
    text = replace_unicode_quotes(text)
    
    # 3) Apply aggressive cleaning if requested (useful for corrupted PDFs)
    if aggressive_cleaning and english_only:
        text = clean_non_ascii_chars(text)

    # 4) Core cleaning in one shot.
    # - bullets=True strips list bullets at line starts only.
    # - extra_whitespace=True squeezes spaces and fixes random tabs/nbsp.
    # - trailing_punctuation=True trims stray periods/commas at line ends.
    # - dashes=True normalizes various dash types to standard hyphens.
    # - lowercase=... if you want case-insensitive BM25.
    text = clean(
        text,
        bullets=True,
        extra_whitespace=True,
        trailing_punctuation=True,
        dashes=True,  # Now using dashes=True for universal cleaning
        lowercase=lowercase,
    )
    
    # 5) Additional dash normalization (unstructured's dashes=True might not catch all)
    text = re.sub(r'[–—]', '-', text)  # Replace en-dash and em-dash with hyphen

    # 5) Collapse intra-paragraph single newlines to spaces; keep paragraph breaks.
    # Unstructured's clean doesn't change linebreak semantics aggressively, so do a light pass:
    text = re.sub(r"(?<!\n)\n(?!\n)", " ", text)   # single \n -> space
    text = re.sub(r"\n{3,}", "\n\n", text)         # large gaps -> double \n

    return text


def _fix_hyphenated_words(text: str) -> str:
    """
    Fix words that are split across lines with hyphens.
    Handles patterns like:
    - "Tra-\nditionally" -> "Traditionally"
    - "auto-\nmobile" -> "automobile"
    - "tele-\nphone" -> "telephone"
    """
    import re
    
    # Pattern 1: word-hyphen-newline-word (most common case)
    # This matches a word followed by hyphen, then newline, then another word
    pattern1 = r'(\w+)-\s*\n\s*(\w+)'
    
    def merge_hyphenated(match):
        return match.group(1) + match.group(2)
    
    text = re.sub(pattern1, merge_hyphenated, text)
    
    # Pattern 2: word-hyphen-space-word (less common but happens)
    # This matches a word followed by hyphen, then space, then another word
    pattern2 = r'(\w+)-\s+(\w+)'
    
    def merge_hyphenated_space(match):
        # Only merge if it looks like a hyphenated word (not a compound like "well-known")
        word1, word2 = match.group(1), match.group(2)
        # If both parts are reasonably long and the second starts with lowercase, merge
        if len(word1) > 2 and len(word2) > 2 and word2[0].islower():
            return word1 + word2
        else:
            return match.group(0)  # Keep original if it looks like a compound
    
    text = re.sub(pattern2, merge_hyphenated_space, text)
    
    return text


class FileProcessor:
    """File text extraction and cleaning processor for all supported formats."""
    
    def __init__(self, config):
        """Initialize file processor with configuration.
        
        Args:
            config: Configuration object
        """
        self.config = config
        self.pdf_extractor = config.pdf_extractor
        
        # Supported file extensions
        self.supported_extensions = self.config.get("extraction.supported_formats", 
                                                   ["pdf", "docx", "html", "md", "txt", "jpg", "jpeg", "png", "tiff"])
        
        # Map extensions to processing methods
        self.pdf_extensions = {'.pdf'}
        self.unstructured_extensions = {'.docx', '.doc', '.html', '.htm', '.md', '.txt', '.rtf', '.epub', '.msg', '.eml'}
        self.image_extensions = {'.jpg', '.jpeg', '.png', '.tiff', '.tif', '.bmp', '.gif'}
    
    def is_supported(self, file_path: Path) -> bool:
        """Check if file type is supported."""
        ext = file_path.suffix.lower().lstrip('.')
        return ext in self.supported_extensions
        
    def extract_text(self, file_path: Path) -> str:
        """Extract text from any supported file type.
        
        Args:
            file_path: Path to file
            
        Returns:
            Extracted text content
        """
        try:
            ext = file_path.suffix.lower()
            
            if ext in self.pdf_extensions:
                return self._extract_pdf(file_path)
            elif ext in self.unstructured_extensions:
                return self._extract_with_unstructured(file_path)
            elif ext in self.image_extensions:
                return self._extract_image_with_unstructured(file_path)
            else:
                logger.warning(f"Unsupported file type: {ext}")
                return ""
                
        except Exception as e:
            logger.error(f"Failed to extract text from {file_path}: {e}")
            return ""
    
    def _extract_pdf(self, pdf_path: Path) -> str:
        """Extract text from PDF using configured method."""
        try:
            if self.pdf_extractor == "unstructured":
                return self._extract_with_unstructured(pdf_path)
            elif self.pdf_extractor == "pymupdf":
                return self._extract_with_pymupdf(pdf_path)
            elif self.pdf_extractor == "pdfplumber":
                return self._extract_with_pdfplumber(pdf_path)
            elif self.pdf_extractor == "pypdf2":
                return self._extract_with_pypdf2(pdf_path)
            else:
                raise ValueError(f"Unknown PDF extractor: {self.pdf_extractor}")
        except Exception as e:
            logger.error(f"Failed to extract PDF {pdf_path}: {e}")
            return ""
    
    def _extract_with_unstructured(self, file_path: Path) -> str:
        """Extract text using unstructured for non-PDF files."""
        try:
            logger.info(f"Extracting with unstructured: {file_path}")
            
            # Handle EML files with email partitioner
            if file_path.suffix.lower() == '.eml':
                from unstructured.partition.email import partition_email
                elements = partition_email(filename=str(file_path))
            else:
                # Use unstructured's auto partition for other formats
                elements = partition(
                    filename=str(file_path),
                    strategy="auto",
                    include_page_breaks=True,
                )
            
            # Extract text from elements
            text_parts = []
            for element in elements:
                if hasattr(element, 'text') and element.text:
                    text_parts.append(element.text)
            
            return "\n\n".join(text_parts)
            
        except Exception as e:
            logger.error(f"Failed to extract with unstructured {file_path}: {e}")
            return ""
    
    def _extract_image_with_unstructured(self, file_path: Path) -> str:
        """Extract text from images using unstructured OCR."""
        try:
            logger.info(f"Extracting image with unstructured OCR: {file_path}")
            
            # Use unstructured for OCR
            elements = partition(
                filename=str(file_path),
                strategy="ocr_only",  # Force OCR for images
                include_page_breaks=True,
            )
            
            # Extract text from elements
            text_parts = []
            for element in elements:
                if hasattr(element, 'text') and element.text:
                    text_parts.append(element.text)
            
            return "\n\n".join(text_parts)
            
        except Exception as e:
            logger.error(f"Failed to extract image {file_path}: {e}")
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
    
    def clean_text(self, text: str, file_format: str = None) -> str:
        """Clean and normalize extracted text using universal approach.
        
        Args:
            text: Raw extracted text
            file_format: Optional file format for conditional special treatment
            
        Returns:
            Cleaned text with preserved paragraph structure
        """
        if not text:
            return ""
        
        # Determine if we need aggressive cleaning based on file format
        aggressive_cleaning = self._needs_aggressive_cleaning(file_format, text)
        
        # Use universal cleaning approach
        text = normalize_for_search(
            text, 
            english_only=self.config.get("text_cleaning.english_only", True),
            lowercase=self.config.get("text_cleaning.lowercase", False),
            aggressive_cleaning=aggressive_cleaning
        )
        
        # Apply format-specific cleaning if needed
        text = self._apply_format_specific_cleaning(text, file_format)
        
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
        
        # Post-process to reduce excessive empty lines
        if cleaned_lines:
            # Remove consecutive empty lines, keeping at most one
            final_lines = []
            prev_empty = False
            for line in cleaned_lines:
                is_empty = not line.strip()
                if is_empty and prev_empty:
                    continue  # Skip consecutive empty lines
                final_lines.append(line)
                prev_empty = is_empty
            cleaned_lines = final_lines
        
        # Join lines and preserve paragraph structure
        if self.config.preserve_paragraphs:
            return self._preserve_paragraphs(cleaned_lines)
        else:
            return '\n'.join(cleaned_lines)
    
    def _needs_aggressive_cleaning(self, file_format: str, text: str) -> bool:
        """Determine if text needs aggressive cleaning based on format and content."""
        # Always use aggressive cleaning for PDFs (they often have unicode corruption)
        if file_format == 'pdf':
            return True
        
        # Check for unicode corruption patterns in any format
        unicode_corruption_patterns = [
            'â\x80\x99',  # Common unicode corruption
            'â\x80\x9c',  # Left double quote corruption
            'â\x80\x9d',  # Right double quote corruption
            'â\x80\x93',  # En dash corruption
            'â\x80\x94',  # Em dash corruption
        ]
        
        for pattern in unicode_corruption_patterns:
            if pattern in text:
                return True
        
        # Check for high ratio of non-ASCII characters (potential corruption)
        if len(text) > 100:  # Only check if text is substantial
            non_ascii_count = sum(1 for c in text if ord(c) > 127)
            if non_ascii_count / len(text) > 0.1:  # More than 10% non-ASCII
                return True
        
        return False
    
    def _apply_format_specific_cleaning(self, text: str, file_format: str) -> str:
        """Apply format-specific cleaning rules."""
        if file_format == 'pdf':
            return self._clean_pdf_artifacts(text)
        elif file_format in ['html', 'htm']:
            return self._clean_html_artifacts(text)
        elif file_format in ['docx', 'doc']:
            return self._clean_docx_artifacts(text)
        else:
            return text
    
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
    
    def _clean_html_artifacts(self, text: str) -> str:
        """Clean common HTML extraction artifacts."""
        import re
        
        # Remove HTML entities that might have been missed
        text = re.sub(r'&[a-zA-Z0-9#]+;', '', text)
        
        # Remove excessive whitespace
        text = re.sub(r'\s+', ' ', text)
        
        return text
    
    def _clean_docx_artifacts(self, text: str) -> str:
        """Clean common DOCX extraction artifacts."""
        import re
        
        # Remove excessive whitespace
        text = re.sub(r'\s+', ' ', text)
        
        # Remove excessive newlines
        text = re.sub(r'\n+', '\n', text)
        
        return text
    
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
    
    def process_file(self, file_path: Path) -> str:
        """Process any supported file: extract and clean text.
        
        Args:
            file_path: Path to file
            
        Returns:
            Cleaned text content
        """
        logger.info(f"Processing file: {file_path}")
        
        # Extract text
        raw_text = self.extract_text(file_path)
        if not raw_text:
            logger.warning(f"No text extracted from {file_path}")
            return ""
        
        # Check minimum text length
        if len(raw_text.strip()) < self.config.min_text_length:
            logger.warning(f"Text too short in {file_path}: {len(raw_text)} chars")
            return ""
        
        # Get file format for conditional cleaning
        file_format = file_path.suffix.lower().lstrip('.')
        
        # Clean text with format-aware cleaning
        cleaned_text = self.clean_text(raw_text, file_format)
        
        logger.info(f"Processed {file_path}: {len(raw_text)} -> {len(cleaned_text)} chars")
        return cleaned_text
    
    def process_pdf(self, pdf_path: Path) -> str:
        """Legacy method for backward compatibility."""
        return self.process_file(pdf_path)
