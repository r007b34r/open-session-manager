export type SessionListItem = {
  sessionId: string;
  title: string;
  assistant: string;
  progressState: string;
  progressPercent: number;
  lastActivityAt: string;
  environment: string;
  valueScore: number;
};

export type TranscriptHighlight = {
  role: string;
  content: string;
};

export type TranscriptTodo = {
  content: string;
  completed: boolean;
};

export type SessionUsageRecord = {
  model?: string;
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheWriteTokens: number;
  reasoningTokens: number;
  totalTokens: number;
  costUsd: number;
};

export type SessionDetailRecord = SessionListItem & {
  summary: string;
  projectPath: string;
  sourcePath: string;
  tags: string[];
  riskFlags: string[];
  keyArtifacts: string[];
  transcriptHighlights: TranscriptHighlight[];
  todoItems: TranscriptTodo[];
  usage?: SessionUsageRecord;
};

export type ConfigRiskRecord = {
  artifactId: string;
  assistant: string;
  scope: string;
  path: string;
  provider: string;
  model?: string;
  baseUrl: string;
  maskedSecret: string;
  officialOrProxy: string;
  risks: string[];
};

export type AuditEventRecord = {
  eventId: string;
  type: string;
  target: string;
  actor: string;
  createdAt: string;
  result: string;
  detail: string;
  outputPath?: string;
  quarantinedPath?: string;
  manifestPath?: string;
};

export type DashboardMetric = {
  label: string;
  value: string;
  note: string;
};

export type DashboardRuntime = {
  auditDbPath: string;
  exportRoot: string;
  defaultExportRoot: string;
  exportRootSource: "default" | "custom";
  quarantineRoot: string;
  preferencesPath: string;
};

export type UsageTotalsRecord = {
  sessionsWithUsage: number;
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheWriteTokens: number;
  reasoningTokens: number;
  totalTokens: number;
  costUsd: number;
};

export type AssistantUsageRecord = {
  assistant: string;
  sessionCount: number;
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheWriteTokens: number;
  reasoningTokens: number;
  totalTokens: number;
  costUsd: number;
};

export type UsageOverviewRecord = {
  totals: UsageTotalsRecord;
  assistants: AssistantUsageRecord[];
};

export type DashboardSnapshot = {
  metrics: DashboardMetric[];
  sessions: SessionDetailRecord[];
  configs: ConfigRiskRecord[];
  auditEvents: AuditEventRecord[];
  usageOverview: UsageOverviewRecord;
  runtime: DashboardRuntime;
};

export type DashboardPreferencesUpdate = {
  exportRoot: string | null;
};

const DEMO_DATA_STORAGE_KEY = "open-session-manager.enable-demo-data";
const EMPTY_USAGE_OVERVIEW: UsageOverviewRecord = {
  totals: {
    sessionsWithUsage: 0,
    inputTokens: 0,
    outputTokens: 0,
    cacheReadTokens: 0,
    cacheWriteTokens: 0,
    reasoningTokens: 0,
    totalTokens: 0,
    costUsd: 0
  },
  assistants: []
};
const EMPTY_RUNTIME: DashboardRuntime = {
  auditDbPath: "",
  exportRoot: "",
  defaultExportRoot: "",
  exportRootSource: "default",
  quarantineRoot: "",
  preferencesPath: ""
};

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown;
  }
}

