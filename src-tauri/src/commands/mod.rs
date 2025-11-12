mod app;
mod brain;

pub use app::{
    check_model_installed, check_neural_affect_matrix_running, check_ollama_installed, check_ollama_running,
    download_model,
};

pub use brain::{current_emotion, destroy_brain, process_new_message, process_welcome_back_message};
