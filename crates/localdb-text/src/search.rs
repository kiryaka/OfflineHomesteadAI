use anyhow::Result;
use tantivy::{Index, collector::TopDocs, query::QueryParser, TantivyDocument};
use tantivy::query::{BoostQuery, BooleanQuery, Occur, Query};
use tantivy::schema::Value;
use localdb_core::traits::TextIndexer;
use localdb_core::types::{DocumentChunk, SearchHit, SourceKind};

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
    /// Open a searcher over an existing index path.
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

    /// Run a BM25 search with AND/phrase boosting and return top `limit` results.
    pub fn search(&self, query_text: &str, limit: usize) -> Result<Vec<SearchResult>, anyhow::Error> {
        // OR query (default behavior)
        let parser_or = QueryParser::for_index(&self.index, vec![self.text_field]);
        let or_q = parser_or.parse_query(query_text)?;

        // AND query (conjunction by default)
        let mut parser_and = QueryParser::for_index(&self.index, vec![self.text_field]);
        parser_and.set_conjunction_by_default();
        let and_q = parser_and.parse_query(query_text)?;

        // Phrase query if multiword
        let phrase_q: Option<Box<dyn Query>> = if query_text.split_whitespace().count() > 1 {
            let phrase_text = format!("\"{}\"", query_text);
            match parser_or.parse_query(&phrase_text) {
                Ok(q) => Some(q.box_clone()),
                Err(_) => None,
            }
        } else { None };

        // Combine with boosts: phrase (x4) > AND (x2) > OR (x1)
        let mut subs: Vec<(Occur, Box<dyn Query>)> = Vec::new();
        subs.push((Occur::Should, Box::new(BoostQuery::new(or_q.box_clone(), 1.0))));
        subs.push((Occur::Should, Box::new(BoostQuery::new(and_q.box_clone(), 2.0))));
        if let Some(pq) = phrase_q { subs.push((Occur::Should, Box::new(BoostQuery::new(pq, 4.0)))); }
        let combined = BooleanQuery::new(subs);

        let top_docs = self.searcher.search(&combined, &TopDocs::with_limit(limit))?;
        let mut results = Vec::new();
        for (score, doc_address) in top_docs { let doc: TantivyDocument = self.searcher.doc(doc_address)?;
            let id = doc.get_first(self.id_field).unwrap().as_str().unwrap();
            let category = doc.get_first(self.category_text_field).unwrap().as_str().unwrap();
            let path = doc.get_first(self.path_field).unwrap().as_str().unwrap();
            let snippet_generator = tantivy::snippet::SnippetGenerator::create(&self.searcher, &combined, self.text_field)?;
            let snippet = snippet_generator.snippet_from_doc(&doc);
            results.push(SearchResult { score, id: id.to_string(), category: category.to_string(), path: path.to_string(), snippet: snippet.to_html() }); }
		Ok(results)
	}

    /// Compute facet counts for the root facet under the given query.
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

impl TextIndexer for TantivySearchEngine {
    fn index(&self, _chunks: &[DocumentChunk]) -> anyhow::Result<()> {
        // Read-only search adapter; indexing should be done via TantivyIndexer.
        Ok(())
    }

    fn search(&self, query: &str, k: usize) -> anyhow::Result<Vec<SearchHit>> {
        let query_parser = QueryParser::for_index(&self.index, vec![self.text_field]);
        let query = query_parser.parse_query(query)?;
        let top_docs = self.searcher.search(&query, &TopDocs::with_limit(k))?;
        let mut hits = Vec::new();
        for (score, doc_address) in top_docs {
            let doc: TantivyDocument = self.searcher.doc(doc_address)?;
            let id = doc.get_first(self.id_field).and_then(|v| v.as_str()).unwrap_or("").to_string();
            hits.push(SearchHit { id, score, source: SourceKind::Text });
        }
//! BM25 search over the Tantivy index with boosted AND/phrase variants.
//!
//! Builds three subqueries (OR, AND-by-default, and phrase if applicable) and
//! combines them with a Boolean SHOULD query using weights (OR×1, AND×2, PHRASE×4).
        Ok(hits)
    }
}
