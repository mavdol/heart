use std::fmt;
use std::sync::Arc;
use std::time::UNIX_EPOCH;
use tokio::sync::Mutex;

use npc_neural_affect_matrix::{
    EmotionPrediction, EmotionPredictor, EmotionPredictorError, Identity, MemoryConfig, MemoryEmotionEvaluator,
    MemoryRecord as EmotionMemoryRecord, MemoryStore, NpcConfig, PersonalityTraits,
};

use crate::services::brain::memory::{MemoryManagerService, MemoryManagerServiceError, MemoryRecord};

#[derive(Debug)]
pub enum EmotionServiceError {
    PredictorError(String),
    MemoryManagerError(String),
}

impl fmt::Display for EmotionServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmotionServiceError::PredictorError(msg) => write!(f, "Predictor error: {}", msg),
            EmotionServiceError::MemoryManagerError(msg) => write!(f, "Memory manager error: {}", msg),
        }
    }
}

impl From<EmotionPredictorError> for EmotionServiceError {
    fn from(error: EmotionPredictorError) -> Self {
        EmotionServiceError::PredictorError(error.to_string())
    }
}

impl From<MemoryManagerServiceError> for EmotionServiceError {
    fn from(error: MemoryManagerServiceError) -> Self {
        EmotionServiceError::MemoryManagerError(error.to_string())
    }
}

#[derive(Clone)]
pub struct EmotionService {
    pub emotion_predictor: Arc<Mutex<EmotionPredictor>>,
    pub npc_emotion: Arc<Mutex<MemoryEmotionEvaluator>>,
    pub npc_birthday: u64,
}

impl EmotionService {
    pub async fn new(
        memory_manager_service: MemoryManagerService,
        npc_birthday: u64,
    ) -> Result<Self, EmotionServiceError> {
        let npc_config = NpcConfig {
            identity: Identity {
                name: "Heart".to_string(),
                background: "You are a local AI companion that feels emotions".to_string(),
            },
            personality: PersonalityTraits::default(),
            memory: MemoryConfig {
                decay_rate: 0.000014, // emotion loses 70% of its weight every day
            },
        };

        let emotion_predictor = EmotionPredictor::new()?;
        let npc_emotion = MemoryEmotionEvaluator::new(npc_config, None)?;
        let records = memory_manager_service.export_all_records().await?;

        Self::import_emotion_memory(npc_emotion.npc_id.clone(), records).await?;

        Ok(Self {
            emotion_predictor: Arc::new(Mutex::new(emotion_predictor)),
            npc_emotion: Arc::new(Mutex::new(npc_emotion)),
            npc_birthday,
        })
    }

    async fn import_emotion_memory(npc_id: String, records: Vec<MemoryRecord>) -> Result<(), EmotionServiceError> {
        let exctrated_records: Vec<EmotionMemoryRecord> = records
            .into_iter()
            .filter_map(|record| {
                let content = record.metadata.get("content")?.to_string();
                let valence = record
                    .metadata
                    .get("valence")
                    .and_then(|v| v.parse::<f32>().ok())
                    .unwrap_or(0.0);

                let arousal = record
                    .metadata
                    .get("arousal")
                    .and_then(|a| a.parse::<f32>().ok())
                    .unwrap_or(0.0);

                let past_time = record
                    .metadata
                    .get("past_time")
                    .and_then(|p| p.parse::<i64>().ok())
                    .unwrap_or(0);

                Some(EmotionMemoryRecord {
                    id: record.id,
                    source_id: "user".to_string(),
                    content,
                    valence,
                    arousal,
                    past_time,
                })
            })
            .collect();

        let _ = MemoryStore::import(&npc_id, exctrated_records);

        Ok(())
    }

    pub async fn predict_emotion(self, text: String) -> Result<EmotionPrediction, EmotionServiceError> {
        let current_time = UNIX_EPOCH.elapsed().unwrap().as_secs();
        let past_time = (current_time - self.npc_birthday) as i64;

        let emotion = self.emotion_predictor.lock().await.predict_emotion_from_text(&text)?;
        let npc_current_emotion = self
            .npc_emotion
            .lock()
            .await
            .evaluate_npc_emotion(&text, &emotion, past_time, None)?;

        Ok(npc_current_emotion)
    }

    pub async fn get_current_emotion(self) -> Result<EmotionPrediction, EmotionServiceError> {
        let mut npc_current_emotion = self.npc_emotion.lock().await.calculate_current_emotion()?;

        npc_current_emotion.valence = (npc_current_emotion.valence * 100.0).round() / 100.0;
        npc_current_emotion.arousal = (npc_current_emotion.arousal * 100.0).round() / 100.0;

        Ok(npc_current_emotion)
    }

    pub async fn is_neural_affect_matrix_running(&self) -> Result<bool, EmotionServiceError> {
        match self
            .emotion_predictor
            .lock()
            .await
            .predict_emotion_from_text("Hello, are you okay?")
        {
            Ok(_) => Ok(true),
            Err(e) => {
                eprintln!("Error predicting emotion: {}", e);
                Ok(false)
            }
        }
    }
}
