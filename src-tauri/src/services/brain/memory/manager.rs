use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use crate::services::brain::memory::{ColdMemoryService, HotMemoryService, WarmMemoryService};

#[derive(Clone, Debug)]
enum MemoryTier {
    Hot,
    Warm,
    Cold,
}

#[derive(Debug)]
pub enum MemoryManagerServiceError {
    ParseError(String),
    FsError(String),
    LanceDbError(String),
}

impl fmt::Display for MemoryManagerServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryManagerServiceError::FsError(msg) => write!(f, "File system error: {}", msg),
            MemoryManagerServiceError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            MemoryManagerServiceError::LanceDbError(msg) => write!(f, "LanceDB error (vector database): {}", msg),
        }
    }
}

impl From<std::io::Error> for MemoryManagerServiceError {
    fn from(err: std::io::Error) -> Self {
        MemoryManagerServiceError::FsError(err.to_string())
    }
}

impl From<serde_json::Error> for MemoryManagerServiceError {
    fn from(err: serde_json::Error) -> Self {
        MemoryManagerServiceError::ParseError(err.to_string())
    }
}

impl From<lancedb::Error> for MemoryManagerServiceError {
    fn from(err: lancedb::Error) -> Self {
        MemoryManagerServiceError::LanceDbError(err.to_string())
    }
}

impl From<arrow::error::ArrowError> for MemoryManagerServiceError {
    fn from(err: arrow::error::ArrowError) -> Self {
        MemoryManagerServiceError::LanceDbError(err.to_string())
    }
}

impl std::error::Error for MemoryManagerServiceError {}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryRecord {
    pub id: String,
    pub vector: Vec<f32>,
    pub metadata: HashMap<String, String>,
    pub created_at: String,
    pub updated_at: String,

    #[serde(default)]
    pub last_accessed_at: String,
    #[serde(default)]
    pub access_count: u32,
}

const COLD_TO_WARM_ACCESS_THRESHOLD: u32 = 3;
const WARM_TO_HOT_ACCESS_THRESHOLD: u32 = 5;
const RECENT_ACCESS_HOURS: i64 = 24;

#[derive(Clone)]
pub struct MemoryManagerService {
    cold_memory: ColdMemoryService,
    warm_memory: WarmMemoryService,
    pub hot_memory: HotMemoryService,
}

impl MemoryManagerService {
    pub async fn new(app_data_dir: PathBuf) -> Result<Self, MemoryManagerServiceError> {
        let memory_path = app_data_dir.join("heart_memory.lance");

        if let Some(parent) = memory_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let connection = Arc::new(lancedb::connect(&memory_path.to_string_lossy()).execute().await?);
        let cold_memory = ColdMemoryService::new(Arc::clone(&connection)).await?;
        let warm_memory = WarmMemoryService::new(Arc::clone(&connection)).await?;
        let hot_memory = HotMemoryService::new(Arc::clone(&connection)).await?;

        Ok(Self {
            cold_memory,
            warm_memory,
            hot_memory,
        })
    }

    pub async fn migrate_hot_to_warm(&mut self) -> Result<(), MemoryManagerServiceError> {
        let hot_count = self.hot_memory.count_records().await?;

        if hot_count > 50 {
            let records_to_move = hot_count - 50;
            let oldest_records = self.hot_memory.get_oldest_records(records_to_move).await?;

            for record in oldest_records.iter() {
                self.warm_memory.upsert_record(record.clone()).await?;
            }

            self.hot_memory.delete_oldest_records(records_to_move).await?;
        }

        Ok(())
    }

    pub async fn migrate_warm_to_cold(&mut self) -> Result<(), MemoryManagerServiceError> {
        let warm_count = self.warm_memory.count_records().await?;

        if warm_count > 500 {
            let records_to_move = warm_count - 500;
            let oldest_records = self.warm_memory.get_oldest_records(records_to_move).await?;

            for record in oldest_records.iter() {
                self.cold_memory.upsert_record(record.clone()).await?;
            }

            self.warm_memory.delete_oldest_records(records_to_move).await?;
        }

        Ok(())
    }

    pub async fn add_memory(&mut self, record: MemoryRecord) -> Result<(), MemoryManagerServiceError> {
        self.hot_memory.upsert_record(record).await?;

        self.migrate_hot_to_warm().await?;
        self.migrate_warm_to_cold().await?;

        Ok(())
    }

