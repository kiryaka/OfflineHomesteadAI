#!/usr/bin/env python3
"""
ETL Load Script - Convert all supported files to cleaned text files.

This script is the main entry point for the ETL (Extract, Transform, Load) pipeline.
It processes files of all supported types from a hierarchical directory structure,
extracts text content using the FileProcessor, cleans it with universal text cleaning,
and saves as text files while preserving the directory structure.

Key Features:
- Processes multiple file formats (PDF, DOCX, HTML, Markdown, TXT, etc.)
- Preserves directory structure in output
- Universal text cleaning with format-specific optimizations
- Progress tracking with tqdm
- Comprehensive logging and error handling
- Dry-run mode for testing
- Environment-specific configuration

Supported File Formats:
- PDF (text): PyMuPDF, pdfplumber, PyPDF2, unstructured
- DOCX/DOC: unstructured
- HTML/HTM: unstructured
- Markdown: unstructured
- TXT: unstructured
- RTF: unstructured (requires pandoc)
- EPUB: unstructured (requires pandoc)
- MSG/EML: unstructured email partitioner
- Images: unstructured OCR (requires tesseract)

Usage:
    python load.py [--env dev|prod|test] [--dry-run] [--verbose]

Examples:
    # Process files in development environment
    python load.py --env dev
    
    # Test what would be processed without writing files
    python load.py --env dev --dry-run
    
    # Process with verbose logging
    python load.py --env prod --verbose

Configuration:
    The script uses environment-specific configuration files:
    - etl/config/dev.yaml (development)
    - etl/config/prod.yaml (production)
    - etl/config/test.yaml (testing)
"""

import argparse
import logging
import sys
from pathlib import Path
from typing import List, Tuple
from tqdm import tqdm

# Add src to path for imports
sys.path.insert(0, str(Path(__file__).parent / "src"))

from config.settings import Config
from pdf_processor import FileProcessor


def setup_logging(config: Config) -> None:
    """Setup logging configuration."""
    logging.basicConfig(
        level=getattr(logging, config.log_level.upper()),
        format=config.get("logging.format", "%(asctime)s - %(name)s - %(levelname)s - %(message)s"),
        handlers=[
            logging.StreamHandler(),
            logging.FileHandler("etl_load.log")
        ]
    )


def find_supported_files(raw_dir: Path, processor: FileProcessor) -> List[Path]:
    """Find all supported files in the raw directory recursively.
    
    Args:
        raw_dir: Root directory to search
        processor: Processor instance to check file support
        
    Returns:
        List of supported file paths
    """
    supported_files = []
    for file_path in raw_dir.rglob("*"):
        if file_path.is_file() and processor.is_supported(file_path):
            supported_files.append(file_path)
    return sorted(supported_files)


def get_output_path(file_path: Path, raw_dir: Path, txt_dir: Path) -> Path:
    """Get output text file path preserving directory structure.
    
    Args:
        file_path: Source file path
        raw_dir: Source root directory
        txt_dir: Target root directory
        
    Returns:
        Output text file path
    """
    # Get relative path from raw_dir
    rel_path = file_path.relative_to(raw_dir)
    
    # Change extension to .txt
    txt_filename = rel_path.with_suffix('.txt')
    
    # Create full output path
    output_path = txt_dir / txt_filename
    
    return output_path


def process_file(file_path: Path, output_path: Path, processor: FileProcessor, 
                dry_run: bool = False) -> Tuple[bool, str]:
    """Process a single file.
    
    Args:
        file_path: Source file
        output_path: Target text file path
        processor: File processor instance
        dry_run: If True, don't write files
        
    Returns:
        Tuple of (success, message)
    """
    try:
        # Create output directory if it doesn't exist
        if not dry_run:
            output_path.parent.mkdir(parents=True, exist_ok=True)
        
        # Process file
        cleaned_text = processor.process_file(file_path)
        
        if not cleaned_text:
            return False, "No text extracted or text too short"
        
        # Write output file
        if not dry_run:
            with open(output_path, 'w', encoding='utf-8') as f:
                f.write(cleaned_text)
        
        return True, f"Processed successfully ({len(cleaned_text)} chars)"
        
    except Exception as e:
        return False, f"Error: {str(e)}"


