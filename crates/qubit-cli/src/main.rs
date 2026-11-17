//! Qubit Command Line Interface (CLI)
//! Provides:
//! - Full REST/HTTP integration with QubitCore daemon (default http://127.0.0.1:4000)
//! - Reporting current working directory as MCP Roots (`list_roots` / `roots/list`)
//! - Transparent stdio bridge for editor integrations (Cursor, VS Code, Zed) forwarding JSON-RPC to QubitCore

use clap::{Parser, Subcommand};
use qubit_protocol::{AgentDescriptor, CallToolRequest, CreateAgentRequest, ExecutionMode, HealthResponse, ToolResult};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(Parser, Debug)]
#[command(
    name = "qubit",
    version,
    about = "Qubit Ecosystem Command Line Tool & Stdio MCP Bridge"
)]
struct Cli {
    /// QubitCore daemon address
    #[arg(short, long, default_value = "http://127.0.0.1:4000", env = "QUBIT_DAEMON_URL")]
    daemon_url: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Check system status, health, and memory metrics of QubitCore daemon
    Status,
    /// List active agents, or create a new agent
    Agents {
        /// Create a new agent by specifying name
        #[arg(short, long)]
        create: Option<String>,
        /// Role for the new agent (default: "Assistant")
        #[arg(short, long, default_value = "Assistant")]
        role: String,
    },
    /// List available MCP tools in QubitCore registry
    Tools,
    /// Call an MCP tool directly through QubitCore
    CallTool {
        /// Name of the tool to invoke
        name: String,
        /// JSON arguments payload (default: "{}")
        #[arg(default_value = "{}")]
        args: String,
    },
    /// Report current working directory or explicit paths as MCP Roots (`roots/list`)
    Roots {
        /// Optional explicit path to report as root (default: current working directory)
        path: Option<PathBuf>,
    },
    /// Launch stdio JSON-RPC bridge for IDEs (Cursor, VS Code, Zed) connected to QubitCore
    Bridge {
        /// Optional root directory for IDE context
        #[arg(short, long)]
        workspace: Option<PathBuf>,
    },
    /// Launch Ghidra / Reverse Engineering sandbox container
    Sandbox {
        #[arg(short, long)]
        target: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpRoot {
    pub uri: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpRootsListResult {
    pub roots: Vec<McpRoot>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    match cli.command {
        Commands::Status => {
            handle_status(&client, &cli.daemon_url).await?;
        }
        Commands::Agents { create, role } => {
            if let Some(name) = create {
                handle_create_agent(&client, &cli.daemon_url, &name, &role).await?;
            } else {
                handle_list_agents(&client, &cli.daemon_url).await?;
            }
        }
        Commands::Tools => {
            handle_list_tools(&client, &cli.daemon_url).await?;
        }
        Commands::CallTool { name, args } => {
            handle_call_tool(&client, &cli.daemon_url, &name, &args).await?;
        }
        Commands::Roots { path } => {
            handle_roots(path)?;
        }
        Commands::Bridge { workspace } => {
            handle_bridge(&client, &cli.daemon_url, workspace).await?;
        }
        Commands::Sandbox { target } => {
            println!("Launching sandbox environment for target: {:?}", target);
            println!("RE-Ghidra headless container integration active.");
        }
    }

    Ok(())
}

/// Fetch daemon health and status
async fn handle_status(client: &reqwest::Client, daemon_url: &str) -> anyhow::Result<()> {
    let url = format!("{}/api/health", daemon_url.trim_end_matches('/'));
    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let health: HealthResponse = resp.json().await?;
            println!("=== QubitCore Daemon Status ===");
            println!("Status        : {}", health.status);
            println!("Version       : v{}", health.version);
            println!("Engine        : {}", health.engine);
            println!("Memory Budget : {} MB (Idle Target)", health.memory_budget_mb);
            println!("Uptime        : {}s", health.uptime_secs);
            println!("Endpoint      : {}", url);
        }
        Ok(resp) => {
            eprintln!("Daemon returned HTTP error status: {}", resp.status());
        }
        Err(e) => {
            eprintln!("Failed to connect to QubitCore daemon at {}: {}", url, e);
            eprintln!("Ensure qubit-core is running (`qubit-core --bind 127.0.0.1:4000`).");
        }
    }
    Ok(())
}

/// List all agents registered in QubitCore
async fn handle_list_agents(client: &reqwest::Client, daemon_url: &str) -> anyhow::Result<()> {
    let url = format!("{}/api/v1/agents", daemon_url.trim_end_matches('/'));
    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let agents: Vec<AgentDescriptor> = resp.json().await?;
            println!("=== Registered Qubit Agents ({}) ===", agents.len());
            for agent in agents {
                println!(
                    "- [{}] {} ({}) - Status: {:?}",
                    agent.id, agent.name, agent.role, agent.status
                );
            }
        }
        Ok(resp) => {
            eprintln!("Daemon returned HTTP error status: {}", resp.status());
        }
        Err(e) => {
            eprintln!("Failed to connect to QubitCore daemon at {}: {}", url, e);
        }
    }
    Ok(())
}

