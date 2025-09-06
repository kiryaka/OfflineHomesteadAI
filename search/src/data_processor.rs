use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a processed document chunk ready for indexing
#[derive(Debug, Clone)]
pub struct DocumentChunk {
    pub id: String,
    pub doc_id: String,
    pub doc_path: String,
    pub category: String,
    pub category_text: String,
    pub content: String,
    pub chunk_index: usize,
    pub total_chunks: usize,
}

/// Configuration for text chunking
#[derive(Debug, Clone)]
pub struct ChunkingConfig {
    pub max_tokens: usize,
    pub overlap_percent: f32,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            max_tokens: 500,
            overlap_percent: 0.2,
        }
    }
}

/// Unified data processor that handles file loading, chunking, and metadata extraction
#[derive(Default)]
pub struct DataProcessor {
    chunking_config: ChunkingConfig,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Process all text files in a directory and return document chunks
    pub fn process_directory(&self, data_dir: &Path) -> Result<Vec<DocumentChunk>> {
        let files = self.list_txt_files(data_dir);
        if files.is_empty() {
            println!("No .txt files found under {}.", data_dir.display());
            return Ok(vec![]);
        }

        let mut all_chunks = Vec::new();

        for (file_index, file_path) in files.iter().enumerate() {
            println!(
                "Processing file {}/{}: {}",
                file_index + 1,
                files.len(),
                file_path.display()
            );

            let content = self.read_file_content(file_path)?;
            let doc_id = self.extract_doc_id(file_path);
            let category = self.get_facet_from_path(file_path, data_dir);

            // Process the content into chunks
            let chunks = self.chunk_content(&content, &doc_id, file_path, &category)?;
            all_chunks.extend(chunks);
        }

        println!(
            "Processed {} files into {} chunks",
            files.len(),
            all_chunks.len()
        );
        Ok(all_chunks)
    }

    /// Process a limited number of text files in a directory and return document chunks
    pub fn process_directory_limited(&self, data_dir: &Path, limit: usize) -> Result<Vec<DocumentChunk>> {
        let mut files = self.list_txt_files(data_dir);
        if files.is_empty() {
            println!("No .txt files found under {}.", data_dir.display());
            return Ok(vec![]);
        }

        // Limit the number of files
        if files.len() > limit {
            files.truncate(limit);
            println!("🔢 Limited to first {} files out of {} total", limit, self.list_txt_files(data_dir).len());
        }

        let mut all_chunks = Vec::new();

        for (file_index, file_path) in files.iter().enumerate() {
            println!(
                "Processing file {}/{}: {}",
                file_index + 1,
                files.len(),
                file_path.display()
            );

            let content = self.read_file_content(file_path)?;
            let doc_id = self.extract_doc_id(file_path);
            let category = self.get_facet_from_path(file_path, data_dir);

            // Process the content into chunks
            let chunks = self.chunk_content(&content, &doc_id, file_path, &category)?;
            all_chunks.extend(chunks);
        }

        println!(
            "Processed {} files into {} chunks",
            files.len(),
            all_chunks.len()
        );

        Ok(all_chunks)
    }

    /// Read file content with UTF-8 error handling
    fn read_file_content(&self, file_path: &Path) -> Result<String> {
        match fs::read_to_string(file_path) {
            Ok(content) => Ok(content),
            Err(_) => {
                // Handle UTF-8 encoding issues by reading as bytes and converting with lossy conversion
                let bytes = fs::read(file_path)?;
                Ok(String::from_utf8_lossy(&bytes).to_string())
            }
        }
    }

    /// Extract document ID from file path
    fn extract_doc_id(&self, file_path: &Path) -> String {
        file_path.file_stem().unwrap().to_string_lossy().to_string()
    }

    /// Get facet category from directory structure
    fn get_facet_from_path(&self, file_path: &Path, data_dir: &Path) -> String {
        let relative_path = file_path.strip_prefix(data_dir).unwrap_or(file_path);

        if let Some(parent) = relative_path.parent() {
            if let Some(facet) = parent.to_str() {
                return facet.to_string();
            }
        }

        "misc".to_string()
    }

