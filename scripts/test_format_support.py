#!/usr/bin/env python3
"""
Test script to validate ETL pipeline support for all file formats.
"""

import sys
import os
from pathlib import Path

# Add src to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent / "etl" / "src"))

from config.settings import Config
from pdf_processor import FileProcessor

def test_format_support():
    """Test that all supported formats are properly handled."""
    
    # Load test configuration
    os.environ["ETL_ENV"] = "test"
    config = Config()
    processor = FileProcessor(config)
    
    print("🧪 Testing ETL Pipeline Format Support")
    print("=" * 50)
    
    # Test data directory
    test_raw_dir = Path("../test_data/raw/formats")
    
    # Expected formats and their test files
    test_files = {
        'html': test_raw_dir / "html" / "index.html",
        'htm': test_raw_dir / "htm" / "survival_guide.htm", 
        'md': test_raw_dir / "md" / "README.md",
        'txt': test_raw_dir / "txt" / "survival_guide.txt",
        'rtf': test_raw_dir / "rtf" / "survival_guide.rtf",
        'jpg': test_raw_dir / "jpg" / "survival_image.jpg",
        'jpeg': test_raw_dir / "jpeg" / "survival_photo.jpeg",
        'png': test_raw_dir / "png" / "survival_diagram.png",
        'tiff': test_raw_dir / "tiff" / "survival_map.tiff",
        'tif': test_raw_dir / "tif" / "survival_chart.tif",
        'bmp': test_raw_dir / "bmp" / "survival_icon.bmp",
        'gif': test_raw_dir / "gif" / "survival_animation.gif",
        'docx': test_raw_dir / "docx" / "survival_guide.docx",
        'doc': test_raw_dir / "doc" / "survival_guide.doc",
        'epub': test_raw_dir / "epub" / "survival_guide.epub",
        'msg': test_raw_dir / "msg" / "survival_email.msg",
        'eml': test_raw_dir / "eml" / "survival_email.eml",
    }
    
    results = {}
    
    for format_name, file_path in test_files.items():
        print(f"\n📄 Testing {format_name.upper()} format...")
        
        if not file_path.exists():
            print(f"   ❌ Test file not found: {file_path}")
            results[format_name] = False
            continue
            
        # Test if format is supported
        if not processor.is_supported(file_path):
            print(f"   ❌ Format not supported: {format_name}")
            results[format_name] = False
            continue
            
        # Test text extraction
        try:
            text = processor.extract_text(file_path)
            if text and len(text.strip()) > 0:
                print(f"   ✅ Successfully extracted {len(text)} characters")
                results[format_name] = True
            else:
                print(f"   ⚠️  No text extracted (may be expected for binary formats)")
                results[format_name] = True  # Still counts as supported
        except Exception as e:
            print(f"   ❌ Error extracting text: {e}")
            results[format_name] = False
    
    # Summary
    print("\n" + "=" * 50)
    print("📊 Test Results Summary")
    print("=" * 50)
    
    supported_count = sum(1 for success in results.values() if success)
    total_count = len(results)
    
    for format_name, success in results.items():
        status = "✅" if success else "❌"
        print(f"{status} {format_name.upper()}")
    
    print(f"\n🎯 {supported_count}/{total_count} formats supported")
    
    if supported_count == total_count:
        print("🎉 All formats are properly supported!")
        return True
    else:
        print("⚠️  Some formats need attention")
        return False

if __name__ == "__main__":
    success = test_format_support()
    sys.exit(0 if success else 1)
