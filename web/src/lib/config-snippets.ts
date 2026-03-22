import type { ConfigWritebackInput } from "./api";

export const CONFIG_SNIPPET_STORAGE_KEY = "open-session-manager.config-snippets";

export type ConfigSnippetRecord = {
  version: 1;
  name: string;
  provider: string;
  model?: string;
  baseUrl: string;
  originAssistant: string;
  createdAt: string;
};

export type StoredConfigSnippetRecord = ConfigSnippetRecord & {
  id: string;
};

export function buildConfigSnippet(
  draft: ConfigWritebackInput,
  options: {
    name: string;
    createdAt?: string;
  }
): ConfigSnippetRecord {
  const createdAt = options.createdAt ?? new Date().toISOString();

  return {
    version: 1,
    name: normalizeSnippetName(options.name, draft),
    provider: draft.provider.trim(),
    model: normalizeOptionalText(draft.model),
    baseUrl: draft.baseUrl.trim(),
    originAssistant: draft.assistant,
    createdAt
  };
}

export function serializeConfigSnippet(snippet: ConfigSnippetRecord) {
  return JSON.stringify(snippet, null, 2);
}

export function parseConfigSnippet(raw: string): ConfigSnippetRecord {
  let parsed: unknown;

  try {
    parsed = JSON.parse(raw);
  } catch {
    throw new Error("invalid config snippet");
  }

  if (
    !isRecord(parsed) ||
    parsed.version !== 1 ||
    typeof parsed.name !== "string" ||
    typeof parsed.provider !== "string" ||
    typeof parsed.baseUrl !== "string" ||
    typeof parsed.originAssistant !== "string" ||
    typeof parsed.createdAt !== "string"
  ) {
    throw new Error("invalid config snippet");
  }

  if (
    parsed.name.trim().length === 0 ||
    parsed.provider.trim().length === 0 ||
    parsed.baseUrl.trim().length === 0
  ) {
    throw new Error("invalid config snippet");
  }

  return {
    version: 1,
    name: parsed.name.trim(),
    provider: parsed.provider.trim(),
    model: typeof parsed.model === "string" ? normalizeOptionalText(parsed.model) : undefined,
    baseUrl: parsed.baseUrl.trim(),
    originAssistant: parsed.originAssistant.trim(),
    createdAt: parsed.createdAt
  };
}

export function applyConfigSnippetToDraft(
  draft: ConfigWritebackInput,
  snippet: ConfigSnippetRecord
): ConfigWritebackInput {
  return {
    ...draft,
    provider: snippet.provider,
    model: snippet.model,
    baseUrl: snippet.baseUrl
  };
}

export function loadSavedConfigSnippets(): StoredConfigSnippetRecord[] {
  if (typeof window === "undefined") {
    return [];
  }

  try {
    const raw = window.localStorage.getItem(CONFIG_SNIPPET_STORAGE_KEY);
    if (!raw) {
      return [];
    }

    const parsed: unknown = JSON.parse(raw);
    if (!Array.isArray(parsed)) {
      return [];
    }

    return parsed
      .map(normalizeStoredSnippet)
      .filter((snippet): snippet is StoredConfigSnippetRecord => Boolean(snippet));
  } catch {
    return [];
  }
}

export function saveConfigSnippetRecord(
  snippet: ConfigSnippetRecord
): StoredConfigSnippetRecord[] {
  const current = loadSavedConfigSnippets();
  const storedSnippet: StoredConfigSnippetRecord = {
    ...snippet,
    id: `${safeSnippetSegment(snippet.name)}-${safeSnippetSegment(snippet.createdAt)}`
  };
  const next = [
    storedSnippet,
    ...current.filter((item) => item.id !== storedSnippet.id)
  ];

  persistSavedConfigSnippets(next);
  return next;
}

function persistSavedConfigSnippets(snippets: StoredConfigSnippetRecord[]) {
  if (typeof window === "undefined") {
    return;
  }

  try {
    window.localStorage.setItem(CONFIG_SNIPPET_STORAGE_KEY, JSON.stringify(snippets));
  } catch {
    // Ignore storage failures and keep in-memory behavior.
  }
}

function normalizeStoredSnippet(value: unknown) {
  if (!isRecord(value) || typeof value.id !== "string") {
    return undefined;
  }

  try {
    return {
      id: value.id,
      ...parseConfigSnippet(JSON.stringify(value))
    };
  } catch {
    return undefined;
  }
}

function normalizeSnippetName(name: string, draft: ConfigWritebackInput) {
  const normalized = name.trim();
  if (normalized) {
    return normalized;
  }

  return [draft.assistant, draft.provider, draft.model]
    .filter((value) => typeof value === "string" && value.trim().length > 0)
    .join(" ");
}

function normalizeOptionalText(value: string | undefined) {
  const normalized = value?.trim();
  return normalized ? normalized : undefined;
}

function safeSnippetSegment(value: string) {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
