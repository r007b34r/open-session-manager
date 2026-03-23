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

export type CostSource = "reported" | "estimated" | "mixed" | "unknown";

export type SessionUsageRecord = {
  model?: string;
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheWriteTokens: number;
  reasoningTokens: number;
  totalTokens: number;
  costUsd?: number;
  costSource: CostSource;
};

export type SessionControlRecord = {
  supported: boolean;
  available: boolean;
  controller: string;
  command: string;
  attached: boolean;
  lastCommand?: string;
  lastPrompt?: string;
  lastResponse?: string;
  lastError?: string;
  lastResumedAt?: string;
  lastContinuedAt?: string;
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
  sessionControl?: SessionControlRecord;
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
  mcpServers?: ConfigMcpServerRecord[];
};

export type ConfigMcpServerRecord = {
  serverId: string;
  name: string;
  enabled: boolean;
  status: string;
  transport: string;
  command?: string;
  args: string[];
  url?: string;
  configJson: string;
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

export type DoctorFindingRecord = {
  code: string;
  severity: string;
  assistant: string;
  path: string;
  detail: string;
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

export type GitCommitRecord = {
  sha: string;
  summary: string;
  author: string;
  authoredAt: string;
};

export type GitProjectRecord = {
  projectPath: string;
  repoRoot: string;
  branch: string;
  status: string;
  sessionCount: number;
  dirty: boolean;
  stagedChanges: number;
  unstagedChanges: number;
  untrackedFiles: number;
  ahead: number;
  behind: number;
  lastCommitSummary?: string;
  lastCommitAt?: string;
  recentCommits: GitCommitRecord[];
};

export type UsageTotalsRecord = {
  sessionsWithUsage: number;
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheWriteTokens: number;
  reasoningTokens: number;
  totalTokens: number;
  costUsd?: number;
  costSource: CostSource;
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
  costUsd?: number;
  costSource: CostSource;
};

export type UsageTimelineRecord = {
  date: string;
  sessionsWithUsage: number;
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheWriteTokens: number;
  reasoningTokens: number;
  totalTokens: number;
  costUsd?: number;
  costSource: CostSource;
};

export type UsageOverviewRecord = {
  totals: UsageTotalsRecord;
  assistants: AssistantUsageRecord[];
};

export type DashboardSnapshot = {
  metrics: DashboardMetric[];
  sessions: SessionDetailRecord[];
  configs: ConfigRiskRecord[];
  gitProjects?: GitProjectRecord[];
  doctorFindings: DoctorFindingRecord[];
  auditEvents: AuditEventRecord[];
  usageOverview: UsageOverviewRecord;
  usageTimeline: UsageTimelineRecord[];
  runtime: DashboardRuntime;
};

export type DashboardPreferencesUpdate = {
  exportRoot: string | null;
};

export type GitProjectCommitInput = {
  repoRoot: string;
  message: string;
};

export type GitProjectBranchSwitchInput = {
  repoRoot: string;
  branch: string;
};

export type GitProjectPushInput = {
  repoRoot: string;
  remote?: string;
};

export type LocalAuditEventInput = {
  type: string;
  target: string;
  detail: string;
  result?: string;
};

export type ConfigWritebackInput = {
  artifactId: string;
  assistant: string;
  scope: string;
  path: string;
  provider: string;
  model?: string;
  baseUrl: string;
  secret?: string;
};

export type SessionContinueInput = {
  sessionId: string;
  prompt: string;
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
    costSource: "unknown"
  },
  assistants: []
};
const EMPTY_USAGE_TIMELINE: UsageTimelineRecord[] = [];
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
        totalTokens: 1024
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
        totalTokens: 2835
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
      risks: ["third_party_base_url", "dangerous_sandbox", "dangerous_approval_policy"],
      mcpServers: []
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
      risks: ["dangerous_permissions", "shell_hook"],
      mcpServers: []
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
      risks: ["third_party_provider", "dangerous_permissions"],
      mcpServers: [
        {
          serverId: "cfg-003:filesystem",
          name: "filesystem",
          enabled: true,
          status: "enabled",
          transport: "embedded",
          args: [],
          configJson: '{\n  "enabled": true\n}'
        }
      ]
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
      risks: ["third_party_base_url", "dangerous_permissions"],
      mcpServers: [
        {
          serverId: "cfg-004:filesystem",
          name: "filesystem",
          enabled: true,
          status: "configured",
          transport: "stdio",
          command: "node",
          args: ["mcp-filesystem.js"],
          configJson:
            '{\n  "command": "node",\n  "args": ["mcp-filesystem.js"]\n}'
        }
      ]
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
      ],
      mcpServers: [
        {
          serverId: "cfg-005:postgres",
          name: "postgres",
          enabled: true,
          status: "configured",
          transport: "stdio",
          command: "uvx",
          args: ["mcp-postgres"],
          configJson: '{\n  "command": "uvx",\n  "args": ["mcp-postgres"]\n}'
        }
      ]
    }
  ],
  gitProjects: [
    {
      projectPath: "C:/Users/Max/Desktop/2026年3月15日",
      repoRoot: "C:/Users/Max/Desktop/2026年3月15日",
      branch: "feat/usability-clarity",
      status: "dirty",
      sessionCount: 3,
      dirty: true,
      stagedChanges: 1,
      unstagedChanges: 2,
      untrackedFiles: 1,
      ahead: 0,
      behind: 0,
      lastCommitSummary: "feat: add active session cockpit",
      lastCommitAt: "2026-03-23T01:40:00.000Z",
      recentCommits: [
        {
          sha: "7fd57a6",
          summary: "feat: add active session cockpit",
          author: "r007b34r",
          authoredAt: "2026-03-23T01:40:00.000Z"
        },
        {
          sha: "9042ddf",
          summary: "test: add mobile viewport matrix",
          author: "r007b34r",
          authoredAt: "2026-03-23T01:10:00.000Z"
        }
      ]
    }
  ],
  doctorFindings: [
    {
      code: "malformed_session_skipped",
      severity: "warn",
      assistant: "Claude Code",
      path: "C:/Users/Max/.claude/projects/C--Users-Max/broken-session.jsonl",
      detail:
        "Skipped malformed session file for Claude Code because the session id was missing."
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
      costSource: "unknown"
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
        costUsd: 0.01301,
        costSource: "estimated"
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
        costSource: "unknown"
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
        costUsd: 0.02,
        costSource: "reported"
      }
    ]
  },
  usageTimeline: [
    {
      date: "2026-03-15",
      sessionsWithUsage: 3,
      inputTokens: 1994,
      outputTokens: 775,
      cacheReadTokens: 1146,
      cacheWriteTokens: 144,
      reasoningTokens: 10,
      totalTokens: 4069,
      costSource: "unknown"
    }
  ],
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
    ? normalizeDashboardSnapshot(seedDemoSessionControls(fallbackSnapshot))
    : buildEmptyDashboardSnapshot();

  return applyBrowserRuntimePreferences(browserSnapshot);
}

