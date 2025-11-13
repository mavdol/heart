pub mod emotion;
pub mod memory;
pub mod thought;

pub use emotion::{EmotionService, EmotionServiceError};
pub use memory::{MemoryManagerService, MemoryManagerServiceError, MemoryRecord};
pub use thought::{ThoughtService, ThoughtServiceError};

use chrono::{DateTime, Local, TimeZone, Utc};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use tauri::{AppHandle, Manager};
use tauri_plugin_store::StoreExt;

use crate::services::{ChatMessage, ChatResponse, LLMServiceError};
use crate::services::{EmbeddingService, EmbeddingServiceError, LlmService};
use crate::utils::{chat_prompt_builder, get_emotion_label, welcome_back_prompt_builder};

#[derive(Debug)]
pub enum BrainError {
    MemoryManager(MemoryManagerServiceError),
    EmotionService(EmotionServiceError),
    ThoughtService(ThoughtServiceError),
    LlmService(LLMServiceError),
    EmbeddingService(EmbeddingServiceError),
    PathError(String),
    StoreError(String),
    InvalidMessageError(String),
    DateTimeError(String),
    SerdeError(serde_json::Error),
    OrderingError(std::cmp::Ordering),
}

impl fmt::Display for BrainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrainError::MemoryManager(e) => write!(f, "Memory manager error: {}", e),
            BrainError::EmotionService(e) => write!(f, "Emotion service error: {}", e),
            BrainError::ThoughtService(e) => write!(f, "Thought service error: {}", e),
            BrainError::LlmService(e) => write!(f, "LLM service error: {}", e),
            BrainError::EmbeddingService(e) => write!(f, "Embedding service error: {}", e),
            BrainError::PathError(e) => write!(f, "Path error: {}", e),
            BrainError::StoreError(e) => write!(f, "Store error: {}", e),
            BrainError::InvalidMessageError(e) => write!(f, "Invalid message error: {}", e),
            BrainError::DateTimeError(e) => write!(f, "Date time error: {}", e),
            BrainError::SerdeError(e) => write!(f, "Serde error: {}", e),
            BrainError::OrderingError(e) => write!(f, "Ordering error: {:?}", e),
        }
    }
}

impl std::error::Error for BrainError {}

impl From<MemoryManagerServiceError> for BrainError {
    fn from(error: MemoryManagerServiceError) -> Self {
        BrainError::MemoryManager(error)
    }
}

impl From<EmotionServiceError> for BrainError {
    fn from(error: EmotionServiceError) -> Self {
        BrainError::EmotionService(error)
    }
}

impl From<ThoughtServiceError> for BrainError {
    fn from(error: ThoughtServiceError) -> Self {
        BrainError::ThoughtService(error)
    }
}

impl From<LLMServiceError> for BrainError {
    fn from(error: LLMServiceError) -> Self {
        BrainError::LlmService(error)
    }
}

impl From<EmbeddingServiceError> for BrainError {
    fn from(error: EmbeddingServiceError) -> Self {
        BrainError::EmbeddingService(error)
    }
}

impl From<serde_json::Error> for BrainError {
    fn from(error: serde_json::Error) -> Self {
        BrainError::SerdeError(error)
    }
}

impl From<std::cmp::Ordering> for BrainError {
    fn from(error: std::cmp::Ordering) -> Self {
        BrainError::OrderingError(error)
    }
}

pub struct Brain {
    pub memory: MemoryManagerService,
    pub emotion: EmotionService,
    pub embedding_service: Arc<EmbeddingService>,
    pub llm_service: Arc<LlmService>,
    pub app_handle: AppHandle,
}

impl Brain {
    pub async fn new(
        app_handle: AppHandle,
        embedding_service: Arc<EmbeddingService>,
        llm_service: Arc<LlmService>,
    ) -> Result<Self, BrainError> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .map_err(|e| BrainError::PathError(e.to_string()))?;

        let now = Local::now();
        let store = app_handle
            .store("heart.json")
            .map_err(|e| BrainError::StoreError(e.to_string()))?;

        if store.get("birthday").is_none() {
            store.set("birthday", now.timestamp());
        }

        if store.get("personality_updated_at").is_none() {
            store.set("personality_updated_at", now.timestamp());
        }

        if store.get("first_message_sent").is_none() {
            store.set("first_message_sent", false);
        }