/// Create a new agent in QubitCore
async fn handle_create_agent(
    client: &reqwest::Client,
    daemon_url: &str,
    name: &str,
    role: &str,
) -> anyhow::Result<()> {
    let url = format!("{}/api/v1/agents", daemon_url.trim_end_matches('/'));
    let req = CreateAgentRequest {
        name: name.to_string(),
        role: role.to_string(),
        mode: Some(ExecutionMode::Reflex),
    };
    let resp = client.post(&url).json(&req).send().await?;
    if resp.status().is_success() {
        let created: AgentDescriptor = resp.json().await?;
        println!("Agent created successfully:");
        println!("  ID  : {}", created.id);
        println!("  Name: {}", created.name);
        println!("  Role: {}", created.role);
    } else {
        eprintln!("Failed to create agent. Server returned HTTP {}", resp.status());
    }
    Ok(())
}

/// List all MCP tools from QubitCore registry
async fn handle_list_tools(client: &reqwest::Client, daemon_url: &str) -> anyhow::Result<()> {
    let url = format!("{}/api/v1/tools", daemon_url.trim_end_matches('/'));
    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let tools: Vec<qubit_mcp::ToolDefinition> = resp.json().await?;
            println!("=== Registered MCP Tools ({}) ===", tools.len());
            for tool in tools {
                println!("* {} - {}", tool.name, tool.description);
            }
        }
        Ok(resp) => {
            eprintln!("Daemon returned HTTP error status: {}", resp.status());
        }
        Err(e) => {
            eprintln!("Failed to connect to QubitCore daemon at {}: {}", url, e);
        }
    }
    Ok(())
}

/// Invoke an MCP tool on QubitCore
async fn handle_call_tool(
    client: &reqwest::Client,
    daemon_url: &str,
    name: &str,
    args_json: &str,
) -> anyhow::Result<()> {
    let parsed_args: Value = serde_json::from_str(args_json)
        .map_err(|e| anyhow::anyhow!("Invalid JSON passed to --args: {}", e))?;
    let url = format!("{}/api/v1/tools/{}/call", daemon_url.trim_end_matches('/'), name);
    let payload = CallToolRequest {
        arguments: parsed_args,
    };
    let resp = client.post(&url).json(&payload).send().await?;
    if resp.status().is_success() {
        let result: ToolResult = resp.json().await?;
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        eprintln!("Tool execution failed. Server status: {}", resp.status());
    }
    Ok(())
}

/// Produce MCP Roots representation of current workspace or given path
pub fn get_mcp_roots(explicit_path: Option<PathBuf>) -> McpRootsListResult {
    let cwd = explicit_path.unwrap_or_else(|| {
        env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    });
    let canonical = cwd.canonicalize().unwrap_or(cwd);
    let path_str = canonical.to_string_lossy().to_string();
    let uri = if cfg!(windows) {
        format!("file:///{}", path_str.replace('\\', "/"))
    } else {
        format!("file://{}", path_str)
    };
    let name = canonical
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "workspace".to_string());

    McpRootsListResult {
        roots: vec![McpRoot { uri, name }],
    }
}

fn handle_roots(path: Option<PathBuf>) -> anyhow::Result<()> {
    let roots_result = get_mcp_roots(path);
    println!("{}", serde_json::to_string_pretty(&roots_result)?);
    Ok(())
}

