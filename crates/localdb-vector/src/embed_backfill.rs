use anyhow::{Result, anyhow};
use lancedb::Connection;
use arrow_array::{RecordBatch, RecordBatchIterator, StringArray, FixedSizeListArray};
use futures::TryStreamExt;
use lancedb::query::ExecutableQuery;
use std::collections::HashSet;
use std::sync::Arc;
use chrono::Utc;

use crate::embed_provider::EmbedProvider;
use crate::cache::{get_many as cache_get_many, put_many as cache_put_many, CacheEntry};
use crate::schema::{build_embeddings_schema, EMBEDDING_DIM};

fn hash_content(s: &str) -> String {
    let h = blake3::hash(s.as_bytes());
    h.to_hex().to_string()
}

pub async fn backfill_embeddings(
    conn: &Connection,
    docs_table: &str,
    emb_table: &str,
    cache_table: &str,
    provider: &dyn EmbedProvider,
    batch_size: usize,
    limit_rows: Option<usize>,
) -> Result<usize> {
    let t = conn.open_table(docs_table).execute().await?;
    let mut processed = 0usize;
    let mut to_process: Vec<(String, String, String)> = Vec::new();
    // Scan documents and collect (id, content, content_hash, embedding_status)
    let mut stream = t.query().execute().await?;
    while let Some(batch) = futures::TryStreamExt::try_next(&mut stream).await? {
        let id_col = batch.column_by_name("id").and_then(|c| c.as_any().downcast_ref::<StringArray>()).ok_or_else(|| anyhow!("missing id"))?;
        let content_col = batch.column_by_name("content").and_then(|c| c.as_any().downcast_ref::<StringArray>()).ok_or_else(|| anyhow!("missing content"))?;
        let status_col = batch.column_by_name("embedding_status").and_then(|c| c.as_any().downcast_ref::<StringArray>());
        for i in 0..batch.num_rows() {
            let id = id_col.value(i).to_string();
            let content = content_col.value(i).to_string();
            let chash = hash_content(&content);
            // Select rows that are not ready
            let take = match status_col { Some(sc) => sc.value(i) != "ready", None => true };
            if take { to_process.push((id, content, chash)); }
            if let Some(lim) = limit_rows { if to_process.len() >= lim { break; } }
        }
        if let Some(lim) = limit_rows { if to_process.len() >= lim { break; } }
    }
    if to_process.is_empty() { return Ok(0); }

    // Ensure embeddings table exists
    super::table::ensure_embeddings_table(conn, emb_table).await?;
    super::table::ensure_cache_table(conn, cache_table).await?;
    let emb = conn.open_table(emb_table).execute().await?;

    // Process in batches
    for chunk in to_process.chunks(batch_size) {
        // Mark in_progress for this chunk
        let ids_list = chunk.iter().map(|(id,_,_)| format!("'{}'", id.replace("'","''"))).collect::<Vec<_>>().join(",");
        let filter = format!("id IN ({})", ids_list);
        let _ = t.update()
            .only_if(filter.clone())
            .column("embedding_status", "'in_progress'")
            .execute().await?;
        // Cache lookup
        let hashes: Vec<String> = chunk.iter().map(|(_,_,h)| h.clone()).collect();
        let cache_map = cache_get_many(conn, cache_table, provider.embedder_id(), &hashes).await?;
        // Build embed inputs for misses
        let mut texts = Vec::new();
        let mut miss_indices = Vec::new();
        for (idx, (_id, content, h)) in chunk.iter().enumerate() {
            if !cache_map.contains_key(h) { texts.push(content.clone()); miss_indices.push(idx); }
        }
        let mut new_cache_entries = Vec::new();
        let mut vectors: Vec<Vec<f32>> = vec![Vec::new(); chunk.len()];
        // Hits
        for (i, (_id, _content, h)) in chunk.iter().enumerate() {
            if let Some(v) = cache_map.get(h) { vectors[i] = v.clone(); }
        }
        // Misses
        if !texts.is_empty() {
            match provider.embed_batch(&texts) {
                Ok(embs) => {
                    if embs.len() != texts.len() { return Err(anyhow!("embedder returned wrong count")); }
                    for (j, &i) in miss_indices.iter().enumerate() {
                        let v = &embs[j];
                        if v.len() != EMBEDDING_DIM as usize { return Err(anyhow!("dim mismatch: got {} expected {}", v.len(), EMBEDDING_DIM)); }
                        vectors[i] = v.clone();
                        new_cache_entries.push(CacheEntry { content_hash: chunk[i].2.clone(), embedder_id: provider.embedder_id().to_string(), vector: v.clone() });
                    }
                }
                Err(e) => {
                    // Mark errors and continue
                    let err = format!("{}", e);
                    let ids_err = miss_indices.iter().map(|&i| format!("'{}'", chunk[i].0.replace("'","''"))).collect::<Vec<_>>().join(",");
                    let filter_err = format!("id IN ({})", ids_err);
                    let _ = t.update().only_if(filter_err)
                        .column("embedding_status", "'error'")
                        .column("embedding_error", format!("'{}'", err.replace("'","''")))
                        .execute().await?;
                    // Skip writing embeddings for this batch
                    continue;
                }
            }
        }
        // Write new cache entries
        if !new_cache_entries.is_empty() { cache_put_many(conn, cache_table, &new_cache_entries).await?; }

        // Write to embeddings table
        let schema = build_embeddings_schema();
        let mut ids = Vec::new();
        let mut eids = Vec::new();
        let mut hashes = Vec::new();
        let mut times = Vec::new();
        let mut vecs: Vec<Option<Vec<Option<f32>>>> = Vec::new();
        let now = Utc::now().timestamp_millis();
        for i in 0..chunk.len() {
            ids.push(chunk[i].0.clone());
            eids.push(provider.embedder_id().to_string());
            hashes.push(chunk[i].2.clone());
            times.push(now);
            vecs.push(Some(vectors[i].iter().map(|&x| Some(x)).collect()));
        }
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(ids)),
                Arc::new(StringArray::from(eids)),
                Arc::new(StringArray::from(hashes)),
                Arc::new(arrow_array::TimestampMillisecondArray::from(times)),
                Arc::new(FixedSizeListArray::from_iter_primitive::<arrow_array::types::Float32Type, _, _>(vecs.into_iter(), EMBEDDING_DIM)),
            ],
        )?;
        let reader = Box::new(RecordBatchIterator::new(vec![Ok(batch)].into_iter(), schema));
        emb.add(reader).execute().await?;
        // Mark ready for all processed ids
        let now = Utc::now().timestamp_millis();
        let _ = t.update().only_if(filter)
            .column("embedding_status", "'ready'")
            .column("embedding_error", "NULL")
            .column("embedding_version", "embedding_version + 1")
            .column("embedded_at", format!("CAST({} AS TIMESTAMP)", now))
            .column("content_hash", "content_hash")
            .execute().await?;
        processed += chunk.len();
    }

    Ok(processed)
}
//! Resumable embedding backfill into side `embeddings` with write-through cache.
//!
//! Selection is status-driven: `embedding_status != 'ready'`. For each batch we
//! mark rows `in_progress`, consult the cache, embed misses, write to
//! `embeddings` + cache, and finally mark rows `ready` (or `error`).
