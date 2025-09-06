use std::env;
use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, FuzzyTermQuery, Occur, QueryParser, TermQuery};
use tantivy::schema::*;
use tantivy::snippet::SnippetGenerator;
use tantivy::{Index, Term};

mod config;
use config::Config;

/// Create a hybrid fuzzy query: exact match + fuzzy match with scoring
fn create_fuzzy_query(
    query_string: &str,
    text_field: Field,
) -> anyhow::Result<Box<dyn tantivy::query::Query>> {
    // Split query into words
    let words: Vec<&str> = query_string.split_whitespace().collect();

    if words.is_empty() {
        return Err(anyhow::anyhow!("Empty query"));
    }

    // For each word, create both exact and fuzzy queries
    let mut word_queries = Vec::new();

    for word in words {
        let word_lower = word.to_lowercase();
        let word_len = word_lower.len();

        if word_len < 3 {
            // For very short words, search exactly only
            let term = Term::from_field_text(text_field, &word_lower);
            word_queries.push(Box::new(TermQuery::new(
                term,
                tantivy::schema::IndexRecordOption::Basic,
            )) as Box<dyn tantivy::query::Query>);
        } else {
            // For longer words, try exact match first, then fuzzy
            let term = Term::from_field_text(text_field, &word_lower);

            // Exact match (higher priority)
            let exact_query =
                TermQuery::new(term.clone(), tantivy::schema::IndexRecordOption::Basic);

            // Fuzzy match (lower priority, only for typos)
            let max_distance = match word_len {
                3..=4 => 1,  // 1 edit for short words
                5..=7 => 2,  // 2 edits for medium words
                8..=10 => 3, // 3 edits for long words
                _ => 4,      // 4 edits for very long words
            };
            let fuzzy_query = FuzzyTermQuery::new(term, max_distance, true);

            // Combine exact OR fuzzy (exact will score higher)
            let word_query = BooleanQuery::new(vec![
                (
                    Occur::Should,
                    Box::new(exact_query) as Box<dyn tantivy::query::Query>,
                ),
                (
                    Occur::Should,
                    Box::new(fuzzy_query) as Box<dyn tantivy::query::Query>,
                ),
            ]);

            word_queries.push(Box::new(word_query) as Box<dyn tantivy::query::Query>);
        }
    }

    // Combine all words with AND (all words must match)
    if word_queries.len() == 1 {
        Ok(word_queries.into_iter().next().unwrap())
    } else {
        Ok(Box::new(BooleanQuery::new(
            word_queries.into_iter().map(|q| (Occur::Must, q)).collect(),
        )) as Box<dyn tantivy::query::Query>)
    }
}

