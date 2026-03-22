use std::{
    env, fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};

use serde_json::Value;

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/fixtures")
        .canonicalize()
        .expect("fixtures root resolves")
}

#[test]
fn serve_command_exposes_health_and_session_routes() {
    let sandbox = temp_root();
    let home_dir = sandbox.join("home");

    seed_session_fixture(
        &home_dir
            .join(".factory")
            .join("sessions")
            .join("project-a")
            .join("droid-session-1.jsonl"),
        "factory/sessions/project-a/droid-session-1.jsonl",
    );
    seed_session_fixture(
        &home_dir
            .join(".factory")
            .join("projects")
            .join("project-a")
            .join("stream-session-1.jsonl"),
        "factory/projects/project-a/stream-session-1.jsonl",
    );
    seed_session_fixture(
        &home_dir
            .join(".claude")
            .join("projects")
            .join("C--Projects-Claude-Demo")
            .join("claude-ses-1.jsonl"),
        "claude/projects/C--Projects-Claude-Demo/claude-ses-1.jsonl",
    );
    seed_session_fixture(
        &home_dir.join(".claude").join("settings.json"),
        "configs/claude/settings.json",
    );

    let port = reserve_port();
    let mut server = spawn_server(&home_dir, port, None);

    wait_for_server(&mut server, port);

    let health = get_json(port, "/health");
    assert_eq!(
        health.get("status").and_then(Value::as_str),
        Some("ok")
    );

    let list = get_json(
        port,
        "/api/v1/sessions?assistant=factory-droid&limit=1&offset=1&sortBy=lastActivityAt&descending=true",
    );
    assert_eq!(list.get("total").and_then(Value::as_u64), Some(2));
    assert_eq!(
        list.get("sessions")
            .and_then(Value::as_array)
            .and_then(|sessions| sessions.first())
            .and_then(|session| session.get("sessionId"))
            .and_then(Value::as_str),
        Some("droid-session-1")
    );

    let search = get_json(
        port,
        "/api/v1/sessions/search?query=branch&assistant=factory-droid&limit=1&offset=0&sortBy=title&descending=false",
    );
    assert_eq!(search.get("total").and_then(Value::as_u64), Some(2));
    assert_eq!(
        search
            .get("hits")
            .and_then(Value::as_array)
            .and_then(|hits| hits.first())
            .and_then(|hit| hit.get("sessionId"))
            .and_then(Value::as_str),
        Some("droid-session-1")
    );

    let detail = get_json(port, "/api/v1/sessions/claude-ses-1");
    assert_eq!(
        detail.get("assistant").and_then(Value::as_str),
        Some("claude-code")
    );

    let view = get_json(port, "/api/v1/sessions/claude-ses-1/view");
    assert!(
        view.get("content")
            .and_then(Value::as_str)
            .is_some_and(|content| content.contains("# 扫描 Claude transcripts"))
    );

    let expand = get_json(port, "/api/v1/sessions/claude-ses-1/expand");
    assert!(
        expand
            .get("relatedConfigs")
            .and_then(Value::as_array)
            .is_some_and(|configs| configs.iter().any(|config| {
                config.get("assistant").and_then(Value::as_str) == Some("claude-code")
            }))
    );

    let openapi = get_json(port, "/openapi.json");
    assert_eq!(openapi.get("openapi").and_then(Value::as_str), Some("3.1.0"));
    assert!(
        openapi
            .get("paths")
            .and_then(|paths| paths.get("/api/v1/sessions"))
            .and_then(|path| path.get("get"))
            .is_some()
    );
    assert!(
        openapi
            .get("paths")
            .and_then(|paths| paths.get("/api/v1/sessions/search"))
            .and_then(|path| path.get("get"))
            .is_some()
    );
    assert!(
        openapi
            .get("paths")
            .and_then(|paths| paths.get("/api/v1/sessions/{sessionId}"))
            .and_then(|path| path.get("get"))
            .is_some()
    );
    assert!(
        openapi
            .get("components")
            .and_then(|components| components.get("securitySchemes"))
            .and_then(|schemes| schemes.get("bearerAuth"))
            .is_some()
    );
}

