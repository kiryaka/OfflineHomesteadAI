use std::{
    env, fs,
    path::{Path, PathBuf},
};
// RNG imports removed - using deterministic facet allocation
use tantivy::collector::{FacetCollector, TopDocs};
use tantivy::doc;
use tantivy::query::{BooleanQuery, Occur, QueryParser, TermQuery};
use tantivy::schema::*;
use tantivy::snippet::SnippetGenerator;
use tantivy::tokenizer::{TextAnalyzer, SimpleTokenizer, LowerCaser, StopWordFilter};
use tantivy::{Index, Term};
use walkdir::WalkDir;

mod config;
use config::Config;

// VECTOR_DIM removed - no vector search needed

/// Build the search schema with all required fields
fn build_schema() -> Schema {
    let mut schema_builder = Schema::builder();

    // Required fields as per specification
    let _id_field = schema_builder.add_text_field("id", STRING | STORED);
    let _doc_id_field = schema_builder.add_text_field("doc_id", STRING | STORED);
    let _doc_path_field = schema_builder.add_text_field("doc_path", STRING | STORED);
    // vector_field removed - no vector search needed
    // Regular text field for exact matching with stop word filtering
    let text_field_indexing = TextFieldIndexing::default()
        .set_tokenizer("text_with_stopwords")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    let text_options = TextOptions::default()
        .set_indexing_options(text_field_indexing)
        .set_stored();
    let _text_field = schema_builder.add_text_field("text", text_options);

    // Note: N-gram field removed - using FuzzyTermQuery on regular text field instead

    // Facet field for hierarchical categories (2-level tree)
    let _category_field = schema_builder.add_facet_field("category", FacetOptions::default());
    // Also store category as text for display purposes
    let _category_text_field = schema_builder.add_text_field("category_text", STRING | STORED);

    schema_builder.build()
}

/// Generate facet category based on directory structure
fn get_facet_from_path(file_path: &Path, data_dir: &Path) -> String {
    // Get the relative path from the data directory
    let relative_path = file_path.strip_prefix(data_dir).unwrap_or(file_path);
    
    // Get the parent directory (the facet category)
    if let Some(parent) = relative_path.parent() {
        if let Some(facet) = parent.to_str() {
            return facet.to_string();
        }
    }
    
    // Fallback to "misc" if no parent directory
    "misc".to_string()
}

/// List all .txt files in the given directory recursively
fn list_txt_files(root: &Path) -> Vec<PathBuf> {
    let mut txt_files = Vec::new();

    for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file() && path.extension().map(|ext| ext == "txt").unwrap_or(false) {
            txt_files.push(path.to_path_buf());
        }
    }

    txt_files.sort();
    txt_files
}

// seed_from_path function removed - using directory-based facet allocation

// generate_vector function removed - no vector search needed

/// Index all text files from the data directory
fn index_files(index: &Index, data_dir: &Path, _config: &Config) -> anyhow::Result<()> {
    let schema = index.schema();
    let id_field = schema.get_field("id")?;
    let doc_id_field = schema.get_field("doc_id")?;
    let doc_path_field = schema.get_field("doc_path")?;
    // vector_field removed - no vector search needed
    let text_field = schema.get_field("text")?;
    // text_ngram_field removed - using FuzzyTermQuery instead
    let category_field = schema.get_field("category")?;
    let category_text_field = schema.get_field("category_text")?;

    let files = list_txt_files(data_dir);
    if files.is_empty() {
        println!("No .txt files found under {}.", data_dir.display());
        return Ok(());
    }

    let mut writer = index.writer(50_000_000)?; // 50MB memory buffer

    for (i, file_path) in files.iter().enumerate() {
        let content = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(_) => {
                // Handle UTF-8 encoding issues by reading as bytes and converting with lossy conversion
                let bytes = fs::read(file_path)?;
                String::from_utf8_lossy(&bytes).to_string()
            }
        };
        let doc_id = file_path.file_stem().unwrap().to_string_lossy().to_string();

        // vector generation removed - no vector search needed

        // Generate facet category based on directory structure
        let _filename = file_path.file_name().unwrap().to_string_lossy();
        let category_str = get_facet_from_path(file_path, &data_dir);
        let category_facet = Facet::from(&format!("/{}", category_str));

        // Create document with the specified schema
        let doc_id_str = if !doc_id.is_empty() {
            format!("{}:{}", doc_id, i)
        } else {
            format!("row:{}", i)
        };

        writer.add_document(doc!(
            id_field => doc_id_str,
            doc_id_field => doc_id,
            doc_path_field => file_path.to_string_lossy().to_string(),
            // vector_field removed - no vector search needed
            text_field => content,
            // text_ngram_field removed - using FuzzyTermQuery instead
            category_field => category_facet,
            category_text_field => category_str
        ))?;
    }

    writer.commit()?;
    println!("Successfully indexed {} documents", files.len());
    Ok(())
}

