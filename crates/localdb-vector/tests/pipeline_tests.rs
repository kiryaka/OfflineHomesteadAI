use std::path::PathBuf;

use localdb_core::types::DocumentChunk;
use localdb_vector::embed_provider::EmbedProvider;
use arrow_array::{RecordBatch, RecordBatchIterator, StringArray, Int32Array, FixedSizeListArray, TimestampMillisecondArray};
use std::sync::Arc;
use localdb_vector::schema::build_arrow_schema;

fn blake3_hash(s: &str) -> String { blake3::hash(s.as_bytes()).to_hex().to_string() }

#[tokio::test]
async fn backfill_and_sync_in_memory_fast() -> anyhow::Result<()> {
    std::env::set_var("APP_USE_FAKE_EMBEDDINGS", "1");

    let tmp = tempfile::tempdir()?;
    let db_uri = tmp.path().to_string_lossy().to_string();
    let docs_table = "documents";
    let emb_table = "embeddings";
    let cache_table = "emb_cache";

    // 1) Seed a small number of chunks with empty vectors by creating the documents table directly
    let n = 32usize;
    let chunks: Vec<DocumentChunk> = (0..n)
        .map(|i| DocumentChunk {
            id: format!("doc:{}", i),
            doc_id: format!("doc:{}", i),
            doc_path: format!("/tmp/doc{}.txt", i),
            category: "/test".to_string(),
            category_text: "/test".to_string(),
            content: format!("hello world {}", i),
            chunk_index: i as usize,
            total_chunks: n,
        })
        .collect();
    let conn = localdb_vector::table::open_db(&db_uri).await?;
    let schema = build_arrow_schema();
    let mut ids = Vec::new();
    let mut doc_ids = Vec::new();
    let mut doc_paths = Vec::new();
    let mut categories = Vec::new();
    let mut category_texts = Vec::new();
    let mut contents = Vec::new();
    let mut chunk_indices = Vec::new();
    let mut total_chunks = Vec::new();
    let mut vectors: Vec<Option<Vec<Option<f32>>>> = Vec::new();
    let mut content_hashes = Vec::new();
    let mut emb_status = Vec::new();
    let mut emb_error: Vec<Option<&str>> = Vec::new();
    let mut emb_version = Vec::new();
    let mut embedded_at: Vec<Option<i64>> = Vec::new();
    let mut index_status = Vec::new();
    let mut index_version = Vec::new();
    for c in &chunks {
        ids.push(c.id.clone());
        doc_ids.push(c.doc_id.clone());
        doc_paths.push(c.doc_path.clone());
        categories.push(c.category.clone());
        category_texts.push(c.category_text.clone());
        contents.push(c.content.clone());
        chunk_indices.push(c.chunk_index as i32);
        total_chunks.push(c.total_chunks as i32);
        vectors.push(None);
        content_hashes.push(blake3_hash(&c.content));
        emb_status.push("new".to_string());
        emb_error.push(None);
        emb_version.push(0);
        embedded_at.push(None);
        index_status.push("stale".to_string());
        index_version.push(0);
    }
    let rb = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(ids)),
            Arc::new(StringArray::from(doc_ids)),
            Arc::new(StringArray::from(doc_paths)),
            Arc::new(StringArray::from(categories)),
            Arc::new(StringArray::from(category_texts)),
            Arc::new(StringArray::from(contents)),
            Arc::new(Int32Array::from(chunk_indices)),
            Arc::new(Int32Array::from(total_chunks)),
            Arc::new(FixedSizeListArray::from_iter_primitive::<arrow_array::types::Float32Type, _, _>(vectors.into_iter(), localdb_vector::schema::EMBEDDING_DIM)),
            Arc::new(StringArray::from(content_hashes)),
            Arc::new(StringArray::from(emb_status)),
            Arc::new(StringArray::from(emb_error)),
            Arc::new(Int32Array::from(emb_version)),
            Arc::new(TimestampMillisecondArray::from(embedded_at)),
            Arc::new(StringArray::from(index_status)),
            Arc::new(Int32Array::from(index_version)),
        ],
    )?;
    let reader = Box::new(RecordBatchIterator::new(vec![Ok(rb)].into_iter(), schema));
    conn.create_table(docs_table, reader).execute().await?;

    // 2) Backfill via local provider into embeddings + cache
    localdb_vector::table::ensure_embeddings_table(&conn, emb_table).await?;
    localdb_vector::table::ensure_cache_table(&conn, cache_table).await?;
    let provider = localdb_vector::embed_provider::local::LocalProvider::new()?;
    let processed = localdb_vector::embed_backfill::backfill_embeddings(
        &conn,
        docs_table,
        emb_table,
        cache_table,
        &provider,
        16,
        None,
    )
    .await?;
    assert_eq!(processed, chunks.len());

    // 3) Sync serving vectors from embeddings
    let updated = localdb_vector::index_build::sync_serving_vectors_from_embeddings(
        &conn,
        docs_table,
        emb_table,
        provider.embedder_id(),
    )
    .await?;
    assert!(updated >= chunks.len());

    Ok(())
}

