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

export type SessionDetailRecord = SessionListItem & {
  summary: string;
  projectPath: string;
  sourcePath: string;
  tags: string[];
  riskFlags: string[];
  keyArtifacts: string[];
};

export type ConfigRiskRecord = {
  artifactId: string;
  assistant: string;
  scope: string;
  path: string;
  provider: string;
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
};

export type DashboardMetric = {
  label: string;
  value: string;
  note: string;
};

export type DashboardSnapshot = {
  metrics: DashboardMetric[];
  sessions: SessionDetailRecord[];
  configs: ConfigRiskRecord[];
  auditEvents: AuditEventRecord[];
};

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
      projectPath: "/home/max/src/agent-session-governance",
      sourcePath:
        "C:/Users/Max/.codex/sessions/2026/03/15/rollout-2026-03-15.jsonl",
      tags: ["wsl", "collector", "transport"],
      riskFlags: ["stale_followup_needed"],
      keyArtifacts: [
        "Defined distro handshake checkpoints",
        "Separated Windows path discovery from Linux payload collection",
        "Logged retry edge case for restore flow"
      ]
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
        "Mapped ANTHROPIC_BASE_URL override",
        "Captured hook command chain",
        "Flagged accept-edits default mode"
      ]
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
      projectPath: "/home/max/labs/session-governance",
      sourcePath:
        "/home/max/.local/share/opencode/storage/session/info/ses_demo.json",
      tags: ["export", "quarantine", "audit"],
      riskFlags: [],
      keyArtifacts: [
        "Wrote Markdown frontmatter template",
        "Added audit_events inserts for every destructive path",
        "Verified restore from manifest"
      ]
    }
  ],
  configs: [
    {
      artifactId: "cfg-001",
      assistant: "Codex",
      scope: "Global",
      path: "~/.codex/config.toml",
      provider: "cch",
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
      baseUrl: "https://openrouter.ai/api/v1",
      maskedSecret: "***3456",
      officialOrProxy: "Proxy",
      risks: ["third_party_provider", "dangerous_permissions"]
    }
  ],
  auditEvents: [
    {
      eventId: "evt-001",
      type: "export_markdown",
      target: "ses-003",
      actor: "Max",
      createdAt: "2026-03-15 13:12",
      result: "success",
      detail: "Exported Markdown briefing for cleanup-ready OpenCode session."
    },
    {
      eventId: "evt-002",
      type: "soft_delete",
      target: "ses-003",
      actor: "Max",
      createdAt: "2026-03-15 13:13",
      result: "success",
      detail: "Moved original transcript into quarantine manifest."
    },
    {
      eventId: "evt-003",
      type: "restore",
      target: "ses-003",
      actor: "Max",
      createdAt: "2026-03-15 13:14",
      result: "success",
      detail: "Restored transcript to original provider storage path."
    }
  ]
};

export async function fetchDashboardSnapshot(): Promise<DashboardSnapshot> {
  const realSnapshot = await tryFetchRealSnapshot();
  return realSnapshot ?? fallbackSnapshot;
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
        `Exported Markdown digest for ${session.title}.`
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

function createAuditEvent(
  type: string,
  target: string,
  detail: string
): AuditEventRecord {
  return {
    eventId: `${type}-${target}-${Date.now()}`,
    type,
    target,
    actor: "Max",
    createdAt: "2026-03-15 13:40",
    result: "success",
    detail
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
