use anyhow::Result;
use std::path::Path;
use tantivy::{doc, Index, TantivyDocument};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::Value;

use localdb_core::traits::TextIndexer;
use localdb_core::types::{DocumentChunk, SearchHit, SourceKind};

use crate::tantivy_utils::{build_schema, register_tokenizer};

pub struct TantivyIndexer {
	index: Index,
	id_field: tantivy::schema::Field,
	text_field: tantivy::schema::Field,
	category_field: tantivy::schema::Field,
	category_text_field: tantivy::schema::Field,
	path_field: tantivy::schema::Field,
}

impl TantivyIndexer {
	pub fn new(index_dir: std::path::PathBuf) -> Result<Self, anyhow::Error> {
		let schema = build_schema();
		if index_dir.exists() { std::fs::remove_dir_all(&index_dir)?; }
		std::fs::create_dir_all(&index_dir)?;
		let index = Index::create_in_dir(&index_dir, schema.clone())?;
		register_tokenizer(&index);
		let id_field = schema.get_field("id")?;
		let text_field = schema.get_field("text")?;
		let category_field = schema.get_field("category")?;
		let category_text_field = schema.get_field("category_text")?;
		let path_field = schema.get_field("doc_path")?;
		Ok(Self { index, id_field, text_field, category_field, category_text_field, path_field })
	}

	pub fn index_files(&self, data_dir: &Path) -> Result<usize, anyhow::Error> {
		let mut index_writer = self.index.writer(50_000_000)?;
		let mut file_count = 0;
		for entry in walkdir::WalkDir::new(data_dir).into_iter().filter_map(|e| e.ok()) {
			if entry.file_type().is_file() && entry.path().extension().is_some_and(|ext| ext == "txt") {
				let file_path = entry.path();
				let relative_path = file_path.strip_prefix(data_dir).unwrap_or(file_path);
				let category = Self::extract_category_from_path(relative_path);
				if let Ok(content) = std::fs::read_to_string(file_path) {
					let doc_id = format!("{}", relative_path.display());
					let doc = doc!(
						self.id_field => doc_id.clone(),
						self.text_field => content.clone(),
						self.category_field => tantivy::schema::Facet::from(&category),
						self.category_text_field => category.clone(),
						self.path_field => file_path.to_string_lossy().to_string()
					);
					index_writer.add_document(doc)?;
					file_count += 1;
				}
			}
		}
		index_writer.commit()?; Ok(file_count)
	}

	fn extract_category_from_path(path: &Path) -> String {
		let components: Vec<_> = path.components().collect();
		if components.len() >= 2 { let category = components[0].as_os_str().to_string_lossy(); let subcategory = components[1].as_os_str().to_string_lossy(); format!("/{}/{}", category, subcategory) }
		else if components.len() == 1 { let category = components[0].as_os_str().to_string_lossy(); format!("/{}", category) }
		else { "/misc".to_string() }
	}
}

impl TextIndexer for TantivyIndexer {
    fn index(&self, chunks: &[DocumentChunk]) -> anyhow::Result<()> {
        let mut index_writer = self.index.writer(50_000_000)?;
        for c in chunks {
            let doc = doc!(
                self.id_field => c.id.clone(),
                self.text_field => c.content.clone(),
                self.category_field => tantivy::schema::Facet::from(&c.category),
                self.category_text_field => c.category_text.clone(),
                self.path_field => c.doc_path.clone(),
            );
            index_writer.add_document(doc)?;
        }
        index_writer.commit()?;
        Ok(())
    }

    fn search(&self, query: &str, k: usize) -> anyhow::Result<Vec<SearchHit>> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        let qp = QueryParser::for_index(&self.index, vec![self.text_field]);
        let q = qp.parse_query(query)?;
        let top_docs = searcher.search(&q, &TopDocs::with_limit(k))?;
        let mut hits = Vec::new();
        for (score, addr) in top_docs {
            let doc: TantivyDocument = searcher.doc(addr)?;
            let id = doc.get_first(self.id_field).and_then(|v| v.as_str()).unwrap_or("").to_string();
            hits.push(SearchHit { id, score, source: SourceKind::Text });
        }
        Ok(hits)
    }
}
