use std::io::{self, Write};
use tantivy::collector::{FacetCollector, TopDocs};
use tantivy::query::{BooleanQuery, Occur, QueryParser, TermQuery};
use tantivy::schema::*;
use tantivy::snippet::SnippetGenerator;
use tantivy::tokenizer::{TextAnalyzer, SimpleTokenizer, LowerCaser, StopWordFilter};
use tantivy::{Index, Term};

mod config;
use config::Config;

/// Setup search system and return necessary components
fn setup_search() -> anyhow::Result<(tantivy::Searcher, tantivy::schema::Schema, Field, Field, Field, Field, Field)> {
    // Load configuration
    let config = Config::load().map_err(|e| {
        eprintln!("Error loading config: {}", e);
        e
    })?;

    let index_dir: String = config.get("data.tantivy_index_dir").unwrap_or_else(|_| "../dev_data/indexes/tantivy".to_string());
    let index_dir = std::path::PathBuf::from(index_dir);

    // Check if index exists
    if !index_dir.exists() {
        println!("❌ Index not found. Please run the indexer first:");
        println!("   cargo run --release --bin tantivy-lancedb-hybrid-search");
        std::process::exit(1);
    }

    // Open existing index
    let index = Index::open_in_dir(&index_dir)?;
    
    // Register custom tokenizer with stop word filtering (same as in main.rs)
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
    
    let reader = index.reader()?;
    let searcher = reader.searcher();
    let schema = index.schema();

    // Get schema fields
    let text_field = schema.get_field("text")?;
    let category_field = schema.get_field("category")?;
    let category_text_field = schema.get_field("category_text")?;
    let doc_path_field = schema.get_field("doc_path")?;
    let id_field = schema.get_field("id")?;

    Ok((searcher, schema, text_field, category_field, category_text_field, doc_path_field, id_field))
}

/// Interactive search CLI for the search system
fn main() -> anyhow::Result<()> {
    println!("🔍 Interactive Search CLI");
    println!("========================");
    
    // Setup search system
    let (searcher, _schema, text_field, category_field, category_text_field, doc_path_field, id_field) = setup_search()?;
    
    println!("✅ Index loaded successfully!");
    println!("📊 Total documents: {}", searcher.num_docs());
    println!();

    // Start interactive loop
    interactive_search_loop(&searcher, text_field, category_field, category_text_field, doc_path_field, id_field)?;

    Ok(())
}

/// Main interactive search loop
fn interactive_search_loop(
    searcher: &tantivy::Searcher,
    text_field: Field,
    category_field: Field,
    category_text_field: Field,
    doc_path_field: Field,
    id_field: Field,
) -> anyhow::Result<()> {
    let mut query_parser = QueryParser::for_index(searcher.index(), vec![text_field]);

    println!("🎯 Interactive Search Commands:");
    println!("  /help     - Show this help message");
    println!("  /facets   - List all available facets");
    println!("  /stats    - Show index statistics");
    println!("  /quit     - Exit the search");
    println!("  <query>   - Search for text");
    println!();

    loop {
        print!("search> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        match input {
            "/help" | "/h" => {
                show_help();
            }
            "/facets" | "/f" => {
                if let Err(e) = show_facets(searcher, &category_text_field) {
                    println!("❌ Error showing facets: {}", e);
                }
            }
            "/stats" | "/s" => {
                show_stats(searcher);
            }
            "/quit" | "/q" | "quit" | "exit" => {
                println!("👋 Goodbye!");
                break;
            }
            _ => {
                // Parse search query with options
                let (query, facet_filter, limit, fuzzy, snippet_chars) = parse_search_input(input);
                
                if let Err(e) = execute_search(
                    searcher,
                    &query,
                    facet_filter.as_deref(),
                    limit,
                    fuzzy,
                    snippet_chars,
                    text_field,
                    category_field,
                    category_text_field,
                    doc_path_field,
                    id_field,
                    &mut query_parser,
                ) {
                    println!("❌ Search error: {}", e);
                }
            }
        }
        println!();
    }

    Ok(())
}

