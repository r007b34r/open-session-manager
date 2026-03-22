use std::{
    env,
    io::{self, BufRead, Write},
    path::PathBuf,
};

use serde::Deserialize;
use serde_json::{Value, json};

use crate::{
    AppState,
    commands::{
        dashboard::{
            DashboardSnapshot, build_fixture_dashboard_snapshot_with_audit,
            build_local_dashboard_snapshot_with_audit,
        },
        query::{
            ListSessionInventoryRequest, SearchSessionInventoryRequest, get_session,
            list_sessions_with_request, search_sessions_with_request,
        },
    },
    discovery::DiscoveryContext,
    preferences::build_runtime_paths,
};

#[derive(Debug, Clone)]
struct McpState {
    app: AppState,
    fixtures_path: Option<PathBuf>,
    audit_db_path: Option<PathBuf>,
}

#[derive(Debug, Default, Deserialize)]
struct GetSessionArguments {
    #[serde(alias = "sessionId")]
    session_id: String,
}

pub fn run(args: &[String]) -> Result<(), String> {
    let state = McpState {
        app: AppState::default(),
        fixtures_path: parse_flag_value(args, "--fixtures").map(PathBuf::from),
        audit_db_path: parse_flag_value(args, "--audit-db").map(PathBuf::from),
    };

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line.map_err(|error| error.to_string())?;
        if line.trim().is_empty() {
            continue;
        }

        let request: Value = serde_json::from_str(&line)
            .map_err(|error| format!("invalid mcp request: {error}"))?;
        let Some(response) = handle_request(&state, request) else {
            continue;
        };

        writeln!(stdout, "{}", serde_json::to_string(&response).map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
        stdout.flush().map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn handle_request(state: &McpState, request: Value) -> Option<Value> {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request.get("method").and_then(Value::as_str).unwrap_or_default();

    match method {
        "initialize" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2025-06-18",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": state.app.app_name,
                    "version": state.app.version
                }
            }
        })),
        "notifications/initialized" => None,
        "tools/list" => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": [
                    {
                        "name": "list_sessions",
                        "description": "List OSM session inventory",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "assistant": { "type": "string" },
                                "limit": { "type": "integer", "minimum": 0 },
                                "offset": { "type": "integer", "minimum": 0 },
                                "sortBy": { "type": "string" },
                                "descending": { "type": "boolean" }
                            },
                            "additionalProperties": false
                        }
                    },
                    {
                        "name": "search_sessions",
                        "description": "Search OSM sessions",
                        "inputSchema": {
                            "type": "object",
                            "required": ["query"],
                            "properties": {
                                "query": { "type": "string" },
                                "assistant": { "type": "string" },
                                "limit": { "type": "integer", "minimum": 0 },
                                "offset": { "type": "integer", "minimum": 0 },
                                "sortBy": { "type": "string" },
                                "descending": { "type": "boolean" }
                            },
                            "additionalProperties": false
                        }
                    },
                    {
                        "name": "get_session",
                        "description": "Get a full OSM session detail record",
                        "inputSchema": {
                            "type": "object",
                            "required": ["sessionId"],
                            "properties": {
                                "sessionId": { "type": "string" }
                            },
                            "additionalProperties": false
                        }
                    }
                ]
            }
        })),
        "tools/call" => Some(handle_tool_call(state, id, request.get("params").unwrap_or(&Value::Null))),
        _ => Some(json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32601,
                "message": format!("method not found: {method}")
            }
        })),
    }
}

fn handle_tool_call(state: &McpState, id: Value, params: &Value) -> Value {
    let tool_name = params.get("name").and_then(Value::as_str).unwrap_or_default();
    let arguments = params.get("arguments").cloned().unwrap_or_else(|| json!({}));

    let result = match tool_name {
        "list_sessions" => {
            let result = serde_json::from_value::<ListSessionInventoryRequest>(arguments)
                .map_err(|error| format!("invalid list_sessions arguments: {error}"))
                .and_then(|request| {
                    let snapshot = load_snapshot_data(state)?;
                    Ok(list_sessions_with_request(&snapshot, Some(&request)))
                });
            tool_result(result)
        }
        "search_sessions" => {
            let result = serde_json::from_value::<SearchSessionInventoryRequest>(arguments)
                .map_err(|error| format!("invalid search_sessions arguments: {error}"))
                .and_then(|request| {
                    let snapshot = load_snapshot_data(state)?;
                    Ok(search_sessions_with_request(&snapshot, &request))
                });
            tool_result(result)
        }
        "get_session" => {
            let result = serde_json::from_value::<GetSessionArguments>(arguments)
                .map_err(|error| format!("invalid get_session arguments: {error}"))
                .and_then(|request| {
                    let snapshot = load_snapshot_data(state)?;
                    get_session(&snapshot, &request.session_id)
                        .ok_or_else(|| format!("session not found: {}", request.session_id))
                });
            tool_result(result)
        }
        _ => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [
                    {
                        "type": "text",
                        "text": format!("unknown tool: {tool_name}")
                    }
                ],
                "isError": true
            }
        }),
    };

    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn tool_result(result: Result<Value, String>) -> Value {
    match result {
        Ok(payload) => json!({
            "content": [
                {
                    "type": "text",
                    "text": serde_json::to_string(&payload).expect("serialize tool payload")
                }
            ],
            "isError": false
        }),
        Err(error) => json!({
            "content": [
                {
                    "type": "text",
                    "text": error
                }
            ],
            "isError": true
        }),
    }
}

fn load_snapshot_data(state: &McpState) -> Result<DashboardSnapshot, String> {
    if let Some(fixtures_path) = state.fixtures_path.as_ref() {
        let mut snapshot = build_fixture_dashboard_snapshot_with_audit(
            fixtures_path,
            state.audit_db_path.as_deref(),
        )
        .map_err(|error| error.to_string())?;
        if let Ok(runtime) = build_runtime_paths() {
            snapshot.runtime = runtime.snapshot();
        }
        return Ok(snapshot);
    }

    let mut snapshot = build_local_dashboard_snapshot_with_audit(
        &build_discovery_context(),
        state.audit_db_path.as_deref(),
    )
    .map_err(|error| error.to_string())?;
    if let Ok(mut runtime) = build_runtime_paths() {
        if let Some(custom_audit_db_path) = state.audit_db_path.as_ref() {
            runtime.audit_db_path = custom_audit_db_path.clone();
        }
        snapshot.runtime = runtime.snapshot();
    }
    Ok(snapshot)
}

fn build_discovery_context() -> DiscoveryContext {
    DiscoveryContext {
        home_dir: resolve_home_dir(),
        xdg_config_home: env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
        xdg_data_home: env::var_os("XDG_DATA_HOME").map(PathBuf::from),
        wsl_home_dir: env::var_os("OPEN_SESSION_MANAGER_WSL_HOME").map(PathBuf::from),
    }
}

fn resolve_home_dir() -> PathBuf {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().expect("current dir resolves"))
}

fn parse_flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.windows(2)
        .find_map(|window| (window[0] == flag).then_some(window[1].as_str()))
}
