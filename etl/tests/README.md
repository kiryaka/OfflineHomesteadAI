# Text Cleaning Validation Tests

This directory contains comprehensive tests for validating text cleaning quality across all supported file formats.

## Overview

The test suite validates that our universal text cleaning approach produces high-quality, consistent results across all supported formats.

## Test Structure

### `test_text_cleaning.py`
Main test file containing:
- `TextValidator` class with universal validation methods
- `TestTextCleaning` class with format-specific test cases
- Comprehensive validation rules for text quality

### `run_text_cleaning_tests.py`
Standalone test runner that provides detailed output and analysis.

### `run_tests.sh`
Shell script to run tests with both pytest and the custom runner.

## Validation Rules

The `TextValidator` class checks for:

### Universal Validations
- **Unicode Clean**: No unicode corruption (e.g., `â\x80\x99`)
- **Hyphenation Fixed**: Words split across lines are properly merged
- **Whitespace Normalized**: No excessive spaces, tabs, or newlines
- **Bullets Cleaned**: List bullets and markers are removed
- **Quotes Normalized**: Smart quotes converted to standard quotes
- **Dashes Normalized**: En-dash and em-dash converted to hyphens
- **Minimum Length**: Text meets minimum length requirements
- **No Empty Lines**: Reasonable ratio of empty lines

### Format-Specific Validations
- **PDF**: No excessive dots, PDF artifacts
- **HTML**: No HTML entities
- **DOCX**: No null bytes
- **Images**: OCR-specific validations (when implemented)

## Supported Formats

### ✅ Currently Supported (5/19 formats)
- **PDF** (text): Full validation suite - ✅ PASSING
- **TXT**: Full validation suite - ✅ PASSING
- **Markdown**: Full validation suite - ✅ PASSING
- **HTML**: Full validation suite - ✅ PASSING
- **DOCX**: Full validation suite - ✅ PASSING

### ⏳ Pending Implementation (6/19 formats)
- **RTF**: Format not yet implemented - needs unstructured support
- **EPUB**: Format not yet implemented - needs unstructured support
- **MSG**: Format not yet implemented - needs unstructured support
- **EML**: Format not yet implemented - needs unstructured support
- **HTM**: Format not yet implemented - should work like HTML
- **DOC**: Format not yet implemented - needs unstructured support

### 🔍 OCR Pending (8/19 formats)
- **PDF** (image): Requires OCR implementation - needs Tesseract setup
- **JPG**: Requires OCR implementation - needs Tesseract setup
- **JPEG**: Requires OCR implementation - needs Tesseract setup
- **PNG**: Requires OCR implementation - needs Tesseract setup
- **TIFF**: Requires OCR implementation - needs Tesseract setup
- **TIF**: Requires OCR implementation - needs Tesseract setup
- **BMP**: Requires OCR implementation - needs Tesseract setup
- **GIF**: Requires OCR implementation - needs Tesseract setup

## Running Tests

### Quick Test Run
```bash
cd etl/tests
python run_text_cleaning_tests.py
```

### With pytest
```bash
cd etl/tests
python -m pytest test_text_cleaning.py -v -s
```

### Using Shell Script
```bash
cd etl/tests
./run_tests.sh
```

## Test Results

### ✅ Currently Supported Formats (5/5 passing)
All currently supported formats pass all validation tests:

```
✅ PDF Text        PASSED (9/9) - 733 chars
✅ TXT             PASSED (9/9) - 731 chars  
✅ Markdown        PASSED (9/9) - 756 chars
✅ HTML            PASSED (9/9) - 742 chars
✅ DOCX            PASSED (9/9) - 730 chars
```

### 📊 Overall Statistics
- **✅ Supported formats**: 5/19 (26%)
- **⏳ Pending formats**: 6/19 (32%)
- **🔍 OCR pending formats**: 8/19 (42%)
- **📁 Total formats**: 19

### 🎯 Progress Tracking
The test suite provides a clear roadmap of what needs to be implemented:

1. **Next Priority**: Implement pending document formats (RTF, EPUB, MSG, EML, HTM, DOC)
2. **Future Work**: Set up OCR for image formats and PDF images
3. **Current Status**: All supported formats are working perfectly

## Key Features

1. **Universal Cleaning**: Same cleaning logic applied to all formats
2. **Conditional Treatment**: Aggressive cleaning for corrupted text (e.g., PDFs)
3. **Format Awareness**: Format-specific post-processing
4. **Comprehensive Validation**: 9 different quality checks per format
5. **Easy Extension**: Simple to add new formats and validation rules

## Adding New Formats

To add a new format:

1. Add the format to `test_files` fixture in `test_text_cleaning.py`
2. Create a test method following the pattern `test_{format}_cleaning`
3. Add format-specific validation rules to `validate_format_specific`
4. Remove the `@pytest.mark.skip` decorator when ready

## Adding New Validations

To add new validation rules:

1. Add a new method to `TextValidator` class
2. Call it in the `validate_all` method
3. Update test assertions as needed

## Example Output

```
==================== PDF TEXT ====================
Processing: test_data/raw/formats/pdf/format_pdf_text.pdf

Validation Results (9/9 passed):
----------------------------------------
unicode_clean        ✅ PASS
hyphenation_fixed    ✅ PASS
whitespace_normalized ✅ PASS
bullets_cleaned      ✅ PASS
quotes_normalized    ✅ PASS
dashes_normalized    ✅ PASS
minimum_length       ✅ PASS
no_empty_lines       ✅ PASS
format_specific      ✅ PASS

Cleaned Text Sample (first 200 chars):
----------------------------------------
Project: Hyphenation & Unicode CleanTrial (v1.2.3+meta) 2025 09 06 Author: kirill@example.com | URL: https://example.com/test?q=Eagles%E2%80%99 UUID: 550e8400 e29b 41d4 a716 446655440000 | Range: 10 1...
```

This test suite ensures that our text cleaning pipeline produces consistent, high-quality results across all supported formats.
