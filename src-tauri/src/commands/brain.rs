use crate::services::brain::Brain;
use crate::services::{ChatMessage, ChatResponse};
use crate::utils::{ApiSuccessResponse, AppError, AppResult};
use npc_neural_affect_matrix::EmotionPrediction;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;

#[tauri::command]
pub async fn process_new_message(
    brain: State<'_, Mutex<Brain>>,
    messages: Vec<ChatMessage>,
) -> AppResult<ApiSuccessResponse<ChatResponse>> {
    let mut brain = brain.lock().await;
    let response = brain
        .process_new_message(messages)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(ApiSuccessResponse::new(response))
}

#[tauri::command]
pub async fn process_welcome_back_message(
    brain: State<'_, Mutex<Brain>>,
) -> AppResult<ApiSuccessResponse<ChatResponse>> {
    let mut brain = brain.lock().await;
    let response = brain
        .welcome_back_message()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(ApiSuccessResponse::new(response))
}

#[tauri::command]
pub async fn destroy_brain(app_handle: AppHandle) -> AppResult<ApiSuccessResponse<()>> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(e.to_string()))?;

    std::fs::remove_dir_all(app_data_dir.join("heart_memory.lance")).map_err(|e| AppError::Internal(e.to_string()))?;

    let store = app_handle.store("heart.json").map_err(|e| AppError::Internal(e.to_string()))?;
    store.clear();

    eprintln!("Brain destroyed");
    Ok(ApiSuccessResponse::new(()))
}

#[tauri::command]
pub async fn current_emotion(brain: State<'_, Mutex<Brain>>) -> AppResult<ApiSuccessResponse<EmotionPrediction>> {
    let brain = brain.lock().await;
    let emotion = brain
        .emotion
        .clone()
        .get_current_emotion()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(ApiSuccessResponse::new(emotion))
}
