use serde::{Deserialize, Serialize};
use std::env;
use std::fmt;
use std::path::PathBuf;
use std::process::Command;

use reqwest::Client;

#[derive(Debug)]
pub enum LLMServiceError {
    ReqwestError(String),
    ParseError(String),
    Other(String),
}

impl fmt::Display for LLMServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LLMServiceError::ReqwestError(msg) => write!(f, "Reqwest error: {}", msg),
            LLMServiceError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            LLMServiceError::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl From<reqwest::Error> for LLMServiceError {
    fn from(error: reqwest::Error) -> Self {
        eprintln!("reqwest::Error: {:?}", error);
        LLMServiceError::ReqwestError(error.to_string())
    }
}

impl From<serde_json::Error> for LLMServiceError {
    fn from(error: serde_json::Error) -> Self {
        eprintln!("serde_json::Error: {:?}", error);
        LLMServiceError::ParseError(error.to_string())
    }
}

impl From<std::io::Error> for LLMServiceError {
    fn from(error: std::io::Error) -> Self {
        eprintln!("std::io::Error: {:?}", error);
        LLMServiceError::Other(error.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaChatResponse {
    model: String,
    created_at: String,
    message: ChatResponse,
    done: bool,
    done_reason: String,
    total_duration: u64,
    load_duration: u64,
    prompt_eval_count: u64,
    prompt_eval_duration: u64,
    eval_count: u64,
    eval_duration: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct OllamaResponse {
    model: String,
    created_at: String,
    response: String,
    done: bool,
    done_reason: String,
    total_duration: u64,
    load_duration: u64,
    prompt_eval_count: u64,
    prompt_eval_duration: u64,
    eval_count: u64,
    eval_duration: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub role: String,
    pub content: String,
}

#[derive(Clone)]
pub struct LlmService {
    client: Client,
    ollama_host: String,
    pub(crate) ollama_model: String,
    ollama_binary_path: Option<PathBuf>,
}

impl LlmService {
    pub fn new() -> Result<Self, LLMServiceError> {
        let ollama_host = env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_string());
        let ollama_model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".to_string());
        let client = Client::new();

        let ollama_binary_path = Self::find_ollama_binary();

        Ok(Self {
            client,
            ollama_host,
            ollama_model,
            ollama_binary_path,
        })
    }

    fn find_ollama_binary() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        let (which_cmd, binary_name) = ("where", "ollama.exe");

        #[cfg(not(target_os = "windows"))]
        let (which_cmd, binary_name) = ("which", "ollama");

        if let Ok(output) = Command::new(which_cmd).arg(binary_name).output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path_str.is_empty() {
                    let first_path = path_str.lines().next().unwrap_or(&path_str);
                    return Some(PathBuf::from(first_path));
                }
            }
        }

        #[cfg(target_os = "windows")]
        let common_paths = vec![
            r"C:\Program Files\Ollama\ollama.exe",
            r"C:\Program Files (x86)\Ollama\ollama.exe",
            format!(r"{}\AppData\Local\Programs\Ollama\ollama.exe",
                env::var("USERPROFILE").unwrap_or_default()),
        ];

        #[cfg(target_os = "macos")]
        let common_paths = vec![
            "/usr/local/bin/ollama",
            "/opt/homebrew/bin/ollama",
            "/usr/bin/ollama",
            "/opt/local/bin/ollama",
        ];

        #[cfg(target_os = "linux")]
        let common_paths = vec![
            "/usr/local/bin/ollama",
            "/usr/bin/ollama",
            "/opt/ollama/bin/ollama",
            format!("{}/.local/bin/ollama",
                env::var("HOME").unwrap_or_default()),
        ];

        for path in common_paths {
            let path_buf = PathBuf::from(path);
            if path_buf.exists() && path_buf.is_file() {
                return Some(path_buf);
            }
        }

        None
    }

    fn get_ollama_command(&self) -> Command {
        if let Some(ref path) = self.ollama_binary_path {
            Command::new(path)
        } else {
            Command::new("ollama")
        }
    }

    pub fn get_ollama_binary_path(&self) -> String {
        if let Some(ref path) = self.ollama_binary_path {
            path.to_string_lossy().to_string()
        } else {
            "ollama".to_string()
        }
    }

    pub fn is_model_installed(&self) -> Result<bool, LLMServiceError> {
        let mut cmd = self.get_ollama_command();
        let output = cmd.arg("list").output()?;

        if !output.status.success() {
            return Ok(false);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        Ok(stdout.contains(&self.ollama_model))
    }

    pub fn check_ollama_installed(&self) -> bool {
        if self.ollama_binary_path.is_some() {
            return true;
        }

        let mut cmd = self.get_ollama_command();
        cmd.arg("--version").output().is_ok()
    }

    pub async fn is_ollama_running(&self) -> Result<bool, LLMServiceError> {
        match self.client.get(&self.ollama_host).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    pub async fn generate_chat_response(&self, messages: Vec<ChatMessage>) -> Result<ChatResponse, LLMServiceError> {
        let request = OllamaChatRequest {
            model: self.ollama_model.clone(),
            messages,
            stream: false,
        };

        let ollama_response: OllamaChatResponse = self
            .client
            .post(format!("{}/api/chat", self.ollama_host))
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        Ok(ollama_response.message)
    }

    pub async fn generate_response(&self, prompt: &str) -> Result<String, LLMServiceError> {
        let request = OllamaRequest {
            model: self.ollama_model.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let ollama_response: OllamaResponse = self
            .client
            .post(format!("{}/api/generate", self.ollama_host))
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        Ok(ollama_response.response)
    }
}

