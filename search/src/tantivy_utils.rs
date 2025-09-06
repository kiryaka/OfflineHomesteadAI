use std::path::{Path, PathBuf};
use tantivy::{
    collector::{FacetCollector, TopDocs},
    doc,
    query::QueryParser,
    schema::{Facet, Value, Schema, TextFieldIndexing, TextOptions, IndexRecordOption, FacetOptions, STRING, STORED},
    snippet::SnippetGenerator,
    tokenizer::{TextAnalyzer, SimpleTokenizer, LowerCaser, StopWordFilter},
    Index, TantivyDocument,
};
use walkdir::WalkDir;

pub struct TantivySearchEngine {
    index: Index,
    searcher: tantivy::Searcher,
    id_field: tantivy::schema::Field,
    text_field: tantivy::schema::Field,
    category_text_field: tantivy::schema::Field,
    path_field: tantivy::schema::Field,
}

pub struct TantivyIndexer {
    index: Index,
    id_field: tantivy::schema::Field,
    text_field: tantivy::schema::Field,
    category_field: tantivy::schema::Field,
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
    pub fn new(index_dir: PathBuf) -> Result<Self, anyhow::Error> {
        // Load existing index
        let index = Index::open_in_dir(&index_dir)?;
        
        // Register the same tokenizer used during indexing
        let stop_words = vec![
            "a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "has", "he", "in", "is",
            "it", "its", "of", "on", "that", "the", "to", "was", "will", "with", "or", "but", "not",
            "this", "these", "they", "them", "their", "there", "then", "than", "so", "if", "when",
            "where", "why", "how", "what", "which", "who", "whom", "whose", "can", "could", "should",
            "would", "may", "might", "must", "shall", "do", "does", "did", "have", "had", "having",
        ];

        let tokenizer = TextAnalyzer::builder(SimpleTokenizer::default())
            .filter(LowerCaser)
            .filter(StopWordFilter::remove(
                stop_words.into_iter().map(|s| s.to_string()),
            ))
            .build();

        index
            .tokenizers()
            .register("text_with_stopwords", tokenizer);
        
        let reader = index.reader()?;
        let searcher = reader.searcher();

        // Get schema fields
        let schema = index.schema();
        let id_field = schema.get_field("id")?;
        let text_field = schema.get_field("text")?;
        let category_text_field = schema.get_field("category_text")?;
        let path_field = schema.get_field("doc_path")?;

        Ok(Self {
            index,
            searcher,
            id_field,
            text_field,
            category_text_field,
            path_field,
        })
    }

    pub fn search(&self, query_text: &str, limit: usize) -> Result<Vec<SearchResult>, anyhow::Error> {
        // Parse query
        let query_parser = QueryParser::for_index(&self.index, vec![self.text_field]);
        let query = query_parser.parse_query(query_text)?;

        // Execute search
        let top_docs = self.searcher.search(&query, &TopDocs::with_limit(limit))?;

        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let doc: TantivyDocument = self.searcher.doc(doc_address)?;
            
            let id = doc.get_first(self.id_field).unwrap().as_str().unwrap();
            let category = doc.get_first(self.category_text_field).unwrap().as_str().unwrap();
            let path = doc.get_first(self.path_field).unwrap().as_str().unwrap();

            // Generate snippet
            let snippet_generator = SnippetGenerator::create(&self.searcher, &query, self.text_field)?;
            let snippet = snippet_generator.snippet_from_doc(&doc);

            results.push(SearchResult {
                score,
                id: id.to_string(),
                category: category.to_string(),
                path: path.to_string(),
                snippet: snippet.to_html(),
            });
        }

        Ok(results)
    }

    pub fn get_facet_counts(&self, query_text: &str) -> Result<Vec<(String, u64)>, anyhow::Error> {
        let query_parser = QueryParser::for_index(&self.index, vec![self.text_field]);
        let query = query_parser.parse_query(query_text)?;

        let mut facet_collector = FacetCollector::for_field("category");
        facet_collector.add_facet(Facet::root());
        let facet_counts = self.searcher.search(&query, &facet_collector)?;
        
        let mut facets = Vec::new();
        for (facet, count) in facet_counts.get(&Facet::root().to_string()) {
            facets.push((facet.to_string(), count));
        }
        
        Ok(facets)
    }
}

