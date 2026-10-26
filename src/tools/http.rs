use super::{Tool, ToolDefinition};
use async_trait::async_trait;
use serde_json::json;

pub struct HttpFetchTool {
    client: reqwest::Client,
}

impl HttpFetchTool {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl Tool for HttpFetchTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "http_fetch".to_string(),
            description: "Send HTTP GET/POST request to fetch data or call external APIs.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "The URL to fetch"
                    },
                    "method": {
                        "type": "string",
                        "description": "HTTP method: GET or POST (default: GET)"
                    },
                    "body": {
                        "type": "string",
                        "description": "Optional request body string for POST requests"
                    }
                },
                "required": ["url"]
            }),
        }
    }

    async fn execute(&self, arguments: serde_json::Value) -> Result<String, String> {
        let url = arguments
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'url' argument".to_string())?;

        let method = arguments
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET")
            .to_uppercase();

        let req = match method.as_str() {
            "POST" => {
                let mut builder = self.client.post(url);
                if let Some(body) = arguments.get("body").and_then(|v| v.as_str()) {
                    builder = builder.body(body.to_string());
                }
                builder
            }
            _ => self.client.get(url),
        };

        let response = req
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read HTTP body: {}", e))?;

        let truncated_body = if body.len() > 10000 {
            format!("{}... [Truncated, total {} bytes]", &body[..10000], body.len())
        } else {
            body
        };

        Ok(format!("HTTP Status: {}\nBody:\n{}", status, truncated_body))
    }
}
