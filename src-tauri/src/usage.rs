use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    adapters::{
        gemini_cli::gemini_messages,
        openclaw::{openclaw_kind, openclaw_role},
        traits::collect_files,
    },
    domain::session::SessionRecord,
};

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CostSource {
    Reported,
    Estimated,
    Mixed,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionUsageRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub reasoning_tokens: u64,
    pub total_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_usd: Option<f64>,
    pub cost_source: CostSource,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageTotalsRecord {
    pub sessions_with_usage: u32,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub reasoning_tokens: u64,
    pub total_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_usd: Option<f64>,
    pub cost_source: CostSource,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantUsageRecord {
    pub assistant: String,
    pub session_count: u32,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub reasoning_tokens: u64,
    pub total_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_usd: Option<f64>,
    pub cost_source: CostSource,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageTimelineRecord {
    pub date: String,
    pub sessions_with_usage: u32,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub reasoning_tokens: u64,
    pub total_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_usd: Option<f64>,
    pub cost_source: CostSource,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageOverviewRecord {
    pub totals: UsageTotalsRecord,
    pub assistants: Vec<AssistantUsageRecord>,
}

#[derive(Debug, Clone, Copy)]
struct ModelPricing {
    input_per_million_usd: f64,
    output_per_million_usd: f64,
    cache_read_per_million_usd: f64,
    cache_write_per_million_usd: f64,
    reasoning_per_million_usd: f64,
}

#[derive(Debug, Clone, Default)]
struct UsageAccumulator {
    model: Option<String>,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_write_tokens: u64,
    reasoning_tokens: u64,
    reported_cost_usd: f64,
    has_reported_cost_data: bool,
    missing_reported_cost_for_usage: bool,
}

#[derive(Debug, Clone, Default)]
struct AggregateAccumulator {
    session_count: u32,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_write_tokens: u64,
    reasoning_tokens: u64,
    total_tokens: u64,
    cost_usd: f64,
    has_known_cost: bool,
    has_unknown_cost: bool,
    has_reported_cost: bool,
    has_estimated_cost: bool,
}

impl UsageAccumulator {
    fn add(
        &mut self,
        input_tokens: u64,
        output_tokens: u64,
        cache_read_tokens: u64,
        cache_write_tokens: u64,
        reasoning_tokens: u64,
        cost_usd: Option<f64>,
    ) {
        let has_usage_signal = input_tokens > 0
            || output_tokens > 0
            || cache_read_tokens > 0
            || cache_write_tokens > 0
            || reasoning_tokens > 0
            || cost_usd.is_some();
        if !has_usage_signal {
            return;
        }

        self.input_tokens += input_tokens;
        self.output_tokens += output_tokens;
        self.cache_read_tokens += cache_read_tokens;
        self.cache_write_tokens += cache_write_tokens;
        self.reasoning_tokens += reasoning_tokens;
        match cost_usd {
            Some(value) => {
                self.has_reported_cost_data = true;
                self.reported_cost_usd = round_cost(self.reported_cost_usd + value);
            }
            None => {
                self.missing_reported_cost_for_usage = true;
            }
        }
    }

    fn total_tokens(&self) -> u64 {
        self.input_tokens
            + self.output_tokens
            + self.cache_read_tokens
            + self.cache_write_tokens
            + self.reasoning_tokens
    }

    fn has_usage(&self) -> bool {
        self.total_tokens() > 0
            || self.has_reported_cost_data
            || self.missing_reported_cost_for_usage
    }

    fn resolved_cost(&self) -> (Option<f64>, CostSource) {
        if !self.has_usage() {
            return (None, CostSource::Unknown);
        }

        if self.has_reported_cost_data && !self.missing_reported_cost_for_usage {
            return (
                Some(round_cost(self.reported_cost_usd)),
                CostSource::Reported,
            );
        }

        if !self.has_reported_cost_data {
            if let Some(model) = self.model.as_deref() {
                if let Some(estimated) = estimate_usage_cost(
                    model,
                    self.input_tokens,
                    self.output_tokens,
                    self.cache_read_tokens,
                    self.cache_write_tokens,
                    self.reasoning_tokens,
                ) {
                    return (Some(estimated), CostSource::Estimated);
                }
            }
        }

        (None, CostSource::Unknown)
    }

    fn into_session_usage(self) -> Option<SessionUsageRecord> {
        let total_tokens = self.total_tokens();
        let (cost_usd, cost_source) = self.resolved_cost();
        self.has_usage().then(|| SessionUsageRecord {
            model: self.model,
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens,
            cache_read_tokens: self.cache_read_tokens,
            cache_write_tokens: self.cache_write_tokens,
            reasoning_tokens: self.reasoning_tokens,
            total_tokens,
            cost_usd,
            cost_source,
        })
    }
}

impl AggregateAccumulator {
    fn add_usage(&mut self, usage: &SessionUsageRecord) {
        self.session_count += 1;
        self.input_tokens += usage.input_tokens;
        self.output_tokens += usage.output_tokens;
        self.cache_read_tokens += usage.cache_read_tokens;
        self.cache_write_tokens += usage.cache_write_tokens;
        self.reasoning_tokens += usage.reasoning_tokens;
        self.total_tokens += usage.total_tokens;

        match usage.cost_usd {
            Some(value) => {
                self.has_known_cost = true;
                self.cost_usd = round_cost(self.cost_usd + value);
            }
            None => {
                self.has_unknown_cost = true;
            }
        }

        match usage.cost_source {
            CostSource::Reported => self.has_reported_cost = true,
            CostSource::Estimated => self.has_estimated_cost = true,
            CostSource::Mixed => {
                self.has_reported_cost = true;
                self.has_estimated_cost = true;
            }
            CostSource::Unknown => self.has_unknown_cost = true,
        }
    }

    fn resolved_cost(&self) -> Option<f64> {
        if self.has_unknown_cost || !self.has_known_cost {
            return None;
        }

        Some(round_cost(self.cost_usd))
    }

    fn resolved_cost_source(&self) -> CostSource {
        if self.has_unknown_cost || !self.has_known_cost {
            return CostSource::Unknown;
        }

        match (self.has_reported_cost, self.has_estimated_cost) {
            (true, true) => CostSource::Mixed,
            (false, true) => CostSource::Estimated,
            _ => CostSource::Reported,
        }
    }
}

pub fn extract_session_usage(session: &SessionRecord) -> Option<SessionUsageRecord> {
    match session.assistant.as_str() {
        "codex" => extract_codex_usage(Path::new(&session.source_path)),
        "claude-code" => extract_claude_usage(Path::new(&session.source_path)),
        "opencode" => extract_opencode_usage(Path::new(&session.source_path)),
        "gemini-cli" => extract_gemini_usage(Path::new(&session.source_path)),
        "openclaw" => extract_openclaw_usage(Path::new(&session.source_path)),
        _ => None,
    }
}

pub fn build_usage_overview(
    usage_by_session: impl IntoIterator<Item = (String, Option<SessionUsageRecord>)>,
) -> UsageOverviewRecord {
    let mut totals = AggregateAccumulator::default();
    let mut assistants = BTreeMap::<String, AggregateAccumulator>::new();

    for (assistant, usage) in usage_by_session {
        let Some(usage) = usage else {
            continue;
        };

        totals.add_usage(&usage);
        assistants.entry(assistant).or_default().add_usage(&usage);
    }

    let mut assistants = assistants
        .into_iter()
        .map(|(assistant, entry)| AssistantUsageRecord {
            assistant,
            session_count: entry.session_count,
            input_tokens: entry.input_tokens,
            output_tokens: entry.output_tokens,
            cache_read_tokens: entry.cache_read_tokens,
            cache_write_tokens: entry.cache_write_tokens,
            reasoning_tokens: entry.reasoning_tokens,
            total_tokens: entry.total_tokens,
            cost_usd: entry.resolved_cost(),
            cost_source: entry.resolved_cost_source(),
        })
        .collect::<Vec<_>>();
    assistants.sort_by(|left, right| {
        right
            .total_tokens
            .cmp(&left.total_tokens)
            .then_with(|| left.assistant.cmp(&right.assistant))
    });

    UsageOverviewRecord {
        totals: UsageTotalsRecord {
            sessions_with_usage: totals.session_count,
            input_tokens: totals.input_tokens,
            output_tokens: totals.output_tokens,
            cache_read_tokens: totals.cache_read_tokens,
            cache_write_tokens: totals.cache_write_tokens,
            reasoning_tokens: totals.reasoning_tokens,
            total_tokens: totals.total_tokens,
            cost_usd: totals.resolved_cost(),
            cost_source: totals.resolved_cost_source(),
        },
        assistants,
    }
}

pub fn build_usage_timeline(
    usage_by_session: impl IntoIterator<Item = (Option<String>, Option<SessionUsageRecord>)>,
) -> Vec<UsageTimelineRecord> {
    let mut buckets = BTreeMap::<String, AggregateAccumulator>::new();

    for (timestamp, usage) in usage_by_session {
        let Some(usage) = usage else {
            continue;
        };
        let Some(date) = timestamp.as_deref().and_then(bucket_usage_date) else {
            continue;
        };

        buckets.entry(date).or_default().add_usage(&usage);
    }

    buckets
        .into_iter()
        .map(|(date, entry)| UsageTimelineRecord {
            date,
            sessions_with_usage: entry.session_count,
            input_tokens: entry.input_tokens,
            output_tokens: entry.output_tokens,
            cache_read_tokens: entry.cache_read_tokens,
            cache_write_tokens: entry.cache_write_tokens,
            reasoning_tokens: entry.reasoning_tokens,
            total_tokens: entry.total_tokens,
            cost_usd: entry.resolved_cost(),
            cost_source: entry.resolved_cost_source(),
        })
        .collect()
}

fn extract_codex_usage(source: &Path) -> Option<SessionUsageRecord> {
    let reader = BufReader::new(File::open(source).ok()?);
    let mut usage = UsageAccumulator::default();

    for line in reader.lines() {
        let line = line.ok()?;
        let parsed: Value = serde_json::from_str(&line).ok()?;

        let info = parsed
            .get("payload")
            .filter(|payload| payload.get("type").and_then(Value::as_str) == Some("token_count"))
            .and_then(|payload| payload.get("info"))
            .and_then(|info| {
                info.get("last_token_usage")
                    .or_else(|| info.get("token_usage"))
            });
        let Some(info) = info else {
            continue;
        };

        usage.model = info
            .get("model")
            .or_else(|| info.get("model_slug"))
            .or_else(|| {
                parsed
                    .get("payload")
                    .and_then(|payload| payload.get("model"))
            })
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or(usage.model);

        usage.add(
            read_u64(info, &["input_tokens", "input"]),
            read_u64(info, &["output_tokens", "output"]),
            read_u64(
                info,
                &[
                    "cached_input_tokens",
                    "cache_read_input_tokens",
                    "input_cache_read",
                    "cache_read",
                ],
            ),
            read_u64(
                info,
                &[
                    "cache_creation_input_tokens",
                    "input_cache_creation",
                    "cache_write",
                ],
            ),
            read_u64(info, &["reasoning_tokens", "reasoning"]),
            read_cost(info),
        );
    }

    usage.into_session_usage()
}

fn extract_claude_usage(source: &Path) -> Option<SessionUsageRecord> {
    let reader = BufReader::new(File::open(source).ok()?);
    let mut usage = UsageAccumulator::default();

    for line in reader.lines() {
        let line = line.ok()?;
        let parsed: Value = serde_json::from_str(&line).ok()?;
        if parsed.get("type").and_then(Value::as_str) != Some("assistant") {
            continue;
        }

        let Some(message) = parsed.get("message") else {
            continue;
        };
        let Some(message_usage) = message.get("usage") else {
            continue;
        };

        usage.model = message
            .get("model")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or(usage.model);
        usage.add(
            read_u64(message_usage, &["input_tokens", "input"]),
            read_u64(message_usage, &["output_tokens", "output"]),
            read_u64(
                message_usage,
                &[
                    "cache_read_input_tokens",
                    "cached_input_tokens",
                    "cache_read",
                ],
            ),
            read_u64(
                message_usage,
                &["cache_creation_input_tokens", "cache_write"],
            ),
            read_u64(message_usage, &["reasoning_tokens", "reasoning"]),
            read_cost(message_usage),
        );
    }

    usage.into_session_usage()
}

fn extract_opencode_usage(source: &Path) -> Option<SessionUsageRecord> {
    let session_info: Value = serde_json::from_slice(&fs::read(source).ok()?).ok()?;
    let session_id = session_info.get("id").and_then(Value::as_str)?;
    let storage_root = source.parent()?.parent()?.parent()?;
    let message_dir = storage_root
        .join("session")
        .join("message")
        .join(session_id);
    let message_files = collect_files(&message_dir, &|path| {
        path.extension().and_then(|value| value.to_str()) == Some("json")
    })
    .ok()?;
    let mut usage = UsageAccumulator::default();

    for message_file in message_files {
        let message: Value = serde_json::from_slice(&fs::read(message_file).ok()?).ok()?;
        if message.get("role").and_then(Value::as_str) != Some("assistant") {
            continue;
        }

        usage.model = message
            .get("modelID")
            .or_else(|| message.get("modelId"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or(usage.model);
        let token_usage = message.get("tokens").unwrap_or(&Value::Null);
        let cache_usage = token_usage.get("cache").unwrap_or(&Value::Null);
        usage.add(
            read_u64(token_usage, &["input"]),
            read_u64(token_usage, &["output"]),
            read_u64(cache_usage, &["read"]),
            read_u64(cache_usage, &["write"]),
            read_u64(token_usage, &["reasoning"]),
            message.get("cost").and_then(Value::as_f64).map(round_cost),
        );
    }

    usage.into_session_usage()
}

fn extract_gemini_usage(source: &Path) -> Option<SessionUsageRecord> {
    let parsed: Value = serde_json::from_slice(&fs::read(source).ok()?).ok()?;
    let mut usage = UsageAccumulator::default();

    for message in gemini_messages(&parsed) {
        let Some(token_usage) = message.get("tokens") else {
            continue;
        };
        usage.model = message
            .get("model")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or(usage.model);
        usage.add(
            read_u64(token_usage, &["input"]),
            read_u64(token_usage, &["output"]),
            read_u64(token_usage, &["cached", "cacheRead"]),
            read_u64(token_usage, &["cacheWrite"]),
            read_u64(token_usage, &["thoughts", "reasoning"]),
            read_cost(token_usage),
        );
    }

    usage.into_session_usage()
}

fn extract_openclaw_usage(source: &Path) -> Option<SessionUsageRecord> {
    let reader = BufReader::new(File::open(source).ok()?);
    let mut usage = UsageAccumulator::default();

    for line in reader.lines() {
        let line = line.ok()?;
        let parsed: Value = serde_json::from_str(&line).ok()?;

        if openclaw_kind(&parsed) == Some("modelchange") {
            usage.model = parsed
                .get("modelId")
                .or_else(|| parsed.get("model"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
                .or(usage.model);
            continue;
        }

        if openclaw_kind(&parsed) != Some("message") {
            continue;
        }

        let Some(message) = parsed.get("message") else {
            continue;
        };
        if openclaw_role(message) != Some("assistant") {
            continue;
        }

        let Some(message_usage) = message.get("usage") else {
            continue;
        };
        usage.model = message
            .get("model")
            .or_else(|| message.get("modelId"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or(usage.model);
        usage.add(
            read_u64(message_usage, &["input", "input_tokens"]),
            read_u64(message_usage, &["output", "output_tokens"]),
            read_u64(message_usage, &["cacheRead", "cache_read_tokens"]),
            read_u64(message_usage, &["cacheWrite", "cache_write_tokens"]),
            read_u64(message_usage, &["reasoning", "reasoning_tokens"]),
            read_cost(message_usage),
        );
    }

    usage.into_session_usage()
}

fn read_u64(value: &Value, keys: &[&str]) -> u64 {
    keys.iter()
        .find_map(|key| {
            value.get(*key).and_then(|field| {
                field
                    .as_u64()
                    .or_else(|| field.as_i64().map(|v| v.max(0) as u64))
            })
        })
        .unwrap_or(0)
}

fn read_cost(value: &Value) -> Option<f64> {
    if let Some(cost) = value.get("cost") {
        if let Some(number) = cost.as_f64() {
            return Some(round_cost(number));
        }

        if let Some(number) = cost
            .get("total")
            .and_then(Value::as_f64)
            .or_else(|| cost.get("usd").and_then(Value::as_f64))
        {
            return Some(round_cost(number));
        }
    }

    None
}

fn estimate_usage_cost(
    model: &str,
    input_tokens: u64,
    output_tokens: u64,
    cache_read_tokens: u64,
    cache_write_tokens: u64,
    reasoning_tokens: u64,
) -> Option<f64> {
    let pricing = lookup_model_pricing(model)?;
    let total = (input_tokens as f64 * pricing.input_per_million_usd
        + output_tokens as f64 * pricing.output_per_million_usd
        + cache_read_tokens as f64 * pricing.cache_read_per_million_usd
        + cache_write_tokens as f64 * pricing.cache_write_per_million_usd
        + reasoning_tokens as f64 * pricing.reasoning_per_million_usd)
        / 1_000_000.0;

    Some(round_cost(total))
}

fn lookup_model_pricing(model: &str) -> Option<ModelPricing> {
    let normalized = model.trim().to_ascii_lowercase();

    if normalized.starts_with("claude-sonnet-4")
        || normalized.starts_with("openrouter/anthropic/claude-sonnet-4")
    {
        return Some(ModelPricing {
            input_per_million_usd: 3.0,
            output_per_million_usd: 15.0,
            cache_read_per_million_usd: 0.3,
            cache_write_per_million_usd: 3.75,
            reasoning_per_million_usd: 15.0,
        });
    }

    if normalized.starts_with("gemini-2.5-pro") {
        return Some(ModelPricing {
            input_per_million_usd: 1.25,
            output_per_million_usd: 10.0,
            cache_read_per_million_usd: 0.125,
            cache_write_per_million_usd: 1.25,
            reasoning_per_million_usd: 10.0,
        });
    }

    match normalized.as_str() {
        "gpt-5" | "gpt-5-codex" | "gpt-5-chat-latest" => Some(ModelPricing {
            input_per_million_usd: 1.25,
            output_per_million_usd: 10.0,
            cache_read_per_million_usd: 0.125,
            cache_write_per_million_usd: 1.25,
            reasoning_per_million_usd: 10.0,
        }),
        _ => None,
    }
}

fn bucket_usage_date(value: &str) -> Option<String> {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
        return Some(parsed.with_timezone(&Utc).format("%Y-%m-%d").to_string());
    }

    let milliseconds = value.parse::<i64>().ok()?;
    DateTime::from_timestamp_millis(milliseconds)
        .map(|timestamp| timestamp.format("%Y-%m-%d").to_string())
}

fn round_cost(value: f64) -> f64 {
    (value * 100_000.0).round() / 100_000.0
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::Value;

    use crate::{
        adapters::{
            claude_code::ClaudeCodeAdapter, codex::CodexAdapter, gemini_cli::GeminiCliAdapter,
            openclaw::OpenClawAdapter, opencode::OpenCodeAdapter, traits::SessionAdapter,
        },
        domain::session::SessionRecord,
    };

    use super::{build_usage_overview, extract_session_usage};

    #[test]
    fn extracts_usage_from_fixture_sessions() {
        let opencode = parse_fixture(OpenCodeAdapter, fixtures_root().join("opencode"));
        let claude = parse_fixture(ClaudeCodeAdapter, fixtures_root().join("claude"));
        let codex = parse_fixture(CodexAdapter, fixtures_root().join("codex"));
        let gemini = parse_fixture(GeminiCliAdapter, fixtures_root().join("gemini").join("tmp"));
        let openclaw = parse_fixture(OpenClawAdapter, fixtures_root().join("openclaw"));

        let opencode_usage = extract_session_usage(&opencode).expect("opencode usage exists");
        let claude_usage = extract_session_usage(&claude).expect("claude usage exists");
        let codex_usage = extract_session_usage(&codex).expect("codex usage exists");
        let gemini_usage = extract_session_usage(&gemini).expect("gemini usage exists");
        let openclaw_usage = extract_session_usage(&openclaw).expect("openclaw usage exists");

        assert_eq!(opencode_usage.model.as_deref(), Some("gpt-5"));
        assert_eq!(opencode_usage.total_tokens, 210);
        assert_eq!(claude_usage.cache_write_tokens, 144);
        assert_eq!(codex_usage.cache_read_tokens, 256);
        assert_eq!(gemini_usage.reasoning_tokens, 45);
        assert_eq!(openclaw_usage.cost_usd, Some(0.02));
    }

    #[test]
    fn aggregates_usage_by_assistant() {
        let overview = build_usage_overview([
            (
                "opencode".to_string(),
                Some(super::SessionUsageRecord {
                    model: Some("gpt-5".to_string()),
                    input_tokens: 120,
                    output_tokens: 80,
                    cache_read_tokens: 0,
                    cache_write_tokens: 0,
                    reasoning_tokens: 10,
                    total_tokens: 210,
                    cost_usd: Some(0.02),
                    cost_source: super::CostSource::Reported,
                }),
            ),
            (
                "opencode".to_string(),
                Some(super::SessionUsageRecord {
                    model: Some("gpt-5".to_string()),
                    input_tokens: 10,
                    output_tokens: 5,
                    cache_read_tokens: 1,
                    cache_write_tokens: 0,
                    reasoning_tokens: 0,
                    total_tokens: 16,
                    cost_usd: Some(0.01),
                    cost_source: super::CostSource::Reported,
                }),
            ),
            ("codex".to_string(), None),
        ]);

        assert_eq!(overview.totals.sessions_with_usage, 2);
        assert_eq!(overview.totals.total_tokens, 226);
        assert_eq!(overview.totals.cost_usd, Some(0.03));
        assert_eq!(overview.assistants.len(), 1);
        assert_eq!(overview.assistants[0].assistant, "opencode");
        assert_eq!(overview.assistants[0].session_count, 2);
    }

    #[test]
    fn omits_unknown_cost_from_serialized_session_usage() {
        let codex = parse_fixture(CodexAdapter, fixtures_root().join("codex"));
        let codex_usage = extract_session_usage(&codex).expect("codex usage exists");
        let serialized = serde_json::to_value(&codex_usage).expect("usage serializes");

        assert!(serialized.get("costUsd").is_none());
        assert_eq!(
            serialized.get("costSource").and_then(Value::as_str),
            Some("unknown")
        );
        assert_eq!(
            serialized.get("totalTokens").and_then(Value::as_u64),
            Some(codex_usage.total_tokens)
        );
    }

    #[test]
    fn estimates_cost_from_local_price_catalog_when_upstream_cost_is_missing() {
        let claude = parse_fixture(ClaudeCodeAdapter, fixtures_root().join("claude"));
        let claude_usage = extract_session_usage(&claude).expect("claude usage exists");
        let serialized = serde_json::to_value(&claude_usage).expect("usage serializes");

        assert_eq!(
            serialized.get("costUsd").and_then(Value::as_f64),
            Some(0.01301)
        );
        assert_eq!(
            serialized.get("costSource").and_then(Value::as_str),
            Some("estimated")
        );
    }

    #[test]
    fn preserves_reported_cost_source_when_session_already_has_cost() {
        let openclaw = parse_fixture(OpenClawAdapter, fixtures_root().join("openclaw"));
        let openclaw_usage = extract_session_usage(&openclaw).expect("openclaw usage exists");
        let serialized = serde_json::to_value(&openclaw_usage).expect("usage serializes");

        assert_eq!(
            serialized.get("costUsd").and_then(Value::as_f64),
            Some(0.02)
        );
        assert_eq!(
            serialized.get("costSource").and_then(Value::as_str),
            Some("reported")
        );
    }

    #[test]
    fn omits_aggregate_cost_when_any_session_lacks_reliable_cost() {
        let codex = parse_fixture(CodexAdapter, fixtures_root().join("codex"));
        let openclaw = parse_fixture(OpenClawAdapter, fixtures_root().join("openclaw"));
        let overview = build_usage_overview([
            ("codex".to_string(), extract_session_usage(&codex)),
            ("openclaw".to_string(), extract_session_usage(&openclaw)),
        ]);
        let serialized = serde_json::to_value(&overview).expect("overview serializes");
        let assistants = serialized
            .get("assistants")
            .and_then(Value::as_array)
            .expect("assistants array exists");
        let codex_assistant = assistants
            .iter()
            .find(|assistant| assistant.get("assistant").and_then(Value::as_str) == Some("codex"))
            .expect("codex assistant exists");
        let openclaw_assistant = assistants
            .iter()
            .find(|assistant| {
                assistant.get("assistant").and_then(Value::as_str) == Some("openclaw")
            })
            .expect("openclaw assistant exists");

        assert!(
            serialized
                .get("totals")
                .and_then(|totals| totals.get("costUsd"))
                .is_none()
        );
        assert!(codex_assistant.get("costUsd").is_none());
        assert_eq!(
            openclaw_assistant.get("costUsd").and_then(Value::as_f64),
            Some(0.02)
        );
    }

    fn parse_fixture(adapter: impl SessionAdapter, root: PathBuf) -> SessionRecord {
        let sources = adapter
            .discover_session_files(&root)
            .expect("fixture sources discover");
        adapter
            .parse_session(sources.first().expect("fixture source exists"))
            .expect("fixture session parses")
    }

    fn fixtures_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../tests/fixtures")
            .canonicalize()
            .expect("fixtures root resolves")
    }
}