        store.set("last_connection", now.timestamp());

        let birthday_secs = store.get("birthday").and_then(|v| v.as_u64()).unwrap_or(0);

        let memory = MemoryManagerService::new(app_data_dir).await?;
        let emotion = EmotionService::new(memory.clone(), birthday_secs).await?;
        let thought = ThoughtService::new(app_handle.clone(), memory.clone(), llm_service.clone())?;

        thought.start_thinking()?;

        Ok(Self {
            memory,
            emotion,
            embedding_service,
            llm_service,
            app_handle,
        })
    }

    pub async fn process_new_message(&mut self, mut messages: Vec<ChatMessage>) -> Result<ChatResponse, BrainError> {
        if messages.is_empty() {
            return Err(BrainError::InvalidMessageError("Messages vector is empty".to_string()));
        }

        let last_message = messages
            .last()
            .ok_or_else(|| BrainError::InvalidMessageError("Failed to get last message".to_string()))?;

        let store = self
            .app_handle
            .store("heart.json")
            .map_err(|e| BrainError::StoreError(e.to_string()))?;

        let first_message_sent = store
            .get("first_message_sent")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !first_message_sent {
            store.set("first_message_sent", true);
        }

        let last_message_content = last_message.content.clone();

        let emotion = self
            .emotion
            .clone()
            .predict_emotion(last_message_content.clone())
            .await?;

        let birthday_timestamp = store.get("birthday").and_then(|v| v.as_i64()).unwrap_or(0);
        let last_connection_timestamp = store.get("last_connection").and_then(|v| v.as_i64()).unwrap_or(0);

        let birthday = Utc
            .timestamp_opt(birthday_timestamp, 0)
            .single()
            .ok_or_else(|| BrainError::DateTimeError(format!("Invalid birthday timestamp: {}", birthday_timestamp)))?;

        let last_connection: DateTime<Utc> =
            Utc.timestamp_opt(last_connection_timestamp, 0)
                .single()
                .ok_or_else(|| {
                    BrainError::DateTimeError(format!(
                        "Invalid last connection timestamp: {}",
                        last_connection_timestamp
                    ))
                })?;

        let now = Local::now();

        let writing_style: Option<String> = store
            .get("writing_style")
            .and_then(|v| v.as_str().map(|s| s.to_string()));

        let current_context = match first_message_sent {
            true => format!(
                "Date of your birth : {}, \nToday date : {}, \nLast time you spoke to each other  : {}",
                birthday.format("%A %d %B %Y %H:%M:%S"),
                now.format("%A %d %B %Y %H:%M:%S"),
                last_connection.format("%A %d %B %Y %H:%M:%S")
            ),
            false => format!(
                "Date of your birth : {}, \nToday date : {} \nLast time you spoke to each other",
                birthday.format("%A %d %B %Y %H:%M:%S"),
                now.format("%A %d %B %Y %H:%M:%S")
            ),
        };

        let emotion_label = get_emotion_label(emotion.valence, emotion.arousal, 0.7)?;

        let memory_retrieved_string = self.retrieve_relevant_memories(&messages).await?;

        let chat_prompt = chat_prompt_builder(
            &current_context,
            &emotion_label,
            (!memory_retrieved_string.is_empty()).then_some(&memory_retrieved_string),
            writing_style.as_deref().filter(|s| !s.is_empty()),
        );

        let system_message = ChatMessage {
            role: "system".to_string(),
            content: chat_prompt,
        };

        if !messages.is_empty() && messages[0].role == "system" {
            messages[0] = system_message;
        } else {
            messages.insert(0, system_message);
        }

        let llm_response = self.llm_service.generate_chat_response(messages).await?;

        let response_content = llm_response.content.clone();
        let user_content = last_message_content.clone();
        let emotion_valence = emotion.valence;
        let emotion_arousal = emotion.arousal;
        let user_timestamp = now;
        let mut memory_manager = self.memory.clone();
        let embedding_service = self.embedding_service.clone();

        tokio::spawn(async move {
            let assistant_now = Local::now();

            let assistant_message_metadata = HashMap::from([
                ("role".to_string(), "assistant".to_string()),
                ("content".to_string(), response_content.clone()),
                ("valence".to_string(), "0.0".to_string()),
                ("arousal".to_string(), "0.0".to_string()),
                (
                    "message_sent".to_string(),
                    assistant_now.format("%A %d %B %Y %H:%M:%S").to_string(),
                )
            ]);

            let user_message_metadata = HashMap::from([
                ("role".to_string(), "user".to_string()),
                ("content".to_string(), user_content.clone()),
                ("valence".to_string(), emotion_valence.to_string()),
                ("arousal".to_string(), emotion_arousal.to_string()),
                (
                    "message_sent".to_string(),
                    user_timestamp.format("%A %d %B %Y %H:%M:%S").to_string(),
                ),
                ("past_time".to_string(), (user_timestamp.timestamp() - birthday.timestamp()).to_string()),
            ]);

            let user_metadata_json = match serde_json::to_string(&user_message_metadata) {
                Ok(json) => json,
                Err(e) => {
                    eprintln!("Failed to serialize user message metadata: {}", e);
                    return;
                }
            };

            let user_message_vector = match embedding_service.embed_text(&user_metadata_json).await {
                Ok(vec) => vec,
                Err(e) => {
                    eprintln!("Failed to embed user message: {}", e);
                    return;
                }
            };

            let assistant_metadata_json = match serde_json::to_string(&assistant_message_metadata) {
                Ok(json) => json,
                Err(e) => {
                    eprintln!("Failed to serialize assistant message metadata: {}", e);
                    return;
                }
            };

            let assistant_message_vector = match embedding_service.embed_text(&assistant_metadata_json).await {
                Ok(vec) => vec,
                Err(e) => {
                    eprintln!("Failed to embed assistant message: {}", e);
                    return;
                }
            };

            let user_record = MemoryRecord {
                id: uuid::Uuid::new_v4().to_string(),
                vector: user_message_vector,
                metadata: user_message_metadata,
                created_at: user_timestamp.timestamp_millis().to_string(),
                updated_at: user_timestamp.timestamp_millis().to_string(),
                last_accessed_at: String::new(),
                access_count: 0,
            };

            if let Err(e) = memory_manager.add_memory(user_record).await {
                eprintln!("Failed to add user memory: {}", e);
                return;
            }

            let assistant_record = MemoryRecord {
                id: uuid::Uuid::new_v4().to_string(),
                vector: assistant_message_vector,
                metadata: assistant_message_metadata,
                created_at: assistant_now.timestamp_millis().to_string(),
                updated_at: assistant_now.timestamp_millis().to_string(),
                last_accessed_at: String::new(),
                access_count: 0,
            };

            if let Err(e) = memory_manager.add_memory(assistant_record).await {
                eprintln!("Failed to add assistant memory: {}", e);
                return;
            }

            if let Err(e) = memory_manager.check_and_promote_memories().await {
                eprintln!("Failed to check and promote memories: {}", e);
            }
        });

        Ok(llm_response)
    }

    pub async fn welcome_back_message(&mut self) -> Result<ChatResponse, BrainError> {
        let store = self
            .app_handle
            .store("heart.json")
            .map_err(|e| BrainError::StoreError(e.to_string()))?;

        let birthday_timestamp = store.get("birthday").and_then(|v| v.as_i64()).unwrap_or(0);
        let last_connection_timestamp = store.get("last_connection").and_then(|v| v.as_i64()).unwrap_or(0);
        let now = Local::now();

        let birthday = Utc
            .timestamp_opt(birthday_timestamp, 0)
            .single()
            .ok_or_else(|| BrainError::DateTimeError(format!("Invalid birthday timestamp: {}", birthday_timestamp)))?;

        let last_connection: DateTime<Utc> =
            Utc.timestamp_opt(last_connection_timestamp, 0)
                .single()
                .ok_or_else(|| {
                    BrainError::DateTimeError(format!(
                        "Invalid last connection timestamp: {}",
                        last_connection_timestamp
                    ))
                })?;

        let writing_style: Option<String> = store
            .get("writing_style")
            .and_then(|v| v.as_str().map(|s| s.to_string()));

        let current_context = format!(
            "Date of your birth : {}, \nToday date : {}, \nLast time you spoke to each other  : {}",
            birthday.format("%A %d %B %Y %H:%M:%S"),
            now.format("%A %d %B %Y %H:%M:%S"),
            last_connection.format("%A %d %B %Y %H:%M:%S")
        );

        let current_emotion = self.emotion.clone().get_current_emotion().await?;

        let emotion_label = get_emotion_label(current_emotion.valence, current_emotion.arousal, 0.7)?;

        let memory_retrieved_string = self.retrieve_significant_memories(3).await?;

        let chat_prompt = welcome_back_prompt_builder(
            &current_context,
            &memory_retrieved_string,
            &emotion_label,
            writing_style.as_deref().filter(|s| !s.is_empty()),
        );

        let system_message = ChatMessage {
            role: "system".to_string(),
            content: chat_prompt,
        };

        let messages = vec![
            system_message,
            ChatMessage {
                role: "user".to_string(),
                content: "*user just came back*".to_string(),
            },
        ];

        let llm_response = self.llm_service.generate_chat_response(messages).await?;

        let response_content = llm_response.content.clone();
        let mut memory_manager = self.memory.clone();
        let embedding_service = self.embedding_service.clone();

        tokio::spawn(async move {
            let assistant_now = Local::now();

            let assistant_message_metadata = HashMap::from([
                ("role".to_string(), "assistant".to_string()),
                ("content".to_string(), response_content.clone()),
                ("valence".to_string(), "0.0".to_string()),
                ("arousal".to_string(), "0.0".to_string()),
                (
                    "message_sent".to_string(),
                    assistant_now.format("%A %d %B %Y %H:%M:%S").to_string(),
                ),
            ]);

            let assistant_metadata_json = match serde_json::to_string(&assistant_message_metadata) {
                Ok(json) => json,
                Err(e) => {
                    eprintln!("Failed to serialize assistant message metadata: {}", e);
                    return;
                }
            };

            let assistant_message_vector = match embedding_service.embed_text(&assistant_metadata_json).await {
                Ok(vec) => vec,
                Err(e) => {
                    eprintln!("Failed to embed assistant message: {}", e);
                    return;
                }
            };

            let assistant_record = MemoryRecord {
                id: uuid::Uuid::new_v4().to_string(),
                vector: assistant_message_vector,
                metadata: assistant_message_metadata,
                created_at: assistant_now.timestamp_millis().to_string(),
                updated_at: assistant_now.timestamp_millis().to_string(),
                last_accessed_at: String::new(),
                access_count: 0,
            };

            if let Err(e) = memory_manager.add_memory(assistant_record).await {
                eprintln!("Failed to add assistant memory: {}", e);
                return;
            }

            if let Err(e) = memory_manager.check_and_promote_memories().await {
                eprintln!("Failed to check and promote memories: {}", e);
            }
        });

        Ok(llm_response)
    }

    async fn retrieve_relevant_memories(&mut self, messages: &[ChatMessage]) -> Result<String, BrainError> {
        let last_3_messages: Vec<String> = messages.iter().rev().take(3).map(|msg| msg.content.clone()).collect();

        if last_3_messages.is_empty() {
            return Ok(String::new());
        }

        let query_text = last_3_messages.join("\n");

        let query_vector = self.embedding_service.embed_text(&query_text).await?;
        let memory_records = self.memory.search(query_vector, 3).await?;

        let memory_strings: Vec<String> = memory_records
            .iter()
            .filter_map(|record| record.metadata.get("content").cloned())
            .collect();

        Ok(memory_strings.join("\n"))
    }

    async fn retrieve_significant_memories(&mut self, limit: usize) -> Result<String, BrainError> {
        let mut memory_records = self.memory.hot_memory.get_all_records().await?;

        memory_records.sort_by(|a, b| {
            let significance_a = a
                .metadata
                .get("valence")
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(0.0)
                + a.metadata
                    .get("arousal")
                    .and_then(|v| v.parse::<f32>().ok())
                    .unwrap_or(0.0);

            let significance_b = b
                .metadata
                .get("valence")
                .and_then(|v| v.parse::<f32>().ok())
                .unwrap_or(0.0)
                + b.metadata
                    .get("arousal")
                    .and_then(|v| v.parse::<f32>().ok())
                    .unwrap_or(0.0);

            significance_b
                .partial_cmp(&significance_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let memory_strings: Vec<String> = memory_records
            .iter()
            .take(limit)
            .filter_map(|record| record.metadata.get("content").cloned())
            .collect();

        Ok(memory_strings.join("\n"))
    }
}
