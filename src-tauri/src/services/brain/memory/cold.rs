use futures::TryStreamExt;
use std::sync::Arc;

use lancedb::query::{ExecutableQuery, IntoQueryVector, QueryBase};
use lancedb::Connection;

use crate::services::brain::memory::{MemoryManagerServiceError, MemoryRecord};
use crate::utils::{batches_to_memory_records, memory_record_schema, memory_record_to_batches};

const COLD_MEMORY_TABLE: &str = "cold_memory";

#[derive(Clone)]
pub struct ColdMemoryService {
    pub connection: Arc<Connection>,
}

impl ColdMemoryService {
    pub async fn new(connection: Arc<Connection>) -> Result<Self, MemoryManagerServiceError> {
        let service = Self { connection };

        if !service.is_collection_exists().await? {
            service.create_collection().await?;
        }

        Ok(service)
    }

    pub async fn create_collection(&self) -> Result<(), MemoryManagerServiceError> {
        if self.is_collection_exists().await? {
            return Ok(());
        }

        let schema = memory_record_schema();

        self.connection
            .create_empty_table(COLD_MEMORY_TABLE, schema)
            .execute()
            .await?;

        Ok(())
    }

    pub async fn is_collection_exists(&self) -> Result<bool, MemoryManagerServiceError> {
        let table_names = self.connection.table_names().execute().await?;
        Ok(table_names.contains(&COLD_MEMORY_TABLE.to_string()))
    }

    pub async fn upsert_record(&self, record: MemoryRecord) -> Result<(), MemoryManagerServiceError> {
        let row = MemoryRecord {
            id: record.id,
            vector: record.vector,
            metadata: record.metadata,
            created_at: record.created_at,
            updated_at: record.updated_at,
            last_accessed_at: record.last_accessed_at,
            access_count: record.access_count,
        };

        let table = self.connection.open_table(COLD_MEMORY_TABLE).execute().await?;

        let batches = memory_record_to_batches(row)?;
        table.add(batches).execute().await?;

        Ok(())
    }

    pub async fn record_exists(&self, record_id: &str) -> Result<bool, MemoryManagerServiceError> {
        let table = self.connection.open_table(COLD_MEMORY_TABLE).execute().await?;

        let batches = table
            .query()
            .only_if(&format!("id = '{}'", record_id))
            .limit(1)
            .execute()
            .await?
            .try_collect::<Vec<_>>()
            .await?;

        Ok(!batches.is_empty() && batches[0].num_rows() > 0)
    }

    pub async fn delete_record(&self, record_id: &str) -> Result<(), MemoryManagerServiceError> {
        let table = self.connection.open_table(COLD_MEMORY_TABLE).execute().await?;
        table.delete(&format!("id = '{}'", record_id)).await?;
        Ok(())
    }

    pub async fn search(
        &self,
        query_vector: impl IntoQueryVector,
        limit: usize,
    ) -> Result<Vec<MemoryRecord>, MemoryManagerServiceError> {
        let table = self.connection.open_table(COLD_MEMORY_TABLE).execute().await?;

        let batches = table
            .query()
            .nearest_to(query_vector)?
            .limit(limit)
            .execute()
            .await?
            .try_collect::<Vec<_>>()
            .await?;

        let records = batches_to_memory_records(batches)?;

        Ok(records)
    }

    pub async fn get_all_records(&self) -> Result<Vec<MemoryRecord>, MemoryManagerServiceError> {
        let table = self.connection.open_table(COLD_MEMORY_TABLE).execute().await?;
        let batches = table.query().execute().await?.try_collect::<Vec<_>>().await?;

        let records = batches_to_memory_records(batches)?;

        Ok(records)
    }

    pub async fn get_promotion_candidates(&self, limit: usize) -> Result<Vec<MemoryRecord>, MemoryManagerServiceError> {
        let table = self.connection.open_table(COLD_MEMORY_TABLE).execute().await?;
        let batches = table
            .query()
            .only_if("access_count >= 2")
            .limit(limit)
            .execute()
            .await?
            .try_collect::<Vec<_>>()
            .await?;

        let records = batches_to_memory_records(batches)?;

        Ok(records)
    }
}
