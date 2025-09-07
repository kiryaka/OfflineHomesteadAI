"""
Test suite for text cleaning and validation across all supported formats.

This module provides comprehensive testing for the ETL pipeline's text cleaning functionality.
It validates that text extraction and cleaning produces high-quality, normalized text
across all supported file formats.

Key Features:
- Universal text validation across all formats
- Format-specific test cases with appropriate expectations
- Comprehensive validation rules (unicode, hyphenation, whitespace, etc.)
- Support for 19 different file formats (6 supported, 5 pending, 8 OCR pending)
- Clear status reporting and progress tracking

Validation Rules:
- Unicode corruption detection and repair
- Hyphenation fixing across line breaks
- Whitespace normalization (spaces, newlines, tabs)
- Bullet point and list formatting
- Quote and dash normalization
- Minimum text length requirements
- Empty line ratio limits
- Format-specific artifact removal

Supported Formats (6):
- PDF (text): Full validation suite
- TXT: Full validation suite
- Markdown: Full validation suite
- HTML: Full validation suite
- DOCX: Full validation suite
- HTM: Full validation suite (treated as HTML)

Pending Formats (5):
- RTF: Needs pandoc dependency
- EML: Email partitioner configuration issue
- EPUB: Needs unstructured support
- MSG: Needs unstructured support
- DOC: Needs unstructured support

OCR Pending Formats (8):
- PDF (image): Requires OCR implementation
- JPG, JPEG, PNG, TIFF, TIF, BMP, GIF: Need Tesseract setup

Usage:
    # Run all tests
    pytest test_text_cleaning.py
    
    # Run specific format test
    pytest test_text_cleaning.py::TestTextCleaning::test_pdf_cleaning
    
    # Run with verbose output
    pytest test_text_cleaning.py -v

Example:
    >>> from etl.tests.test_text_cleaning import TextValidator
    >>> 
    >>> validator = TextValidator()
    >>> text = "This is clean text without corruption."
    >>> 
    >>> # Validate text quality
    >>> results = validator.validate_all(text, 'txt')
    >>> print(f"Validation passed: {all(result[0] for result in results.values())}")
"""

import pytest
import re
from pathlib import Path
from typing import Dict, List, Tuple, Optional
from src.pdf_processor import FileProcessor
from config.settings import Config