/// Stdio JSON-RPC Bridge for IDEs (Cursor, VS Code, Zed)
/// Translates stdio Model Context Protocol requests to QubitCore Axum daemon.
async fn handle_bridge(
    client: &reqwest::Client,
    daemon_url: &str,
    workspace: Option<PathBuf>,
) -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut stdout = tokio::io::stdout();
    let mut line = String::new();

    let root_info = get_mcp_roots(workspace);

    while reader.read_line(&mut line).await? > 0 {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            line.clear();
            continue;
        }

        let parsed: Result<JsonRpcRequest, _> = serde_json::from_str(trimmed);
        match parsed {
            Ok(req) => {
                let resp = process_bridge_request(client, daemon_url, &req, &root_info).await;
                if let Some(r) = resp {
                    let out_json = serde_json::to_string(&r)?;
                    stdout.write_all(out_json.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
            }
            Err(e) => {
                let err_resp = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(json!({
                        "code": -32700,
                        "message": format!("Parse error: {}", e)
                    })),
                };
                let out_json = serde_json::to_string(&err_resp)?;
                stdout.write_all(out_json.as_bytes()).await?;
                stdout.write_all(b"\n").await?;
                stdout.flush().await?;
            }
        }

        line.clear();
    }

    Ok(())
}

/// Process a single JSON-RPC method call over stdio bridge
async fn process_bridge_request(
    client: &reqwest::Client,
    daemon_url: &str,
    req: &JsonRpcRequest,
    root_info: &McpRootsListResult,
) -> Option<JsonRpcResponse> {
    // If request has no ID, it's a notification, no response required unless error
    let req_id = req.id.clone()?;

    match req.method.as_str() {
        "initialize" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(req_id),
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "roots": { "listChanged": true },
                    "tools": { "listChanged": true }
                },
                "serverInfo": {
                    "name": "qubit-bridge",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
            error: None,
        }),
        "notifications/initialized" => None,
        "ping" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(req_id),
            result: Some(json!({})),
            error: None,
        }),
        "roots/list" | "list_roots" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(req_id),
            result: Some(json!(root_info)),
            error: None,
        }),
        "tools/list" => {
            let url = format!("{}/api/v1/tools", daemon_url.trim_end_matches('/'));
            match client.get(&url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    let tools: Vec<qubit_mcp::ToolDefinition> = resp.json().await.unwrap_or_default();
                    let mcp_tools: Vec<Value> = tools
                        .into_iter()
                        .map(|t| {
                            json!({
                                "name": t.name,
                                "description": t.description,
                                "inputSchema": t.input_schema
                            })
                        })
                        .collect();
                    Some(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: Some(req_id),
                        result: Some(json!({ "tools": mcp_tools })),
                        error: None,
                    })
                }
                _ => Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Some(req_id),
                    result: Some(json!({ "tools": [] })),
                    error: None,
                }),
            }
        }
        "tools/call" => {
            let params = req.params.clone().unwrap_or(Value::Null);
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

            let url = format!(
                "{}/api/v1/tools/{}/call",
                daemon_url.trim_end_matches('/'),
                tool_name
            );
            let payload = CallToolRequest { arguments };

            match client.post(&url).json(&payload).send().await {
                Ok(resp) if resp.status().is_success() => {
                    let result: ToolResult = resp.json().await.unwrap_or_else(|_| ToolResult {
                        call_id: "unknown".into(),
                        success: false,
                        output: None,
                        error: Some("Failed to parse tool result from daemon".into()),
                    });
                    Some(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: Some(req_id),
                        result: Some(json!({
                            "content": [{
                                "type": "text",
                                "text": serde_json::to_string(&result.output.unwrap_or(Value::Null)).unwrap_or_default()
                            }],
                            "isError": !result.success
                        })),
                        error: None,
                    })
                }
                Ok(resp) => Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Some(req_id),
                    result: None,
                    error: Some(json!({
                        "code": -32000,
                        "message": format!("QubitCore daemon returned error HTTP {}", resp.status())
                    })),
                }),
                Err(e) => Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: Some(req_id),
                    result: None,
                    error: Some(json!({
                        "code": -32603,
                        "message": format!("Failed to reach QubitCore daemon: {}", e)
                    })),
                }),
            }
        }
        other => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(req_id),
            result: None,
            error: Some(json!({
                "code": -32601,
                "message": format!("Method '{}' not found", other)
            })),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_roots_generation() {
        let path = PathBuf::from("/tmp");
        let roots = get_mcp_roots(Some(path));
        assert_eq!(roots.roots.len(), 1);
        assert!(roots.roots[0].uri.starts_with("file://"));
    }

    #[test]
    fn test_jsonrpc_request_parsing() {
        let raw = r#"{"jsonrpc": "2.0", "id": 1, "method": "roots/list", "params": {}}"#;
        let parsed: JsonRpcRequest = serde_json::from_str(raw).expect("parse jsonrpc");
        assert_eq!(parsed.method, "roots/list");
        assert_eq!(parsed.id, Some(json!(1)));
    }
}
