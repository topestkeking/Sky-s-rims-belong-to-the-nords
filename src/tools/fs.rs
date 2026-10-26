use super::{Tool, ToolDefinition};
use async_trait::async_trait;
use serde_json::json;
use std::path::Path;
use tokio::fs;

pub struct FileReadTool;

#[async_trait]
impl Tool for FileReadTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "read_file".to_string(),
            description: "Read the content of a file on the local filesystem.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "filepath": {
                        "type": "string",
                        "description": "Path to the file to read"
                    }
                },
                "required": ["filepath"]
            }),
        }
    }

    async fn execute(&self, arguments: serde_json::Value) -> Result<String, String> {
        let filepath = arguments
            .get("filepath")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'filepath' argument".to_string())?;

        fs::read_to_string(filepath)
            .await
            .map_err(|e| format!("Failed to read file '{}': {}", filepath, e))
    }
}

pub struct FileWriteTool;

#[async_trait]
impl Tool for FileWriteTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "write_file".to_string(),
            description: "Write content to a file on the local filesystem (creates parent directories if needed).".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "filepath": {
                        "type": "string",
                        "description": "Path to the file to write"
                    },
                    "content": {
                        "type": "string",
                        "description": "Text content to write"
                    }
                },
                "required": ["filepath", "content"]
            }),
        }
    }

    async fn execute(&self, arguments: serde_json::Value) -> Result<String, String> {
        let filepath = arguments
            .get("filepath")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'filepath' argument".to_string())?;

        let content = arguments
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'content' argument".to_string())?;

        if let Some(parent) = Path::new(filepath).parent() {
            let _ = fs::create_dir_all(parent).await;
        }

        fs::write(filepath, content)
            .await
            .map_err(|e| format!("Failed to write file '{}': {}", filepath, e))?;

        Ok(format!("Successfully wrote {} bytes to '{}'", content.len(), filepath))
    }
}

pub struct DirectoryListTool;

#[async_trait]
impl Tool for DirectoryListTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "list_directory".to_string(),
            description: "List files and directories in a given path.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Directory path (defaults to current directory '.' if empty)"
                    }
                }
            }),
        }
    }

    async fn execute(&self, arguments: serde_json::Value) -> Result<String, String> {
        let dir_path = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let mut entries = fs::read_dir(dir_path)
            .await
            .map_err(|e| format!("Failed to read directory '{}': {}", dir_path, e))?;

        let mut list = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            let meta = entry.metadata().await;
            let kind = match meta {
                Ok(m) if m.is_dir() => "[DIR] ",
                Ok(m) if m.is_file() => "[FILE]",
                _ => "[OTHER]",
            };
            if let Ok(file_name) = entry.file_name().into_string() {
                list.push(format!("{} {}", kind, file_name));
            }
        }

        list.sort();
        Ok(list.join("\n"))
    }
}
