use std::sync::Arc;

use lancedb::query::{ExecutableQuery, IntoQueryVector, QueryBase};
use lancedb::Connection;

use futures::TryStreamExt;

use crate::services::brain::memory::{MemoryManagerServiceError, MemoryRecord};
use crate::utils::{batches_to_memory_records, memory_record_schema, memory_record_to_batches};

const WARM_MEMORY_TABLE: &str = "warm_memory";
const WARM_MEMORY_MAX_CAPACITY: usize = 500;

#[derive(Clone)]
pub struct WarmMemoryService {
    pub connection: Arc<Connection>,
}

impl WarmMemoryService {
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
            .create_empty_table(WARM_MEMORY_TABLE, schema)
            .execute()
            .await?;

        Ok(())
    }

    pub async fn is_collection_exists(&self) -> Result<bool, MemoryManagerServiceError> {
        let table_names = self.connection.table_names().execute().await?;
        Ok(table_names.contains(&WARM_MEMORY_TABLE.to_string()))
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

        let table = self.connection.open_table(WARM_MEMORY_TABLE).execute().await?;

        let batches = memory_record_to_batches(row)?;
        table.add(batches).execute().await?;

        Ok(())
    }

    pub async fn record_exists(&self, record_id: &str) -> Result<bool, MemoryManagerServiceError> {
        let table = self.connection.open_table(WARM_MEMORY_TABLE).execute().await?;

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
        let table = self.connection.open_table(WARM_MEMORY_TABLE).execute().await?;
        table.delete(&format!("id = '{}'", record_id)).await?;
        Ok(())
    }

    pub async fn search(
        &self,
        query_vector: impl IntoQueryVector,
        limit: usize,
    ) -> Result<Vec<MemoryRecord>, MemoryManagerServiceError> {
        let table = self.connection.open_table(WARM_MEMORY_TABLE).execute().await?;

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

    pub async fn count_records(&self) -> Result<usize, MemoryManagerServiceError> {
        let table = self.connection.open_table(WARM_MEMORY_TABLE).execute().await?;
        let count = table.count_rows(None).await?;
        Ok(count)
    }

    pub async fn get_all_records(&self) -> Result<Vec<MemoryRecord>, MemoryManagerServiceError> {
        let table = self.connection.open_table(WARM_MEMORY_TABLE).execute().await?;
        let batches = table.query().execute().await?.try_collect::<Vec<_>>().await?;

        let records = batches_to_memory_records(batches)?;

        Ok(records)
    }

    pub async fn get_oldest_records(&self, limit: usize) -> Result<Vec<MemoryRecord>, MemoryManagerServiceError> {
        let table = self.connection.open_table(WARM_MEMORY_TABLE).execute().await?;

        let batches = table
            .query()
            .limit(WARM_MEMORY_MAX_CAPACITY)
            .execute()
            .await?
            .try_collect::<Vec<_>>()
            .await?;

        let mut records = batches_to_memory_records(batches)?;

        records.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        records.truncate(limit);

        Ok(records)
    }

    pub async fn delete_oldest_records(&self, count: usize) -> Result<(), MemoryManagerServiceError> {
        let oldest = self.get_oldest_records(count).await?;
        let table = self.connection.open_table(WARM_MEMORY_TABLE).execute().await?;

        for record in oldest {
            table.delete(&format!("id = '{}'", record.id)).await?;
        }

        Ok(())
    }

    pub async fn get_promotion_candidates(&self, limit: usize) -> Result<Vec<MemoryRecord>, MemoryManagerServiceError> {
        let table = self.connection.open_table(WARM_MEMORY_TABLE).execute().await?;
        let batches = table
            .query()
            .only_if("access_count >= 3")
            .limit(limit)
            .execute()
            .await?
            .try_collect::<Vec<_>>()
            .await?;

        let records = batches_to_memory_records(batches)?;

        Ok(records)
    }
}
