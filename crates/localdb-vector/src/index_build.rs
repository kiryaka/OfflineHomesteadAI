use anyhow::Result;
use lancedb::{Connection, index::{Index, vector::IvfPqIndexBuilder}};
use lancedb::DistanceType;
use lancedb::query::{ExecutableQuery, QueryBase, Select};
use arrow_array::Array;
use arrow_array::{RecordBatch, RecordBatchIterator, StringArray, FixedSizeListArray};
use arrow_array::cast::AsArray;
use std::sync::Arc;

use crate::schema::{EMBEDDING_DIM};
use crate::table::{set_meta, ensure_meta_table};

pub struct IvfPqParams {
    pub nlist: usize,
    pub m: usize,
    pub nbits: usize,
}

pub async fn count_ready_vectors(conn: &Connection, docs_table: &str) -> Result<usize> {
    let tbl = conn.open_table(docs_table).execute().await?;
    let mut cnt = 0usize;
    let mut stream = tbl.query().select(Select::columns(&["vector"])).execute().await?;
    while let Some(batch) = futures::TryStreamExt::try_next(&mut stream).await? {
        if let Some(arr) = batch.column_by_name("vector") {
            if let Some(fsl) = arr.as_any().downcast_ref::<FixedSizeListArray>() {
                for i in 0..batch.num_rows() { if fsl.is_valid(i) { cnt += 1; } }
            }
        }
    }
    Ok(cnt)
}

pub fn compute_ivfpq_params(total_ready: usize, dim: usize) -> IvfPqParams {
    let sqrt_n = (total_ready as f64).sqrt() as usize;
    let mut nlist = std::cmp::max(2048, 2 * sqrt_n);
    nlist = std::cmp::min(nlist, 65536);
    // Clamp nlist to be less than total_ready for tiny datasets
    if total_ready > 1 {
        nlist = std::cmp::min(nlist, total_ready - 1);
    } else {
        nlist = 1;
    }
    let m = if dim >= 1024 { 32 } else { 16 };
    IvfPqParams { nlist, m, nbits: 8 }
}

/// Copy vectors from embeddings (for a given embedder_id) into documents.vector via merge_insert
pub async fn sync_serving_vectors_from_embeddings(
    conn: &Connection,
    docs_table: &str,
    emb_table: &str,
    embedder_id: &str,
) -> Result<usize> {
    let docs = conn.open_table(docs_table).execute().await?;
    let emb = conn.open_table(emb_table).execute().await?;
    // Build a RecordBatchReader with (id, vector) for this embedder_id
    let mut src_batches: Vec<Result<RecordBatch, arrow_schema::ArrowError>> = Vec::new();
    let mut stream = emb.query().select(Select::columns(&["id","embedder_id","vector"])).execute().await?;
    while let Some(batch) = futures::TryStreamExt::try_next(&mut stream).await? {
        let eid = batch.column_by_name("embedder_id").unwrap().as_any().downcast_ref::<StringArray>().unwrap();
        let id = batch.column_by_name("id").unwrap().as_any().downcast_ref::<StringArray>().unwrap();
        let vecs = batch.column_by_name("vector").unwrap().as_any().downcast_ref::<FixedSizeListArray>().unwrap();
        let mut ids = Vec::new();
        let mut vectors: Vec<Option<Vec<Option<f32>>>> = Vec::new();
        for i in 0..batch.num_rows() {
            if eid.value(i) != embedder_id { continue; }
            ids.push(id.value(i).to_string());
            let arr = vecs.value(i);
            let vals = arr.as_primitive::<arrow_array::types::Float32Type>();
            let v = vals.values().iter().copied().map(Some).collect::<Vec<_>>();
            vectors.push(Some(v));
        }
        if !ids.is_empty() {
            let schema = Arc::new(arrow_schema::Schema::new(vec![
                arrow_schema::Field::new("id", arrow_schema::DataType::Utf8, false),
                arrow_schema::Field::new(
                    "vector",
                    arrow_schema::DataType::FixedSizeList(
                        Arc::new(arrow_schema::Field::new("item", arrow_schema::DataType::Float32, true)),
                        EMBEDDING_DIM,
                    ),
                    true,
                ),
            ]));
            let rb = RecordBatch::try_new(
                schema,
                vec![
                    Arc::new(StringArray::from(ids)),
                    Arc::new(FixedSizeListArray::from_iter_primitive::<arrow_array::types::Float32Type, _, _>(vectors.into_iter(), EMBEDDING_DIM)),
                ],
            )?;
            src_batches.push(Ok(rb));
        }
    }
    if src_batches.is_empty() { return Ok(0); }
    // Merge insert: update existing rows by id; insert all if not matched (shouldn’t happen)
    let reader = Box::new(RecordBatchIterator::new(src_batches.into_iter(), Arc::new(arrow_schema::Schema::new(vec![
        arrow_schema::Field::new("id", arrow_schema::DataType::Utf8, false),
        arrow_schema::Field::new(
            "vector",
            arrow_schema::DataType::FixedSizeList(
                Arc::new(arrow_schema::Field::new("item", arrow_schema::DataType::Float32, true)),
                EMBEDDING_DIM,
            ),
            true,
        ),
    ]))));
    let mut mi = docs.merge_insert(&["id"]);
    mi.when_matched_update_all(None).when_not_matched_insert_all();
    let res = mi.execute(reader).await?;
    Ok((res.num_inserted_rows + res.num_updated_rows) as usize)
}

