"""
Command line interface for the ETL pipeline.

This module provides a comprehensive CLI for the ETL (Extract, Transform, Load) pipeline
using Click. It supports individual pipeline steps as well as running the complete
pipeline from raw documents to search-ready indexes.

Key Features:
- Individual pipeline steps (extract, chunk, embed, load)
- Complete pipeline execution
- Configurable parameters for each step
- Support for multiple file formats
- Integration with both Tantivy and LanceDB

Pipeline Steps:
1. Extract: Extract text from documents (PDF, DOCX, images)
2. Chunk: Split text into manageable chunks
3. Embed: Generate vector embeddings for chunks
4. Load: Export to search systems (Tantivy, LanceDB)

Usage:
    # Run individual steps
    python -m etl.src.cli extract --input raw/ --output processed/
    python -m etl.src.cli chunk --input processed/ --output chunks/
    python -m etl.src.cli embed --input chunks/ --output embeddings/
    python -m etl.src.cli load --embeddings embeddings/ --output indexes/
    
    # Run complete pipeline
    python -m etl.src.cli pipeline --input raw/ --output output/

Example:
    >>> # Extract text from documents
    >>> python -m etl.src.cli extract -i documents/ -o processed/ -f pdf docx
    
    >>> # Chunk processed documents
    >>> python -m etl.src.cli chunk -i processed/ -o chunks/ --chunk-size 1000
    
    >>> # Generate embeddings
    >>> python -m etl.src.cli embed -i chunks/ -o embeddings/ --model all-MiniLM-L6-v2
    
    >>> # Load into search systems
    >>> python -m etl.src.cli load -e embeddings/ -o indexes/
"""

import logging
import click
from pathlib import Path
from typing import List

from .extractors import PDFExtractor, DOCXExtractor, ImageExtractor
from .processors import TextChunker, Tokenizer, Embedder
from .loaders import TantivyLoader, LanceDBLoader

# Configure logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(name)s - %(levelname)s - %(message)s')
logger = logging.getLogger(__name__)


@click.group()
def main():
    """Universe ELT Pipeline - Extract, Load, Transform documents."""
    pass


@main.command()
@click.option('--input', '-i', required=True, help='Input directory with raw documents')
@click.option('--output', '-o', required=True, help='Output directory for processed data')
@click.option('--formats', '-f', multiple=True, default=['pdf', 'docx', 'jpg', 'png'], 
              help='File formats to process')
def extract(input: str, output: str, formats: List[str]):
    """
    Extract text from documents.
    
    This command processes raw documents and extracts text content using
    format-specific extractors. It supports PDF, DOCX, and image formats
    with OCR capabilities.
    
    Args:
        input: Directory containing raw documents
        output: Directory to save processed JSON files
        formats: List of file formats to process (pdf, docx, jpg, png, etc.)
    """
    input_dir = Path(input)
    output_dir = Path(output)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Initialize extractors
    pdf_extractor = PDFExtractor()
    docx_extractor = DOCXExtractor()
    image_extractor = ImageExtractor()
    
    # Process files
    processed_files = []
    
    for file_path in input_dir.rglob('*'):
        if file_path.is_file() and file_path.suffix.lower().lstrip('.') in formats:
            logger.info(f"Processing: {file_path}")
            
            # Choose extractor based on file type
            if file_path.suffix.lower() == '.pdf':
                result = pdf_extractor.extract(file_path)
            elif file_path.suffix.lower() == '.docx':
                result = docx_extractor.extract(file_path)
            elif file_path.suffix.lower() in ['.jpg', '.jpeg', '.png', '.tiff']:
                result = image_extractor.extract(file_path)
            else:
                logger.warning(f"Unsupported format: {file_path}")
                continue
            
            # Save processed data
            output_file = output_dir / f"{file_path.stem}.json"
            import json
            with open(output_file, 'w', encoding='utf-8') as f:
                json.dump(result, f, indent=2, ensure_ascii=False)
            
            processed_files.append(output_file)
    
    logger.info(f"Processed {len(processed_files)} files")


@main.command()
@click.option('--input', '-i', required=True, help='Input directory with processed documents')
@click.option('--output', '-o', required=True, help='Output directory for chunks')
@click.option('--chunk-size', default=1000, help='Chunk size in characters')
@click.option('--chunk-overlap', default=200, help='Chunk overlap in characters')
def chunk(input: str, output: str, chunk_size: int, chunk_overlap: int):
    """
    Chunk processed documents.
    
    This command takes processed documents and splits them into smaller,
    manageable chunks for vector search. It preserves semantic boundaries
    and applies configurable overlap between chunks.
    
    Args:
        input: Directory containing processed JSON documents
        output: Directory to save chunked data
        chunk_size: Maximum size of each chunk in characters
        chunk_overlap: Overlap between chunks in characters
    """
    input_dir = Path(input)
    output_dir = Path(output)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Initialize chunker
    chunker = TextChunker(chunk_size=chunk_size, chunk_overlap=chunk_overlap)
    
    # Process files
    all_chunks = []
    
    for file_path in input_dir.glob('*.json'):
        logger.info(f"Chunking: {file_path}")
        
        import json
        with open(file_path, 'r', encoding='utf-8') as f:
            data = json.load(f)
        
        content = data.get('content', '')
        metadata = data.get('metadata', {})
        
        chunks = chunker.chunk_document(content, metadata)
        all_chunks.extend(chunks)
    
    # Save chunks
    output_file = output_dir / "chunks.json"
    import json
    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(all_chunks, f, indent=2, ensure_ascii=False)
    
    logger.info(f"Created {len(all_chunks)} chunks")