    pub async fn search(
        &mut self,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<MemoryRecord>, MemoryManagerServiceError> {
        let mut results = Vec::new();
        let mut tier_map: HashMap<String, MemoryTier> = HashMap::new();

        let hot_results = self.hot_memory.search(query_vector.clone(), limit).await?;
        for record in &hot_results {
            tier_map.insert(record.id.clone(), MemoryTier::Hot);
        }
        results.extend(hot_results);

        if results.len() < limit {
            let remaining = limit - results.len();
            let warm_results = self.warm_memory.search(query_vector.clone(), remaining).await?;
            for record in &warm_results {
                tier_map.insert(record.id.clone(), MemoryTier::Warm);
            }
            results.extend(warm_results);
        }

        if results.len() < limit {
            let remaining = limit - results.len();
            let cold_results = self.cold_memory.search(query_vector, remaining).await?;
            for record in &cold_results {
                tier_map.insert(record.id.clone(), MemoryTier::Cold);
            }
            results.extend(cold_results);
        }

        let results_for_update = results.clone();
        let tier_map_clone = tier_map.clone();

        let hot_memory = self.hot_memory.clone();
        let warm_memory = self.warm_memory.clone();
        let cold_memory = self.cold_memory.clone();

        tokio::spawn(async move {
            Self::update_access_tracking_async(hot_memory, warm_memory, cold_memory, results_for_update, tier_map_clone)
                .await
        });

        Ok(results)
    }

    async fn update_access_tracking_async(
        hot_memory: HotMemoryService,
        warm_memory: WarmMemoryService,
        cold_memory: ColdMemoryService,
        mut records: Vec<MemoryRecord>,
        tier_map: HashMap<String, MemoryTier>,
    ) -> Result<(), MemoryManagerServiceError> {
        let now = chrono::Utc::now().to_rfc3339();

        for record in records.iter_mut() {
            record.access_count += 1;
            record.last_accessed_at = now.clone();
            record.updated_at = now.clone();

            match tier_map.get(&record.id) {
                Some(MemoryTier::Hot) => {
                    hot_memory.upsert_record(record.clone()).await?;
                }
                Some(MemoryTier::Warm) => {
                    warm_memory.upsert_record(record.clone()).await?;
                }
                Some(MemoryTier::Cold) => {
                    cold_memory.upsert_record(record.clone()).await?;
                }
                None => {}
            }
        }

        Ok(())
    }

    pub async fn check_and_promote_memories(&mut self) -> Result<usize, MemoryManagerServiceError> {
        let mut promoted_count = 0;

        let cold_candidates = self.cold_memory.get_promotion_candidates(20).await?;
        for record in cold_candidates {
            if self.should_promote_memory(&record) {
                self.promote_memory(record).await?;
                promoted_count += 1;
            }
        }

        let warm_candidates = self.warm_memory.get_promotion_candidates(20).await?;
        for record in warm_candidates {
            if self.should_promote_memory(&record) {
                self.promote_memory(record).await?;
                promoted_count += 1;
            }
        }

        Ok(promoted_count)
    }

    fn should_promote_memory(&self, record: &MemoryRecord) -> bool {
        let is_recently_accessed = if !record.last_accessed_at.is_empty() {
            if let Ok(last_access) = chrono::DateTime::parse_from_rfc3339(&record.last_accessed_at) {
                let now = chrono::Utc::now();
                let diff = now.signed_duration_since(last_access.with_timezone(&chrono::Utc));
                diff.num_hours() < RECENT_ACCESS_HOURS
            } else {
                false
            }
        } else {
            false
        };

        (record.access_count >= COLD_TO_WARM_ACCESS_THRESHOLD && is_recently_accessed)
            || record.access_count >= WARM_TO_HOT_ACCESS_THRESHOLD
    }

    async fn promote_memory(&mut self, mut record: MemoryRecord) -> Result<(), MemoryManagerServiceError> {
        if self.cold_memory.record_exists(&record.id).await? {
            if record.access_count >= COLD_TO_WARM_ACCESS_THRESHOLD {
                self.cold_memory.delete_record(&record.id).await?;

                record.access_count = 0;
                self.warm_memory.upsert_record(record).await?;

                let warm_count = self.warm_memory.count_records().await?;
                if warm_count > 500 {
                    self.migrate_warm_to_cold().await?;
                }
            }
        } else if self.warm_memory.record_exists(&record.id).await? {
            if record.access_count >= WARM_TO_HOT_ACCESS_THRESHOLD {
                self.warm_memory.delete_record(&record.id).await?;

                record.access_count = 0;
                self.hot_memory.upsert_record(record).await?;

                let hot_count = self.hot_memory.count_records().await?;
                if hot_count > 50 {
                    self.migrate_hot_to_warm().await?;
                }
            }
        }

        Ok(())
    }

    pub async fn export_all_records(&self) -> Result<Vec<MemoryRecord>, MemoryManagerServiceError> {
        let (cold_records, warm_records, hot_records) = tokio::try_join!(
            self.cold_memory.get_all_records(),
            self.warm_memory.get_all_records(),
            self.hot_memory.get_all_records()
        )?;

        let export_records: Vec<MemoryRecord> = cold_records
            .into_iter()
            .chain(warm_records.into_iter())
            .chain(hot_records.into_iter())
            .collect();

        Ok(export_records)
    }
}