const fallbackSnapshot: DashboardSnapshot = {
  metrics: [
    {
      label: "Indexed Sessions",
      value: "184",
      note: "Across Windows, Linux, and WSL surfaces"
    },
    {
      label: "High-Value Candidates",
      value: "27",
      note: "Worth exporting before cleanup"
    },
    {
      label: "Risky Configs",
      value: "9",
      note: "Relay, wide permissions, or shell hooks"
    },
    {
      label: "Cold Storage Saved",
      value: "11.3 GB",
      note: "Potential reclaim from soft-delete queue"
    }
  ],
  sessions: [
    {
      sessionId: "ses-001",
      title: "Refactor WSL collector handshake",
      assistant: "Codex",
      progressState: "In Progress",
      progressPercent: 65,
      lastActivityAt: "2026-03-15 12:40",
      environment: "WSL: Ubuntu",
      valueScore: 84,
      summary:
        "Collector can enumerate distros and config roots, but transport framing still needs a stable manifest layer.",
      projectPath: "/home/max/src/open-session-manager",
      sourcePath:
        "C:/Users/Max/.codex/sessions/2026/03/15/rollout-2026-03-15.jsonl",
      tags: ["wsl", "collector", "transport"],
      riskFlags: ["stale_followup_needed"],
      keyArtifacts: [
        "Defined distro handshake checkpoints",
        "Separated Windows path discovery from Linux payload collection",
        "Logged retry edge case for restore flow"
      ],
      transcriptHighlights: [
        {
          role: "User",
          content: "Normalize the WSL collector handshake before adding more adapters."
        },
        {
          role: "Assistant",
          content:
            "Discovery roots are stable now. The next step is locking transport framing and manifest retries."
        }
      ],
      todoItems: [
        {
          content: "Finalize manifest framing",
          completed: true
        },
        {
          content: "Verify retry path in WSL",
          completed: false
        }
      ],
      usage: {
        model: "gpt-5-codex",
        inputTokens: 640,
        outputTokens: 128,
        cacheReadTokens: 256,
        cacheWriteTokens: 0,
        reasoningTokens: 0,
        totalTokens: 1024,
        costUsd: 0
      }
    },
    {
      sessionId: "ses-002",
      title: "Audit Anthropic relay settings",
      assistant: "Claude Code",
      progressState: "Blocked",
      progressPercent: 15,
      lastActivityAt: "2026-03-14 22:10",
      environment: "Windows 11",
      valueScore: 47,
      summary:
        "Proxy endpoint and permissive shell hooks were identified, but remediation steps were not applied yet.",
      projectPath: "C:/Users/Max/Desktop/ops",
      sourcePath: "C:/Users/Max/.claude/projects/ops/claude-ses-1.jsonl",
      tags: ["relay", "risk", "claude"],
      riskFlags: ["dangerous_permissions", "shell_hook"],
      keyArtifacts: [
        "Documented ANTHROPIC_BASE_URL override path",
        "Captured hook command chain",
        "Flagged accept-edits default mode"
      ],
      transcriptHighlights: [
        {
          role: "User",
          content:
            "Audit the relay endpoint, shell hooks, and decide whether this Claude session should be archived."
        },
        {
          role: "Assistant",
          content:
            "Mapped ANTHROPIC_BASE_URL override and traced the permissive shell hook chain, but remediation has not been applied yet."
        }
      ],
      todoItems: [
        {
          content: "Review shell hook chain",
          completed: false
        },
        {
          content: "Export remediation summary before cleanup",
          completed: true
        }
      ],
      usage: {
        model: "claude-sonnet-4-20250514",
        inputTokens: 1234,
        outputTokens: 567,
        cacheReadTokens: 890,
        cacheWriteTokens: 144,
        reasoningTokens: 0,
        totalTokens: 2835,
        costUsd: 0
      }
    },
    {
      sessionId: "ses-003",
      title: "Package export and quarantine workflow",
      assistant: "OpenCode",
      progressState: "Completed",
      progressPercent: 100,
      lastActivityAt: "2026-03-13 18:05",
      environment: "Linux",
      valueScore: 91,
      summary:
        "Markdown export, quarantine manifest, and restore chain were implemented and fully verified.",
      projectPath: "/home/max/labs/open-session-manager",
      sourcePath:
        "/home/max/.local/share/opencode/storage/session/info/ses_demo.json",
      tags: ["export", "quarantine", "audit"],
      riskFlags: [],
      keyArtifacts: [
        "Wrote Markdown frontmatter template",
        "Added audit_events inserts for every destructive path",
        "Verified restore from manifest"
      ],
      transcriptHighlights: [
        {
          role: "User",
          content: "Package the export and quarantine workflow for release verification."
        },
        {
          role: "Assistant",
          content:
            "Markdown export, quarantine manifest, and restore validation are complete and ready for release checks."
        }
      ],
      todoItems: [
        {
          content: "Confirm restore from manifest",
          completed: true
        }
      ],
      usage: {
        model: "gpt-5",
        inputTokens: 120,
        outputTokens: 80,
        cacheReadTokens: 0,
        cacheWriteTokens: 0,
        reasoningTokens: 10,
        totalTokens: 210,
        costUsd: 0.02
      }
    }
  ],
  configs: [
    {
      artifactId: "cfg-001",
      assistant: "Codex",
      scope: "Global",
      path: "~/.codex/config.toml",
      provider: "cch",
      model: "gpt-5-codex",
      baseUrl: "https://relay.cch.example/v1",
      maskedSecret: "***6789",
      officialOrProxy: "Proxy",
      risks: ["third_party_base_url", "dangerous_sandbox", "dangerous_approval_policy"]
    },
    {
      artifactId: "cfg-002",
      assistant: "Claude Code",
      scope: "Global",
      path: "~/.claude/settings.json",
      provider: "anthropic",
      model: "claude-sonnet-4",
      baseUrl: "https://relay.anthropic-proxy.example/v1",
      maskedSecret: "***4321",
      officialOrProxy: "Proxy",
      risks: ["dangerous_permissions", "shell_hook"]
    },
    {
      artifactId: "cfg-003",
      assistant: "OpenCode",
      scope: "Project",
      path: ".opencode/opencode.json",
      provider: "openrouter",
      model: "openrouter/anthropic/claude-sonnet-4",
      baseUrl: "https://openrouter.ai/api/v1",
      maskedSecret: "***3456",
      officialOrProxy: "Proxy",
      risks: ["third_party_provider", "dangerous_permissions"]
    },
    {
      artifactId: "cfg-004",
      assistant: "GitHub Copilot CLI",
      scope: "Global",
      path: "~/.copilot/config.json",
      provider: "github",
      model: "gpt-5",
      baseUrl: "https://copilot.enterprise-relay.example",
      maskedSecret: "***7890",
      officialOrProxy: "Proxy",
      risks: ["third_party_base_url", "dangerous_permissions"]
    },
    {
      artifactId: "cfg-005",
      assistant: "Factory Droid",
      scope: "Global",
      path: "~/.factory/settings.local.json",
      provider: "openrouter",
      model: "openrouter/anthropic/claude-sonnet-4",
      baseUrl: "https://factory-relay.example/v1",
      maskedSecret: "***7890",
      officialOrProxy: "Proxy",
      risks: [
        "third_party_provider",
        "third_party_base_url",
        "dangerous_permissions"
      ]
    }
  ],
  auditEvents: [
    {
      eventId: "evt-001",
      type: "export_markdown",
      target: "ses-003",
      actor: "r007b34r",
      createdAt: "2026-03-15 13:12",
      result: "success",
      detail: "Exported Markdown briefing for cleanup-ready OpenCode session.",
      outputPath:
        "C:/Users/Max/Documents/OpenSessionManager/exports/session-ses-003.md"
    },
    {
      eventId: "evt-002",
      type: "soft_delete",
      target: "ses-003",
      actor: "r007b34r",
      createdAt: "2026-03-15 13:13",
      result: "success",
      detail: "Moved original transcript into quarantine manifest.",
      quarantinedPath:
        "C:/Users/Max/AppData/Local/OpenSessionManager/quarantine/ses-003/payload/ses_demo.json",
      manifestPath:
        "C:/Users/Max/AppData/Local/OpenSessionManager/quarantine/ses-003/manifest.json"
    },
    {
      eventId: "evt-003",
      type: "restore",
      target: "ses-003",
      actor: "r007b34r",
      createdAt: "2026-03-15 13:14",
      result: "success",
      detail: "Restored transcript to original provider storage path."
    }
  ],
  usageOverview: {
    totals: {
      sessionsWithUsage: 3,
      inputTokens: 1994,
      outputTokens: 775,
      cacheReadTokens: 1146,
      cacheWriteTokens: 144,
      reasoningTokens: 10,
      totalTokens: 4069,
      costUsd: 0.02
    },
    assistants: [
      {
        assistant: "Claude Code",
        sessionCount: 1,
        inputTokens: 1234,
        outputTokens: 567,
        cacheReadTokens: 890,
        cacheWriteTokens: 144,
        reasoningTokens: 0,
        totalTokens: 2835,
        costUsd: 0
      },
      {
        assistant: "Codex",
        sessionCount: 1,
        inputTokens: 640,
        outputTokens: 128,
        cacheReadTokens: 256,
        cacheWriteTokens: 0,
        reasoningTokens: 0,
        totalTokens: 1024,
        costUsd: 0
      },
      {
        assistant: "OpenCode",
        sessionCount: 1,
        inputTokens: 120,
        outputTokens: 80,
        cacheReadTokens: 0,
        cacheWriteTokens: 0,
        reasoningTokens: 10,
        totalTokens: 210,
        costUsd: 0.02
      }
    ]
  },
  runtime: {
    auditDbPath: "C:/Users/Max/AppData/Local/OpenSessionManager/audit/audit.db",
    exportRoot: "C:/Users/Max/Documents/OpenSessionManager/exports",
    defaultExportRoot: "C:/Users/Max/Documents/OpenSessionManager/exports",
    exportRootSource: "default",
    quarantineRoot: "C:/Users/Max/AppData/Local/OpenSessionManager/quarantine",
    preferencesPath:
      "C:/Users/Max/AppData/Local/OpenSessionManager/preferences.json"
  }
};

