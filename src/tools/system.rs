use super::{Tool, ToolDefinition};
use async_trait::async_trait;
use serde_json::json;
use sysinfo::{CpuRefreshKind, ProcessRefreshKind, RefreshKind, System};

pub struct SystemInfoTool;

#[async_trait]
impl Tool for SystemInfoTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "system_info".to_string(),
            description: "Get detailed CPU, Memory, OS, and system status information.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn execute(&self, _arguments: serde_json::Value) -> Result<String, String> {
        let mut sys = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_processes(ProcessRefreshKind::new()),
        );
        sys.refresh_all();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        sys.refresh_cpu();

        let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
        let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
        let host_name = System::host_name().unwrap_or_else(|| "Unknown".to_string());
        let total_mem_mb = sys.total_memory() / 1024 / 1024;
        let used_mem_mb = sys.used_memory() / 1024 / 1024;
        let cpu_count = sys.cpus().len();
        let global_cpu_usage = sys.global_cpu_info().cpu_usage();

        Ok(format!(
            "System Information:\n- Host: {}\n- OS: {} {}\n- CPU Cores: {}\n- CPU Usage: {:.1}%\n- RAM Used: {} MB / {} MB ({:.1}%)",
            host_name,
            os_name,
            os_version,
            cpu_count,
            global_cpu_usage,
            used_mem_mb,
            total_mem_mb,
            (used_mem_mb as f64 / total_mem_mb as f64) * 100.0
        ))
    }
}

pub struct ProcessListTool;

#[async_trait]
impl Tool for ProcessListTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "list_processes".to_string(),
            description: "List running system processes, sorted by memory usage or filter by name.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "filter": {
                        "type": "string",
                        "description": "Optional process name filter (case-insensitive)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Max processes to return (default 25)"
                    }
                }
            }),
        }
    }

    async fn execute(&self, arguments: serde_json::Value) -> Result<String, String> {
        let filter = arguments
            .get("filter")
            .and_then(|v| v.as_str())
            .map(|s| s.to_lowercase());
        let limit = arguments
            .get("limit")
            .and_then(|v| v.as_u64())
            .unwrap_or(25) as usize;

        let mut sys = System::new_all();
        sys.refresh_all();

        let mut procs: Vec<_> = sys
            .processes()
            .values()
            .filter(|p| {
                if let Some(f) = &filter {
                    p.name().to_lowercase().contains(f)
                } else {
                    true
                }
            })
            .collect();

        procs.sort_by(|a, b| b.memory().cmp(&a.memory()));

        let mut lines = Vec::new();
        lines.push(format!("{:<8} {:<25} {:<10} {:<10}", "PID", "NAME", "RAM (MB)", "CPU (%)"));
        lines.push("-".repeat(60));

        for p in procs.into_iter().take(limit) {
            lines.push(format!(
                "{:<8} {:<25} {:<10.1} {:<10.1}",
                p.pid(),
                p.name(),
                p.memory() as f64 / 1024.0 / 1024.0,
                p.cpu_usage()
            ));
        }

        Ok(lines.join("\n"))
    }
}
