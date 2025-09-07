use anyhow::Result;
use lancedb::Connection;
use lancedb::query::ExecutableQuery;
use futures::TryStreamExt;
use arrow_array::{RecordBatch, RecordBatchIterator, StringArray, FixedSizeListArray};
use arrow_array::cast::AsArray;
use std::collections::HashMap;
use std::sync::Arc;
use chrono::Utc;

use crate::schema::{build_cache_schema, EMBEDDING_DIM};

#[derive(Clone, Debug)]
pub struct CacheEntry {
    pub content_hash: String,
    pub embedder_id: String,
    pub vector: Vec<f32>,
}

pub async fn get_many(
    conn: &Connection,
    table: &str,
    embedder_id: &str,
    hashes: &[String],
) -> Result<HashMap<String, Vec<f32>>> {
    let names = conn.table_names().execute().await?;
    if !names.contains(&table.to_string()) { return Ok(HashMap::new()); }
    let t = conn.open_table(table).execute().await?;
    // naive scan; TODO: add predicate pushdown when API allows
    let mut out = HashMap::new();
    let mut stream = t.query().execute().await?;
    while let Some(batch) = futures::TryStreamExt::try_next(&mut stream).await? {
        let hash_col = batch
            .column_by_name("content_hash")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .expect("content_hash col");
        let eid_col = batch
            .column_by_name("embedder_id")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .expect("embedder_id col");
        let vec_col = batch
            .column_by_name("vector")
            .and_then(|c| c.as_any().downcast_ref::<FixedSizeListArray>())
            .expect("vector col");
        for i in 0..batch.num_rows() {
            let h = hash_col.value(i);
            if eid_col.value(i) != embedder_id { continue; }
            if !hashes.iter().any(|x| x == h) { continue; }
            let list = vec_col.value(i);
            let vals = list
                .as_primitive::<arrow_array::types::Float32Type>()
                .values()
                .iter()
                .copied()
                .collect::<Vec<f32>>();
            if vals.len() == EMBEDDING_DIM as usize { out.insert(h.to_string(), vals); }
        }
    }
    Ok(out)
}

pub async fn put_many(conn: &Connection, table: &str, entries: &[CacheEntry]) -> Result<()> {
    if entries.is_empty() { return Ok(()); }
    let names = conn.table_names().execute().await?;
    if !names.contains(&table.to_string()) {
        // create table
        let schema = build_cache_schema();
        let iter = RecordBatchIterator::new(vec![].into_iter(), schema.clone());
        conn.create_table(table, Box::new(iter)).execute().await?;
    }
    let t = conn.open_table(table).execute().await?;
    // Build columns
    let mut hashes = Vec::new();
    let mut eids = Vec::new();
    let mut created = Vec::new();
    let mut vectors: Vec<Option<Vec<Option<f32>>>> = Vec::new();
    let now = Utc::now().timestamp_millis();
    for e in entries {
        hashes.push(e.content_hash.clone());
        eids.push(e.embedder_id.clone());
        created.push(now);
        vectors.push(Some(e.vector.iter().map(|&x| Some(x)).collect()));
    }
    let batch = RecordBatch::try_new(
        build_cache_schema(),
        vec![
            Arc::new(StringArray::from(hashes)),
            Arc::new(StringArray::from(eids)),
            Arc::new(arrow_array::TimestampMillisecondArray::from(created)),
            Arc::new(FixedSizeListArray::from_iter_primitive::<arrow_array::types::Float32Type, _, _>(vectors.into_iter(), EMBEDDING_DIM)),
        ],
    )?;
    let reader = Box::new(RecordBatchIterator::new(vec![Ok(batch)].into_iter(), build_cache_schema()));
    t.add(reader).execute().await?;
    Ok(())
}
//! Lance-backed embedding cache keyed by `(content_hash, embedder_id)`.
//!
//! The cache is consulted prior to calling a provider and written through on
//! cache misses. This enables offline operation and reduces repeated work.