export async function fetchDashboardSnapshot(): Promise<DashboardSnapshot> {
  const nativeSnapshot =
    await tryInvokeNativeCommand<DashboardSnapshot>("load_dashboard_snapshot");
  if (nativeSnapshot && isDashboardSnapshot(nativeSnapshot)) {
    return normalizeDashboardSnapshot(nativeSnapshot);
  }

  const realSnapshot = await tryFetchRealSnapshot();
  if (realSnapshot) {
    return applyBrowserRuntimePreferences(normalizeDashboardSnapshot(realSnapshot));
  }

  const browserSnapshot = shouldUseDemoData()
    ? normalizeDashboardSnapshot(fallbackSnapshot)
    : buildEmptyDashboardSnapshot();

  return applyBrowserRuntimePreferences(browserSnapshot);
}

export function recordMarkdownExport(
  current: DashboardSnapshot,
  sessionId: string
): DashboardSnapshot {
  const session = current.sessions.find((item) => item.sessionId === sessionId);
  if (!session) {
    return current;
  }

  return {
    ...current,
    auditEvents: [
      createAuditEvent(
        "export_markdown",
        session.sessionId,
        `Exported Markdown digest for ${session.title}.`,
        {
          outputPath: buildMarkdownOutputPath(current.runtime.exportRoot, session.sessionId)
        }
      ),
      ...current.auditEvents
    ]
  };
}

