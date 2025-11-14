mod commands;
mod prompts;
mod services;
mod utils;

use crate::commands::*;
use crate::services::{Brain, EmbeddingService, LlmService};
use crate::utils::AppError;
use tauri_plugin_store::StoreExt;

use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            check_ollama_installed,
            check_ollama_running,
            check_model_installed,
            check_neural_affect_matrix_running,
            download_model,
            process_new_message,
            process_welcome_back_message,
            current_emotion,
            destroy_brain,
        ])
        .setup(|app| {
            let store = app
                .store("settings.json")
                .map_err(|e| AppError::Internal(e.to_string()))?;

            if store.get("theme").is_none() {
                store.set("theme", "system");
            }

            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| AppError::Internal(e.to_string()))?;

            let embedding_cache_dir = app_data_dir.join("embedding_models");

            let embedding_service = Arc::new(
                EmbeddingService::new(Some(embedding_cache_dir))
                    .map_err(|e| AppError::Internal(e.to_string()))?
            );
            let llm_service = Arc::new(LlmService::new().map_err(|e| AppError::Internal(e.to_string()))?);

            let brain = tauri::async_runtime::block_on(async {
                Brain::new(app.handle().clone(), embedding_service.clone(), llm_service.clone())
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))
            })?;

            app.manage(embedding_service);
            app.manage(llm_service);
            app.manage(Mutex::new(brain));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