impl TantivyIndexer {
    pub fn new(index_dir: PathBuf) -> Result<Self, anyhow::Error> {
        let schema = Self::build_schema();
        
        // Clean up existing index if it exists
        if index_dir.exists() {
            std::fs::remove_dir_all(&index_dir)?;
        }
        std::fs::create_dir_all(&index_dir)?;

        let index = Index::create_in_dir(&index_dir, schema.clone())?;

        // Register custom tokenizer with stop words
        let stop_words = vec![
            "a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "has", "he", "in", "is",
            "it", "its", "of", "on", "that", "the", "to", "was", "will", "with", "or", "but", "not",
            "this", "these", "they", "them", "their", "there", "then", "than", "so", "if", "when",
            "where", "why", "how", "what", "which", "who", "whom", "whose", "can", "could", "should",
            "would", "may", "might", "must", "shall", "do", "does", "did", "have", "had", "having",
        ];

        let tokenizer = TextAnalyzer::builder(SimpleTokenizer::default())
            .filter(LowerCaser)
            .filter(StopWordFilter::remove(
                stop_words.into_iter().map(|s| s.to_string()),
            ))
            .build();

        index
            .tokenizers()
            .register("text_with_stopwords", tokenizer);

        // Get schema fields
        let id_field = schema.get_field("id")?;
        let text_field = schema.get_field("text")?;
        let category_field = schema.get_field("category")?;
        let category_text_field = schema.get_field("category_text")?;
        let path_field = schema.get_field("doc_path")?;

        Ok(Self {
            index,
            id_field,
            text_field,
            category_field,
            category_text_field,
            path_field,
        })
    }

    fn build_schema() -> Schema {
        let mut schema_builder = Schema::builder();

        let _id_field = schema_builder.add_text_field("id", STRING | STORED);
        let _doc_id_field = schema_builder.add_text_field("doc_id", STRING | STORED);
        let _doc_path_field = schema_builder.add_text_field("doc_path", STRING | STORED);
        
        let text_field_indexing = TextFieldIndexing::default()
            .set_tokenizer("text_with_stopwords")
            .set_index_option(IndexRecordOption::WithFreqsAndPositions);
        let text_options = TextOptions::default()
            .set_indexing_options(text_field_indexing)
            .set_stored();
        let _text_field = schema_builder.add_text_field("text", text_options);

        let _category_field = schema_builder.add_facet_field("category", FacetOptions::default());
        let _category_text_field = schema_builder.add_text_field("category_text", STRING | STORED);

        schema_builder.build()
    }

    pub fn index_files(&self, data_dir: &Path) -> Result<usize, anyhow::Error> {
        let mut index_writer = self.index.writer(50_000_000)?;
        let mut file_count = 0;

        for entry in WalkDir::new(data_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() && entry.path().extension().is_some_and(|ext| ext == "txt") {
                let file_path = entry.path();
                let relative_path = file_path.strip_prefix(data_dir).unwrap_or(file_path);
                let category = Self::extract_category_from_path(relative_path);
                
                if let Ok(content) = std::fs::read_to_string(file_path) {
                    let doc_id = format!("{}", relative_path.display());
                    let doc = doc!(
                        self.id_field => doc_id.clone(),
                        self.text_field => content.clone(),
                        self.category_field => Facet::from(&category),
                        self.category_text_field => category.clone(),
                        self.path_field => file_path.to_string_lossy().to_string()
                    );

                    index_writer.add_document(doc)?;
                    file_count += 1;
                }
            }
        }

        index_writer.commit()?;
        Ok(file_count)
    }

    fn extract_category_from_path(path: &Path) -> String {
        let components: Vec<_> = path.components().collect();
        if components.len() >= 2 {
            let category = components[0].as_os_str().to_string_lossy();
            let subcategory = components[1].as_os_str().to_string_lossy();
            format!("/{}/{}", category, subcategory)
        } else if components.len() == 1 {
            let category = components[0].as_os_str().to_string_lossy();
            format!("/{}", category)
        } else {
            "/misc".to_string()
        }
    }
}