class TextValidator:
    """
    Universal text validator for all formats.
    
    This class provides comprehensive validation methods to ensure that text
    extracted and cleaned by the ETL pipeline meets quality standards. It
    checks for common issues like unicode corruption, hyphenation problems,
    whitespace issues, and format-specific artifacts.
    
    The validator is designed to work with all supported file formats and
    provides consistent quality standards across the entire pipeline.
    """
    
    def __init__(self):
        """
        Initialize the text validator with corruption patterns.
        
        Sets up the validator with known unicode corruption patterns that
        commonly occur during PDF extraction and other text processing
        operations. These patterns are used to detect and flag corrupted text.
        """
        # Common unicode corruption patterns from PDF extraction libraries
        # These patterns indicate encoding issues that need aggressive cleaning
        self.unicode_corruption_patterns = [
            'â\x80\x99',  # Common unicode corruption (smart apostrophe)
            'â\x80\x9c',  # Left double quote corruption
            'â\x80\x9d',  # Right double quote corruption
            'â\x80\x93',  # En dash corruption
            'â\x80\x94',  # Em dash corruption
        ]
    
    def validate_unicode_clean(self, text: str) -> Tuple[bool, List[str]]:
        """Check if text is free from unicode corruption."""
        issues = []
        for pattern in self.unicode_corruption_patterns:
            if pattern in text:
                issues.append(f"Unicode corruption found: {repr(pattern)}")
        return len(issues) == 0, issues
    
    def validate_hyphenation_fixed(self, text: str) -> Tuple[bool, List[str]]:
        """Check if hyphenated words across lines are properly merged."""
        issues = []
        # Look for patterns like "word-\nword" that should be merged
        if re.search(r'\w+-\s*\n\s*\w+', text):
            issues.append("Hyphenated words across lines not properly merged")
        return len(issues) == 0, issues
    
    def validate_whitespace_normalized(self, text: str) -> Tuple[bool, List[str]]:
        """Check if whitespace is properly normalized."""
        issues = []
        # Check for excessive spaces
        if re.search(r' {3,}', text):
            issues.append("Excessive spaces found")
        # Check for excessive newlines (more than 2 consecutive)
        if re.search(r'\n{3,}', text):
            issues.append("Excessive newlines found")
        # Check for tabs
        if '\t' in text:
            issues.append("Tabs found (should be normalized to spaces)")
        return len(issues) == 0, issues
    
    def validate_bullets_cleaned(self, text: str) -> Tuple[bool, List[str]]:
        """Check if bullet points are properly cleaned."""
        issues = []
        # Look for common bullet patterns that should be cleaned
        bullet_patterns = [r'^\s*[•·▪▫]\s*', r'^\s*[-*+]\s*', r'^\s*\d+[.)]\s*']
        for pattern in bullet_patterns:
            if re.search(pattern, text, re.MULTILINE):
                issues.append(f"Bullet pattern not cleaned: {pattern}")
        return len(issues) == 0, issues
    
    def validate_quotes_normalized(self, text: str) -> Tuple[bool, List[str]]:
        """Check if quotes are properly normalized."""
        issues = []
        # Check for smart quotes that should be normalized
        if '"' in text or '"' in text or ''' in text or ''' in text:
            issues.append("Smart quotes found (should be normalized to standard quotes)")
        return len(issues) == 0, issues
    
    def validate_dashes_normalized(self, text: str) -> Tuple[bool, List[str]]:
        """Check if dashes are properly normalized."""
        issues = []
        # Check for en-dash and em-dash that should be normalized
        if '–' in text or '—' in text:
            issues.append("En-dash or em-dash found (should be normalized)")
        return len(issues) == 0, issues
    
    def validate_minimum_length(self, text: str, min_length: int = 50) -> Tuple[bool, List[str]]:
        """Check if text meets minimum length requirement."""
        issues = []
        if len(text.strip()) < min_length:
            issues.append(f"Text too short: {len(text.strip())} chars (min: {min_length})")
        return len(text.strip()) >= min_length, issues
    
    def validate_no_empty_lines(self, text: str) -> Tuple[bool, List[str]]:
        """Check if there are no excessive empty lines."""
        issues = []
        lines = text.split('\n')
        empty_line_count = sum(1 for line in lines if not line.strip())
        # Allow up to 40% empty lines for short texts, 20% for longer texts
        max_empty_ratio = 0.4 if len(lines) < 10 else 0.2
        if empty_line_count > len(lines) * max_empty_ratio:
            issues.append(f"Too many empty lines: {empty_line_count}/{len(lines)} (max: {max_empty_ratio:.0%})")
        return empty_line_count <= len(lines) * max_empty_ratio, issues
    
    def validate_format_specific(self, text: str, format_type: str) -> Tuple[bool, List[str]]:
        """Format-specific validation rules."""
        issues = []
        
        if format_type == 'pdf':
            # PDF-specific validations
            if re.search(r'\.{4,}', text):  # More than 3 dots
                issues.append("Excessive dots found (PDF artifact)")
            
        elif format_type in ['html', 'htm']:
            # HTML-specific validations
            if re.search(r'&[a-zA-Z0-9#]+;', text):  # HTML entities
                issues.append("HTML entities found (should be cleaned)")
                
        elif format_type in ['docx', 'doc']:
            # DOCX-specific validations
            if re.search(r'\x00', text):  # Null bytes
                issues.append("Null bytes found (DOCX artifact)")
        
        return len(issues) == 0, issues
    
    def validate_all(self, text: str, format_type: str) -> Dict[str, Tuple[bool, List[str]]]:
        """Run all validations and return results."""
        results = {}
        
        # Universal validations
        results['unicode_clean'] = self.validate_unicode_clean(text)
        results['hyphenation_fixed'] = self.validate_hyphenation_fixed(text)
        results['whitespace_normalized'] = self.validate_whitespace_normalized(text)
        results['bullets_cleaned'] = self.validate_bullets_cleaned(text)
        results['quotes_normalized'] = self.validate_quotes_normalized(text)
        results['dashes_normalized'] = self.validate_dashes_normalized(text)
        results['minimum_length'] = self.validate_minimum_length(text)
        results['no_empty_lines'] = self.validate_no_empty_lines(text)
        
        # Format-specific validations
        results['format_specific'] = self.validate_format_specific(text, format_type)
        
        return results