export function recordSoftDelete(
  current: DashboardSnapshot,
  sessionId: string
): DashboardSnapshot {
  const session = current.sessions.find((item) => item.sessionId === sessionId);
  if (!session) {
    return current;
  }

  const remainingSessions = current.sessions.filter(
    (item) => item.sessionId !== sessionId
  );

  return {
    ...current,
    sessions: remainingSessions,
    auditEvents: [
      createAuditEvent(
        "soft_delete",
        session.sessionId,
        `Moved ${session.title} into the quarantine queue.`
      ),
      ...current.auditEvents
    ]
  };
}

export async function applyMarkdownExport(
  current: DashboardSnapshot,
  sessionId: string
): Promise<DashboardSnapshot> {
  const nativeSnapshot = await tryInvokeNativeCommand<DashboardSnapshot>(
    "export_session_markdown",
    { sessionId }
  );

  if (nativeSnapshot && isDashboardSnapshot(nativeSnapshot)) {
    return normalizeDashboardSnapshot(nativeSnapshot);
  }

  return normalizeDashboardSnapshot(recordMarkdownExport(current, sessionId));
}

export async function applyDashboardPreferences(
  current: DashboardSnapshot,
  update: DashboardPreferencesUpdate
): Promise<DashboardSnapshot> {
  const nativeSnapshot = await tryInvokeNativeCommand<DashboardSnapshot>(
    "save_dashboard_preferences",
    {
      exportRoot: normalizeDashboardPreferencePath(update.exportRoot)
    }
  );

  if (nativeSnapshot && isDashboardSnapshot(nativeSnapshot)) {
    return normalizeDashboardSnapshot(nativeSnapshot);
  }

  const nextSnapshot = normalizeDashboardSnapshot(
    updateDashboardRuntime(current, update)
  );
  persistBrowserRuntimePreferences(nextSnapshot.runtime);
  return nextSnapshot;
}

export async function applySoftDelete(
  current: DashboardSnapshot,
  sessionId: string
): Promise<DashboardSnapshot> {
  const nativeSnapshot = await tryInvokeNativeCommand<DashboardSnapshot>(
    "soft_delete_session",
    { sessionId }
  );

  if (nativeSnapshot && isDashboardSnapshot(nativeSnapshot)) {
    return normalizeDashboardSnapshot(nativeSnapshot);
  }

  return normalizeDashboardSnapshot(recordSoftDelete(current, sessionId));
}