export function isConfigWritebackAvailable() {
  return isTauriRuntime() || shouldUseDemoData();
}

export function isGitProjectActionAvailable() {
  return isTauriRuntime() || shouldUseDemoData();
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

export function recordConfigWriteback(
  current: DashboardSnapshot,
  input: ConfigWritebackInput
): DashboardSnapshot {
  const nextConfigs = current.configs.map((config) => {
    if (config.artifactId !== input.artifactId) {
      return config;
    }

    return {
      ...config,
      assistant: input.assistant,
      scope: input.scope,
      path: input.path,
      provider: input.provider,
      model: normalizeOptionalText(input.model),
      baseUrl: input.baseUrl,
      maskedSecret: input.secret ? maskSecret(input.secret) : config.maskedSecret,
      officialOrProxy: inferProxyModeFromEndpoint(input.provider, input.baseUrl),
      risks: reconcileConfigRisks(config.risks, input.provider, input.baseUrl)
    };
  });

  return {
    ...current,
    configs: nextConfigs,
    auditEvents: [
      createAuditEvent(
        "config_writeback",
        input.artifactId,
        `Updated config fields for ${input.assistant}.`,
        {
          manifestPath: buildConfigBackupManifestPath(current.runtime, input.artifactId)
        }
      ),
      ...current.auditEvents
    ]
  };
}

export function recordGitProjectCommit(
  current: DashboardSnapshot,
  input: GitProjectCommitInput
): DashboardSnapshot {
  const message = normalizeOptionalText(input.message);
  if (!message) {
    return current;
  }

  const committedAt = new Date().toISOString();
  let matched = false;
  const nextProjects = (current.gitProjects ?? []).map((project) => {
    if (project.repoRoot !== input.repoRoot) {
      return project;
    }

    matched = true;
    const ahead = project.ahead + 1;
    return {
      ...project,
      dirty: false,
      status: ahead > 0 || project.behind > 0 ? "diverged" : "clean",
      stagedChanges: 0,
      unstagedChanges: 0,
      untrackedFiles: 0,
      ahead,
      lastCommitSummary: message,
      lastCommitAt: committedAt,
      recentCommits: [
        {
          sha: createDemoGitSha(committedAt, message),
          summary: message,
          author: "r007b34r",
          authoredAt: committedAt
        },
        ...project.recentCommits
      ].slice(0, 5)
    };
  });

  if (!matched) {
    return current;
  }

  return {
    ...current,
    gitProjects: nextProjects,
    auditEvents: [
      createAuditEvent("git_commit", input.repoRoot, `Committed ${message}.`),
      ...current.auditEvents
    ]
  };
}

export function recordGitProjectBranchSwitch(
  current: DashboardSnapshot,
  input: GitProjectBranchSwitchInput
): DashboardSnapshot {
  const branch = normalizeOptionalText(input.branch);
  if (!branch) {
    return current;
  }

  let matched = false;
  const nextProjects = (current.gitProjects ?? []).map((project) => {
    if (project.repoRoot !== input.repoRoot) {
      return project;
    }

    matched = true;
    return {
      ...project,
      branch
    };
  });

  if (!matched) {
    return current;
  }

  return {
    ...current,
    gitProjects: nextProjects,
    auditEvents: [
      createAuditEvent(
        "git_branch_switch",
        input.repoRoot,
        `Switched to ${branch}.`
      ),
      ...current.auditEvents
    ]
  };
}

export function recordGitProjectPush(
  current: DashboardSnapshot,
  input: GitProjectPushInput
): DashboardSnapshot {
  const remote = normalizeOptionalText(input.remote) ?? "origin";
  let detail = `Pushed current branch to ${remote}.`;
  let matched = false;
  const nextProjects = (current.gitProjects ?? []).map((project) => {
    if (project.repoRoot !== input.repoRoot) {
      return project;
    }

    matched = true;
    detail = `Pushed ${project.branch} to ${remote}.`;
    return {
      ...project,
      ahead: 0,
      status: project.dirty ? "dirty" : project.behind > 0 ? "diverged" : "clean"
    };
  });

  if (!matched) {
    return current;
  }

  return {
    ...current,
    gitProjects: nextProjects,
    auditEvents: [createAuditEvent("git_push", input.repoRoot, detail), ...current.auditEvents]
  };
}

export function recordLocalAuditEvent(
  current: DashboardSnapshot,
  input: LocalAuditEventInput
): DashboardSnapshot {
  return {
    ...current,
    auditEvents: [
      createAuditEvent(input.type, input.target, input.detail, {
        result: input.result
      }),
      ...current.auditEvents
    ]
  };
}

export function recordSessionResume(
  current: DashboardSnapshot,
  sessionId: string
): DashboardSnapshot {
  const nextSessions = current.sessions.map((session) => {
    if (session.sessionId !== sessionId) {
      return session;
    }

    return {
      ...session,
      sessionControl: {
        ...resolveSessionControlState(session),
        attached: true,
        available: true,
        lastResponse: "READY from demo resume",
        lastResumedAt: new Date().toISOString()
      }
    };
  });

  return {
    ...current,
    sessions: nextSessions,
    auditEvents: [
      createAuditEvent(
        "session_resume",
        sessionId,
        `Resumed ${sessionId} from the session detail panel.`
      ),
      ...current.auditEvents
    ]
  };
}

export function recordSessionContinue(
  current: DashboardSnapshot,
  input: SessionContinueInput
): DashboardSnapshot {
  const nextSessions = current.sessions.map((session) => {
    if (session.sessionId !== input.sessionId) {
      return session;
    }

    return {
      ...session,
      sessionControl: {
        ...resolveSessionControlState(session),
        attached: true,
        available: true,
        lastPrompt: input.prompt,
        lastResponse: `READY from demo continue: ${input.prompt}`,
        lastContinuedAt: new Date().toISOString()
      }
    };
  });

  return {
    ...current,
    sessions: nextSessions,
    auditEvents: [
      createAuditEvent(
        "session_continue",
        input.sessionId,
        `Sent a follow-up prompt to ${input.sessionId}.`
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

export async function applySessionResume(
  current: DashboardSnapshot,
  sessionId: string
): Promise<DashboardSnapshot> {
  const nativeSnapshot = await tryInvokeNativeCommand<DashboardSnapshot>(
    "resume_existing_session",
    { sessionId }
  );

  if (nativeSnapshot && isDashboardSnapshot(nativeSnapshot)) {
    return normalizeDashboardSnapshot(nativeSnapshot);
  }

  if (shouldUseDemoData()) {
    return normalizeDashboardSnapshot(recordSessionResume(current, sessionId));
  }

  return current;
}

export async function applySessionContinue(
  current: DashboardSnapshot,
  input: SessionContinueInput
): Promise<DashboardSnapshot> {
  const nativeSnapshot = await tryInvokeNativeCommand<DashboardSnapshot>(
    "continue_existing_session",
    input
  );

  if (nativeSnapshot && isDashboardSnapshot(nativeSnapshot)) {
    return normalizeDashboardSnapshot(nativeSnapshot);
  }

  if (shouldUseDemoData()) {
    return normalizeDashboardSnapshot(recordSessionContinue(current, input));
  }

  return current;
}

export async function applyConfigWriteback(
  current: DashboardSnapshot,
  input: ConfigWritebackInput
): Promise<DashboardSnapshot> {
  const nativeSnapshot = await tryInvokeNativeCommand<DashboardSnapshot>(
    "write_config_artifact",
    {
      artifactId: input.artifactId,
      assistant: input.assistant,
      scope: input.scope,
      path: input.path,
      provider: input.provider,
      model: normalizeOptionalText(input.model),
      baseUrl: input.baseUrl,
      secret: normalizeOptionalText(input.secret)
    }
  );

  if (nativeSnapshot && isDashboardSnapshot(nativeSnapshot)) {
    return normalizeDashboardSnapshot(nativeSnapshot);
  }

  if (shouldUseDemoData()) {
    return normalizeDashboardSnapshot(recordConfigWriteback(current, input));
  }

  return current;
}

export async function applyGitProjectCommit(
  current: DashboardSnapshot,
  input: GitProjectCommitInput
): Promise<DashboardSnapshot> {
  const nativeSnapshot = await tryInvokeNativeCommand<DashboardSnapshot>(
    "commit_git_project",
    {
      repoRoot: input.repoRoot,
      message: normalizeOptionalText(input.message)
    }
  );

  if (nativeSnapshot && isDashboardSnapshot(nativeSnapshot)) {
    return normalizeDashboardSnapshot(nativeSnapshot);
  }

  if (shouldUseDemoData()) {
    return normalizeDashboardSnapshot(recordGitProjectCommit(current, input));
  }

  return current;
}

export async function applyGitProjectBranchSwitch(
  current: DashboardSnapshot,
  input: GitProjectBranchSwitchInput
): Promise<DashboardSnapshot> {
  const nativeSnapshot = await tryInvokeNativeCommand<DashboardSnapshot>(
    "switch_git_project_branch",
    {
      repoRoot: input.repoRoot,
      branch: normalizeOptionalText(input.branch)
    }
  );

  if (nativeSnapshot && isDashboardSnapshot(nativeSnapshot)) {
    return normalizeDashboardSnapshot(nativeSnapshot);
  }

  if (shouldUseDemoData()) {
    return normalizeDashboardSnapshot(recordGitProjectBranchSwitch(current, input));
  }

  return current;
}

export async function applyGitProjectPush(
  current: DashboardSnapshot,
  input: GitProjectPushInput
): Promise<DashboardSnapshot> {
  const nativeSnapshot = await tryInvokeNativeCommand<DashboardSnapshot>(
    "push_git_project",
    {
      repoRoot: input.repoRoot,
      remote: normalizeOptionalText(input.remote)
    }
  );

  if (nativeSnapshot && isDashboardSnapshot(nativeSnapshot)) {
    return normalizeDashboardSnapshot(nativeSnapshot);
  }

  if (shouldUseDemoData()) {
    return normalizeDashboardSnapshot(recordGitProjectPush(current, input));
  }

  return current;
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
  paths: Partial<
    Pick<AuditEventRecord, "outputPath" | "quarantinedPath" | "manifestPath" | "result">
  > = {}
): AuditEventRecord {
  return {
    eventId: `${type}-${target}-${Date.now()}`,
    type,
    target,
    actor: "r007b34r",
    createdAt: "2026-03-15 13:40",
    ...paths,
    result: paths.result ?? "success",
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
    (!("gitProjects" in value) ||
      (Array.isArray(value.gitProjects) &&
        value.gitProjects.every(isGitProjectRecord))) &&
    (!("doctorFindings" in value) ||
      (Array.isArray(value.doctorFindings) &&
        value.doctorFindings.every(isDoctorFindingRecord))) &&
    Array.isArray(value.auditEvents) &&
    (!("usageTimeline" in value) ||
      (Array.isArray(value.usageTimeline) &&
        value.usageTimeline.every(isUsageTimelineRecord)))
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
    configs: Array.isArray(snapshot.configs)
      ? snapshot.configs.filter(isConfigRiskRecord).map(normalizeConfigRiskRecord)
      : [],
    gitProjects: Array.isArray(snapshot.gitProjects)
      ? snapshot.gitProjects.filter(isGitProjectRecord).map(normalizeGitProjectRecord)
      : [],
    doctorFindings: Array.isArray(snapshot.doctorFindings)
      ? snapshot.doctorFindings.filter(isDoctorFindingRecord)
      : [],
    auditEvents: Array.isArray(snapshot.auditEvents)
      ? snapshot.auditEvents
          .filter(isAuditEventRecord)
          .map(normalizeAuditEventRecord)
      : [],
    runtime: normalizeDashboardRuntime(snapshot.runtime),
    sessions,
    usageOverview: normalizeUsageOverview(snapshot.usageOverview, sessions),
    usageTimeline: normalizeUsageTimeline(snapshot.usageTimeline, sessions)
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
  const usage = isSessionUsageRecord(session.usage)
    ? normalizeSessionUsageRecord(session.usage)
    : undefined;
  const sessionControl = isSessionControlRecord(session.sessionControl)
    ? normalizeSessionControlRecord(session.sessionControl)
    : undefined;

  return {
    ...session,
    transcriptHighlights: Array.isArray(session.transcriptHighlights)
      ? session.transcriptHighlights.filter(isTranscriptHighlight)
      : [],
    todoItems: Array.isArray(session.todoItems)
      ? session.todoItems.filter(isTranscriptTodo)
      : [],
    ...(usage ? { usage } : {}),
    ...(sessionControl ? { sessionControl } : {})
  };
}

function normalizeSessionControlRecord(
  control: SessionControlRecord
): SessionControlRecord {
  return {
    ...control,
    lastCommand:
      typeof control.lastCommand === "string" ? control.lastCommand : undefined,
    lastPrompt:
      typeof control.lastPrompt === "string" ? control.lastPrompt : undefined,
    lastResponse:
      typeof control.lastResponse === "string" ? control.lastResponse : undefined,
    lastError:
      typeof control.lastError === "string" ? control.lastError : undefined,
    lastResumedAt:
      typeof control.lastResumedAt === "string" ? control.lastResumedAt : undefined,
    lastContinuedAt:
      typeof control.lastContinuedAt === "string"
        ? control.lastContinuedAt
        : undefined
  };
}

function seedDemoSessionControls(snapshot: DashboardSnapshot): DashboardSnapshot {
  return {
    ...snapshot,
    sessions: snapshot.sessions.map((session) => ({
      ...session,
      sessionControl: session.sessionControl ?? buildDemoSessionControl(session)
    }))
  };
}

function buildDemoSessionControl(
  session: Pick<SessionDetailRecord, "assistant">
): SessionControlRecord | undefined {
  if (session.assistant === "Codex" || session.assistant === "codex") {
    return {
      supported: true,
      available: true,
      controller: "codex",
      command: "codex",
      attached: false
    };
  }

  if (
    session.assistant === "Claude Code" ||
    session.assistant === "claude-code"
  ) {
    return {
      supported: true,
      available: true,
      controller: "claude-code",
      command: "claude",
      attached: false
    };
  }

  return undefined;
}

function resolveSessionControlState(
  session: Pick<SessionDetailRecord, "assistant" | "sessionControl">
): SessionControlRecord {
  return (
    session.sessionControl ??
    buildDemoSessionControl(session) ?? {
      supported: false,
      available: false,
      controller: "unsupported",
      command: "",
      attached: false
    }
  );
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

function isConfigRiskRecord(value: unknown): value is ConfigRiskRecord {
  return (
    isRecord(value) &&
    typeof value.artifactId === "string" &&
    typeof value.assistant === "string" &&
    typeof value.scope === "string" &&
    typeof value.path === "string" &&
    typeof value.provider === "string" &&
    typeof value.baseUrl === "string" &&
    typeof value.maskedSecret === "string" &&
    typeof value.officialOrProxy === "string" &&
    Array.isArray(value.risks)
  );
}

function normalizeConfigRiskRecord(config: ConfigRiskRecord): ConfigRiskRecord {
  return {
    ...config,
    mcpServers: Array.isArray(config.mcpServers)
      ? config.mcpServers.filter(isConfigMcpServerRecord).map(normalizeConfigMcpServerRecord)
      : []
  };
}

function isConfigMcpServerRecord(value: unknown): value is ConfigMcpServerRecord {
  return (
    isRecord(value) &&
    typeof value.serverId === "string" &&
    typeof value.name === "string" &&
    typeof value.enabled === "boolean" &&
    typeof value.status === "string" &&
    typeof value.transport === "string" &&
    Array.isArray(value.args) &&
    value.args.every((item) => typeof item === "string")
  );
}

function normalizeConfigMcpServerRecord(
  server: ConfigMcpServerRecord
): ConfigMcpServerRecord {
  return {
    ...server,
    command: typeof server.command === "string" ? server.command : undefined,
    args: Array.isArray(server.args)
      ? server.args.filter((item) => typeof item === "string")
      : [],
    url: typeof server.url === "string" ? server.url : undefined,
    configJson:
      typeof server.configJson === "string"
        ? server.configJson
        : JSON.stringify(server, null, 2)
  };
}

function isGitProjectRecord(value: unknown): value is GitProjectRecord {
  return (
    isRecord(value) &&
    typeof value.projectPath === "string" &&
    typeof value.repoRoot === "string" &&
    typeof value.branch === "string" &&
    typeof value.status === "string" &&
    typeof value.sessionCount === "number" &&
    typeof value.dirty === "boolean" &&
    typeof value.stagedChanges === "number" &&
    typeof value.unstagedChanges === "number" &&
    typeof value.untrackedFiles === "number" &&
    typeof value.ahead === "number" &&
    typeof value.behind === "number" &&
    Array.isArray(value.recentCommits) &&
    value.recentCommits.every(isGitCommitRecord)
  );
}

function isGitCommitRecord(value: unknown): value is GitCommitRecord {
  return (
    isRecord(value) &&
    typeof value.sha === "string" &&
    typeof value.summary === "string" &&
    typeof value.author === "string" &&
    typeof value.authoredAt === "string"
  );
}

function normalizeGitProjectRecord(project: GitProjectRecord): GitProjectRecord {
  return {
    ...project,
    lastCommitSummary:
      typeof project.lastCommitSummary === "string" ? project.lastCommitSummary : undefined,
    lastCommitAt: typeof project.lastCommitAt === "string" ? project.lastCommitAt : undefined,
    recentCommits: Array.isArray(project.recentCommits)
      ? project.recentCommits.filter(isGitCommitRecord).map(normalizeGitCommitRecord)
      : []
  };
}

function normalizeGitCommitRecord(commit: GitCommitRecord): GitCommitRecord {
  return {
    ...commit,
    sha: commit.sha.trim(),
    summary: commit.summary.trim(),
    author: commit.author.trim(),
    authoredAt: commit.authoredAt.trim()
  };
}

function isSessionControlRecord(value: unknown): value is SessionControlRecord {
  return (
    isRecord(value) &&
    typeof value.supported === "boolean" &&
    typeof value.available === "boolean" &&
    typeof value.controller === "string" &&
    typeof value.command === "string" &&
    typeof value.attached === "boolean"
  );
}

function isDoctorFindingRecord(value: unknown): value is DoctorFindingRecord {
  return (
    isRecord(value) &&
    typeof value.code === "string" &&
    typeof value.severity === "string" &&
    typeof value.assistant === "string" &&
    typeof value.path === "string" &&
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
    hasOptionalCostValue(value.costUsd) &&
    hasOptionalCostSource(value.costSource)
  );
}

function normalizeUsageOverview(
  usageOverview: UsageOverviewRecord | undefined,
  sessions: SessionDetailRecord[]
): UsageOverviewRecord {
  if (isUsageOverviewRecord(usageOverview)) {
    return {
      totals: normalizeUsageTotalsRecord(usageOverview.totals),
      assistants: usageOverview.assistants
        .map(normalizeAssistantUsageRecord)
        .sort(compareAssistantUsage)
    };
  }

  return deriveUsageOverviewFromSessions(sessions);
}

function normalizeUsageTimeline(
  usageTimeline: UsageTimelineRecord[] | undefined,
  sessions: SessionDetailRecord[]
): UsageTimelineRecord[] {
  if (Array.isArray(usageTimeline) && usageTimeline.every(isUsageTimelineRecord)) {
    return usageTimeline
      .map(normalizeUsageTimelineRecord)
      .sort((left, right) => left.date.localeCompare(right.date));
  }

  return deriveUsageTimelineFromSessions(sessions);
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
    hasOptionalCostValue(value.costUsd) &&
    hasOptionalCostSource(value.costSource)
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
    hasOptionalCostValue(value.costUsd) &&
    hasOptionalCostSource(value.costSource)
  );
}

function isUsageTimelineRecord(value: unknown): value is UsageTimelineRecord {
  return (
    isRecord(value) &&
    typeof value.date === "string" &&
    typeof value.sessionsWithUsage === "number" &&
    typeof value.inputTokens === "number" &&
    typeof value.outputTokens === "number" &&
    typeof value.cacheReadTokens === "number" &&
    typeof value.cacheWriteTokens === "number" &&
    typeof value.reasoningTokens === "number" &&
    typeof value.totalTokens === "number" &&
    hasOptionalCostValue(value.costUsd) &&
    hasOptionalCostSource(value.costSource)
  );
}

type UsageAggregateState = {
  sessionCount: number;
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheWriteTokens: number;
  reasoningTokens: number;
  totalTokens: number;
  costUsd: number;
  hasKnownCost: boolean;
  hasUnknownCost: boolean;
  hasReported: boolean;
  hasEstimated: boolean;
};

function deriveUsageOverviewFromSessions(
  sessions: SessionDetailRecord[]
): UsageOverviewRecord {
  const assistants = new Map<string, UsageAggregateState>();
  const totals = createUsageAggregateState();

  for (const session of sessions) {
    if (!session.usage) {
      continue;
    }

    accumulateUsageAggregate(totals, session.usage);

    const entry = assistants.get(session.assistant) ?? createUsageAggregateState();
    accumulateUsageAggregate(entry, session.usage);
    assistants.set(session.assistant, entry);
  }

  const totalCost = resolveAggregateCost(totals);
  return {
    totals: {
      sessionsWithUsage: totals.sessionCount,
      inputTokens: totals.inputTokens,
      outputTokens: totals.outputTokens,
      cacheReadTokens: totals.cacheReadTokens,
      cacheWriteTokens: totals.cacheWriteTokens,
      reasoningTokens: totals.reasoningTokens,
      totalTokens: totals.totalTokens,
      costUsd: totalCost.costUsd,
      costSource: totalCost.costSource
    },
    assistants: [...assistants.entries()]
      .map(([assistant, entry]) => {
        const cost = resolveAggregateCost(entry);
        return {
          assistant,
          sessionCount: entry.sessionCount,
          inputTokens: entry.inputTokens,
          outputTokens: entry.outputTokens,
          cacheReadTokens: entry.cacheReadTokens,
          cacheWriteTokens: entry.cacheWriteTokens,
          reasoningTokens: entry.reasoningTokens,
          totalTokens: entry.totalTokens,
          costUsd: cost.costUsd,
          costSource: cost.costSource
        };
      })
      .sort(compareAssistantUsage)
  };
}

function deriveUsageTimelineFromSessions(
  sessions: SessionDetailRecord[]
): UsageTimelineRecord[] {
  const buckets = new Map<string, UsageAggregateState>();

  for (const session of sessions) {
    if (!session.usage) {
      continue;
    }

    const date = normalizeUsageDate(session.lastActivityAt);
    if (!date) {
      continue;
    }

    const entry = buckets.get(date) ?? createUsageAggregateState();
    accumulateUsageAggregate(entry, session.usage);
    buckets.set(date, entry);
  }

  return [...buckets.entries()]
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([date, entry]) => {
      const cost = resolveAggregateCost(entry);
      return {
        date,
        sessionsWithUsage: entry.sessionCount,
        inputTokens: entry.inputTokens,
        outputTokens: entry.outputTokens,
        cacheReadTokens: entry.cacheReadTokens,
        cacheWriteTokens: entry.cacheWriteTokens,
        reasoningTokens: entry.reasoningTokens,
        totalTokens: entry.totalTokens,
        costUsd: cost.costUsd,
        costSource: cost.costSource
      };
    });
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

function hasOptionalCostValue(value: unknown) {
  return typeof value === "number" || typeof value === "undefined";
}

function hasOptionalCostSource(value: unknown) {
  return typeof value === "undefined" || isCostSource(value);
}

function isCostSource(value: unknown): value is CostSource {
  return (
    value === "reported" ||
    value === "estimated" ||
    value === "mixed" ||
    value === "unknown"
  );
}

function normalizeOptionalCost(value: unknown) {
  return typeof value === "number" ? value : undefined;
}

function normalizeCostSource(
  value: unknown,
  costUsd: number | undefined
): CostSource {
  if (isCostSource(value)) {
    return value;
  }

  return typeof costUsd === "number" ? "reported" : "unknown";
}

function normalizeSessionUsageRecord(usage: SessionUsageRecord): SessionUsageRecord {
  const costUsd = normalizeOptionalCost(usage.costUsd);
  return {
    ...usage,
    costUsd,
    costSource: normalizeCostSource(usage.costSource, costUsd)
  };
}

function normalizeUsageTotalsRecord(usage: UsageTotalsRecord): UsageTotalsRecord {
  const costUsd = normalizeOptionalCost(usage.costUsd);
  return {
    ...usage,
    costUsd,
    costSource: normalizeCostSource(usage.costSource, costUsd)
  };
}

function normalizeAssistantUsageRecord(
  usage: AssistantUsageRecord
): AssistantUsageRecord {
  const costUsd = normalizeOptionalCost(usage.costUsd);
  return {
    ...usage,
    costUsd,
    costSource: normalizeCostSource(usage.costSource, costUsd)
  };
}

function normalizeUsageTimelineRecord(
  timeline: UsageTimelineRecord
): UsageTimelineRecord {
  const costUsd = normalizeOptionalCost(timeline.costUsd);
  return {
    ...timeline,
    costUsd,
    costSource: normalizeCostSource(timeline.costSource, costUsd)
  };
}

function createUsageAggregateState(): UsageAggregateState {
  return {
    sessionCount: 0,
    inputTokens: 0,
    outputTokens: 0,
    cacheReadTokens: 0,
    cacheWriteTokens: 0,
    reasoningTokens: 0,
    totalTokens: 0,
    costUsd: 0,
    hasKnownCost: false,
    hasUnknownCost: false,
    hasReported: false,
    hasEstimated: false
  };
}

function accumulateUsageAggregate(
  state: UsageAggregateState,
  usage: SessionUsageRecord
) {
  state.sessionCount += 1;
  state.inputTokens += usage.inputTokens;
  state.outputTokens += usage.outputTokens;
  state.cacheReadTokens += usage.cacheReadTokens;
  state.cacheWriteTokens += usage.cacheWriteTokens;
  state.reasoningTokens += usage.reasoningTokens;
  state.totalTokens += usage.totalTokens;

  if (typeof usage.costUsd === "number") {
    state.hasKnownCost = true;
    state.costUsd = roundCost(state.costUsd + usage.costUsd);
  } else {
    state.hasUnknownCost = true;
  }

  switch (normalizeCostSource(usage.costSource, usage.costUsd)) {
    case "reported":
      state.hasReported = true;
      break;
    case "estimated":
      state.hasEstimated = true;
      break;
    case "mixed":
      state.hasReported = true;
      state.hasEstimated = true;
      break;
    default:
      state.hasUnknownCost = true;
      break;
  }
}

function resolveAggregateCost(state: UsageAggregateState) {
  if (state.hasUnknownCost || !state.hasKnownCost) {
    return {
      costUsd: undefined,
      costSource: "unknown" as const
    };
  }

  if (state.hasReported && state.hasEstimated) {
    return {
      costUsd: roundCost(state.costUsd),
      costSource: "mixed" as const
    };
  }

  if (state.hasEstimated) {
    return {
      costUsd: roundCost(state.costUsd),
      costSource: "estimated" as const
    };
  }

  return {
    costUsd: roundCost(state.costUsd),
    costSource: "reported" as const
  };
}

function normalizeUsageDate(value: string) {
  const timestamp = parseActivityTimestamp(value);
  if (timestamp <= 0) {
    return undefined;
  }

  return new Date(timestamp).toISOString().slice(0, 10);
}

function buildEmptyDashboardSnapshot(): DashboardSnapshot {
  return normalizeDashboardSnapshot({
    metrics: [],
    sessions: [],
    configs: [],
    gitProjects: [],
    doctorFindings: [],
    auditEvents: [],
    usageOverview: EMPTY_USAGE_OVERVIEW,
    usageTimeline: EMPTY_USAGE_TIMELINE,
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

function buildConfigBackupManifestPath(
  runtime: DashboardRuntime,
  artifactId: string
) {
  const backupRoot = buildConfigBackupRoot(runtime);
  if (!backupRoot) {
    return undefined;
  }

  return `${trimTrailingSlashes(backupRoot)}/${safeManagedName(artifactId)}/manifest.json`;
}

function buildConfigBackupRoot(runtime: DashboardRuntime) {
  const normalizedAuditDbPath = runtime.auditDbPath.trim();
  if (!normalizedAuditDbPath) {
    return "";
  }

  const normalized = normalizedAuditDbPath.replace(/\\/g, "/");
  const segments = normalized.split("/").filter(Boolean);
  if (segments.length < 3) {
    return "";
  }

  const root = segments.slice(0, -2).join("/");
  const drivePrefix = normalized.startsWith("/") ? "/" : "";
  return `${drivePrefix}${root}/config-backups`;
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

function normalizeOptionalText(value: string | undefined) {
  const normalized = value?.trim();
  return normalized ? normalized : undefined;
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

function maskSecret(value: string) {
  const normalized = value.trim();
  if (normalized.length <= 4) {
    return "***";
  }

  return `***${normalized.slice(-4)}`;
}

function createDemoGitSha(timestamp: string, summary: string) {
  const seed = `${timestamp}:${summary}`;
  let hash = 0;
  for (const character of seed) {
    hash = (hash * 31 + character.charCodeAt(0)) >>> 0;
  }

  return hash.toString(16).padStart(7, "0").slice(0, 7);
}

function reconcileConfigRisks(
  currentRisks: string[],
  provider: string,
  baseUrl: string
) {
  const nextRisks = currentRisks.filter(
    (risk) => risk !== "third_party_provider" && risk !== "third_party_base_url"
  );

  if (!isOfficialProvider(provider)) {
    nextRisks.push("third_party_provider");
  }

  if (!isOfficialBaseUrl(provider, baseUrl)) {
    nextRisks.push("third_party_base_url");
  }

  return [...new Set(nextRisks)];
}

function inferProxyModeFromEndpoint(provider: string, baseUrl: string) {
  return isOfficialProvider(provider) && isOfficialBaseUrl(provider, baseUrl)
    ? "Official"
    : "Proxy";
}

function isOfficialProvider(provider: string) {
  return ["openai", "anthropic", "opencode", "google", "github"].includes(
    provider.trim().toLowerCase()
  );
}

function isOfficialBaseUrl(provider: string, baseUrl: string) {
  const host = extractHost(baseUrl);
  if (!host) {
    return false;
  }

  switch (provider.trim().toLowerCase()) {
    case "openai":
      return host.endsWith("openai.com");
    case "anthropic":
      return host.endsWith("anthropic.com");
    case "google":
      return host.endsWith("googleapis.com") || host.endsWith("google.com");
    case "github":
      return host.endsWith("github.com");
    case "opencode":
      return host.endsWith("opencode.ai");
    case "openrouter":
      return host.endsWith("openrouter.ai");
    default:
      return false;
  }
}

function extractHost(baseUrl: string) {
  const withoutScheme = baseUrl.split("://")[1] ?? baseUrl;
  const hostPort = withoutScheme.split(/[/?#]/)[0] ?? withoutScheme;
  const host = hostPort.split(":")[0]?.trim();

  return host || null;
}