pub async fn build_ivfpq_index(
    conn: &Connection,
    docs_table: &str,
    index_name: &str,
    params: &IvfPqParams,
) -> Result<()> {
    let table = conn.open_table(docs_table).execute().await?;
    table
        .create_index(
            &["vector"],
            Index::IvfPq(
                IvfPqIndexBuilder::default()
                    .distance_type(DistanceType::Cosine)
                    .num_partitions(params.nlist as u32)
                    .num_sub_vectors(params.m as u32),
            ),
        )
        .name(index_name.to_string())
        .execute()
        .await?;
    Ok(())
}

/// Very simple validation: sample up to `sample` vectors and ensure top-k returns non-empty.
pub async fn validate_index(conn: &Connection, docs_table: &str, k: usize, sample: usize) -> Result<bool> {
    let tbl = conn.open_table(docs_table).execute().await?;
    let mut stream = tbl.query().select(Select::columns(&["vector"])).limit(sample).execute().await?;
    let mut ok = 0usize;
    while let Some(batch) = futures::TryStreamExt::try_next(&mut stream).await? {
        if let Some(arr) = batch.column_by_name("vector") {
            if let Some(fsl) = arr.as_any().downcast_ref::<FixedSizeListArray>() {
                for i in 0..batch.num_rows() {
                    if !fsl.is_valid(i) { continue; }
                    let inner = fsl.value(i);
                    let vals = inner.as_primitive::<arrow_array::types::Float32Type>();
                    let q = vals.values().to_vec();
                    let mut s = tbl.vector_search(q)?.distance_type(DistanceType::Cosine).limit(k).execute().await?;
                    if let Some(rb) = futures::TryStreamExt::try_next(&mut s).await? {
                        if rb.num_rows() > 0 { ok += 1; }
                    }
                }
            }
        }
    }
    Ok(ok > 0)
}

/// Flip active index pointer in meta table (keyed by docs table name)
pub async fn flip_active_index(conn: &Connection, docs_table: &str, index_id: &str) -> Result<()> {
    // Store in a global meta table named "meta"
    ensure_meta_table(conn, "meta").await?;
    let key = format!("active_index_id:{}", docs_table);
    set_meta(conn, "meta", &key, index_id).await
}
//! Training/build/flip utilities for IVF_PQ indices in Lance.
//!
//! Typical flow:
//! 1) Copy vectors from `embeddings` to `documents.vector` for the target `embedder_id`
//! 2) Compute params based on ready rows; build IVF_PQ under a unique name
//! 3) Validate on a tiny sample; flip the active index pointer in `meta`