export function hasSuccessfulMarkdownExport(
  auditEvents: AuditEventRecord[],
  sessionId: string
) {
  return auditEvents.some(
    (event) =>
      event.type === "export_markdown" &&
      event.target === sessionId &&
      event.result === "success"
  );
}

function createAuditEvent(
  type: string,
  target: string,
  detail: string,
  paths: Partial<Pick<AuditEventRecord, "outputPath" | "quarantinedPath" | "manifestPath">> = {}
): AuditEventRecord {
  return {
    eventId: `${type}-${target}-${Date.now()}`,
    type,
    target,
    actor: "r007b34r",
    createdAt: "2026-03-15 13:40",
    result: "success",
    detail,
    ...paths
  };
}

async function tryFetchRealSnapshot() {
  if (typeof fetch !== "function") {
    return null;
  }

  try {
    const response = await fetch("/dashboard-snapshot.json", {
      cache: "no-store"
    });

    if (!response.ok) {
      return null;
    }

    const payload = await response.json();
    return isDashboardSnapshot(payload) ? payload : null;
  } catch {
    return null;
  }
}

async function tryInvokeNativeCommand<T>(
  command: string,
  args: Record<string, unknown> = {}
): Promise<T | null> {
  if (!isTauriRuntime()) {
    return null;
  }

  try {
    const { invoke } = await import("@tauri-apps/api/core");
    return await invoke<T>(command, args);
  } catch {
    return null;
  }
}

function isTauriRuntime() {
  return typeof window !== "undefined" && typeof window.__TAURI_INTERNALS__ !== "undefined";
}

