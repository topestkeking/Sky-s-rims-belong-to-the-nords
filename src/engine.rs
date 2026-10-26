use crate::config::Config;
use crate::providers::{AIProvider, ChatMessage, OllamaProvider, OpenAIProvider, ProviderModelInfo};
use crate::tools::{ToolRegistry, ToolResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSessionRequest {
    pub provider: String,
    pub model: String,
    pub user_message: String,
    #[serde(default)]
    pub history: Vec<ChatMessage>,
    pub max_tool_iterations: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSessionResponse {
    pub assistant_reply: String,
    pub updated_history: Vec<ChatMessage>,
    pub executed_tools: Vec<ToolResult>,
}

pub struct AgentEngine {
    config: Arc<RwLock<Config>>,
    registry: Arc<ToolRegistry>,
    openai_provider: Arc<OpenAIProvider>,
    ollama_provider: Arc<OllamaProvider>,
}

impl AgentEngine {
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            registry: Arc::new(ToolRegistry::new()),
            openai_provider: Arc::new(OpenAIProvider::new()),
            ollama_provider: Arc::new(OllamaProvider::new()),
        }
    }

    pub fn get_tool_registry(&self) -> Arc<ToolRegistry> {
        self.registry.clone()
    }

    pub async fn get_config(&self) -> Config {
        self.config.read().await.clone()
    }

    pub async fn update_config(&self, new_config: Config) {
        let mut w = self.config.write().await;
        *w = new_config;
    }

    pub async fn list_all_models(&self) -> Vec<ProviderModelInfo> {
        let cfg = self.get_config().await;
        let mut models = Vec::new();

        if let Ok(mut m) = self.openai_provider.list_models(&cfg).await {
            models.append(&mut m);
        }

        if let Ok(mut m) = self.ollama_provider.list_models(&cfg).await {
            models.append(&mut m);
        }

        models
    }

    pub async fn process_chat(
        &self,
        req: ChatSessionRequest,
    ) -> Result<ChatSessionResponse, String> {
        let cfg = self.get_config().await;
        let provider_name = req.provider.to_lowercase();
        let max_iterations = req.max_tool_iterations.unwrap_or(5);

        let provider: Arc<dyn AIProvider> = match provider_name.as_str() {
            "ollama" => self.ollama_provider.clone(),
            _ => self.openai_provider.clone(),
        };

        let mut history = req.history.clone();

        if history.is_empty() || history[0].role != "system" {
            history.insert(
                0,
                ChatMessage {
                    role: "system".to_string(),
                    content: cfg.system_prompt.clone(),
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                },
            );
        }

        history.push(ChatMessage {
            role: "user".to_string(),
            content: req.user_message.clone(),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        });

        let mut executed_tools = Vec::new();
        let mut iterations = 0;
        let tools_def = self.registry.get_definitions();

        loop {
            iterations += 1;
            if iterations > max_iterations {
                break;
            }

            let resp = provider
                .chat_completion(&cfg, &req.model, &history, &tools_def)
                .await?;

            history.push(resp.message.clone());

            if resp.tool_calls.is_empty() {
                break;
            }

            for call in resp.tool_calls {
                let res = self.registry.execute_call(&call).await;
                executed_tools.push(res.clone());

                history.push(ChatMessage {
                    role: "tool".to_string(),
                    content: res.output.clone(),
                    tool_calls: None,
                    tool_call_id: Some(call.id.clone()),
                    name: Some(call.name.clone()),
                });
            }
        }

        let assistant_reply = history
            .last()
            .map(|m| m.content.clone())
            .unwrap_or_default();

        Ok(ChatSessionResponse {
            assistant_reply,
            updated_history: history,
            executed_tools,
        })
    }
}
