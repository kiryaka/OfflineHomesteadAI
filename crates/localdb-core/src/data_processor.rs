use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

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

#[derive(Debug, Clone)]
pub struct ChunkingConfig {
    pub max_tokens: usize,
    pub overlap_percent: f32,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self { max_tokens: 500, overlap_percent: 0.2 }
    }
}

#[derive(Default)]
pub struct DataProcessor {
    chunking_config: ChunkingConfig,
}

impl DataProcessor {
    pub fn new() -> Self { Self::default() }

    pub fn process_directory(&self, data_dir: &Path) -> Result<Vec<DocumentChunk>> {
        let files = self.list_txt_files(data_dir);
        if files.is_empty() {
            println!("No .txt files found under {}.", data_dir.display());
            return Ok(vec![]);
        }
        let mut all_chunks = Vec::new();
        for (file_index, file_path) in files.iter().enumerate() {
            println!("Processing file {}/{}: {}", file_index + 1, files.len(), file_path.display());
            let content = self.read_file_content(file_path)?;
            let doc_id = self.extract_doc_id(file_path);
            let category = self.get_facet_from_path(file_path, data_dir);
            let chunks = self.chunk_content(&content, &doc_id, file_path, &category)?;
            all_chunks.extend(chunks);
        }
        println!("Processed {} files into {} chunks", files.len(), all_chunks.len());
        Ok(all_chunks)
    }

    pub fn process_directory_limited(&self, data_dir: &Path, limit: usize) -> Result<Vec<DocumentChunk>> {
        let mut files = self.list_txt_files(data_dir);
        if files.is_empty() { println!("No .txt files found under {}.", data_dir.display()); return Ok(vec![]); }
        if files.len() > limit { files.truncate(limit); println!("🔢 Limited to first {} files", limit); }
        let mut all_chunks = Vec::new();
        for (file_index, file_path) in files.iter().enumerate() {
            println!("Processing file {}/{}: {}", file_index + 1, files.len(), file_path.display());
            let content = self.read_file_content(file_path)?;
            let doc_id = self.extract_doc_id(file_path);
            let category = self.get_facet_from_path(file_path, data_dir);
            let chunks = self.chunk_content(&content, &doc_id, file_path, &category)?;
            all_chunks.extend(chunks);
        }
        println!("Processed {} files into {} chunks", files.len(), all_chunks.len());
        Ok(all_chunks)
    }

    fn read_file_content(&self, file_path: &Path) -> Result<String> {
        match fs::read_to_string(file_path) {
            Ok(content) => Ok(content),
            Err(_) => Ok(String::from_utf8_lossy(&fs::read(file_path)?).to_string()),
        }
    }

    fn extract_doc_id(&self, file_path: &Path) -> String { file_path.file_stem().unwrap().to_string_lossy().to_string() }

    fn get_facet_from_path(&self, file_path: &Path, data_dir: &Path) -> String {
        let relative_path = file_path.strip_prefix(data_dir).unwrap_or(file_path);
        if let Some(parent) = relative_path.parent() { if let Some(facet) = parent.to_str() { return facet.to_string(); } }
        "misc".to_string()
    }

    fn chunk_content(&self, content: &str, doc_id: &str, file_path: &Path, category: &str) -> Result<Vec<DocumentChunk>> {
        let paragraphs: Vec<&str> = content.split("\n\n").collect();
        let mut document_chunks = Vec::new();
        let mut chunk_index = 0;
        for paragraph in paragraphs {
            let paragraph = paragraph.trim(); if paragraph.is_empty() { continue; }
            let tokens = self.count_tokens(paragraph);
            if tokens <= self.chunking_config.max_tokens {
                document_chunks.push(DocumentChunk { id: format!("{}:{}", doc_id, chunk_index), doc_id: doc_id.to_string(), doc_path: file_path.to_string_lossy().to_string(), category: category.to_string(), category_text: category.to_string(), content: paragraph.to_string(), chunk_index, total_chunks: 0 });
                chunk_index += 1;
            } else {
                for sub_chunk in self.split_paragraph_with_overlap(paragraph) {
                    document_chunks.push(DocumentChunk { id: format!("{}:{}", doc_id, chunk_index), doc_id: doc_id.to_string(), doc_path: file_path.to_string_lossy().to_string(), category: category.to_string(), category_text: category.to_string(), content: sub_chunk, chunk_index, total_chunks: 0 });
                    chunk_index += 1;
                }
            }
        }
        let total_chunks = document_chunks.len(); for chunk in &mut document_chunks { chunk.total_chunks = total_chunks; }
        Ok(document_chunks)
    }

    fn count_tokens(&self, text: &str) -> usize { let word_count = text.split_whitespace().count(); (word_count as f32 / 0.75) as usize }

    fn split_paragraph_with_overlap(&self, paragraph: &str) -> Vec<String> {
        let words: Vec<&str> = paragraph.split_whitespace().collect();
        let words_per_chunk = 300; let overlap_words = (words_per_chunk as f32 * self.chunking_config.overlap_percent) as usize;
        let mut chunks = Vec::new(); let mut start = 0;
        while start < words.len() {
            let end = (start + words_per_chunk).min(words.len());
            chunks.push(words[start..end].join(" "));
            if end >= words.len() { break; }
            start = end - overlap_words;
        }
        chunks
    }

    fn list_txt_files(&self, root: &Path) -> Vec<PathBuf> {
        let mut txt_files = Vec::new();
        for entry in walkdir::WalkDir::new(root).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
            let path = entry.path(); if path.extension().and_then(|s| s.to_str()) == Some("txt") { txt_files.push(path.to_path_buf()); }
        }
        txt_files.sort(); txt_files
    }
}

