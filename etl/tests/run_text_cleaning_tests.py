#!/usr/bin/env python3
"""
Test runner for text cleaning validation tests.

This script provides a comprehensive test runner for the ETL pipeline's text cleaning
functionality. It runs individual format tests with detailed output and provides
a complete summary of the validation results across all supported formats.

Key Features:
- Individual format testing with detailed validation results
- Comprehensive status reporting (PASS, FAIL, PENDING, OCR_PENDING)
- Progress tracking and statistics
- Clear roadmap for format implementation
- Support for 19 different file formats

Test Categories:
- SUPPORTED: Currently working formats (6 formats)
- PENDING: Formats needing implementation (5 formats)
- OCR_PENDING: Image formats needing OCR setup (8 formats)

Usage:
    # Run all tests with detailed output
    python run_text_cleaning_tests.py
    
    # Run with pytest (alternative)
    pytest test_text_cleaning.py -v

Output:
    The script provides detailed output for each format including:
    - Validation results for each test case
    - Text length and quality metrics
    - Specific issues found (if any)
    - Overall statistics and progress tracking
    - Implementation roadmap for pending formats

Example Output:
    ==================== PDF TEXT ====================
    ✅ PDF Text        PASSED (9/9) - 733 chars
    
    ==================== RTF ====================
    ⏳ PENDING - RTF needs pandoc dependency
       Needs: Install pandoc system dependency
    
    📊 STATISTICS:
      ✅ Supported formats: 6
      ⏳ Pending formats: 5
      🔍 OCR pending formats: 8
      📁 Total formats: 19
"""

import sys
import os
from pathlib import Path

# Add the etl directory to the Python path
etl_dir = Path(__file__).parent.parent
sys.path.insert(0, str(etl_dir))

# Import and run the tests
from test_text_cleaning import TestTextCleaning, TextValidator
from src.pdf_processor import FileProcessor
from config.settings import Config
import pytest


