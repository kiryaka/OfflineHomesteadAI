#!/usr/bin/env python3
"""ETL Load Script - Convert PDFs to cleaned text files.

This script processes PDF files from a hierarchical directory structure,
extracts text content, cleans it, and saves as text files while preserving
the directory structure.

Usage:
    python load.py [--env dev|prod] [--dry-run]
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
from pdf_processor import PDFProcessor


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


def find_pdf_files(raw_dir: Path) -> List[Path]:
    """Find all PDF files in the raw directory recursively.
    
    Args:
        raw_dir: Root directory to search
        
    Returns:
        List of PDF file paths
    """
    pdf_files = []
    for pdf_path in raw_dir.rglob("*.pdf"):
        if pdf_path.is_file():
            pdf_files.append(pdf_path)
    return sorted(pdf_files)


def get_output_path(pdf_path: Path, raw_dir: Path, txt_dir: Path) -> Path:
    """Get output text file path preserving directory structure.
    
    Args:
        pdf_path: Source PDF file path
        raw_dir: Source root directory
        txt_dir: Target root directory
        
    Returns:
        Output text file path
    """
    # Get relative path from raw_dir
    rel_path = pdf_path.relative_to(raw_dir)
    
    # Change extension from .pdf to .txt
    txt_filename = rel_path.with_suffix('.txt')
    
    # Create full output path
    output_path = txt_dir / txt_filename
    
    return output_path


def process_pdf_file(pdf_path: Path, output_path: Path, processor: PDFProcessor, 
                    dry_run: bool = False) -> Tuple[bool, str]:
    """Process a single PDF file.
    
    Args:
        pdf_path: Source PDF file
        output_path: Target text file path
        processor: PDF processor instance
        dry_run: If True, don't write files
        
    Returns:
        Tuple of (success, message)
    """
    try:
        # Create output directory if it doesn't exist
        if not dry_run:
            output_path.parent.mkdir(parents=True, exist_ok=True)
        
        # Process PDF
        cleaned_text = processor.process_pdf(pdf_path)
        
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
    """Main function."""
    parser = argparse.ArgumentParser(description="ETL Load Script - Convert PDFs to text")
    parser.add_argument("--env", choices=["dev", "prod"], default="dev",
                       help="Environment (dev/prod)")
    parser.add_argument("--dry-run", action="store_true",
                       help="Show what would be processed without writing files")
    parser.add_argument("--verbose", "-v", action="store_true",
                       help="Enable verbose logging")
    
    args = parser.parse_args()
    
    # Set environment variable
    import os
    os.environ["ETL_ENV"] = args.env
    
    # Load configuration
    try:
        config = Config()
    except FileNotFoundError as e:
        print(f"❌ Configuration error: {e}")
        sys.exit(1)
    
    # Setup logging
    if args.verbose:
        config.config["logging"]["level"] = "DEBUG"
    setup_logging(config)
    
    logger = logging.getLogger(__name__)
    logger.info(f"Starting ETL Load process (env={args.env}, dry_run={args.dry_run})")
    
    # Get directories
    raw_dir = config.raw_dir
    txt_dir = config.txt_dir
    
    logger.info(f"Raw directory: {raw_dir}")
    logger.info(f"Text directory: {txt_dir}")
    
    # Check if raw directory exists
    if not raw_dir.exists():
        logger.error(f"Raw directory does not exist: {raw_dir}")
        sys.exit(1)
    
    # Find all PDF files
    logger.info("Scanning for PDF files...")
    pdf_files = find_pdf_files(raw_dir)
    
    if not pdf_files:
        logger.warning("No PDF files found in raw directory")
        return
    
    logger.info(f"Found {len(pdf_files)} PDF files")
    
    # Initialize processor
    processor = PDFProcessor(config)
    
    # Process files
    success_count = 0
    error_count = 0
    
    logger.info("Processing PDF files...")
    
    for pdf_path in tqdm(pdf_files, desc="Processing PDFs"):
        # Get output path
        output_path = get_output_path(pdf_path, raw_dir, txt_dir)
        
        # Process file
        success, message = process_pdf_file(pdf_path, output_path, processor, args.dry_run)
        
        if success:
            success_count += 1
            logger.debug(f"✅ {pdf_path.name}: {message}")
        else:
            error_count += 1
            logger.warning(f"❌ {pdf_path.name}: {message}")
    
    # Summary
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
