use chrono::Local;
use serde_json;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use crate::services::brain::memory::{MemoryManagerService, MemoryManagerServiceError};
use crate::services::{LLMServiceError, LlmService};
use crate::utils::writing_style_prompt_builder;

#[derive(Debug)]
pub enum ThoughtServiceError {
    StoreError(String),
    MemoryError(MemoryManagerServiceError),
    LlmError(LLMServiceError),
}

impl fmt::Display for ThoughtServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ThoughtServiceError::StoreError(msg) => write!(f, "Store error: {}", msg),
            ThoughtServiceError::MemoryError(e) => write!(f, "Memory error: {}", e),
            ThoughtServiceError::LlmError(e) => write!(f, "LLM error: {}", e),
        }
    }
}

impl std::error::Error for ThoughtServiceError {}

impl From<MemoryManagerServiceError> for ThoughtServiceError {
    fn from(error: MemoryManagerServiceError) -> Self {
        ThoughtServiceError::MemoryError(error)
    }
}

impl From<LLMServiceError> for ThoughtServiceError {
    fn from(error: LLMServiceError) -> Self {
        ThoughtServiceError::LlmError(error)
    }
}

#[derive(Clone)]
pub struct ThoughtService {
    app: AppHandle,
    memory: MemoryManagerService,
    llm: Arc<LlmService>,
}

impl ThoughtService {
    pub fn new(
        app: AppHandle,
        memory: MemoryManagerService,
        llm: Arc<LlmService>,
    ) -> Result<Self, ThoughtServiceError> {
        Ok(Self { app, memory, llm })
    }

    pub fn start_thinking(&self) -> Result<(), ThoughtServiceError> {
        let app = self.app.clone();
        let memory = self.memory.clone();
        let llm = self.llm.clone();

        tokio::spawn(async move {
            loop {
                if let Err(e) = Self::personality_updates(app.clone(), memory.clone(), llm.clone()).await {
                    eprintln!("Error updating personality: {}", e);
                }
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        });

        Ok(())
    }

    async fn personality_updates(
        app: AppHandle,
        memory: MemoryManagerService,
        llm: Arc<LlmService>,
    ) -> Result<bool, ThoughtServiceError> {
        let store = app
            .store("heart.json")
            .map_err(|e| ThoughtServiceError::StoreError(e.to_string()))?;

        let now = Local::now();
        let personality_updated_at = store
            .get("personality_updated_at")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let time_since_update = now.timestamp() - personality_updated_at;
        if time_since_update < 24 * 60 * 60 {
            return Ok(false);
        }

        let all_records = memory.export_all_records().await?;

        let user_messages: Vec<String> = all_records
            .iter()
            .filter(|record| record.metadata.get("role").map(|role| role == "user").unwrap_or(false))
            .filter_map(|record| record.metadata.get("content").cloned())
            .collect();

        if user_messages.is_empty() {
            return Ok(false);
        }

        let messages_text = user_messages.join("\n");

        let writing_style_prompt = writing_style_prompt_builder(&messages_text);

        let writing_style = llm.generate_response(&writing_style_prompt).await?;

        store.set("writing_style", serde_json::json!(writing_style));
        store.set("personality_updated_at", serde_json::json!(now.timestamp()));

        if let Err(e) = store.save() {
            eprintln!("Failed to save store: {}", e);
        }

        Ok(true)
    }
}