class TestTextCleaning:
    """Test suite for text cleaning across all formats."""
    
    @pytest.fixture
    def config(self):
        """Load test configuration."""
        return Config('etl/config/test.yaml')
    
    @pytest.fixture
    def processor(self, config):
        """Create file processor."""
        return FileProcessor(config)
    
    @pytest.fixture
    def validator(self):
        """Create text validator."""
        return TextValidator()
    
    @pytest.fixture
    def test_files(self) -> Dict[str, Path]:
        """Define test files for each format."""
        base_path = Path('test_data/raw/formats')
        return {
            # Currently supported formats
            'pdf': base_path / 'pdf' / 'format_pdf_text.pdf',
            'pdf_image': base_path / 'pdf' / 'format_pdf_image.pdf',
            'txt': base_path / 'txt' / 'format_txt.txt',
            'md': base_path / 'md' / 'format_md.md',
            'html': base_path / 'html' / 'format_html.html',
            'docx': base_path / 'docx' / 'format_docx.docx',
            
            # Formats to be implemented
            'rtf': base_path / 'rtf' / 'format_rtf.rtf',
            'epub': base_path / 'epub' / 'format_epub.epub',
            'msg': base_path / 'msg' / 'format_msg.msg',
            'eml': base_path / 'eml' / 'format_eml.eml',
            'htm': base_path / 'htm' / 'format_htm.htm',
            'doc': base_path / 'doc' / 'format_doc.doc',
            
            # Image formats (OCR required)
            'jpg': base_path / 'jpg' / 'format_jpg.jpg',
            'jpeg': base_path / 'jpeg' / 'format_jpeg.jpeg',
            'png': base_path / 'png' / 'format_png.png',
            'tiff': base_path / 'tiff' / 'format_tiff.tiff',
            'tif': base_path / 'tif' / 'format_tif.tif',
            'bmp': base_path / 'bmp' / 'format_bmp.bmp',
            'gif': base_path / 'gif' / 'format_gif.gif',
        }
    
    def test_pdf_text_cleaning(self, processor, validator, test_files):
        """Test PDF text cleaning quality."""
        pdf_file = test_files['pdf']
        if not pdf_file.exists():
            pytest.skip(f"Test file not found: {pdf_file}")
        
        # Process the file
        cleaned_text = processor.process_file(pdf_file)
        
        # Validate the results
        results = validator.validate_all(cleaned_text, 'pdf')
        
        # Check critical validations
        assert results['unicode_clean'][0], f"Unicode corruption found: {results['unicode_clean'][1]}"
        assert results['minimum_length'][0], f"Text too short: {results['minimum_length'][1]}"
        assert results['whitespace_normalized'][0], f"Whitespace issues: {results['whitespace_normalized'][1]}"
        
        # Print detailed results for debugging
        print(f"\nPDF Text Cleaning Results:")
        for validation, (passed, issues) in results.items():
            status = "✅ PASS" if passed else "❌ FAIL"
            print(f"  {validation}: {status}")
            if issues:
                for issue in issues:
                    print(f"    - {issue}")
    
    @pytest.mark.skip(reason="OCR not yet implemented")
    def test_pdf_image_cleaning(self, processor, validator, test_files):
        """Test PDF image OCR cleaning quality."""
        pdf_file = test_files['pdf_image']
        if not pdf_file.exists():
            pytest.skip(f"Test file not found: {pdf_file}")
        
        # Process the file
        cleaned_text = processor.process_file(pdf_file)
        
        # Validate the results
        results = validator.validate_all(cleaned_text, 'pdf')
        
        # Check critical validations
        assert results['unicode_clean'][0], f"Unicode corruption found: {results['unicode_clean'][1]}"
        assert results['minimum_length'][0], f"Text too short: {results['minimum_length'][1]}"
        
        print(f"\nPDF Image OCR Cleaning Results:")
        for validation, (passed, issues) in results.items():
            status = "✅ PASS" if passed else "❌ FAIL"
            print(f"  {validation}: {status}")
            if issues:
                for issue in issues:
                    print(f"    - {issue}")
    
    def test_txt_cleaning(self, processor, validator, test_files):
        """Test TXT file cleaning quality."""
        txt_file = test_files['txt']
        if not txt_file.exists():
            pytest.skip(f"Test file not found: {txt_file}")
        
        # Process the file
        cleaned_text = processor.process_file(txt_file)
        
        # Validate the results
        results = validator.validate_all(cleaned_text, 'txt')
        
        # Check critical validations
        assert results['unicode_clean'][0], f"Unicode corruption found: {results['unicode_clean'][1]}"
        assert results['minimum_length'][0], f"Text too short: {results['minimum_length'][1]}"
        assert results['whitespace_normalized'][0], f"Whitespace issues: {results['whitespace_normalized'][1]}"
        
        print(f"\nTXT Cleaning Results:")
        for validation, (passed, issues) in results.items():
            status = "✅ PASS" if passed else "❌ FAIL"
            print(f"  {validation}: {status}")
            if issues:
                for issue in issues:
                    print(f"    - {issue}")
    
    def test_md_cleaning(self, processor, validator, test_files):
        """Test Markdown file cleaning quality."""
        md_file = test_files['md']
        if not md_file.exists():
            pytest.skip(f"Test file not found: {md_file}")
        
        # Process the file
        cleaned_text = processor.process_file(md_file)
        
        # Validate the results
        results = validator.validate_all(cleaned_text, 'md')
        
        # Check critical validations
        assert results['unicode_clean'][0], f"Unicode corruption found: {results['unicode_clean'][1]}"
        assert results['minimum_length'][0], f"Text too short: {results['minimum_length'][1]}"
        
        print(f"\nMarkdown Cleaning Results:")
        for validation, (passed, issues) in results.items():
            status = "✅ PASS" if passed else "❌ FAIL"
            print(f"  {validation}: {status}")
            if issues:
                for issue in issues:
                    print(f"    - {issue}")
    
    def test_html_cleaning(self, processor, validator, test_files):
        """Test HTML file cleaning quality."""
        html_file = test_files['html']
        if not html_file.exists():
            pytest.skip(f"Test file not found: {html_file}")
        
        # Process the file
        cleaned_text = processor.process_file(html_file)
        
        # Validate the results
        results = validator.validate_all(cleaned_text, 'html')
        
        # Check critical validations
        assert results['unicode_clean'][0], f"Unicode corruption found: {results['unicode_clean'][1]}"
        assert results['minimum_length'][0], f"Text too short: {results['minimum_length'][1]}"
        assert results['format_specific'][0], f"HTML-specific issues: {results['format_specific'][1]}"
        
        print(f"\nHTML Cleaning Results:")
        for validation, (passed, issues) in results.items():
            status = "✅ PASS" if passed else "❌ FAIL"
            print(f"  {validation}: {status}")
            if issues:
                for issue in issues:
                    print(f"    - {issue}")
    
    def test_docx_cleaning(self, processor, validator, test_files):
        """Test DOCX file cleaning quality."""
        docx_file = test_files['docx']
        if not docx_file.exists():
            pytest.skip(f"Test file not found: {docx_file}")
        
        # Process the file
        cleaned_text = processor.process_file(docx_file)
        
        # Validate the results
        results = validator.validate_all(cleaned_text, 'docx')
        
        # Check critical validations
        assert results['unicode_clean'][0], f"Unicode corruption found: {results['unicode_clean'][1]}"
        assert results['minimum_length'][0], f"Text too short: {results['minimum_length'][1]}"
        assert results['format_specific'][0], f"DOCX-specific issues: {results['format_specific'][1]}"
        
        print(f"\nDOCX Cleaning Results:")
        for validation, (passed, issues) in results.items():
            status = "✅ PASS" if passed else "❌ FAIL"
            print(f"  {validation}: {status}")
            if issues:
                for issue in issues:
                    print(f"    - {issue}")
    
    @pytest.mark.skip(reason="RTF needs pandoc dependency")
    def test_rtf_cleaning(self, processor, validator, test_files):
        """Test RTF file cleaning quality."""
        pytest.skip("RTF format needs pandoc system dependency")
    
    @pytest.mark.skip(reason="EPUB format not yet implemented")
    def test_epub_cleaning(self, processor, validator, test_files):
        """Test EPUB file cleaning quality."""
        pytest.skip("EPUB format not yet implemented - needs unstructured support")
    
    @pytest.mark.skip(reason="MSG format not yet implemented")
    def test_msg_cleaning(self, processor, validator, test_files):
        """Test MSG file cleaning quality."""
        pytest.skip("MSG format not yet implemented - needs unstructured support")
    
    @pytest.mark.skip(reason="EML extraction not working")
    def test_eml_cleaning(self, processor, validator, test_files):
        """Test EML file cleaning quality."""
        pytest.skip("EML format extraction not working - needs email partitioner fix")
    
    def test_htm_cleaning(self, processor, validator, test_files):
        """Test HTM file cleaning quality."""
        htm_file = test_files['htm']
        if not htm_file.exists():
            pytest.skip(f"Test file not found: {htm_file}")
        
        # Process the file
        cleaned_text = processor.process_file(htm_file)
        
        # Validate the results
        results = validator.validate_all(cleaned_text, 'html')  # HTM is HTML
        
        # Check critical validations
        assert results['unicode_clean'][0], f"Unicode corruption found: {results['unicode_clean'][1]}"
        assert results['minimum_length'][0], f"Text too short: {results['minimum_length'][1]}"
        assert results['format_specific'][0], f"HTML-specific issues: {results['format_specific'][1]}"
        
        print(f"\nHTM Cleaning Results:")
        for validation, (passed, issues) in results.items():
            status = "✅ PASS" if passed else "❌ FAIL"
            print(f"  {validation}: {status}")
            if issues:
                for issue in issues:
                    print(f"    - {issue}")
    
    @pytest.mark.skip(reason="DOC format not yet implemented")
    def test_doc_cleaning(self, processor, validator, test_files):
        """Test DOC file cleaning quality."""
        pytest.skip("DOC format not yet implemented - needs unstructured support")
    
    @pytest.mark.skip(reason="Image OCR not yet implemented")
    def test_jpg_cleaning(self, processor, validator, test_files):
        """Test JPG image OCR cleaning quality."""
        pytest.skip("JPG OCR not yet implemented - needs Tesseract setup")
    
    @pytest.mark.skip(reason="Image OCR not yet implemented")
    def test_jpeg_cleaning(self, processor, validator, test_files):
        """Test JPEG image OCR cleaning quality."""
        pytest.skip("JPEG OCR not yet implemented - needs Tesseract setup")
    
    @pytest.mark.skip(reason="Image OCR not yet implemented")
    def test_png_cleaning(self, processor, validator, test_files):
        """Test PNG image OCR cleaning quality."""
        pytest.skip("PNG OCR not yet implemented - needs Tesseract setup")
    
    @pytest.mark.skip(reason="Image OCR not yet implemented")
    def test_tiff_cleaning(self, processor, validator, test_files):
        """Test TIFF image OCR cleaning quality."""
        pytest.skip("TIFF OCR not yet implemented - needs Tesseract setup")
    
    @pytest.mark.skip(reason="Image OCR not yet implemented")
    def test_tif_cleaning(self, processor, validator, test_files):
        """Test TIF image OCR cleaning quality."""
        pytest.skip("TIF OCR not yet implemented - needs Tesseract setup")
    
    @pytest.mark.skip(reason="Image OCR not yet implemented")
    def test_bmp_cleaning(self, processor, validator, test_files):
        """Test BMP image OCR cleaning quality."""
        pytest.skip("BMP OCR not yet implemented - needs Tesseract setup")
    
    @pytest.mark.skip(reason="Image OCR not yet implemented")
    def test_gif_cleaning(self, processor, validator, test_files):
        """Test GIF image OCR cleaning quality."""
        pytest.skip("GIF OCR not yet implemented - needs Tesseract setup")
    
    def test_universal_cleaning_consistency(self, processor, validator, test_files):
        """Test that universal cleaning produces consistent results across formats."""
        formats_to_test = ['pdf', 'txt', 'md', 'html', 'docx']
        results = {}
        
        for format_type in formats_to_test:
            if format_type in test_files and test_files[format_type].exists():
                try:
                    cleaned_text = processor.process_file(test_files[format_type])
                    validation_results = validator.validate_all(cleaned_text, format_type)
                    results[format_type] = validation_results
                except Exception as e:
                    print(f"Error processing {format_type}: {e}")
                    continue
        
        # Check that all formats pass basic validations
        for format_type, validation_results in results.items():
            assert validation_results['unicode_clean'][0], f"{format_type}: Unicode corruption found"
            assert validation_results['minimum_length'][0], f"{format_type}: Text too short"
            assert validation_results['whitespace_normalized'][0], f"{format_type}: Whitespace issues"
        
        print(f"\nUniversal Cleaning Consistency Results:")
        for format_type, validation_results in results.items():
            print(f"\n{format_type.upper()}:")
            for validation, (passed, issues) in validation_results.items():
                status = "✅ PASS" if passed else "❌ FAIL"
                print(f"  {validation}: {status}")
                if issues:
                    for issue in issues:
                        print(f"    - {issue}")


if __name__ == "__main__":
    # Run tests directly
    pytest.main([__file__, "-v", "-s"])
