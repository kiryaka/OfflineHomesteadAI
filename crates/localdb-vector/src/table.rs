use anyhow::Result;
use lancedb::{connect, Connection};

use arrow_array::{RecordBatch, RecordBatchIterator, StringArray, TimestampMillisecondArray};
use std::sync::Arc;
use chrono::Utc;
use lancedb::query::{QueryBase, ExecutableQuery};

use crate::schema::{build_embeddings_schema, build_cache_schema};

pub async fn open_db(uri: &str) -> Result<Connection> {
    Ok(connect(uri).execute().await?)
}

pub async fn ensure_table(conn: &Connection, name: &str, schema: Arc<arrow_schema::Schema>) -> Result<()> {
    let names = conn.table_names().execute().await?;
    if names.contains(&name.to_string()) {
        return Ok(());
    }
    // create empty table with 0 rows
    let iter = RecordBatchIterator::new(vec![].into_iter(), schema.clone());
    conn.create_table(name, Box::new(iter)).execute().await?;
    Ok(())
}

pub async fn ensure_embeddings_table(conn: &Connection, name: &str) -> Result<()> {
    ensure_table(conn, name, build_embeddings_schema()).await
}

pub async fn ensure_cache_table(conn: &Connection, name: &str) -> Result<()> {
    ensure_table(conn, name, build_cache_schema()).await
}

// Simple key/value meta table management for active index pointers and job state
fn build_meta_schema() -> Arc<arrow_schema::Schema> {
    Arc::new(arrow_schema::Schema::new(vec![
        arrow_schema::Field::new("key", arrow_schema::DataType::Utf8, false),
        arrow_schema::Field::new("value", arrow_schema::DataType::Utf8, false),
        arrow_schema::Field::new("updated_at", arrow_schema::DataType::Timestamp(arrow_schema::TimeUnit::Millisecond, None), false),
    ]))
}

pub async fn ensure_meta_table(conn: &Connection, name: &str) -> Result<()> {
    ensure_table(conn, name, build_meta_schema()).await
}

pub async fn set_meta(conn: &Connection, table: &str, key: &str, value: &str) -> Result<()> {
    ensure_meta_table(conn, table).await?;
    let t = conn.open_table(table).execute().await?;
    let rb = RecordBatch::try_new(
        build_meta_schema(),
        vec![
            Arc::new(StringArray::from(vec![key.to_string()])),
            Arc::new(StringArray::from(vec![value.to_string()])),
            Arc::new(TimestampMillisecondArray::from(vec![Utc::now().timestamp_millis()])),
        ],
    )?;
    let reader = Box::new(RecordBatchIterator::new(vec![Ok(rb)].into_iter(), build_meta_schema()));
    // Upsert behavior via merge_insert: key is unique
    let mut mi = t.merge_insert(&["key"]);
    mi.when_matched_update_all(None).when_not_matched_insert_all();
    let _ = mi.execute(reader).await?;
    Ok(())
}

pub async fn get_meta(conn: &Connection, table: &str, key: &str) -> Result<Option<String>> {
    let names = conn.table_names().execute().await?;
    if !names.contains(&table.to_string()) { return Ok(None); }
    let t = conn.open_table(table).execute().await?;
    let mut stream = t.query().only_if(&format!("key = '{}'", key.replace("'","''"))).execute().await?;
    while let Some(batch) = futures::TryStreamExt::try_next(&mut stream).await? {
        if batch.num_rows() == 0 { continue; }
        let val = batch.column_by_name("value").and_then(|c| c.as_any().downcast_ref::<StringArray>()).ok_or_else(|| anyhow::anyhow!("meta.value column missing"))?;
        return Ok(Some(val.value(0).to_string()));
    }
    Ok(None)
}
//! LanceDB connection and housekeeping helpers.
//!
//! Provides database open functions, ensure-* helpers for tables, and a simple
//! key/value metadata table used to store pointers such as the active index id.