/// Run a search query with optional facet filtering
fn run_search_query(
    index: &Index,
    query_string: &str,
    facet_prefix: Option<&str>,
    limit: usize,
) -> anyhow::Result<()> {
    let reader = index.reader()?;
    let searcher = reader.searcher();
    let schema = index.schema();

    let text_field = schema.get_field("text")?;
    let doc_id_field = schema.get_field("doc_id")?;
    let category_field = schema.get_field("category")?;
    let category_text_field = schema.get_field("category_text")?;
    let id_field = schema.get_field("id")?;
    let doc_path_field = schema.get_field("doc_path")?;

    // Parse the base query
    let query_parser = QueryParser::for_index(index, vec![text_field, doc_id_field]);
    let base_query = query_parser.parse_query(query_string)?;

    // Create snippet generator for context highlighting
    let snippet_generator = SnippetGenerator::create(&searcher, &*base_query, text_field)?;

    // Build the final query (with optional facet filtering)
    let final_query = if let Some(prefix) = facet_prefix {
        let facet = Facet::from(&format!("/{prefix}"));
        let facet_query = TermQuery::new(
            Term::from_facet(category_field, &facet),
            IndexRecordOption::Basic,
        );

        let boolean_query = BooleanQuery::new(vec![
            (Occur::Must, Box::new(facet_query)),
            (Occur::Must, Box::new(base_query)),
        ]);

        Box::new(boolean_query) as Box<dyn tantivy::query::Query>
    } else {
        Box::new(base_query) as Box<dyn tantivy::query::Query>
    };

    // Execute search and display results
    let top_docs = searcher.search(&final_query, &TopDocs::with_limit(limit))?;
    println!(
        "\nQuery: {:?}  Facet: {}",
        query_string,
        facet_prefix.unwrap_or("(none)")
    );

    for (score, doc_address) in top_docs {
        let document = searcher.doc::<TantivyDocument>(doc_address)?;

        let id = document
            .get_first(id_field)
            .and_then(|v| v.as_str())
            .unwrap_or("-");

        let category = document
            .get_first(category_text_field)
            .and_then(|v| v.as_str())
            .unwrap_or("-");

        let path = document
            .get_first(doc_path_field)
            .and_then(|v| v.as_str())
            .unwrap_or("-");

        // Generate snippet showing context where query terms appear
        let snippet = snippet_generator.snippet_from_doc(&document);
        let snippet_text = snippet.to_html();

        println!(
            "  score={:.4}  id={:<20}  category={:<12}  path={}",
            score, id, category, path
        );
        println!("    📝 Context: {}", snippet_text);
    }

    // Show facet counts for drill-down navigation
    let mut facet_collector = FacetCollector::for_field("category");

    if let Some(prefix) = facet_prefix {
        facet_collector.add_facet(Facet::from(&format!("/{prefix}")));
    } else {
        facet_collector.add_facet(Facet::root());
    }

    let facet_counts = searcher.search(&final_query, &facet_collector)?;
    println!("  Facet counts:");

    let root_facet = if let Some(prefix) = facet_prefix {
        Facet::from(&format!("/{prefix}"))
    } else {
        Facet::root()
    };

    for (facet, count) in facet_counts.get(&root_facet.to_string()) {
        println!("    {}: {}", facet, count);
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = Config::load().map_err(|e| {
        eprintln!("Error loading config: {}", e);
        e
    })?;

    // Parse command line arguments
    let args: Vec<String> = env::args().skip(1).collect();
    let data_dir = args
        .get(0)
        .map(|s| PathBuf::from(s))
        .unwrap_or_else(|| {
            let dir: String = config.get("data.raw_txt_dir").unwrap_or_else(|_| "../dev_data/txt".to_string());
            PathBuf::from(dir)
        });
    let query_string = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| r#"old OR coffee OR "ε>0""#.to_string());
    let facet_prefix = args.get(2).map(|s| s.as_str());

    println!("Tantivy Search Demo");
    println!("==================");
    println!("Data directory: {}", data_dir.display());
    println!("Query: {}", query_string);
    println!("Facet filter: {}", facet_prefix.unwrap_or("none"));

    // Create index
    let schema = build_schema();
    let index_dir: String = config.get("data.tantivy_index_dir").unwrap_or_else(|_| "../dev_data/indexes/tantivy".to_string());
    let index_dir = PathBuf::from(index_dir);

    // Clean up existing index
    if index_dir.exists() {
        fs::remove_dir_all(&index_dir)?;
    }
    fs::create_dir_all(&index_dir)?;

    // Create index (no n-gram tokenizer needed - using FuzzyTermQuery instead)
    let index = Index::builder().schema(schema).create_in_dir(&index_dir)?;
    
    // Register custom tokenizer with stop word filtering
    let stop_words = vec![
        "a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "has", "he", "in", "is", 
        "it", "its", "of", "on", "that", "the", "to", "was", "will", "with", "or", "but", "not", 
        "this", "these", "they", "them", "their", "there", "then", "than", "so", "if", "when", 
        "where", "why", "how", "what", "which", "who", "whom", "whose", "can", "could", "should", 
        "would", "may", "might", "must", "shall", "do", "does", "did", "have", "had", "having"
    ];
    
    let tokenizer = TextAnalyzer::builder(SimpleTokenizer::default())
        .filter(LowerCaser)
        .filter(StopWordFilter::remove(stop_words.into_iter().map(|s| s.to_string())))
        .build();
    
    index.tokenizers().register("text_with_stopwords", tokenizer);
    
    println!("Created index at: {}", index_dir.display());

    // Index all text files
    index_files(&index, &data_dir, &config)?;

    // Run the main query
    run_search_query(&index, &query_string, facet_prefix, 10)?;

    // Show additional facet examples if no facet was specified
    if facet_prefix.is_none() {
        println!("\n{}", "=".repeat(50));
        println!("Additional facet examples:");
        // Additional facet examples removed - using real directory-based facets
    }

    Ok(())
}