def main():
    """
    Main function that orchestrates the ETL pipeline.
    
    This function handles command-line arguments, loads configuration, sets up logging,
    finds supported files, processes them through the FileProcessor, and provides
    comprehensive reporting on the results.
    
    The pipeline follows these steps:
    1. Parse command-line arguments
    2. Load environment-specific configuration
    3. Setup logging with appropriate level
    4. Validate input directory exists
    5. Initialize FileProcessor with configuration
    6. Recursively find all supported files
    7. Process each file (extract, clean, save)
    8. Generate summary report
    
    Returns:
        None: Exits with appropriate status code on error
    """
    # Parse command-line arguments
    parser = argparse.ArgumentParser(description="Unified ETL Load Script - Convert all files to text")
    parser.add_argument("--env", choices=["dev", "prod", "test"], default="dev",
                       help="Environment (dev/prod/test)")
    parser.add_argument("--dry-run", action="store_true",
                       help="Show what would be processed without writing files")
    parser.add_argument("--verbose", "-v", action="store_true",
                       help="Enable verbose logging")
    
    args = parser.parse_args()
    
    # Set environment variable for configuration loading
    import os
    os.environ["ETL_ENV"] = args.env
    
    # Load environment-specific configuration
    # This will load the appropriate YAML file based on the environment
    try:
        config = Config()
    except FileNotFoundError as e:
        print(f"❌ Configuration error: {e}")
        sys.exit(1)
    
    # Setup logging with appropriate level
    # Verbose mode enables DEBUG level for detailed troubleshooting
    if args.verbose:
        config.config["logging"]["level"] = "DEBUG"
    setup_logging(config)
    
    logger = logging.getLogger(__name__)
    logger.info(f"Starting ETL Load process (env={args.env}, dry_run={args.dry_run})")
    
    # Get input and output directories from configuration
    raw_dir = config.raw_dir
    txt_dir = config.txt_dir
    
    logger.info(f"Raw directory: {raw_dir}")
    logger.info(f"Text directory: {txt_dir}")
    
    # Validate that input directory exists
    if not raw_dir.exists():
        logger.error(f"Raw directory does not exist: {raw_dir}")
        sys.exit(1)
    
    # Initialize the file processor with configuration
    # This sets up all the extraction and cleaning parameters
    processor = FileProcessor(config)
    
    # Recursively find all supported files in the input directory
    logger.info("Scanning for supported files...")
    supported_files = find_supported_files(raw_dir, processor)
    
    if not supported_files:
        logger.warning("No supported files found in raw directory")
        return
    
    # Analyze and report file type distribution
    # This helps understand what types of files are being processed
    file_types = {}
    for file_path in supported_files:
        ext = file_path.suffix.lower()
        file_types[ext] = file_types.get(ext, 0) + 1
    
    logger.info(f"Found {len(supported_files)} supported files:")
    for ext, count in sorted(file_types.items()):
        logger.info(f"  {ext}: {count} files")
    
    # Process each file through the pipeline
    success_count = 0
    error_count = 0
    
    logger.info("Processing files...")
    
    # Use tqdm for progress tracking during file processing
    for file_path in tqdm(supported_files, desc="Processing files"):
        # Calculate output path preserving directory structure
        output_path = get_output_path(file_path, raw_dir, txt_dir)
        
        # Process the file (extract text, clean, and optionally save)
        success, message = process_file(file_path, output_path, processor, args.dry_run)
        
        if success:
            success_count += 1
            logger.debug(f"✅ {file_path.name}: {message}")
        else:
            error_count += 1
            logger.warning(f"❌ {file_path.name}: {message}")
    
    # Generate comprehensive summary report
    logger.info("=" * 50)
    logger.info(f"Processing complete!")
    logger.info(f"✅ Successfully processed: {success_count}")
    logger.info(f"❌ Errors: {error_count}")
    logger.info(f"📁 Output directory: {txt_dir}")
    
    if args.dry_run:
        logger.info("🔍 Dry run mode - no files were written")
    else:
        logger.info("📄 Text files have been written to the output directory")


if __name__ == "__main__":
    main()