fn search_with_facets(
    index: &Index,
    query_string: &str,
    facets: Option<Vec<&str>>,
    limit: usize,
    fuzzy: bool,
) -> anyhow::Result<()> {
    let reader = index.reader()?;
    let searcher = reader.searcher();
    let schema = index.schema();

    let text_field = schema.get_field("text")?;
    // text_ngram_field removed - using FuzzyTermQuery instead
    let doc_id_field = schema.get_field("doc_id")?;
    let category_field = schema.get_field("category")?;
    let category_text_field = schema.get_field("category_text")?;
    let id_field = schema.get_field("id")?;

    // Parse the base query
    let query_parser = QueryParser::for_index(index, vec![text_field, doc_id_field]);
    let base_query = if fuzzy {
        // For fuzzy search, use edit distance on the regular text field
        create_fuzzy_query(query_string, text_field)?
    } else {
        query_parser.parse_query(query_string)?
    };

    // Create snippet generator for context highlighting
    // Always use the regular text field for snippets (user-friendly display)
    let snippet_generator = SnippetGenerator::create(&searcher, &*base_query, text_field)?;

    // Build final query
    let final_query = if let Some(ref facet_list) = facets {
        // Create facet queries for each specified facet
        let facet_queries: Vec<Box<dyn tantivy::query::Query>> = facet_list
            .iter()
            .map(|facet_prefix| {
                let facet = Facet::from(&format!("/{facet_prefix}"));
                Box::new(TermQuery::new(
                    Term::from_facet(category_field, &facet),
                    tantivy::schema::IndexRecordOption::Basic,
                )) as Box<dyn tantivy::query::Query>
            })
            .collect();

        // Combine: (text_query) AND (any_of_the_facets)
        Box::new(BooleanQuery::new(vec![
            (Occur::Must, base_query),
            (
                Occur::Should,
                Box::new(BooleanQuery::new(
                    facet_queries
                        .into_iter()
                        .map(|q| (Occur::Should, q))
                        .collect(),
                )),
            ),
        ])) as Box<dyn tantivy::query::Query>
    } else {
        // No facet filtering - unified search
        base_query
    };

    // Execute search
    let top_docs = searcher.search(&final_query, &TopDocs::with_limit(limit))?;

    println!("Query: '{}'", query_string);
    if fuzzy {
        println!("Mode: fuzzy search (typo tolerant)");
    }
    if let Some(facet_list) = facets {
        println!("Facets: {}", facet_list.join(", "));
    } else {
        println!("Facets: (unified search)");
    }
    println!("Found {} results:", top_docs.len());
    println!();

    // Count facets in the displayed results
    let mut facet_counts = std::collections::HashMap::new();

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

        // Use the full facet path for counting (e.g., /agriculture, /foraging)
        let facet_key = if category.starts_with("/") {
            &category[1..] // Remove leading slash
        } else {
            category
        };
        *facet_counts.entry(facet_key.to_string()).or_insert(0) += 1;

        // Generate snippet showing context where query terms appear
        let snippet = snippet_generator.snippet_from_doc(&document);
        let snippet_text = snippet.to_html();

        println!(
            "  📄 score={:.4} | id={} | category={}",
            score, id, category
        );
        println!("      🎯 {}", snippet_text);
        println!();
    }

    println!(
        "📊 Results breakdown (out of {}):",
        facet_counts.values().sum::<i32>()
    );
    for (facet, count) in facet_counts {
        println!("  {}: {} docs", facet, count);
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = Config::load().unwrap_or_else(|_| {
        println!("Warning: Could not load config.toml, using defaults");
        Config::default()
    });

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!(
            "Usage: {} [-q <query>] [-f <facet1> <facet2> ...] [-n <number>] [--fuzzy]",
            args[0]
        );
        eprintln!("Examples:");
        eprintln!("  {} -q 'love OR war'", args[0]);
        eprintln!("  {} -q 'coffee' -f agriculture foraging", args[0]);
        eprintln!("  {} -q 'survival' -f survival/gear -n 10", args[0]);
        eprintln!("  {} -q 'coffe' --fuzzy", args[0]);
        std::process::exit(1);
    }

    let index_dir = config.get_tantivy_index_dir();
    let mut query_string = String::new();
    let mut facets: Option<Vec<&str>> = None;
    let mut limit = 5; // default limit
    let mut fuzzy = false;
    let mut i = 1;

    // Parse command line arguments
    while i < args.len() {
        match args[i].as_str() {
            "-q" => {
                if i + 1 < args.len() {
                    query_string = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("Error: -q requires a query string");
                    std::process::exit(1);
                }
            }
            "-f" => {
                let mut facet_list = Vec::new();
                i += 1;
                while i < args.len() && !args[i].starts_with('-') {
                    facet_list.push(args[i].as_str());
                    i += 1;
                }
                facets = Some(facet_list);
            }
            "-n" => {
                if i + 1 < args.len() {
                    limit = args[i + 1].parse().unwrap_or_else(|_| {
                        eprintln!("Error: -n requires a valid number");
                        std::process::exit(1);
                    });
                    i += 2;
                } else {
                    eprintln!("Error: -n requires a number");
                    std::process::exit(1);
                }
            }
            "--fuzzy" => {
                fuzzy = true;
                i += 1;
            }
            _ => {
                eprintln!("Error: Unknown argument '{}'", args[i]);
                std::process::exit(1);
            }
        }
    }

    if query_string.is_empty() {
        eprintln!("Error: Query string is required (-q)");
        std::process::exit(1);
    }

    // Open the index
    let index = Index::open_in_dir(&index_dir)?;

    // Run search
    search_with_facets(&index, &query_string, facets, limit, fuzzy)?;

    Ok(())
}
