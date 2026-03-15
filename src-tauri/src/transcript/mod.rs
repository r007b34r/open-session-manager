use std::{fs, path::Path};

use serde::Serialize;
use serde_json::Value;

use crate::{
    adapters::{
        copilot_cli::copilot_tool_requests,
        factory_droid::{DroidDialect, detect_droid_dialect, normalize_droid_kind},
        gemini_cli::{gemini_messages, gemini_role, gemini_text, gemini_tool_calls},
        traits::collect_files,
    },
    domain::session::SessionRecord,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptDigest {
    pub highlights: Vec<TranscriptHighlight>,
    pub todos: Vec<TranscriptTodo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptHighlight {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptTodo {
    pub content: String,
    pub completed: bool,
}

pub fn build_transcript_digest(session: &SessionRecord) -> TranscriptDigest {
    match session.assistant.as_str() {
        "codex" => build_codex_transcript_digest(Path::new(&session.source_path)),
        "claude-code" => build_claude_transcript_digest(Path::new(&session.source_path)),
        "opencode" => build_opencode_transcript_digest(Path::new(&session.source_path)),
        "gemini-cli" => build_gemini_transcript_digest(Path::new(&session.source_path)),
        "github-copilot-cli" => build_copilot_transcript_digest(Path::new(&session.source_path)),
        "factory-droid" => build_factory_droid_transcript_digest(Path::new(&session.source_path)),
        _ => TranscriptDigest::default(),
    }
}

fn build_codex_transcript_digest(source: &Path) -> TranscriptDigest {
    let Ok(lines) = read_jsonl(source) else {
        return TranscriptDigest::default();
    };

    let mut digest = TranscriptDigest::default();
    for line in lines {
        if line.get("type").and_then(Value::as_str) != Some("response_item") {
            continue;
        }

        let payload = line.get("payload").unwrap_or(&Value::Null);
        if payload.get("type").and_then(Value::as_str) != Some("message") {
            continue;
        }

        let Some(content) = extract_text_array(payload.get("content")) else {
            continue;
        };

        match payload.get("role").and_then(Value::as_str) {
            Some("user") => digest.highlights.push(TranscriptHighlight {
                role: "User".to_string(),
                content,
            }),
            Some("assistant") => digest.highlights.push(TranscriptHighlight {
                role: "Assistant".to_string(),
                content,
            }),
            _ => {}
        }
    }

    digest.highlights.truncate(6);
    digest
}

fn build_claude_transcript_digest(source: &Path) -> TranscriptDigest {
    let Ok(lines) = read_jsonl(source) else {
        return TranscriptDigest::default();
    };

    let mut digest = TranscriptDigest::default();

    for line in lines {
        match line.get("type").and_then(Value::as_str) {
            Some("user") => {
                if let Some(content) = line
                    .get("message")
                    .and_then(|message| message.get("content"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|content| !content.is_empty())
                {
                    digest.highlights.push(TranscriptHighlight {
                        role: "User".to_string(),
                        content: content.to_string(),
                    });
                }

                if let Some(todos) = line.get("todos").and_then(Value::as_array) {
                    digest.todos = todos.iter().filter_map(parse_claude_todo).collect();
                }
            }
            Some("assistant") => {
                if let Some(parts) = line
                    .get("message")
                    .and_then(|message| message.get("content"))
                    .and_then(Value::as_array)
                {
                    if let Some(todos) = extract_claude_todowrite(parts) {
                        digest.todos = todos;
                    }

                    let content = parts
                        .iter()
                        .filter_map(|part| part.get("text").and_then(Value::as_str))
                        .collect::<Vec<_>>()
                        .join(" ")
                        .trim()
                        .to_string();

                    if !content.is_empty() {
                    digest.highlights.push(TranscriptHighlight {
                        role: "Assistant".to_string(),
                        content,
                    });
                    }
                }
            }
            _ => {}
        }
    }

    digest.highlights.truncate(6);
    digest
}

fn build_opencode_transcript_digest(source: &Path) -> TranscriptDigest {
    let Some(session_info) = fs::read(source)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
    else {
        return TranscriptDigest::default();
    };
    let Some(session_id) = session_info.get("id").and_then(Value::as_str) else {
        return TranscriptDigest::default();
    };
    let Some(storage_root) = source
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
    else {
        return TranscriptDigest::default();
    };
    let message_dir = storage_root.join("session").join("message").join(session_id);
    let part_dir = storage_root.join("session").join("part").join(session_id);
    let Ok(mut message_files) = collect_files(&message_dir, &|path| {
        path.extension().and_then(|value| value.to_str()) == Some("json")
    }) else {
        return TranscriptDigest::default();
    };
    message_files.sort();

    let mut messages = Vec::new();
    for message_file in message_files {
        let Some(message) = fs::read(&message_file)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        else {
            continue;
        };
        messages.push((opencode_created_at(&message), message));
    }
    messages.sort_by_key(|(created_at, _)| *created_at);

    let mut digest = TranscriptDigest::default();
    for (_, message) in messages {
        let Some(message_id) = message.get("id").and_then(Value::as_str) else {
            continue;
        };
        let content = collect_opencode_texts(&part_dir.join(message_id));
        if content.is_empty() {
            continue;
        }

        let role = match message.get("role").and_then(Value::as_str) {
            Some("user") => "User",
            Some("assistant") => "Assistant",
            Some("tool") => "Tool",
            _ => continue,
        };
        digest.highlights.push(TranscriptHighlight {
            role: role.to_string(),
            content,
        });
    }

    digest.highlights.truncate(6);
    digest
}

fn build_gemini_transcript_digest(source: &Path) -> TranscriptDigest {
    let Some(parsed) = fs::read(source)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
    else {
        return TranscriptDigest::default();
    };

    let mut digest = TranscriptDigest::default();
    for message in gemini_messages(&parsed) {
        if let Some(content) = gemini_text(message) {
            let role = match gemini_role(message) {
                Some("user") => Some("User"),
                Some("assistant") => Some("Assistant"),
                _ => None,
            };

            if let Some(role) = role {
                digest.highlights.push(TranscriptHighlight {
                    role: role.to_string(),
                    content,
                });
            }
        }

        for tool_call in gemini_tool_calls(message) {
            if let Some(output) = tool_call
                .get("resultDisplay")
                .or_else(|| tool_call.get("output"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|output| !output.is_empty())
            {
                digest.highlights.push(TranscriptHighlight {
                    role: "Tool".to_string(),
                    content: output.to_string(),
                });
            }
        }
    }

    digest.highlights.truncate(6);
    digest
}

fn build_copilot_transcript_digest(source: &Path) -> TranscriptDigest {
    let Ok(lines) = read_jsonl(source) else {
        return TranscriptDigest::default();
    };

    let mut digest = TranscriptDigest::default();
    for line in lines {
        let data = line.get("data").unwrap_or(&Value::Null);

        match line.get("type").and_then(Value::as_str) {
            Some("user.message") => {
                if let Some(content) = data
                    .get("content")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|content| !content.is_empty())
                {
                    digest.highlights.push(TranscriptHighlight {
                        role: "User".to_string(),
                        content: content.to_string(),
                    });
                }
            }
            Some("assistant.message") => {
                if let Some(content) = data
                    .get("content")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|content| !content.is_empty())
                {
                    digest.highlights.push(TranscriptHighlight {
                        role: "Assistant".to_string(),
                        content: content.to_string(),
                    });
                }

                for request in copilot_tool_requests(data) {
                    if let Some(name) = request.get("name").and_then(Value::as_str) {
                        digest.highlights.push(TranscriptHighlight {
                            role: "Tool".to_string(),
                            content: format!("Tool call: {name}"),
                        });
                    }
                }
            }
            Some("tool.execution_complete") => {
                if let Some(content) = data
                    .get("result")
                    .and_then(|result| result.get("content"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|content| !content.is_empty())
                {
                    digest.highlights.push(TranscriptHighlight {
                        role: "Tool".to_string(),
                        content: content.to_string(),
                    });
                }
            }
            _ => {}
        }
    }

    digest.highlights.truncate(6);
    digest
}

fn build_factory_droid_transcript_digest(source: &Path) -> TranscriptDigest {
    match detect_droid_dialect(source) {
        Ok(DroidDialect::SessionStore) => build_factory_droid_session_store_digest(source),
        Ok(DroidDialect::StreamJson) => build_factory_droid_stream_digest(source),
        Err(_) => TranscriptDigest::default(),
    }
}

fn build_factory_droid_session_store_digest(source: &Path) -> TranscriptDigest {
    let Ok(lines) = read_jsonl(source) else {
        return TranscriptDigest::default();
    };

    let mut digest = TranscriptDigest::default();
    for line in lines {
        if line.get("type").and_then(Value::as_str).map(normalize_droid_kind).as_deref()
            != Some("message")
        {
            continue;
        }

        let Some(message) = line.get("message") else {
            continue;
        };
        let role = message.get("role").and_then(Value::as_str).unwrap_or_default();
        let parts = message
            .get("content")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let text = parts
            .iter()
            .filter(|part| {
                part.get("type")
                    .and_then(Value::as_str)
                    .is_some_and(|kind| normalize_droid_kind(kind) == "text")
            })
            .filter_map(|part| part.get("text").and_then(Value::as_str))
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        if !text.is_empty() {
            let role = match role {
                "user" => Some("User"),
                "assistant" => Some("Assistant"),
                _ => None,
            };

            if let Some(role) = role {
                digest.highlights.push(TranscriptHighlight {
                    role: role.to_string(),
                    content: text,
                });
            }
        }

        for part in parts {
            let Some(kind) = part.get("type").and_then(Value::as_str) else {
                continue;
            };
            match normalize_droid_kind(kind).as_str() {
                "tooluse" => {
                    if let Some(name) = part.get("name").and_then(Value::as_str) {
                        digest.highlights.push(TranscriptHighlight {
                            role: "Tool".to_string(),
                            content: format!("Tool call: {name}"),
                        });
                    }
                }
                "toolresult" => {
                    if let Some(content) = part
                        .get("content")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|content| !content.is_empty())
                    {
                        digest.highlights.push(TranscriptHighlight {
                            role: "Tool".to_string(),
                            content: content.to_string(),
                        });
                    }
                }
                _ => {}
            }
        }
    }

    digest.highlights.truncate(6);
    digest
}

fn build_factory_droid_stream_digest(source: &Path) -> TranscriptDigest {
    let Ok(lines) = read_jsonl(source) else {
        return TranscriptDigest::default();
    };

    let mut digest = TranscriptDigest::default();
    for line in lines {
        match line
            .get("type")
            .and_then(Value::as_str)
            .map(normalize_droid_kind)
            .as_deref()
        {
            Some("message") => {
                let role = line.get("role").and_then(Value::as_str).unwrap_or_default();
                let content = line
                    .get("content")
                    .or_else(|| line.get("text"))
                    .or_else(|| line.get("message"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|content| !content.is_empty());

                let role = match role {
                    "user" => Some("User"),
                    "assistant" => Some("Assistant"),
                    _ => None,
                };

                if let (Some(role), Some(content)) = (role, content) {
                    digest.highlights.push(TranscriptHighlight {
                        role: role.to_string(),
                        content: content.to_string(),
                    });
                }
            }
            Some("toolcall") => {
                if let Some(name) = line
                    .get("toolName")
                    .or_else(|| line.get("tool_name"))
                    .and_then(Value::as_str)
                {
                    digest.highlights.push(TranscriptHighlight {
                        role: "Tool".to_string(),
                        content: format!("Tool call: {name}"),
                    });
                }
            }
            Some("toolresult") => {
                let value = line.get("value").unwrap_or(&Value::Null);
                let output = value
                    .get("stdout")
                    .or_else(|| value.get("output"))
                    .or_else(|| value.get("text"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|content| !content.is_empty());

                if let Some(output) = output {
                    digest.highlights.push(TranscriptHighlight {
                        role: "Tool".to_string(),
                        content: output.to_string(),
                    });
                }
            }
            Some("completion") => {
                if let Some(content) = line
                    .get("finalText")
                    .or_else(|| line.get("final"))
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|content| !content.is_empty())
                {
                    digest.highlights.push(TranscriptHighlight {
                        role: "Assistant".to_string(),
                        content: content.to_string(),
                    });
                }
            }
            _ => {}
        }
    }

    digest.highlights.truncate(6);
    digest
}

fn opencode_created_at(message: &Value) -> i64 {
    message
        .get("time")
        .and_then(|time| time.get("created"))
        .and_then(Value::as_i64)
        .unwrap_or(0)
}

fn parse_claude_todo(value: &Value) -> Option<TranscriptTodo> {
    if let Some(content) = value.as_str().map(str::trim).filter(|content| !content.is_empty()) {
        return Some(TranscriptTodo {
            content: content.to_string(),
            completed: false,
        });
    }

    let content = value
        .get("content")
        .or_else(|| value.get("text"))
        .or_else(|| value.get("title"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|content| !content.is_empty())?;
    let status = value
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_ascii_lowercase();

    Some(TranscriptTodo {
        content: content.to_string(),
        completed: matches!(status.as_str(), "completed" | "done" | "closed"),
    })
}

fn extract_claude_todowrite(parts: &[Value]) -> Option<Vec<TranscriptTodo>> {
    let mut latest_todos = None;

    for part in parts {
        if part.get("type").and_then(Value::as_str) != Some("tool_use") {
            continue;
        }

        if part.get("name").and_then(Value::as_str) != Some("TodoWrite") {
            continue;
        }

        let todos = part
            .get("input")
            .and_then(|input| input.get("todos"))
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(parse_claude_todo).collect::<Vec<_>>());

        if let Some(todos) = todos {
            latest_todos = Some(todos);
        }
    }

    latest_todos
}

fn collect_opencode_texts(part_dir: &Path) -> String {
    let Ok(mut files) = collect_files(part_dir, &|path| {
        path.extension().and_then(|value| value.to_str()) == Some("json")
    }) else {
        return String::new();
    };
    files.sort();

    files.into_iter()
        .filter_map(|file| {
            fs::read(file)
                .ok()
                .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        })
        .filter_map(|part| match part.get("type").and_then(Value::as_str) {
            Some("text") => part
                .get("text")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned),
            Some("tool") => part
                .get("tool")
                .and_then(Value::as_str)
                .map(|tool| format!("Tool call: {tool}")),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

fn read_jsonl(path: &Path) -> Result<Vec<Value>, std::io::Error> {
    let content = fs::read_to_string(path)?;
    let parsed = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .collect::<Vec<_>>();
    Ok(parsed)
}

fn extract_text_array(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_array)
        .map(|parts| {
            parts
                .iter()
                .filter_map(|part| part.get("text").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
}
