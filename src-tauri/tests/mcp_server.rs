use std::{
    env,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use serde_json::{Value, json};

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/fixtures")
        .canonicalize()
        .expect("fixtures root resolves")
}

#[test]
fn mcp_server_exposes_session_query_tools() {
    let mut server = spawn_fixture_mcp_server();

    let initialize = server.request(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {
                "name": "osm-test",
                "version": "0.0.0"
            }
        }
    }));

    assert_eq!(
        initialize
            .get("result")
            .and_then(|result| result.get("serverInfo"))
            .and_then(|info| info.get("name"))
            .and_then(Value::as_str),
        Some("open-session-manager")
    );
    assert!(
        initialize
            .get("result")
            .and_then(|result| result.get("capabilities"))
            .and_then(|capabilities| capabilities.get("tools"))
            .is_some()
    );

    server.notify(json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    }));

    let tools = server.request(json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list"
    }));
    let tool_names = tools
        .get("result")
        .and_then(|result| result.get("tools"))
        .and_then(Value::as_array)
        .expect("tools array exists")
        .iter()
        .filter_map(|tool| tool.get("name").and_then(Value::as_str))
        .collect::<Vec<_>>();
    assert!(tool_names.contains(&"list_sessions"));
    assert!(tool_names.contains(&"search_sessions"));
    assert!(tool_names.contains(&"get_session"));

    let list = server.request(json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "tools/call",
        "params": {
            "name": "list_sessions",
            "arguments": {
                "assistant": "claude-code"
            }
        }
    }));
    let list_text = list
        .get("result")
        .and_then(|result| result.get("content"))
        .and_then(Value::as_array)
        .and_then(|content| content.first())
        .and_then(|item| item.get("text"))
        .and_then(Value::as_str)
        .expect("list result text exists");
    let list_payload: Value = serde_json::from_str(list_text).expect("list text is json");
    assert!(
        list_payload
            .get("sessions")
            .and_then(Value::as_array)
            .is_some_and(|sessions| sessions.iter().any(|session| {
                session.get("sessionId").and_then(Value::as_str) == Some("claude-ses-1")
            }))
    );

    let search = server.request(json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "tools/call",
        "params": {
            "name": "search_sessions",
            "arguments": {
                "query": "Claude"
            }
        }
    }));
    let search_text = search
        .get("result")
        .and_then(|result| result.get("content"))
        .and_then(Value::as_array)
        .and_then(|content| content.first())
        .and_then(|item| item.get("text"))
        .and_then(Value::as_str)
        .expect("search result text exists");
    let search_payload: Value = serde_json::from_str(search_text).expect("search text is json");
    assert_eq!(
        search_payload
            .get("hits")
            .and_then(Value::as_array)
            .and_then(|hits| hits.first())
            .and_then(|hit| hit.get("sessionId"))
            .and_then(Value::as_str),
        Some("claude-ses-1")
    );

    let get = server.request(json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "tools/call",
        "params": {
            "name": "get_session",
            "arguments": {
                "sessionId": "claude-ses-1"
            }
        }
    }));
    let get_text = get
        .get("result")
        .and_then(|result| result.get("content"))
        .and_then(Value::as_array)
        .and_then(|content| content.first())
        .and_then(|item| item.get("text"))
        .and_then(Value::as_str)
        .expect("get result text exists");
    let get_payload: Value = serde_json::from_str(get_text).expect("get text is json");
    assert_eq!(
        get_payload.get("assistant").and_then(Value::as_str),
        Some("claude-code")
    );
}

fn spawn_fixture_mcp_server() -> RunningMcpServer {
    let mut child = Command::new(env!("CARGO_BIN_EXE_open-session-manager-core"))
        .args([
            "mcp",
            "--fixtures",
            fixtures_root().to_str().expect("fixtures path as str"),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn mcp server");

    RunningMcpServer {
        stdin: child.stdin.take().expect("stdin is piped"),
        stdout: BufReader::new(child.stdout.take().expect("stdout is piped")),
        child,
    }
}

struct RunningMcpServer {
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    child: Child,
}

impl RunningMcpServer {
    fn notify(&mut self, payload: Value) {
        writeln!(self.stdin, "{}", serde_json::to_string(&payload).expect("serialize payload"))
            .expect("write notification");
        self.stdin.flush().expect("flush notification");
    }

    fn request(&mut self, payload: Value) -> Value {
        self.notify(payload);
        self.read_response()
    }

    fn read_response(&mut self) -> Value {
        let mut line = String::new();
        let bytes = self.stdout.read_line(&mut line).expect("read response line");
        assert!(bytes > 0, "mcp server exited before returning a response");
        serde_json::from_str(line.trim()).expect("response line is json")
    }
}

impl Drop for RunningMcpServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
