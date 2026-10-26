use crate::config::Config;
use crate::tools::{ToolCall, ToolDefinition};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderModelInfo {
    pub provider: String,
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub message: ChatMessage,
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: Option<String>,
}

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn list_models(&self, config: &Config) -> Result<Vec<ProviderModelInfo>, String>;
    async fn chat_completion(
        &self,
        config: &Config,
        model: &str,
        messages: &[ChatMessage],
        tools: &[ToolDefinition],
    ) -> Result<ChatCompletionResponse, String>;
}

pub struct OpenAIProvider {
    client: reqwest::Client,
}

impl OpenAIProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn list_models(&self, _config: &Config) -> Result<Vec<ProviderModelInfo>, String> {
        Ok(vec![
            ProviderModelInfo {
                provider: "openai".to_string(),
                id: "gpt-4o".to_string(),
                name: "GPT-4o (High Speed & Intelligence)".to_string(),
            },
            ProviderModelInfo {
                provider: "openai".to_string(),
                id: "gpt-4o-mini".to_string(),
                name: "GPT-4o Mini (Ultra Fast)".to_string(),
            },
            ProviderModelInfo {
                provider: "openai".to_string(),
                id: "o3-mini".to_string(),
                name: "o3-mini (Reasoning)".to_string(),
            },
        ])
    }

    async fn chat_completion(
        &self,
        config: &Config,
        model: &str,
        messages: &[ChatMessage],
        tools: &[ToolDefinition],
    ) -> Result<ChatCompletionResponse, String> {
        let api_key = config
            .openai_api_key
            .as_ref()
            .ok_or_else(|| "OpenAI API key not configured".to_string())?;

        let url = format!("{}/chat/completions", config.openai_base_url.trim_end_matches('/'));

        let formatted_tools: Vec<Value> = tools
            .iter()
            .map(|t| {
                json!({
                    "type": "function",
                    "function": {
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.parameters
                    }
                })
            })
            .collect();

        let mut payload = json!({
            "model": model,
            "messages": messages,
        });

        if !formatted_tools.is_empty() {
            payload["tools"] = json!(formatted_tools);
            payload["tool_choice"] = json!("auto");
        }

        let res = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("OpenAI request failed: {}", e))?;

        let body: Value = res
            .json()
            .await
            .map_err(|e| format!("Failed to parse OpenAI response: {}", e))?;

        if let Some(err) = body.get("error") {
            return Err(format!("OpenAI Error: {}", err));
        }

        let choice = body["choices"][0].clone();
        let msg_val = &choice["message"];
        let content = msg_val["content"].as_str().unwrap_or("").to_string();

        let mut tool_calls = Vec::new();
        if let Some(t_array) = msg_val["tool_calls"].as_array() {
            for tc in t_array {
                let id = tc["id"].as_str().unwrap_or_default().to_string();
                let name = tc["function"]["name"].as_str().unwrap_or_default().to_string();
                let args_str = tc["function"]["arguments"].as_str().unwrap_or("{}");
                let args: Value = serde_json::from_str(args_str).unwrap_or(json!({}));
                tool_calls.push(ToolCall {
                    id,
                    name,
                    arguments: args,
                });
            }
        }

        let finish_reason = choice["finish_reason"].as_str().map(|s| s.to_string());

        Ok(ChatCompletionResponse {
            message: ChatMessage {
                role: "assistant".to_string(),
                content,
                tool_calls: if tool_calls.is_empty() {
                    None
                } else {
                    Some(tool_calls.clone())
                },
                tool_call_id: None,
                name: None,
            },
            tool_calls,
            finish_reason,
        })
    }
}

pub struct OllamaProvider {
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl AIProvider for OllamaProvider {
    async fn list_models(&self, config: &Config) -> Result<Vec<ProviderModelInfo>, String> {
        let url = format!("{}/api/tags", config.ollama_base_url.trim_end_matches('/'));
        let res = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

        let body: Value = res
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        let mut models = Vec::new();
        if let Some(arr) = body["models"].as_array() {
            for m in arr {
                if let Some(name) = m["name"].as_str() {
                    models.push(ProviderModelInfo {
                        provider: "ollama".to_string(),
                        id: name.to_string(),
                        name: format!("Ollama: {}", name),
                    });
                }
            }
        }

        if models.is_empty() {
            models.push(ProviderModelInfo {
                provider: "ollama".to_string(),
                id: "llama3".to_string(),
                name: "Ollama: llama3 (Default)".to_string(),
            });
        }

        Ok(models)
    }

    async fn chat_completion(
        &self,
        config: &Config,
        model: &str,
        messages: &[ChatMessage],
        _tools: &[ToolDefinition],
    ) -> Result<ChatCompletionResponse, String> {
        let url = format!("{}/api/chat", config.ollama_base_url.trim_end_matches('/'));

        let payload = json!({
            "model": model,
            "messages": messages,
            "stream": false
        });

        let res = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("Ollama request failed: {}", e))?;

        let body: Value = res
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        let content = body["message"]["content"].as_str().unwrap_or("").to_string();

        Ok(ChatCompletionResponse {
            message: ChatMessage {
                role: "assistant".to_string(),
                content,
                tool_calls: None,
                tool_call_id: None,
                name: None,
            },
            tool_calls: vec![],
            finish_reason: Some("stop".to_string()),
        })
    }
}