#[test]
fn serve_command_requires_bearer_token_when_configured() {
    let sandbox = temp_root();
    let home_dir = sandbox.join("home");

    seed_session_fixture(
        &home_dir
            .join(".claude")
            .join("projects")
            .join("C--Projects-Claude-Demo")
            .join("claude-ses-1.jsonl"),
        "claude/projects/C--Projects-Claude-Demo/claude-ses-1.jsonl",
    );

    let port = reserve_port();
    let mut server = spawn_server(&home_dir, port, Some("osm-local-token"));

    wait_for_server(&mut server, port);

    let unauthorized = http_get(port, "/api/v1/sessions", None);
    assert_eq!(unauthorized.status, 401);
    assert!(
        serde_json::from_slice::<Value>(&unauthorized.body)
            .expect("unauthorized body is json")
            .get("error")
            .and_then(Value::as_str)
            .is_some_and(|message| message.contains("bearer token"))
    );

    let authorized = get_json_with_token(port, "/api/v1/sessions", "osm-local-token");
    assert!(
        authorized
            .get("sessions")
            .and_then(Value::as_array)
            .is_some_and(|sessions| sessions.iter().any(|session| {
                session.get("sessionId").and_then(Value::as_str) == Some("claude-ses-1")
            }))
    );

    let health = get_json(port, "/health");
    assert_eq!(health.get("status").and_then(Value::as_str), Some("ok"));
}

fn temp_root() -> PathBuf {
    let root = env::temp_dir().join(format!(
        "open-session-manager-http-api-{}",
        std::process::id()
    ));

    if root.exists() {
        fs::remove_dir_all(&root).expect("reset temp root");
    }

    fs::create_dir_all(&root).expect("create temp root");
    root
}

fn seed_session_fixture(target: &Path, fixture_relative: &str) {
    fs::create_dir_all(target.parent().expect("target parent")).expect("create target dir");
    fs::copy(fixtures_root().join(fixture_relative), target).expect("copy fixture");
}

fn reserve_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind temp listener")
        .local_addr()
        .expect("read temp listener addr")
        .port()
}

fn spawn_server(home_dir: &Path, port: u16, api_token: Option<&str>) -> ServerGuard {
    let mut command = Command::new(env!("CARGO_BIN_EXE_open-session-manager-core"));
    command
        .env("HOME", home_dir)
        .env("USERPROFILE", home_dir)
        .args(["serve", "--host", "127.0.0.1", "--port", &port.to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    if let Some(token) = api_token {
        command.env("OPEN_SESSION_MANAGER_API_TOKEN", token);
    }

    ServerGuard {
        child: command.spawn().expect("spawn serve command"),
    }
}

fn wait_for_server(server: &mut ServerGuard, port: u16) {
    let deadline = Instant::now() + Duration::from_secs(5);

    loop {
        if let Ok(Some(status)) = server.child.try_wait() {
            let mut stderr = String::new();
            if let Some(handle) = server.child.stderr.as_mut() {
                handle.read_to_string(&mut stderr).expect("read stderr");
            }

            panic!("serve command exited early with {status}: {stderr}");
        }

        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return;
        }

        if Instant::now() >= deadline {
            panic!("timed out waiting for serve command to bind port {port}");
        }

        thread::sleep(Duration::from_millis(50));
    }
}

fn get_json(port: u16, path: &str) -> Value {
    let response = http_get(port, path, None);
    assert_eq!(response.status, 200, "unexpected status for {path}");
    serde_json::from_slice(&response.body).expect("response body is json")
}

fn get_json_with_token(port: u16, path: &str, token: &str) -> Value {
    let response = http_get(port, path, Some(token));
    assert_eq!(response.status, 200, "unexpected status for {path}");
    serde_json::from_slice(&response.body).expect("response body is json")
}

fn http_get(port: u16, path: &str, token: Option<&str>) -> HttpResponse {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect server");
    let authorization = token
        .map(|token| format!("Authorization: Bearer {token}\r\n"))
        .unwrap_or_default();
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n{authorization}Connection: close\r\n\r\n"
    );
    stream
        .write_all(request.as_bytes())
        .expect("write request");

    let mut response = Vec::new();
    stream.read_to_end(&mut response).expect("read response");

    let separator = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .expect("response separator exists");
    let (headers, body) = response.split_at(separator + 4);
    let header_text = String::from_utf8(headers.to_vec()).expect("headers are utf8");
    let status = header_text
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .expect("status code exists");

    HttpResponse {
        status,
        body: body.to_vec(),
    }
}

struct HttpResponse {
    status: u16,
    body: Vec<u8>,
}

impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct ServerGuard {
    child: Child,
}