@main.command()
@click.option('--input', '-i', required=True, help='Input directory with chunks')
@click.option('--output', '-o', required=True, help='Output directory for embeddings')
@click.option('--model', default='text-embedding-3-small', help='Embedding model')
def embed(input: str, output: str, model: str):
    """
    Generate embeddings for chunks.
    
    This command takes text chunks and generates vector embeddings using
    the specified model. It also adds token counts for each chunk to
    track text length and processing costs.
    
    Args:
        input: Directory containing chunked JSON data
        output: Directory to save embeddings
        model: Embedding model to use (e.g., 'text-embedding-3-small', 'all-MiniLM-L6-v2')
    """
    input_dir = Path(input)
    output_dir = Path(output)
    output_dir.mkdir(parents=True, exist_ok=True)
    
    # Initialize processors
    tokenizer = Tokenizer(model)
    embedder = Embedder(model)
    
    # Load chunks
    chunks_file = input_dir / "chunks.json"
    if not chunks_file.exists():
        logger.error(f"Chunks file not found: {chunks_file}")
        return
    
    import json
    with open(chunks_file, 'r', encoding='utf-8') as f:
        chunks = json.load(f)
    
    # Add token counts
    chunks = tokenizer.add_token_counts(chunks)
    
    # Generate embeddings
    chunks = embedder.embed_chunks(chunks)
    
    # Save embeddings
    output_file = output_dir / "embeddings.json"
    with open(output_file, 'w', encoding='utf-8') as f:
        json.dump(chunks, f, indent=2, ensure_ascii=False)
    
    logger.info(f"Generated embeddings for {len(chunks)} chunks")


@main.command()
@click.option('--embeddings', '-e', required=True, help='Input directory with embeddings')
@click.option('--output', '-o', required=True, help='Output directory for search indexes')
def load(embeddings: str, output: str):
    """
    Load data into search systems.
    
    This command takes processed chunks with embeddings and exports them
    to both Tantivy (full-text search) and LanceDB (vector search) formats.
    This enables both keyword-based and semantic search capabilities.
    
    Args:
        embeddings: Directory containing embeddings JSON data
        output: Directory to save search indexes
    """
    embeddings_dir = Path(embeddings)
    output_dir = Path(output)
    
    # Load embeddings
    embeddings_file = embeddings_dir / "embeddings.json"
    if not embeddings_file.exists():
        logger.error(f"Embeddings file not found: {embeddings_file}")
        return
    
    import json
    with open(embeddings_file, 'r', encoding='utf-8') as f:
        chunks = json.load(f)
    
    # Initialize loaders
    tantivy_loader = TantivyLoader(output_dir / "tantivy")
    lancedb_loader = LanceDBLoader(output_dir / "lancedb")
    
    # Export to both systems
    tantivy_loader.export_chunks(chunks)
    lancedb_loader.export_chunks(chunks)
    
    logger.info("Data loaded into search systems")


@main.command()
@click.option('--input', '-i', required=True, help='Input directory with raw documents')
@click.option('--output', '-o', required=True, help='Output directory for search indexes')
@click.option('--chunk-size', default=1000, help='Chunk size in characters')
@click.option('--chunk-overlap', default=200, help='Chunk overlap in characters')
@click.option('--model', default='text-embedding-3-small', help='Embedding model')
def pipeline(input: str, output: str, chunk_size: int, chunk_overlap: int, model: str):
    """
    Run the complete ETL pipeline.
    
    This command executes the entire ETL pipeline from raw documents to
    search-ready indexes. It combines all individual steps (extract, chunk,
    embed, load) into a single workflow for convenience.
    
    Pipeline Steps:
    1. Extract text from documents
    2. Chunk text into manageable pieces
    3. Generate vector embeddings
    4. Export to search systems (Tantivy, LanceDB)
    
    Args:
        input: Directory containing raw documents
        output: Directory to save all pipeline outputs
        chunk_size: Maximum size of each chunk in characters
        chunk_overlap: Overlap between chunks in characters
        model: Embedding model to use
    """
    logger.info("Starting complete ELT pipeline")
    
    # Step 1: Extract
    extract(input, str(Path(output) / "processed"))
    
    # Step 2: Chunk
    chunk(str(Path(output) / "processed"), str(Path(output) / "chunks"), chunk_size, chunk_overlap)
    
    # Step 3: Embed
    embed(str(Path(output) / "chunks"), str(Path(output) / "embeddings"), model)
    
    # Step 4: Load
    load(str(Path(output) / "embeddings"), str(Path(output) / "indexes"))
    
    logger.info("ELT pipeline completed")


if __name__ == '__main__':
    main()