function isDashboardSnapshot(value: unknown): value is DashboardSnapshot {
  if (!isRecord(value)) {
    return false;
  }

  return (
    Array.isArray(value.metrics) &&
    Array.isArray(value.sessions) &&
    Array.isArray(value.configs) &&
    Array.isArray(value.auditEvents)
  );
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function normalizeDashboardSnapshot(
  snapshot: DashboardSnapshot
): DashboardSnapshot {
  const sessions = [...snapshot.sessions]
    .map(normalizeSessionDetailRecord)
    .sort(compareSessionsByActivity);

  return {
    ...snapshot,
    auditEvents: Array.isArray(snapshot.auditEvents)
      ? snapshot.auditEvents
          .filter(isAuditEventRecord)
          .map(normalizeAuditEventRecord)
      : [],
    runtime: normalizeDashboardRuntime(snapshot.runtime),
    sessions,
    usageOverview: normalizeUsageOverview(snapshot.usageOverview, sessions)
  };
}

function normalizeDashboardRuntime(
  runtime?: Partial<DashboardRuntime>
): DashboardRuntime {
  return {
    ...EMPTY_RUNTIME,
    ...runtime,
    exportRootSource:
      runtime?.exportRootSource === "custom" ? "custom" : "default"
  };
}

function normalizeAuditEventRecord(event: AuditEventRecord): AuditEventRecord {
  return {
    ...event,
    outputPath: typeof event.outputPath === "string" ? event.outputPath : undefined,
    quarantinedPath:
      typeof event.quarantinedPath === "string" ? event.quarantinedPath : undefined,
    manifestPath:
      typeof event.manifestPath === "string" ? event.manifestPath : undefined
  };
}

function normalizeSessionDetailRecord(
  session: SessionDetailRecord
): SessionDetailRecord {
  return {
    ...session,
    transcriptHighlights: Array.isArray(session.transcriptHighlights)
      ? session.transcriptHighlights.filter(isTranscriptHighlight)
      : [],
    todoItems: Array.isArray(session.todoItems)
      ? session.todoItems.filter(isTranscriptTodo)
      : [],
    usage: isSessionUsageRecord(session.usage) ? session.usage : undefined
  };
}

function compareSessionsByActivity(
  left: SessionDetailRecord,
  right: SessionDetailRecord
) {
  const timestampDelta =
    parseActivityTimestamp(right.lastActivityAt) -
    parseActivityTimestamp(left.lastActivityAt);

  if (timestampDelta !== 0) {
    return timestampDelta;
  }

  const valueDelta = right.valueScore - left.valueScore;
  if (valueDelta !== 0) {
    return valueDelta;
  }

  return left.title.localeCompare(right.title);
}

function parseActivityTimestamp(value: string) {
  const direct = Date.parse(value);
  if (!Number.isNaN(direct)) {
    return direct;
  }

  const numeric = Number(value);
  if (Number.isFinite(numeric)) {
    return numeric;
  }

  return 0;
}

function isTranscriptHighlight(value: unknown): value is TranscriptHighlight {
  return (
    isRecord(value) &&
    typeof value.role === "string" &&
    typeof value.content === "string"
  );
}

function isTranscriptTodo(value: unknown): value is TranscriptTodo {
  return (
    isRecord(value) &&
    typeof value.content === "string" &&
    typeof value.completed === "boolean"
  );
}

function isAuditEventRecord(value: unknown): value is AuditEventRecord {
  return (
    isRecord(value) &&
    typeof value.eventId === "string" &&
    typeof value.type === "string" &&
    typeof value.target === "string" &&
    typeof value.actor === "string" &&
    typeof value.createdAt === "string" &&
    typeof value.result === "string" &&
    typeof value.detail === "string"
  );
}

function isSessionUsageRecord(value: unknown): value is SessionUsageRecord {
  return (
    isRecord(value) &&
    typeof value.inputTokens === "number" &&
    typeof value.outputTokens === "number" &&
    typeof value.cacheReadTokens === "number" &&
    typeof value.cacheWriteTokens === "number" &&
    typeof value.reasoningTokens === "number" &&
    typeof value.totalTokens === "number" &&
    typeof value.costUsd === "number"
  );
}

function normalizeUsageOverview(
  usageOverview: UsageOverviewRecord | undefined,
  sessions: SessionDetailRecord[]
): UsageOverviewRecord {
  if (isUsageOverviewRecord(usageOverview)) {
    return {
      totals: usageOverview.totals,
      assistants: [...usageOverview.assistants].sort(compareAssistantUsage)
    };
  }

  return deriveUsageOverviewFromSessions(sessions);
}

function isUsageOverviewRecord(value: unknown): value is UsageOverviewRecord {
  return (
    isRecord(value) &&
    isUsageTotalsRecord(value.totals) &&
    Array.isArray(value.assistants) &&
    value.assistants.every(isAssistantUsageRecord)
  );
}

function isUsageTotalsRecord(value: unknown): value is UsageTotalsRecord {
  return (
    isRecord(value) &&
    typeof value.sessionsWithUsage === "number" &&
    typeof value.inputTokens === "number" &&
    typeof value.outputTokens === "number" &&
    typeof value.cacheReadTokens === "number" &&
    typeof value.cacheWriteTokens === "number" &&
    typeof value.reasoningTokens === "number" &&
    typeof value.totalTokens === "number" &&
    typeof value.costUsd === "number"
  );
}

function isAssistantUsageRecord(value: unknown): value is AssistantUsageRecord {
  return (
    isRecord(value) &&
    typeof value.assistant === "string" &&
    typeof value.sessionCount === "number" &&
    typeof value.inputTokens === "number" &&
    typeof value.outputTokens === "number" &&
    typeof value.cacheReadTokens === "number" &&
    typeof value.cacheWriteTokens === "number" &&
    typeof value.reasoningTokens === "number" &&
    typeof value.totalTokens === "number" &&
    typeof value.costUsd === "number"
  );
}

function deriveUsageOverviewFromSessions(
  sessions: SessionDetailRecord[]
): UsageOverviewRecord {
  const assistants = new Map<string, AssistantUsageRecord>();
  const totals: UsageTotalsRecord = {
    sessionsWithUsage: 0,
    inputTokens: 0,
    outputTokens: 0,
    cacheReadTokens: 0,
    cacheWriteTokens: 0,
    reasoningTokens: 0,
    totalTokens: 0,
    costUsd: 0
  };

  for (const session of sessions) {
    if (!session.usage) {
      continue;
    }

    totals.sessionsWithUsage += 1;
    totals.inputTokens += session.usage.inputTokens;
    totals.outputTokens += session.usage.outputTokens;
    totals.cacheReadTokens += session.usage.cacheReadTokens;
    totals.cacheWriteTokens += session.usage.cacheWriteTokens;
    totals.reasoningTokens += session.usage.reasoningTokens;
    totals.totalTokens += session.usage.totalTokens;
    totals.costUsd = roundCost(totals.costUsd + session.usage.costUsd);

    const entry =
      assistants.get(session.assistant) ??
      {
        assistant: session.assistant,
        sessionCount: 0,
        inputTokens: 0,
        outputTokens: 0,
        cacheReadTokens: 0,
        cacheWriteTokens: 0,
        reasoningTokens: 0,
        totalTokens: 0,
        costUsd: 0
      };
    entry.sessionCount += 1;
    entry.inputTokens += session.usage.inputTokens;
    entry.outputTokens += session.usage.outputTokens;
    entry.cacheReadTokens += session.usage.cacheReadTokens;
    entry.cacheWriteTokens += session.usage.cacheWriteTokens;
    entry.reasoningTokens += session.usage.reasoningTokens;
    entry.totalTokens += session.usage.totalTokens;
    entry.costUsd = roundCost(entry.costUsd + session.usage.costUsd);
    assistants.set(session.assistant, entry);
  }

  return {
    totals,
    assistants: [...assistants.values()].sort(compareAssistantUsage)
  };
}

function compareAssistantUsage(
  left: AssistantUsageRecord,
  right: AssistantUsageRecord
) {
  const tokenDelta = right.totalTokens - left.totalTokens;
  if (tokenDelta !== 0) {
    return tokenDelta;
  }

  return left.assistant.localeCompare(right.assistant);
}

function roundCost(value: number) {
  return Math.round(value * 100000) / 100000;
}

function buildEmptyDashboardSnapshot(): DashboardSnapshot {
  return normalizeDashboardSnapshot({
    metrics: [],
    sessions: [],
    configs: [],
    auditEvents: [],
    usageOverview: EMPTY_USAGE_OVERVIEW,
    runtime: EMPTY_RUNTIME
  });
}

function shouldUseDemoData() {
  if (typeof window === "undefined") {
    return false;
  }

  try {
    return window.localStorage.getItem(DEMO_DATA_STORAGE_KEY) === "1";
  } catch {
    return false;
  }
}

function buildMarkdownOutputPath(exportRoot: string, sessionId: string) {
  return `${trimTrailingSlashes(exportRoot)}/session-${safeManagedName(sessionId)}.md`;
}

function safeManagedName(value: string) {
  const sanitized = value
    .split("")
    .map((character) =>
      /[A-Za-z0-9_-]/.test(character) ? character : "_"
    )
    .join("")
    .replace(/^_+|_+$/g, "");

  return sanitized || "session";
}

function trimTrailingSlashes(value: string) {
  return value.replace(/[\\/]+$/g, "");
}

function normalizeDashboardPreferencePath(value: string | null) {
  const normalized = value?.trim();
  return normalized ? normalized : null;
}

function updateDashboardRuntime(
  current: DashboardSnapshot,
  update: DashboardPreferencesUpdate
): DashboardSnapshot {
  const exportRoot = normalizeDashboardPreferencePath(update.exportRoot);

  return {
    ...current,
    runtime: {
      ...current.runtime,
      exportRoot: exportRoot ?? current.runtime.defaultExportRoot,
      exportRootSource: exportRoot ? "custom" : "default"
    }
  };
}

function applyBrowserRuntimePreferences(
  snapshot: DashboardSnapshot
): DashboardSnapshot {
  if (typeof window === "undefined") {
    return snapshot;
  }

  try {
    const storedExportRoot = normalizeDashboardPreferencePath(
      window.localStorage.getItem("open-session-manager.export-root")
    );
    if (!storedExportRoot) {
      return snapshot;
    }

    return normalizeDashboardSnapshot(
      updateDashboardRuntime(snapshot, { exportRoot: storedExportRoot })
    );
  } catch {
    return snapshot;
  }
}

function persistBrowserRuntimePreferences(runtime: DashboardRuntime) {
  if (typeof window === "undefined") {
    return;
  }

  try {
    if (runtime.exportRootSource === "custom") {
      window.localStorage.setItem("open-session-manager.export-root", runtime.exportRoot);
    } else {
      window.localStorage.removeItem("open-session-manager.export-root");
    }
  } catch {
    // Ignore storage failures in browser fallback mode.
  }
}
