use crate::services::{Brain, LlmService};

use std::process::Stdio;
use std::sync::Arc;
use tauri::{Emitter, State, Window};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;

use crate::utils::{clean_progress_text, ApiSuccessResponse, AppError, AppResult};

#[tauri::command]
pub fn check_ollama_installed(llm_service: State<'_, Arc<LlmService>>) -> AppResult<ApiSuccessResponse<bool>> {
    Ok(ApiSuccessResponse::new(llm_service.check_ollama_installed()))
}

#[tauri::command]
pub async fn check_ollama_running(llm_service: State<'_, Arc<LlmService>>) -> AppResult<ApiSuccessResponse<bool>> {
    let ollama_running = llm_service
        .is_ollama_running()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(ApiSuccessResponse::new(ollama_running))
}

#[tauri::command]
pub async fn check_model_installed(llm_service: State<'_, Arc<LlmService>>) -> AppResult<ApiSuccessResponse<bool>> {
    let model_installed = llm_service
        .is_model_installed()
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(ApiSuccessResponse::new(model_installed))
}

#[tauri::command]
pub async fn check_neural_affect_matrix_running(brain: State<'_, Mutex<Brain>>) -> AppResult<ApiSuccessResponse<bool>> {
    let brain = brain.lock().await;
    let neural_affect_matrix_running = brain
        .emotion
        .is_neural_affect_matrix_running()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok(ApiSuccessResponse::new(neural_affect_matrix_running))
}

#[tauri::command]
pub async fn download_model(
    llm_service: State<'_, Arc<LlmService>>,
    window: Window,
) -> AppResult<ApiSuccessResponse<String>> {
    window
        .emit(
            "model-download-progress",
            serde_json::json!({
                "status": "starting",
                "message": "Starting model download..."
            }),
        )
        .map_err(|e| AppError::Tauri(e.to_string()))?;

    let ollama_path = llm_service.get_ollama_binary_path();

    let mut child = Command::new(ollama_path)
        .arg("pull")
        .arg(&llm_service.ollama_model)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| AppError::Internal(format!("Failed to spawn ollama: {}", e)))?;

    let stderr = child.stderr.take();

    let window_clone = window.clone();
    let _stderr_handle = tokio::spawn(async move {
        if let Some(stderr) = stderr {
            let reader = BufReader::new(stderr);
            let mut lines_reader = reader.lines();

            while let Ok(Some(line)) = lines_reader.next_line().await {
                if !line.is_empty() {
                    let cleaned = clean_progress_text(&line);

                    if !cleaned.is_empty() {
                        window_clone
                            .emit(
                                "model-download-progress",
                                serde_json::json!({
                                    "status": "downloading",
                                    "message": cleaned
                                }),
                            )
                            .ok();
                    }
                }
            }
        }
    });

    let status = child
        .wait()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to wait for ollama: {}", e)))?;

    if !status.success() {
        window
            .emit(
                "model-download-progress",
                serde_json::json!({
                    "status": "error",
                    "message": "Failed to download model"
                }),
            )
            .ok();
        return Err(AppError::Internal("Failed to download model".to_string()));
    }

    window
        .emit(
            "model-download-progress",
            serde_json::json!({
                "status": "complete",
                "message": "Model downloaded successfully!"
            }),
        )
        .ok();

    Ok(ApiSuccessResponse::new("Model downloaded successfully".to_string()))
}
