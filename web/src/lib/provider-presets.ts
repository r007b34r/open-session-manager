import type { ConfigRiskRecord, ConfigWritebackInput } from "./api";

export type ProviderPresetId =
  | "github_official"
  | "google_ai_studio"
  | "openai_official"
  | "openrouter_official";

export type ProviderPreset = {
  id: ProviderPresetId;
  provider: string;
  model?: string;
  baseUrl: string;
};

const PRESETS_BY_ASSISTANT: Record<string, ProviderPreset[]> = {
  "github-copilot-cli": [
    {
      id: "github_official",
      provider: "github",
      model: "gpt-5",
      baseUrl: "https://api.githubcopilot.com"
    }
  ],
  "factory-droid": [
    {
      id: "openai_official",
      provider: "openai",
      model: "gpt-5-mini",
      baseUrl: "https://api.openai.com/v1"
    },
    {
      id: "openrouter_official",
      provider: "openrouter",
      model: "openrouter/anthropic/claude-sonnet-4",
      baseUrl: "https://openrouter.ai/api/v1"
    }
  ],
  "gemini-cli": [
    {
      id: "google_ai_studio",
      provider: "google",
      model: "gemini-2.5-pro",
      baseUrl: "https://generativelanguage.googleapis.com/v1beta"
    }
  ],
  "qwen-cli": [
    {
      id: "openai_official",
      provider: "openai",
      model: "qwen3-coder-plus",
      baseUrl: "https://api.openai.com/v1"
    }
  ],
  openclaw: [
    {
      id: "openrouter_official",
      provider: "openrouter",
      model: "openrouter/anthropic/claude-sonnet-4",
      baseUrl: "https://openrouter.ai/api/v1"
    }
  ]
};

export function getProviderPresets(assistant: string) {
  return PRESETS_BY_ASSISTANT[normalizeAssistantKey(assistant)] ?? [];
}

export function applyProviderPreset(
  draft: ConfigWritebackInput,
  preset: ProviderPreset
): ConfigWritebackInput {
  return {
    ...draft,
    provider: preset.provider,
    model: preset.model,
    baseUrl: preset.baseUrl
  };
}

export function buildConfigDraft(config: ConfigRiskRecord): ConfigWritebackInput {
  return {
    artifactId: config.artifactId,
    assistant: config.assistant,
    scope: config.scope,
    path: config.path,
    provider: config.provider,
    model: config.model,
    baseUrl: config.baseUrl,
    secret: ""
  };
}

function normalizeAssistantKey(value: string) {
  return value.trim().toLowerCase().replaceAll(" ", "-");
}
