use anyhow::Result;
use tantivy::{Index, collector::TopDocs, query::QueryParser, TantivyDocument};
use tantivy::schema::Value;

pub struct TantivySearchEngine {
	index: Index,
	searcher: tantivy::Searcher,
	id_field: tantivy::schema::Field,
	text_field: tantivy::schema::Field,
	category_text_field: tantivy::schema::Field,
	path_field: tantivy::schema::Field,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
	pub score: f32,
	pub id: String,
	pub category: String,
	pub path: String,
	pub snippet: String,
}

impl TantivySearchEngine {
	pub fn new(index_dir: std::path::PathBuf) -> Result<Self, anyhow::Error> {
		let index = Index::open_in_dir(&index_dir)?;
		crate::tantivy_utils::register_tokenizer(&index);
		let reader = index.reader()?; let searcher = reader.searcher();
		let schema = index.schema();
		let id_field = schema.get_field("id")?;
		let text_field = schema.get_field("text")?;
		let category_text_field = schema.get_field("category_text")?;
		let path_field = schema.get_field("doc_path")?;
		Ok(Self { index, searcher, id_field, text_field, category_text_field, path_field })
	}

	pub fn search(&self, query_text: &str, limit: usize) -> Result<Vec<SearchResult>, anyhow::Error> {
		let query_parser = QueryParser::for_index(&self.index, vec![self.text_field]);
		let query = query_parser.parse_query(query_text)?;
		let top_docs = self.searcher.search(&query, &TopDocs::with_limit(limit))?;
		let mut results = Vec::new();
		for (score, doc_address) in top_docs { let doc: TantivyDocument = self.searcher.doc(doc_address)?;
			let id = doc.get_first(self.id_field).unwrap().as_str().unwrap();
			let category = doc.get_first(self.category_text_field).unwrap().as_str().unwrap();
			let path = doc.get_first(self.path_field).unwrap().as_str().unwrap();
			let snippet_generator = tantivy::snippet::SnippetGenerator::create(&self.searcher, &query, self.text_field)?;
			let snippet = snippet_generator.snippet_from_doc(&doc);
			results.push(SearchResult { score, id: id.to_string(), category: category.to_string(), path: path.to_string(), snippet: snippet.to_html() }); }
		Ok(results)
	}

	pub fn get_facet_counts(&self, query_text: &str) -> Result<Vec<(String, u64)>, anyhow::Error> {
		let query_parser = QueryParser::for_index(&self.index, vec![self.text_field]);
		let query = query_parser.parse_query(query_text)?;
		let mut facet_collector = tantivy::collector::FacetCollector::for_field("category");
		facet_collector.add_facet(tantivy::schema::Facet::root());
		let facet_counts = self.searcher.search(&query, &facet_collector)?;
		let mut facets = Vec::new();
		for (facet, count) in facet_counts.get(&tantivy::schema::Facet::root().to_string()) { facets.push((facet.to_string(), count)); }
		Ok(facets)
	}
}