/// Slow end-to-end test that exercises PQ index training and build.
/// Ignored by default to keep CI fast; run explicitly when needed:
/// `APP_USE_FAKE_EMBEDDINGS=1 cargo test -p localdb-vector --test pipeline_tests -- --ignored`
#[ignore]
#[tokio::test]
async fn backfill_and_build_index_in_memory_slow() -> anyhow::Result<()> {
    std::env::set_var("APP_USE_FAKE_EMBEDDINGS", "1");
    let tmp = tempfile::tempdir()?;
    let db_uri = tmp.path().to_string_lossy().to_string();
    let docs_table = "documents";
    let emb_table = "embeddings";
    let cache_table = "emb_cache";

    // Seed enough rows for PQ training
    let n = 300usize;
    let chunks: Vec<DocumentChunk> = (0..n)
        .map(|i| DocumentChunk {
            id: format!("doc:{}", i),
            doc_id: format!("doc:{}", i),
            doc_path: format!("/tmp/doc{}.txt", i),
            category: "/test".to_string(),
            category_text: "/test".to_string(),
            content: format!("hello world {}", i),
            chunk_index: i as usize,
            total_chunks: n,
        })
        .collect();
    let conn = localdb_vector::table::open_db(&db_uri).await?;
    let schema = localdb_vector::schema::build_arrow_schema();
    use arrow_array::{RecordBatch, RecordBatchIterator, StringArray, Int32Array, FixedSizeListArray, TimestampMillisecondArray};
    use std::sync::Arc;
    let (mut ids, mut doc_ids, mut doc_paths, mut cats, mut cat_txts, mut contents, mut idxs, mut totals) = (Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new());
    let (mut vectors, mut hashes, mut emb_status, mut emb_err, mut emb_ver, mut emb_at, mut idx_status, mut idx_ver) = (Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new(),Vec::new());
    for c in &chunks {
        ids.push(c.id.clone()); doc_ids.push(c.doc_id.clone()); doc_paths.push(c.doc_path.clone());
        cats.push(c.category.clone()); cat_txts.push(c.category_text.clone()); contents.push(c.content.clone());
        idxs.push(c.chunk_index as i32); totals.push(c.total_chunks as i32);
        vectors.push(None);
        hashes.push(blake3::hash(c.content.as_bytes()).to_hex().to_string());
        emb_status.push("new".to_string()); emb_err.push(None::<&str>); emb_ver.push(0); emb_at.push(None::<i64>);
        idx_status.push("stale".to_string()); idx_ver.push(0);
    }
    let rb = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(ids)), Arc::new(StringArray::from(doc_ids)), Arc::new(StringArray::from(doc_paths)),
            Arc::new(StringArray::from(cats)), Arc::new(StringArray::from(cat_txts)), Arc::new(StringArray::from(contents)),
            Arc::new(Int32Array::from(idxs)), Arc::new(Int32Array::from(totals)),
            Arc::new(FixedSizeListArray::from_iter_primitive::<arrow_array::types::Float32Type, _, _>(vectors.into_iter(), localdb_vector::schema::EMBEDDING_DIM)),
            Arc::new(StringArray::from(hashes)), Arc::new(StringArray::from(emb_status)), Arc::new(StringArray::from(emb_err)),
            Arc::new(Int32Array::from(emb_ver)), Arc::new(TimestampMillisecondArray::from(emb_at)),
            Arc::new(StringArray::from(idx_status)), Arc::new(Int32Array::from(idx_ver)),
        ],
    )?;
    let reader = Box::new(RecordBatchIterator::new(vec![Ok(rb)].into_iter(), schema));
    conn.create_table(docs_table, reader).execute().await?;

    localdb_vector::table::ensure_embeddings_table(&conn, emb_table).await?;
    localdb_vector::table::ensure_cache_table(&conn, cache_table).await?;
    let provider = localdb_vector::embed_provider::local::LocalProvider::new()?;
    let processed = localdb_vector::embed_backfill::backfill_embeddings(&conn, docs_table, emb_table, cache_table, &provider, 64, None).await?;
    assert_eq!(processed, chunks.len());
    let updated = localdb_vector::index_build::sync_serving_vectors_from_embeddings(&conn, docs_table, emb_table, provider.embedder_id()).await?;
    assert!(updated >= chunks.len());

    let ready = localdb_vector::index_build::count_ready_vectors(&conn, docs_table).await?;
    let params = localdb_vector::index_build::compute_ivfpq_params(ready, provider.dim());
    let index_name = format!("ivfpq-test-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
    localdb_vector::index_build::build_ivfpq_index(&conn, docs_table, &index_name, &params).await?;
    let ok = localdb_vector::index_build::validate_index(&conn, docs_table, 5, 5).await?;
    assert!(ok);
    localdb_vector::index_build::flip_active_index(&conn, docs_table, &index_name).await?;
    let active = localdb_vector::table::get_meta(&conn, "meta", &format!("active_index_id:{}", docs_table)).await?;
    assert_eq!(active.as_deref(), Some(index_name.as_str()));
    Ok(())
}