    /// Chunk content based on paragraphs with smart splitting
    fn chunk_content(
        &self,
        content: &str,
        doc_id: &str,
        file_path: &Path,
        category: &str,
    ) -> Result<Vec<DocumentChunk>> {
        // Split content into paragraphs (separated by \n\n)
        let paragraphs: Vec<&str> = content.split("\n\n").collect();
        let mut document_chunks = Vec::new();
        let mut chunk_index = 0;

        for paragraph in paragraphs {
            let paragraph = paragraph.trim();
            if paragraph.is_empty() {
                continue;
            }

            let tokens = self.count_tokens(paragraph);

            if tokens <= self.chunking_config.max_tokens {
                // Paragraph fits in one chunk - use it as is
                document_chunks.push(DocumentChunk {
                    id: format!("{}:{}", doc_id, chunk_index),
                    doc_id: doc_id.to_string(),
                    doc_path: file_path.to_string_lossy().to_string(),
                    category: category.to_string(),
                    category_text: category.to_string(),
                    content: paragraph.to_string(),
                    chunk_index,
                    total_chunks: 0, // Will be set later
                });
                chunk_index += 1;
            } else {
                // Paragraph is too large - split it by ~300 tokens with 20% overlap
                let sub_chunks = self.split_paragraph_with_overlap(paragraph);
                for sub_chunk in sub_chunks {
                    document_chunks.push(DocumentChunk {
                        id: format!("{}:{}", doc_id, chunk_index),
                        doc_id: doc_id.to_string(),
                        doc_path: file_path.to_string_lossy().to_string(),
                        category: category.to_string(),
                        category_text: category.to_string(),
                        content: sub_chunk,
                        chunk_index,
                        total_chunks: 0, // Will be set later
                    });
                    chunk_index += 1;
                }
            }
        }

        // Set total_chunks for all chunks
        let total_chunks = document_chunks.len();
        for chunk in &mut document_chunks {
            chunk.total_chunks = total_chunks;
        }

        Ok(document_chunks)
    }

    /// Count tokens using a simple word-based approximation
    fn count_tokens(&self, text: &str) -> usize {
        // Simple approximation: 1 token ≈ 0.75 words
        // This is a rough estimate - for production, you'd want to use a proper tokenizer
        let word_count = text.split_whitespace().count();
        (word_count as f32 / 0.75) as usize
    }

    /// Split a large paragraph into chunks with overlap
    fn split_paragraph_with_overlap(&self, paragraph: &str) -> Vec<String> {
        let words: Vec<&str> = paragraph.split_whitespace().collect();
        let words_per_chunk = 300; // ~300 tokens target
        let overlap_words =
            (words_per_chunk as f32 * self.chunking_config.overlap_percent) as usize;

        let mut chunks = Vec::new();
        let mut start = 0;

        while start < words.len() {
            let end = (start + words_per_chunk).min(words.len());
            let chunk_words = &words[start..end];
            chunks.push(chunk_words.join(" "));

            if end >= words.len() {
                break;
            }

            // Move start position with overlap
            start = end - overlap_words;
        }

        chunks
    }

    /// List all .txt files in the given directory recursively
    fn list_txt_files(&self, root: &Path) -> Vec<PathBuf> {
        let mut txt_files = Vec::new();

        for entry in walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("txt") {
                txt_files.push(path.to_path_buf());
            }
        }

        txt_files.sort();
        txt_files
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_counting() {
        let processor = DataProcessor::new();
        let text = "This is a test sentence with multiple words.";
        let tokens = processor.count_tokens(text);
        assert!(tokens > 0);
    }

    #[test]
    fn test_chunking_small_text() {
        let processor = DataProcessor::new();
        let text = "Short text";
        let chunks = processor
            .chunk_content(text, "test", Path::new("test.txt"), "category")
            .unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].content, text);
    }

}
