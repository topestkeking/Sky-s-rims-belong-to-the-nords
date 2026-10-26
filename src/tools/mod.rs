pub mod fs;
pub mod http;
pub mod shell;
pub mod system;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub call_id: String,
    pub tool_name: String,
    pub success: bool,
    pub output: String,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn definition(&self) -> ToolDefinition;
    async fn execute(&self, arguments: Value) -> Result<String, String>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };

        registry.register(fs::FileReadTool);
        registry.register(fs::FileWriteTool);
        registry.register(fs::DirectoryListTool);
        registry.register(shell::ShellTool);
        registry.register(system::SystemInfoTool);
        registry.register(system::ProcessListTool);
        registry.register(http::HttpFetchTool::new());

        registry
    }

    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        let def = tool.definition();
        self.tools.insert(def.name.clone(), Arc::new(tool));
    }

    pub fn get_definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.definition()).collect()
    }

    pub async fn execute_call(&self, call: &ToolCall) -> ToolResult {
        if let Some(tool) = self.tools.get(&call.name) {
            match tool.execute(call.arguments.clone()).await {
                Ok(output) => ToolResult {
                    call_id: call.id.clone(),
                    tool_name: call.name.clone(),
                    success: true,
                    output,
                },
                Err(err) => ToolResult {
                    call_id: call.id.clone(),
                    tool_name: call.name.clone(),
                    success: false,
                    output: err,
                },
            }
        } else {
            ToolResult {
                call_id: call.id.clone(),
                tool_name: call.name.clone(),
                success: false,
                output: format!("Tool '{}' not found", call.name),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_shell_tool_execution() {
        let registry = ToolRegistry::new();
        let call = ToolCall {
            id: "call_1".to_string(),
            name: "execute_shell".to_string(),
            arguments: json!({ "command": "echo 'Hermes-Claw active'" }),
        };

        let result = registry.execute_call(&call).await;
        assert!(result.success);
        assert!(result.output.contains("Hermes-Claw active"));
    }

    #[tokio::test]
    async fn test_system_info_tool() {
        let registry = ToolRegistry::new();
        let call = ToolCall {
            id: "call_2".to_string(),
            name: "system_info".to_string(),
            arguments: json!({}),
        };

        let result = registry.execute_call(&call).await;
        assert!(result.success);
        assert!(result.output.contains("System Information"));
    }
}