def run_individual_tests():
    """Run individual format tests with detailed output."""
    print("=" * 60)
    print("TEXT CLEANING VALIDATION TESTS")
    print("=" * 60)
    
    # Setup
    config = Config('etl/config/test.yaml')
    processor = FileProcessor(config)
    validator = TextValidator()
    
    # Test files
    base_path = Path('test_data/raw/formats')
    test_files = {
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
    
    # Define test cases with their status
    test_cases = [
        # Currently supported formats
        ('PDF Text', 'pdf', test_files['pdf'], 'SUPPORTED'),
        ('TXT', 'txt', test_files['txt'], 'SUPPORTED'),
        ('Markdown', 'md', test_files['md'], 'SUPPORTED'),
        ('HTML', 'html', test_files['html'], 'SUPPORTED'),
        ('DOCX', 'docx', test_files['docx'], 'SUPPORTED'),
        ('HTM', 'htm', test_files['htm'], 'SUPPORTED'),
        ('RTF', 'rtf', test_files['rtf'], 'PENDING'),
        ('EML', 'eml', test_files['eml'], 'PENDING'),
        
        # Formats to be implemented
        ('EPUB', 'epub', test_files['epub'], 'PENDING'),
        ('MSG', 'msg', test_files['msg'], 'PENDING'),
        ('DOC', 'doc', test_files['doc'], 'PENDING'),
        
        # Image formats (OCR required)
        ('JPG', 'jpg', test_files['jpg'], 'OCR_PENDING'),
        ('JPEG', 'jpeg', test_files['jpeg'], 'OCR_PENDING'),
        ('PNG', 'png', test_files['png'], 'OCR_PENDING'),
        ('TIFF', 'tiff', test_files['tiff'], 'OCR_PENDING'),
        ('TIF', 'tif', test_files['tif'], 'OCR_PENDING'),
        ('BMP', 'bmp', test_files['bmp'], 'OCR_PENDING'),
        ('GIF', 'gif', test_files['gif'], 'OCR_PENDING'),
        
        # Special cases
        ('PDF Image', 'pdf_image', test_files['pdf_image'], 'OCR_PENDING'),
    ]
    
    results = {}
    
    for test_name, format_type, file_path, status in test_cases:
        print(f"\n{'='*20} {test_name.upper()} {'='*20}")
        
        if status == 'PENDING':
            if format_type == 'rtf':
                print(f"⏳ PENDING - RTF needs pandoc dependency")
                print(f"   Needs: Install pandoc system dependency")
            elif format_type == 'eml':
                print(f"⏳ PENDING - EML extraction not working")
                print(f"   Needs: Fix email partitioner configuration")
            else:
                print(f"⏳ PENDING - Format not yet implemented")
                print(f"   Needs: unstructured support for {format_type.upper()}")
            results[test_name] = {'status': 'PENDING', 'reason': 'Format not implemented'}
            continue
        elif status == 'OCR_PENDING':
            print(f"🔍 OCR PENDING - OCR not yet implemented")
            print(f"   Needs: Tesseract setup for {format_type.upper()}")
            results[test_name] = {'status': 'OCR_PENDING', 'reason': 'OCR not implemented'}
            continue
        
        if not file_path.exists():
            print(f"❌ Test file not found: {file_path}")
            results[test_name] = {'status': 'SKIP', 'reason': 'File not found'}
            continue
        
        try:
            # Process the file
            print(f"Processing: {file_path}")
            cleaned_text = processor.process_file(file_path)
            
            # Validate the results
            validation_results = validator.validate_all(cleaned_text, format_type)
            
            # Check results
            passed_tests = sum(1 for passed, _ in validation_results.values() if passed)
            total_tests = len(validation_results)
            
            print(f"\nValidation Results ({passed_tests}/{total_tests} passed):")
            print("-" * 40)
            
            for validation_name, (passed, issues) in validation_results.items():
                status_icon = "✅ PASS" if passed else "❌ FAIL"
                print(f"{validation_name:20} {status_icon}")
                if issues:
                    for issue in issues:
                        print(f"  └─ {issue}")
            
            # Show sample of cleaned text
            print(f"\nCleaned Text Sample (first 200 chars):")
            print("-" * 40)
            print(cleaned_text[:200] + "..." if len(cleaned_text) > 200 else cleaned_text)
            
            results[test_name] = {
                'status': 'PASS' if passed_tests == total_tests else 'FAIL',
                'passed': passed_tests,
                'total': total_tests,
                'text_length': len(cleaned_text)
            }
            
        except Exception as e:
            print(f"❌ Error processing {test_name}: {e}")
            results[test_name] = {'status': 'ERROR', 'reason': str(e)}
    
    # Summary
    print(f"\n{'='*60}")
    print("SUMMARY")
    print("=" * 60)
    
    # Group results by status
    status_groups = {
        'PASS': [],
        'FAIL': [],
        'PENDING': [],
        'OCR_PENDING': [],
        'SKIP': [],
        'ERROR': []
    }
    
    for test_name, result in results.items():
        status_groups[result['status']].append((test_name, result))
    
    # Print each group
    if status_groups['PASS']:
        print("\n✅ SUPPORTED FORMATS (PASSING):")
        for test_name, result in status_groups['PASS']:
            print(f"  ✅ {test_name:15} PASSED ({result['passed']}/{result['total']}) - {result['text_length']} chars")
    
    if status_groups['FAIL']:
        print("\n❌ SUPPORTED FORMATS (FAILING):")
        for test_name, result in status_groups['FAIL']:
            print(f"  ❌ {test_name:15} FAILED ({result['passed']}/{result['total']}) - {result['text_length']} chars")
    
    if status_groups['PENDING']:
        print("\n⏳ PENDING FORMATS (TO BE IMPLEMENTED):")
        for test_name, result in status_groups['PENDING']:
            print(f"  ⏳ {test_name:15} PENDING - {result['reason']}")
    
    if status_groups['OCR_PENDING']:
        print("\n🔍 OCR PENDING FORMATS (NEED TESSERACT):")
        for test_name, result in status_groups['OCR_PENDING']:
            print(f"  🔍 {test_name:15} OCR PENDING - {result['reason']}")
    
    if status_groups['SKIP']:
        print("\n⏭️  SKIPPED FORMATS:")
        for test_name, result in status_groups['SKIP']:
            print(f"  ⏭️  {test_name:15} SKIPPED - {result['reason']}")
    
    if status_groups['ERROR']:
        print("\n💥 ERROR FORMATS:")
        for test_name, result in status_groups['ERROR']:
            print(f"  💥 {test_name:15} ERROR - {result['reason']}")
    
    # Overall statistics
    supported_count = len(status_groups['PASS']) + len(status_groups['FAIL'])
    pending_count = len(status_groups['PENDING'])
    ocr_pending_count = len(status_groups['OCR_PENDING'])
    total_count = len(results)
    
    print(f"\n📊 STATISTICS:")
    print(f"  ✅ Supported formats: {supported_count}")
    print(f"  ⏳ Pending formats: {pending_count}")
    print(f"  🔍 OCR pending formats: {ocr_pending_count}")
    print(f"  📁 Total formats: {total_count}")
    
    # Overall result
    passed_count = len(status_groups['PASS'])
    print(f"\n🎯 SUPPORTED FORMATS: {passed_count}/{supported_count} passing")
    
    return passed_count == supported_count


def run_pytest():
    """Run tests using pytest framework."""
    print("Running tests with pytest...")
    return pytest.main([__file__.replace('run_text_cleaning_tests.py', 'test_text_cleaning.py'), "-v", "-s"])


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--pytest":
        exit_code = run_pytest()
        sys.exit(exit_code)
    else:
        success = run_individual_tests()
        sys.exit(0 if success else 1)
