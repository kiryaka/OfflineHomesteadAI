use tantivy::schema::{Schema, TextFieldIndexing, TextOptions, IndexRecordOption, FacetOptions, STRING, STORED};
use tantivy::tokenizer::{TextAnalyzer, SimpleTokenizer, LowerCaser, StopWordFilter};
use tantivy::Index;

pub fn build_schema() -> Schema {
	let mut schema_builder = Schema::builder();
	let _id_field = schema_builder.add_text_field("id", STRING | STORED);
	let _doc_id_field = schema_builder.add_text_field("doc_id", STRING | STORED);
	let _doc_path_field = schema_builder.add_text_field("doc_path", STRING | STORED);
	let text_field_indexing = TextFieldIndexing::default().set_tokenizer("text_with_stopwords").set_index_option(IndexRecordOption::WithFreqsAndPositions);
	let text_options = TextOptions::default().set_indexing_options(text_field_indexing).set_stored();
	let _text_field = schema_builder.add_text_field("text", text_options);
	let _category_field = schema_builder.add_facet_field("category", FacetOptions::default());
	let _category_text_field = schema_builder.add_text_field("category_text", STRING | STORED);
	schema_builder.build()
}

pub fn register_tokenizer(index: &Index) {
	let stop_words = vec![
		"a","an","and","are","as","at","be","by","for","from","has","he","in","is","it","its","of","on","that","the","to","was","will","with","or","but","not","this","these","they","them","their","there","then","than","so","if","when","where","why","how","what","which","who","whom","whose","can","could","should","would","may","might","must","shall","do","does","did","have","had","having",
	];
	let tokenizer = TextAnalyzer::builder(SimpleTokenizer::default())
		.filter(LowerCaser)
		.filter(StopWordFilter::remove(stop_words.into_iter().map(|s| s.to_string())))
		.build();
	index.tokenizers().register("text_with_stopwords", tokenizer);
}