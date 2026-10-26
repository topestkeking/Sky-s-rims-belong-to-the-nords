use super::{Tool, ToolDefinition};
use async_trait::async_trait;
use serde_json::json;
use tokio::process::Command;

pub struct ShellTool;

#[async_trait]
impl Tool for ShellTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "execute_shell".to_string(),
            description: "Execute a shell / bash command directly on the host machine and return stdout/stderr.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The shell command to run (e.g. 'ls -la', 'ps aux', 'python3 script.py')"
                    }
                },
                "required": ["command"]
            }),
        }
    }

    async fn execute(&self, arguments: serde_json::Value) -> Result<String, String> {
        let command = arguments
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'command' argument".to_string())?;

        let output = Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .await
            .map_err(|e| format!("Failed to execute command: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let mut res = String::new();
        if !stdout.is_empty() {
            res.push_str(&format!("STDOUT:\n{}\n", stdout.trim()));
        }
        if !stderr.is_empty() {
            res.push_str(&format!("STDERR:\n{}\n", stderr.trim()));
        }
        if res.is_empty() {
            res = format!("Command executed successfully with exit code: {}", output.status);
        }

        Ok(res)
    }
}