fn parse_search_input(input: &str) -> (String, Option<String>, usize, bool, Option<usize>) {
    let mut facet_filter = None;
    let mut limit = 5;
    let mut fuzzy = false;
    let mut snippet_chars = None;

    // Parse options like: "coffee -f agriculture -n 10 --fuzzy"
    let parts: Vec<&str> = input.split_whitespace().collect();
    let mut i = 0;
    let mut query_parts = Vec::new();

    while i < parts.len() {
        match parts[i] {
            "-f" | "--facet" => {
                if i + 1 < parts.len() {
                    facet_filter = Some(parts[i + 1].to_string());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "-n" | "--limit" | "--number" => {
                if i + 1 < parts.len() {
                    if let Ok(n) = parts[i + 1].parse::<usize>() {
                        limit = n;
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--fuzzy" => {
                fuzzy = true;
                i += 1;
            }
            "-c" | "--context" => {
                if i + 1 < parts.len() {
                    if let Ok(num) = parts[i + 1].parse::<usize>() {
                        snippet_chars = Some(num);
                        i += 2; // Skip both -c and the number
                    } else {
                        eprintln!("Warning: -c option requires a number.");
                        i += 1;
                    }
                } else {
                    eprintln!("Warning: -c option requires a number.");
                    i += 1;
                }
            }
            _ => {
                query_parts.push(parts[i]);
                i += 1;
            }
        }
    }

    let query = query_parts.join(" ");
    (query, facet_filter, limit, fuzzy, snippet_chars)
}

fn execute_search(
    searcher: &tantivy::Searcher,
    query_str: &str,
    facet_filter: Option<&str>,
    limit: usize,
    _fuzzy: bool,
    snippet_chars: Option<usize>,
    text_field: Field,
    category_field: Field,
    category_text_field: Field,
    doc_path_field: Field,
    id_field: Field,
    query_parser: &mut QueryParser,
) -> anyhow::Result<()> {
    if query_str.is_empty() {
        println!("❌ Empty query");
        return Ok(());
    }

    // Build query
    let text_query = query_parser.parse_query(query_str)?;
    
    let final_query = if let Some(facet) = facet_filter {
        // Add facet filter
        let facet_term = Term::from_facet(category_field, &Facet::from(&format!("/{}", facet)));
        let facet_query = TermQuery::new(facet_term, tantivy::schema::IndexRecordOption::Basic);
        Box::new(BooleanQuery::new(vec![
            (Occur::Must, text_query),
            (Occur::Must, Box::new(facet_query)),
        ]))
    } else {
        text_query
    };

    // Create snippet generator with the actual query for proper highlighting
    let mut snippet_generator = SnippetGenerator::create(searcher, &*final_query, text_field)?;
    
    // Set snippet length if specified
    if let Some(chars) = snippet_chars {
        snippet_generator.set_max_num_chars(chars);
    }

    // Execute search
    let top_docs = searcher.search(&final_query, &TopDocs::with_limit(limit))?;

    if top_docs.is_empty() {
        println!("🔍 No results found for: \"{}\"", query_str);
        if let Some(facet) = facet_filter {
            println!("   (filtered by facet: {})", facet);
        }
        return Ok(());
    }

    println!("🔍 Found {} results for: \"{}\"", top_docs.len(), query_str);
    if let Some(facet) = facet_filter {
        println!("   (filtered by facet: {})", facet);
    }
    println!();

    // Display results
    for (i, (score, doc_address)) in top_docs.iter().enumerate() {
        let retrieved_doc = searcher.doc::<TantivyDocument>(*doc_address)?;
        
        let id = retrieved_doc.get_first(id_field).and_then(|v| v.as_str()).unwrap_or("-");
        let category = retrieved_doc.get_first(category_text_field).and_then(|v| v.as_str()).unwrap_or("-");
        let path = retrieved_doc.get_first(doc_path_field).and_then(|v| v.as_str()).unwrap_or("-");

        println!("  {}. score={:.4}  id={}  category={}", 
                 i + 1, score, id, category);
        println!("     path={}", path);

        // Generate snippet with Tantivy's built-in highlighting
        let snippet = snippet_generator.snippet_from_doc(&retrieved_doc);
        let snippet_text = snippet.to_html();
        println!("     {}", snippet_text);
        println!();
    }

    Ok(())
}


fn show_help() {
    println!("🔍 Search Help");
    println!("==============");
    println!();
    println!("📝 Search Syntax:");
    println!("  Basic: coffee");
    println!("  Boolean: coffee AND survival");
    println!("  Phrases: \"coffee grounds\"");
    println!("  Wildcards: coffee*");
    println!();
    println!("🎯 Search Options:");
    println!("  -f <facet>     Filter by facet (e.g., agriculture, foraging)");
    println!("  -n <number>    Limit results (default: 5, max: 100)");
    println!("  -c <chars>     Set snippet length in characters (default: 150)");
    println!("  --fuzzy        Enable fuzzy matching");
    println!();
    println!("📋 Examples:");
    println!("  coffee -f agriculture -n 10");
    println!("  \"survival gear\" -f survival --fuzzy");
    println!("  hunting AND fishing -n 20 -c 300");
    println!("  chicken coop --fuzzy -c 50");
    println!();
    println!("🔧 Commands:");
    println!("  /help, /h      Show this help");
    println!("  /facets, /f    List available facets");
    println!("  /stats, /s     Show index statistics");
    println!("  /quit, /q      Exit search");
}

fn show_facets(searcher: &tantivy::Searcher, _category_text_field: &Field) -> anyhow::Result<()> {
    println!("📊 Available Facets");
    println!("===================");
    
    // Use FacetCollector to get all facets efficiently
    let mut facet_collector = FacetCollector::for_field("category");
    facet_collector.add_facet(Facet::root());
    
    let all_query = tantivy::query::AllQuery;
    let facet_counts = searcher.search(&all_query, &facet_collector)?;
    
    // Get facets from the root facet
    let root_facet = Facet::root();
    for (facet, count) in facet_counts.get(&root_facet.to_string()) {
        let facet_str = facet.to_string();
        let display_facet = if facet_str.starts_with("/") {
            &facet_str[1..] // Remove leading slash
        } else {
            &facet_str
        };
        println!("  {}: {} documents", display_facet, count);
    }
    
    Ok(())
}

fn show_stats(searcher: &tantivy::Searcher) {
    println!("📈 Index Statistics");
    println!("==================");
    println!("  Total documents: {}", searcher.num_docs());
    println!("  Index size: {} MB", estimate_index_size(searcher));
}

fn estimate_index_size(searcher: &tantivy::Searcher) -> f64 {
    // Rough estimation - in practice you'd want to check actual file sizes
    searcher.num_docs() as f64 * 0.001 // Assume ~1KB per document
}
