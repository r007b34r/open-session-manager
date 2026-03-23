use std::{
    env, fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::{Duration, Instant},
};

use serde_json::Value;

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

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

#[test]
fn serve_command_exposes_session_control_routes() {
    let sandbox = temp_root();
    let home_dir = sandbox.join("home");
    let bin_dir = sandbox.join("bin");
    let log_path = sandbox.join("codex.log");

    seed_session_fixture(
        &home_dir
            .join(".codex")
            .join("sessions")
            .join("2026")
            .join("03")
            .join("15")
            .join("rollout-2026-03-15.jsonl"),
        "codex/2026/03/15/rollout-2026-03-15T12-00-00-codex-ses-1.jsonl",
    );

    fs::create_dir_all(&bin_dir).expect("create fake bin dir");
    write_fake_codex_executable(&bin_dir, &log_path);

    let port = reserve_port();
    let codex_command = fake_command_path(&bin_dir, "codex");
    let mut server = spawn_server_with_options(
        &home_dir,
        port,
        Some("osm-local-token"),
        Some(("OPEN_SESSION_MANAGER_CODEX_COMMAND", codex_command.as_path())),
    );

    wait_for_server(&mut server, port);

    let unauthorized = http_post_json(port, "/api/v1/sessions/codex-ses-1/resume", None, None);
    assert_eq!(unauthorized.status, 401);

    let resume = post_json_with_token(
        port,
        "/api/v1/sessions/codex-ses-1/resume",
        "osm-local-token",
        None,
    );
    assert_eq!(
        resume
            .get("sessionControl")
            .and_then(|control| control.get("attached"))
            .and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        resume
            .get("sessionControl")
            .and_then(|control| control.get("runtimeState"))
            .and_then(Value::as_str),
        Some("idle")
    );
    assert!(
        resume
            .get("sessionControl")
            .and_then(|control| control.get("lastResponse"))
            .and_then(Value::as_str)
            .is_some_and(|response| response.contains("READY"))
    );

    let pause = post_json_with_token(
        port,
        "/api/v1/sessions/codex-ses-1/pause",
        "osm-local-token",
        None,
    );
    assert_eq!(
        pause
            .get("sessionControl")
            .and_then(|control| control.get("paused"))
            .and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        pause
            .get("sessionControl")
            .and_then(|control| control.get("runtimeState"))
            .and_then(Value::as_str),
        Some("paused")
    );

    let resumed_again = post_json_with_token(
        port,
        "/api/v1/sessions/codex-ses-1/resume",
        "osm-local-token",
        None,
    );
    assert_eq!(
        resumed_again
            .get("sessionControl")
            .and_then(|control| control.get("paused"))
            .and_then(Value::as_bool),
        Some(false)
    );

    let continued = post_json_with_token(
        port,
        "/api/v1/sessions/codex-ses-1/continue",
        "osm-local-token",
        Some(r#"{"prompt":"Continue with validation"}"#),
    );
    assert_eq!(
        continued
            .get("sessionControl")
            .and_then(|control| control.get("lastPrompt"))
            .and_then(Value::as_str),
        Some("Continue with validation")
    );
    assert!(
        continued
            .get("sessionControl")
            .and_then(|control| control.get("lastResponse"))
            .and_then(Value::as_str)
            .is_some_and(|response| response.contains("READY"))
    );

    let detached = post_json_with_token(
        port,
        "/api/v1/sessions/codex-ses-1/detach",
        "osm-local-token",
        None,
    );
    assert_eq!(
        detached
            .get("sessionControl")
            .and_then(|control| control.get("attached"))
            .and_then(Value::as_bool),
        Some(false)
    );
    assert_eq!(
        detached
            .get("sessionControl")
            .and_then(|control| control.get("runtimeState"))
            .and_then(Value::as_str),
        Some("detached")
    );

    let attached = post_json_with_token(
        port,
        "/api/v1/sessions/codex-ses-1/attach",
        "osm-local-token",
        None,
    );
    assert_eq!(
        attached
            .get("sessionControl")
            .and_then(|control| control.get("attached"))
            .and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        attached
            .get("sessionControl")
            .and_then(|control| control.get("runtimeState"))
            .and_then(Value::as_str),
        Some("idle")
    );

    let openapi = get_json(port, "/openapi.json");
    assert!(
        openapi
            .get("paths")
            .and_then(|paths| paths.get("/api/v1/sessions/{sessionId}/resume"))
            .and_then(|path| path.get("post"))
            .is_some()
    );
    assert!(
        openapi
            .get("paths")
            .and_then(|paths| paths.get("/api/v1/sessions/{sessionId}/pause"))
            .and_then(|path| path.get("post"))
            .is_some()
    );
    assert!(
        openapi
            .get("paths")
            .and_then(|paths| paths.get("/api/v1/sessions/{sessionId}/attach"))
            .and_then(|path| path.get("post"))
            .is_some()
    );
    assert!(
        openapi
            .get("paths")
            .and_then(|paths| paths.get("/api/v1/sessions/{sessionId}/detach"))
            .and_then(|path| path.get("post"))
            .is_some()
    );
    assert!(
        openapi
            .get("paths")
            .and_then(|paths| paths.get("/api/v1/sessions/{sessionId}/continue"))
            .and_then(|path| path.get("post"))
            .and_then(|operation| operation.get("requestBody"))
            .is_some()
    );
    assert!(
        openapi
            .get("components")
            .and_then(|components| components.get("schemas"))
            .and_then(|schemas| schemas.get("ContinueSessionRequest"))
            .is_some()
    );
}

#[test]
fn serve_command_controls_copilot_session_via_http() {
    let sandbox = temp_root();
    let home_dir = sandbox.join("home");
    let bin_dir = sandbox.join("bin");
    let log_path = sandbox.join("copilot.log");

    seed_session_fixture(
        &home_dir
            .join(".copilot")
            .join("session-state")
            .join("copilot-ses-1.jsonl"),
        "copilot/session-state/copilot-ses-1.jsonl",
    );

    fs::create_dir_all(&bin_dir).expect("create fake bin dir");
    write_fake_copilot_executable(&bin_dir, &log_path);

    let port = reserve_port();
    let copilot_command = fake_command_path(&bin_dir, "copilot");
    let mut server = spawn_server_with_options(
        &home_dir,
        port,
        Some("osm-local-token"),
        Some((
            "OPEN_SESSION_MANAGER_COPILOT_COMMAND",
            copilot_command.as_path(),
        )),
    );

    wait_for_server(&mut server, port);

    let resume = post_json_with_token(
        port,
        "/api/v1/sessions/copilot-ses-1/resume",
        "osm-local-token",
        None,
    );
    assert_eq!(
        resume.get("assistant").and_then(Value::as_str),
        Some("github-copilot-cli")
    );
    assert_eq!(
        resume
            .get("sessionControl")
            .and_then(|control| control.get("controller"))
            .and_then(Value::as_str),
        Some("github-copilot-cli")
    );
    assert!(
        resume
            .get("sessionControl")
            .and_then(|control| control.get("lastResponse"))
            .and_then(Value::as_str)
            .is_some_and(|response| response.contains("READY"))
    );

    let continued = post_json_with_token(
        port,
        "/api/v1/sessions/copilot-ses-1/continue",
        "osm-local-token",
        Some(r#"{"prompt":"Continue with Copilot verification"}"#),
    );
    assert_eq!(
        continued
            .get("sessionControl")
            .and_then(|control| control.get("lastPrompt"))
            .and_then(Value::as_str),
        Some("Continue with Copilot verification")
    );
    assert!(
        fs::read_to_string(&log_path)
            .expect("read copilot log")
            .contains("--resume=copilot-ses-1")
    );
}

fn temp_root() -> PathBuf {
    let suffix = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    let root = env::temp_dir().join(format!(
        "open-session-manager-http-api-{}-{suffix}",
        std::process::id(),
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
    spawn_server_with_options(home_dir, port, api_token, None)
}

fn spawn_server_with_options(
    home_dir: &Path,
    port: u16,
    api_token: Option<&str>,
    extra_env: Option<(&str, &Path)>,
) -> ServerGuard {
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
    if let Some((key, value)) = extra_env {
        command.env(key, value);
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

fn post_json_with_token(port: u16, path: &str, token: &str, body: Option<&str>) -> Value {
    let response = http_post_json(port, path, Some(token), body);
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

fn http_post_json(port: u16, path: &str, token: Option<&str>, body: Option<&str>) -> HttpResponse {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect server");
    let authorization = token
        .map(|token| format!("Authorization: Bearer {token}\r\n"))
        .unwrap_or_default();
    let payload = body.unwrap_or_default();
    let request = format!(
        "POST {path} HTTP/1.1\r\n\
Host: 127.0.0.1:{port}\r\n\
{authorization}Content-Type: application/json\r\n\
Content-Length: {content_length}\r\n\
Connection: close\r\n\r\n\
{payload}",
        path = path,
        port = port,
        authorization = authorization,
        content_length = payload.len()
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

fn write_fake_codex_executable(bin_dir: &Path, log_path: &Path) {
    if cfg!(windows) {
        let script_path = bin_dir.join("codex.cmd");
        fs::write(
            &script_path,
            format!(
                concat!(
                    "@echo off\r\n",
                    "setlocal EnableDelayedExpansion\r\n",
                    "echo %*>>\"{}\"\r\n",
                    "set \"out=\"\r\n",
                    ":next\r\n",
                    "if \"%~1\"==\"\" goto done\r\n",
                    "if \"%~1\"==\"-o\" (\r\n",
                    "  set \"out=%~2\"\r\n",
                    "  shift\r\n",
                    ")\r\n",
                    "shift\r\n",
                    "goto next\r\n",
                    ":done\r\n",
                    "if not \"!out!\"==\"\" (\r\n",
                    "  >\"!out!\" echo READY from fake codex\r\n",
                    ")\r\n",
                    "echo ok\r\n"
                ),
                log_path.display()
            ),
        )
        .expect("write fake codex");
        return;
    }

    let script_path = bin_dir.join("codex");
    fs::write(
        &script_path,
        format!(
            concat!(
                "#!/bin/sh\n",
                "printf '%s\\n' \"$*\" >> '{}'\n",
                "out=''\n",
                "while [ \"$#\" -gt 0 ]; do\n",
                "  if [ \"$1\" = '-o' ]; then\n",
                "    out=\"$2\"\n",
                "    shift 2\n",
                "    continue\n",
                "  fi\n",
                "  shift\n",
                "done\n",
                "if [ -n \"$out\" ]; then\n",
                "  printf 'READY from fake codex\\n' > \"$out\"\n",
                "fi\n",
                "printf 'ok\\n'\n"
            ),
            log_path.display()
        ),
    )
    .expect("write fake codex");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))
            .expect("chmod fake codex");
    }
}

fn write_fake_copilot_executable(bin_dir: &Path, log_path: &Path) {
    if cfg!(windows) {
        let script_path = bin_dir.join("copilot.cmd");
        fs::write(
            &script_path,
            format!(
                concat!(
                    "@echo off\r\n",
                    "echo %*>>\"{}\"\r\n",
                    "echo READY from fake copilot\r\n"
                ),
                log_path.display()
            ),
        )
        .expect("write fake copilot");
        return;
    }

    let script_path = bin_dir.join("copilot");
    fs::write(
        &script_path,
        format!(
            concat!(
                "#!/bin/sh\n",
                "printf '%s\\n' \"$*\" >> '{}'\n",
                "printf 'READY from fake copilot\\n'\n"
            ),
            log_path.display()
        ),
    )
    .expect("write fake copilot");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))
            .expect("chmod fake copilot");
    }
}

fn fake_command_path(bin_dir: &Path, name: &str) -> PathBuf {
    if cfg!(windows) {
        bin_dir.join(format!("{name}.cmd"))
    } else {
        bin_dir.join(name)
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
