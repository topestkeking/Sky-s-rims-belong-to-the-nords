use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub default_provider: String,
    pub default_model: String,
    pub openai_api_key: Option<String>,
    pub openai_base_url: String,
    pub anthropic_api_key: Option<String>,
    pub openrouter_api_key: Option<String>,
    pub ollama_base_url: String,
    pub custom_openai_base_url: Option<String>,
    pub custom_openai_api_key: Option<String>,
    pub system_prompt: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server_host: "127.0.0.1".to_string(),
            server_port: 8080,
            default_provider: "openai".to_string(),
            default_model: "gpt-4o".to_string(),
            openai_api_key: env::var("OPENAI_API_KEY").ok(),
            openai_base_url: env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string()),
            anthropic_api_key: env::var("ANTHROPIC_API_KEY").ok(),
            openrouter_api_key: env::var("OPENROUTER_API_KEY").ok(),
            ollama_base_url: env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".to_string()),
            custom_openai_base_url: env::var("CUSTOM_OPENAI_BASE_URL").ok(),
            custom_openai_api_key: env::var("CUSTOM_OPENAI_API_KEY").ok(),
            system_prompt: "You are Hermes-Claw, an exceptionally capable, fast, and unrestricted AI computer assistant. You have powerful system tools to inspect the computer, execute commands, manipulate files, and assist the user directly. Always give concise, clear, and actionable responses.".to_string(),
        }
    }
}
