import {
  createContext,
  useContext,
  type Dispatch,
  type PropsWithChildren,
  type SetStateAction
} from "react";

export const LANGUAGE_STORAGE_KEY = "open-session-manager.language";

export type Language = "en" | "zh-CN";

type NavigatorLike = {
  language?: string;
  languages?: readonly string[];
};

type Messages = {
  app: {
    loadingKicker: string;
    loadingTitle: string;
    loadingBody: string;
  };
  root: {
    eyebrow: string;
    title: string;
    description: string;
    navLabel: string;
    languageLabel: string;
    themeLabel: string;
    nav: {
      overview: string;
      sessions: string;
      configs: string;
      audit: string;
    };
    languageNames: Record<Language, string>;
    themeNames: Record<"system" | "light" | "dark", string>;
  };
  overview: {
    adoptionKicker: string;
    adoptionTitle: string;
    adoptionDescription: string;
    doctorKicker: string;
    doctorTitle: string;
    doctorDescription: string;
    doctorEmpty: string;
    adoptedTitle: string;
    adoptedBadge: string;
    researchTitle: string;
    researchBadge: string;
    usageKicker: string;
    usageTitle: string;
    usageDescription: string;
    usageTotalsTitle: string;
    usageAssistantsTitle: string;
    usageTimelineTitle: string;
    usageTimelineEmpty: string;
    costUnavailable: string;
    costSources: {
      reported: string;
      estimated: string;
      mixed: string;
      unknown: string;
    };
    usageFields: {
      sessionsWithUsage: string;
      totalTokens: string;
      totalCost: string;
      cacheRead: string;
      assistant: string;
      sessionCount: string;
    };
  };
  sessions: {
    searchLabel: string;
    searchPlaceholder: string;
    searchSummary: string;
    searchSummaryEmpty: string;
    matchReasonLabels: Record<string, string>;
  };
  sessionTable: {
    kicker: string;
    title: string;
    description: string;
    emptyTitle: string;
    emptyBody: string;
    columns: {
      session: string;
      assistant: string;
      progress: string;
      value: string;
      lastActivity: string;
    };
  };
  sessionDetail: {
    kicker: string;
    emptyTitle: string;
    emptyBody: string;
    cleanupRequirement: string;
    controlUnavailable: string;
    continuePlaceholder: string;
    exportPathLabel: string;
    actions: {
      exportMarkdown: string;
      moveToQuarantine: string;
      resumeSession: string;
      continueSession: string;
    };
    sections: {
      sessionControl: string;
      context: string;
      signals: string;
      usage: string;
      transcriptHighlights: string;
      todoSnapshot: string;
      keyArtifacts: string;
      riskFlags: string;
      topicLabels: string;
    };
    fields: {
      controller: string;
      command: string;
      controlStatus: string;
      continuePrompt: string;
      lastPrompt: string;
      lastResponse: string;
      lastError: string;
      lastResumeAt: string;
      lastContinueAt: string;
      assistant: string;
      environment: string;
      project: string;
      source: string;
      progress: string;
      completion: string;
      valueScore: string;
      lastActive: string;
      model: string;
      inputTokens: string;
      outputTokens: string;
      cacheReadTokens: string;
      cacheWriteTokens: string;
      reasoningTokens: string;
      totalTokens: string;
      costUsd: string;
    };
    statuses: {
      attached: string;
      detached: string;
    };
    noRiskFlags: string;
    noTranscriptHighlights: string;
    noTodoItems: string;
    noSessionControl: string;
  };
  configRisk: {
    kicker: string;
    title: string;
    description: string;
    actions: {
      editConfig: string;
      saveConfig: string;
      cancelEdit: string;
    };
    fields: {
      scope: string;
      provider: string;
      model: string;
      endpoint: string;
      maskedKey: string;
      newKey: string;
    };
  };
  runtimePanel: {
    kicker: string;
    title: string;
    description: string;
    exportRootLabel: string;
    defaultExportRootHint: string;
    customExportRootHint: string;
    actions: {
      saveExportRoot: string;
      resetExportRoot: string;
    };
    fields: {
      exportRoot: string;
      auditDb: string;
      quarantineRoot: string;
      preferencesFile: string;
    };
  };
  audit: {
    kicker: string;
    title: string;
    description: string;
  };
  data: {
    unknownValue: string;
    metricLabels: Record<string, string>;
    metricNotes: Record<string, string>;
    progressStates: Record<string, string>;
    scopes: Record<string, string>;
    proxyModes: Record<string, string>;
    riskFlags: Record<string, string>;
    auditTypes: Record<string, string>;
    auditResults: Record<string, string>;
  };
};

type I18nContextValue = {
  language: Language;
  setLanguage: Dispatch<SetStateAction<Language>>;
  copy: Messages;
  translateMetricLabel: (value: string) => string;
  translateMetricNote: (value: string) => string;
  translateProgressState: (value: string) => string;
  translateScope: (value: string) => string;
  translateProxyMode: (value: string) => string;
  translateRiskFlag: (value: string) => string;
  translateAuditType: (value: string) => string;
  translateAuditResult: (value: string) => string;
};

const messages: Record<Language, Messages> = {
  en: {
    app: {
      loadingKicker: "Loading",
      loadingTitle: "Preparing governance snapshot",
      loadingBody:
        "Collecting sessions, config risks, and cleanup recommendations."
    },
    root: {
      eyebrow: "Bootstrap",
      title: "open Session Manager",
      description:
        "Inspect local coding-agent sessions, configs, and cleanup actions before you archive or delete anything.",
      navLabel: "Primary",
      languageLabel: "Language",
      themeLabel: "Theme",
      nav: {
        overview: "Overview",
        sessions: "Sessions",
        configs: "Configs",
        audit: "Audit"
      },
      languageNames: {
        en: "English",
        "zh-CN": "中文"
      },
      themeNames: {
        system: "System",
        light: "Light",
        dark: "Dark"
      }
    },
    overview: {
      adoptionKicker: "Upstream Intake",
      adoptionTitle: "What OSM already absorbed",
      adoptionDescription:
        "These references are visible in the product now, not only in research notes.",
      doctorKicker: "Doctor",
      doctorTitle: "Environment doctor",
      doctorDescription:
        "Recoverable discovery problems are surfaced here so they can be fixed without relying on noisy terminal output.",
      doctorEmpty: "No recoverable discovery problems were detected in the current snapshot.",
      adoptedTitle: "Adopted",
      adoptedBadge: "Landed in product",
      researchTitle: "Researched next",
      researchBadge: "Tracked for follow-up",
      usageKicker: "Usage Analytics",
      usageTitle: "Usage analytics",
      usageDescription:
        "Token and cost signals are now extracted from supported local session formats instead of living only in research notes.",
      usageTotalsTitle: "Totals",
      usageAssistantsTitle: "By assistant",
      usageTimelineTitle: "Daily timeline",
      usageTimelineEmpty: "No usage timeline is available for the current snapshot.",
      costUnavailable: "Cost unavailable",
      costSources: {
        reported: "Reported by session log",
        estimated: "Estimated from local price catalog",
        mixed: "Mixed reported and estimated cost",
        unknown: "Cost source unavailable"
      },
      usageFields: {
        sessionsWithUsage: "Sessions with usage",
        totalTokens: "Total tokens",
        totalCost: "Total cost",
        cacheRead: "Cache read",
        assistant: "Assistant",
        sessionCount: "Sessions"
      }
    },
    sessions: {
      searchLabel: "Search sessions",
      searchPlaceholder: "topic, project, assistant, risk",
      searchSummary: "ranked local matches",
      searchSummaryEmpty: "Type to search across titles, summaries, transcript highlights, and todos.",
      matchReasonLabels: {
        title: "Title",
        assistant: "Assistant",
        environment: "Environment",
        summary: "Summary",
        project: "Project",
        source: "Source",
        tag: "Tag",
        risk: "Risk",
        artifact: "Artifact",
        transcript: "Transcript",
        todo: "To-do"
      }
    },
    sessionTable: {
      kicker: "Session Explorer",
      title: "Retention-first queue",
      description:
        "Review title quality, progress, and recency before exporting or deleting anything.",
      emptyTitle: "No sessions match this filter",
      emptyBody:
        "Adjust the search terms or clear the filter to recover the workspace queue.",
      columns: {
        session: "Session",
        assistant: "Assistant",
        progress: "Progress",
        value: "Value",
        lastActivity: "Last Activity"
      }
    },
    sessionDetail: {
      kicker: "Session Detail",
      emptyTitle: "Select a session",
      emptyBody:
        "Choose a row to inspect summary, evidence, and cleanup readiness.",
      cleanupRequirement:
        "Export Markdown first so the session can be reviewed before moving it into quarantine.",
      controlUnavailable:
        "Session control is only available when the matching assistant command is installed and reachable from the local runtime.",
      continuePlaceholder: "Send a follow-up prompt back into this session",
      exportPathLabel: "Markdown saved to",
      actions: {
        exportMarkdown: "Export Markdown",
        moveToQuarantine: "Move to Quarantine",
        resumeSession: "Resume Session",
        continueSession: "Continue Session"
      },
      sections: {
        sessionControl: "Session Control",
        context: "Context",
        signals: "Signals",
        usage: "Usage",
        transcriptHighlights: "Transcript Highlights",
        todoSnapshot: "Todo Snapshot",
        keyArtifacts: "Key Artifacts",
        riskFlags: "Risk Flags",
        topicLabels: "Topic Labels"
      },
      fields: {
        controller: "Controller",
        command: "Command",
        controlStatus: "Control status",
        continuePrompt: "Continue prompt",
        lastPrompt: "Last prompt",
        lastResponse: "Last response",
        lastError: "Last error",
        lastResumeAt: "Last resume",
        lastContinueAt: "Last continue",
        assistant: "Assistant",
        environment: "Environment",
        project: "Project",
        source: "Source",
        progress: "Progress",
        completion: "Completion",
        valueScore: "Value score",
        lastActive: "Last active",
        model: "Model",
        inputTokens: "Input tokens",
        outputTokens: "Output tokens",
        cacheReadTokens: "Cache read",
        cacheWriteTokens: "Cache write",
        reasoningTokens: "Reasoning",
        totalTokens: "Total tokens",
        costUsd: "Cost (USD)"
      },
      statuses: {
        attached: "Attached",
        detached: "Detached"
      },
      noRiskFlags: "no active risk flags",
      noTranscriptHighlights: "No transcript highlights were extracted for this session.",
      noTodoItems: "No todo evidence was captured for this session.",
      noSessionControl: "No session control adapter is available for this assistant."
    },
    configRisk: {
      kicker: "Config Center",
      title: "Config Risk Center",
      description:
        "Secrets stay masked by default while endpoints, providers, and risk posture remain visible.",
      actions: {
        editConfig: "Edit Config",
        saveConfig: "Save Config",
        cancelEdit: "Cancel"
      },
      fields: {
        scope: "Scope",
        provider: "Provider",
        model: "Model",
        endpoint: "Endpoint",
        maskedKey: "Masked Key",
        newKey: "New Key"
      }
    },
    runtimePanel: {
      kicker: "Storage Paths",
      title: "Export and retention settings",
      description:
        "Set the Markdown export folder, then verify where audits and quarantine data are stored.",
      exportRootLabel: "Markdown export folder",
      defaultExportRootHint: "Using the default export folder.",
      customExportRootHint: "Using a custom export folder override.",
      actions: {
        saveExportRoot: "Save Export Folder",
        resetExportRoot: "Use Default Folder"
      },
      fields: {
        exportRoot: "Current export folder",
        auditDb: "Audit database",
        quarantineRoot: "Quarantine root",
        preferencesFile: "Preferences file"
      }
    },
    audit: {
      kicker: "Audit Center",
      title: "Trace every destructive operation",
      description:
        "Export, quarantine, and restore actions stay attached to an actor, timestamp, and target."
    },
    data: {
      unknownValue: "Unknown",
      metricLabels: {
        indexed_sessions: "Indexed Sessions",
        high_value_candidates: "High-Value Candidates",
        risky_configs: "Risky Configs",
        cold_storage_saved: "Cold Storage Saved"
      },
      metricNotes: {
        across_windows_linux_and_wsl_surfaces:
          "Across Windows, Linux, and WSL surfaces",
        worth_exporting_before_cleanup: "Worth exporting before cleanup",
        relay_wide_permissions_or_shell_hooks:
          "Relay, wide permissions, or shell hooks",
        potential_reclaim_from_soft_delete_queue:
          "Potential reclaim from soft-delete queue"
      },
      progressStates: {
        new: "New",
        in_progress: "In Progress",
        blocked: "Blocked",
        completed: "Completed"
      },
      scopes: {
        global: "Global",
        project: "Project"
      },
      proxyModes: {
        proxy: "Proxy",
        official: "Official"
      },
      riskFlags: {
        stale_followup_needed: "stale_followup_needed",
        blocked_session: "blocked_session",
        error_detected: "error_detected",
        stale_session: "stale_session",
        likely_garbage: "likely_garbage",
        dangerous_permissions: "dangerous_permissions",
        shell_hook: "shell_hook",
        third_party_base_url: "third_party_base_url",
        dangerous_sandbox: "dangerous_sandbox",
        dangerous_approval_policy: "dangerous_approval_policy",
        third_party_provider: "third_party_provider",
        missing_primary_secret: "missing_primary_secret"
      },
      auditTypes: {
        export_markdown: "export_markdown",
        soft_delete: "soft_delete",
        restore: "restore",
        session_resume: "session_resume",
        session_continue: "session_continue"
      },
      auditResults: {
        success: "success",
        failed: "failed"
      }
    }
  },
  "zh-CN": {
    app: {
      loadingKicker: "加载中",
      loadingTitle: "正在准备治理快照",
      loadingBody: "正在汇总会话、配置风险和清理建议。"
    },
    root: {
      eyebrow: "本地优先",
      title: "开放会话管理器",
      description:
        "在归档、迁移或删除之前，先把本地终端编程助手会话和配置看清楚。",
      navLabel: "主导航",
      languageLabel: "语言",
      themeLabel: "主题",
      nav: {
        overview: "总览",
        sessions: "会话",
        configs: "配置",
        audit: "审计"
      },
      languageNames: {
        en: "English",
        "zh-CN": "中文"
      },
      themeNames: {
        system: "跟随系统",
        light: "浅色",
        dark: "深色"
      }
    },
    overview: {
      adoptionKicker: "上游吸收",
      adoptionTitle: "已经落到产品里的能力",
      adoptionDescription: "这些能力已经在界面和导出链路里可见，不只是文档记录。",
      doctorKicker: "环境诊断",
      doctorTitle: "环境诊断",
      doctorDescription: "可恢复的发现问题会在这里展示，不再依赖终端噪声提醒用户。",
      doctorEmpty: "当前快照里没有发现可恢复的发现问题。",
      adoptedTitle: "已吸收",
      adoptedBadge: "已落地",
      researchTitle: "下一批研究对象",
      researchBadge: "已建档",
      usageKicker: "用量分析",
      usageTitle: "Usage analytics",
      usageDescription:
        "支持的本地会话格式现在会直接提取 token 和成本信号，不再只停留在研究文档里。",
      usageTotalsTitle: "总体汇总",
      usageAssistantsTitle: "按助手汇总",
      usageTimelineTitle: "每日趋势",
      usageTimelineEmpty: "当前快照里还没有可展示的用量趋势。",
      costUnavailable: "成本不可用",
      costSources: {
        reported: "来自会话日志上报",
        estimated: "来自本地价格目录估算",
        mixed: "混合了上报值与估算值",
        unknown: "成本来源不可用"
      },
      usageFields: {
        sessionsWithUsage: "含用量数据的会话",
        totalTokens: "总 Token",
        totalCost: "总成本",
        cacheRead: "缓存读取",
        assistant: "助手",
        sessionCount: "会话数"
      }
    },
    sessions: {
      searchLabel: "搜索会话",
      searchPlaceholder: "主题、项目、助手、风险",
      searchSummary: "条排序后的本地命中结果",
      searchSummaryEmpty: "输入关键词后，可在标题、摘要、高亮和待办中检索。",
      matchReasonLabels: {
        title: "标题",
        assistant: "助手",
        environment: "环境",
        summary: "摘要",
        project: "项目",
        source: "来源",
        tag: "标签",
        risk: "风险",
        artifact: "产物",
        transcript: "高亮",
        todo: "待办"
      }
    },
    sessionTable: {
      kicker: "会话浏览",
      title: "保留优先队列",
      description: "先判断标题质量、进度和最近活跃度，再决定导出或删除。",
      emptyTitle: "当前筛选条件下没有会话",
      emptyBody: "调整搜索词，或清空筛选条件后再继续处理会话。",
      columns: {
        session: "会话",
        assistant: "助手",
        progress: "进度",
        value: "价值",
        lastActivity: "最后活动"
      }
    },
    sessionDetail: {
      kicker: "会话详情",
      emptyTitle: "请选择一个会话",
      emptyBody: "选择左侧条目后，可查看摘要、证据和清理准备情况。",
      cleanupRequirement: "必须先导出 Markdown，确认核心内容已保留后才能移入隔离区。",
      controlUnavailable:
        "只有本机已安装且当前运行时能找到对应助手命令时，才允许执行会话恢复与继续运行。",
      continuePlaceholder: "向当前会话继续发送一条跟进提示",
      exportPathLabel: "Markdown 已保存到",
      actions: {
        exportMarkdown: "导出为 Markdown",
        moveToQuarantine: "移入隔离区",
        resumeSession: "恢复会话",
        continueSession: "继续运行"
      },
      sections: {
        sessionControl: "会话控制",
        context: "上下文",
        signals: "信号",
        usage: "Usage",
        transcriptHighlights: "会话高亮",
        todoSnapshot: "待办快照",
        keyArtifacts: "关键产物",
        riskFlags: "风险标记",
        topicLabels: "主题标签"
      },
      fields: {
        controller: "控制器",
        command: "命令",
        controlStatus: "控制状态",
        continuePrompt: "继续提示",
        lastPrompt: "最近提示",
        lastResponse: "最近响应",
        lastError: "最近错误",
        lastResumeAt: "最近恢复",
        lastContinueAt: "最近继续运行",
        assistant: "助手",
        environment: "环境",
        project: "项目",
        source: "来源",
        progress: "进度",
        completion: "完成度",
        valueScore: "价值分",
        lastActive: "最后活跃",
        model: "模型",
        inputTokens: "输入 Token",
        outputTokens: "输出 Token",
        cacheReadTokens: "缓存读取",
        cacheWriteTokens: "缓存写入",
        reasoningTokens: "推理 Token",
        totalTokens: "总 Token",
        costUsd: "成本（USD）"
      },
      statuses: {
        attached: "已附着",
        detached: "未附着"
      },
      noRiskFlags: "当前没有风险标记",
      noTranscriptHighlights: "当前没有提取到可展示的会话高亮。",
      noTodoItems: "当前没有捕获到待办证据。",
      noSessionControl: "当前助手还没有接入会话控制适配器。"
    },
    configRisk: {
      kicker: "配置中心",
      title: "配置风险中心",
      description: "默认隐藏密钥明文，同时保留端点、提供商和风险态势可见。",
      actions: {
        editConfig: "编辑配置",
        saveConfig: "保存配置",
        cancelEdit: "取消"
      },
      fields: {
        scope: "范围",
        provider: "提供商",
        model: "模型",
        endpoint: "端点",
        maskedKey: "脱敏密钥",
        newKey: "新密钥"
      }
    },
    runtimePanel: {
      kicker: "存储路径",
      title: "导出与保留设置",
      description: "可以直接修改 Markdown 导出目录，并确认审计库和隔离区当前落在哪里。",
      exportRootLabel: "Markdown 导出目录",
      defaultExportRootHint: "当前使用默认导出目录。",
      customExportRootHint: "当前使用自定义导出目录。",
      actions: {
        saveExportRoot: "保存导出目录",
        resetExportRoot: "恢复默认目录"
      },
      fields: {
        exportRoot: "当前导出目录",
        auditDb: "审计数据库",
        quarantineRoot: "隔离区目录",
        preferencesFile: "偏好设置文件"
      }
    },
    audit: {
      kicker: "审计中心",
      title: "追踪每一次破坏性操作",
      description: "导出、隔离和恢复动作都绑定到操作者、时间戳和目标。"
    },
    data: {
      unknownValue: "未知",
      metricLabels: {
        indexed_sessions: "已索引会话",
        high_value_candidates: "高价值候选",
        risky_configs: "高风险配置",
        cold_storage_saved: "可回收冷存储"
      },
      metricNotes: {
        across_windows_linux_and_wsl_surfaces:
          "覆盖 Windows、Linux 与 WSL 环境",
        worth_exporting_before_cleanup: "建议在清理前优先导出",
        relay_wide_permissions_or_shell_hooks:
          "存在中转地址、宽权限或 shell hook",
        potential_reclaim_from_soft_delete_queue:
          "软删除队列可释放的潜在空间"
      },
      progressStates: {
        new: "新建",
        in_progress: "进行中",
        blocked: "阻塞",
        completed: "已完成"
      },
      scopes: {
        global: "全局",
        project: "项目"
      },
      proxyModes: {
        proxy: "中转",
        official: "官方"
      },
      riskFlags: {
        stale_followup_needed: "需要后续跟进",
        blocked_session: "会话已阻塞",
        error_detected: "检测到错误",
        stale_session: "会话已过期",
        likely_garbage: "疑似垃圾会话",
        dangerous_permissions: "高危权限",
        shell_hook: "Shell Hook",
        third_party_base_url: "第三方 Base URL",
        dangerous_sandbox: "高危沙箱设置",
        dangerous_approval_policy: "高危审批策略",
        third_party_provider: "第三方提供商",
        missing_primary_secret: "缺少主凭据"
      },
      auditTypes: {
        export_markdown: "导出 Markdown",
        soft_delete: "软删除",
        restore: "恢复",
        session_resume: "恢复会话",
        session_continue: "继续运行"
      },
      auditResults: {
        success: "成功",
        failed: "失败"
      }
    }
  }
};

export function detectLanguage(navigatorLike?: NavigatorLike): Language {
  const candidates = [
    ...(navigatorLike?.languages ?? []),
    navigatorLike?.language ?? ""
  ];

  for (const candidate of candidates) {
    const coerced = coerceLanguage(candidate);
    if (coerced) {
      return coerced;
    }
  }

  return "en";
}

export function getInitialLanguage(): Language {
  if (typeof window === "undefined") {
    return "en";
  }

  try {
    const stored = coerceLanguage(
      window.localStorage.getItem(LANGUAGE_STORAGE_KEY)
    );
    if (stored) {
      return stored;
    }
  } catch {
    return detectLanguage(window.navigator);
  }

  return detectLanguage(window.navigator);
}

export function getMessages(language: Language): Messages {
  return messages[language];
}

export function I18nProvider({
  children,
  language,
  setLanguage
}: PropsWithChildren<{
  language: Language;
  setLanguage: Dispatch<SetStateAction<Language>>;
}>) {
  return (
    <I18nContext.Provider value={createContextValue(language, setLanguage)}>
      {children}
    </I18nContext.Provider>
  );
}

export function useI18n() {
  return useContext(I18nContext);
}

function coerceLanguage(value?: string | null): Language | null {
  const normalized = value?.trim().toLowerCase();

  if (!normalized) {
    return null;
  }

  if (normalized.startsWith("zh")) {
    return "zh-CN";
  }

  if (normalized.startsWith("en")) {
    return "en";
  }

  return null;
}

function createContextValue(
  language: Language,
  setLanguage: Dispatch<SetStateAction<Language>>
): I18nContextValue {
  const copy = getMessages(language);

  return {
    language,
    setLanguage,
    copy,
    translateMetricLabel: (value) => translateLookup(copy.data.metricLabels, value),
    translateMetricNote: (value) => translateLookup(copy.data.metricNotes, value),
    translateProgressState: (value) =>
      translateLookup(copy.data.progressStates, value),
    translateScope: (value) => translateLookup(copy.data.scopes, value),
    translateProxyMode: (value) => translateLookup(copy.data.proxyModes, value),
    translateRiskFlag: (value) => translateLookup(copy.data.riskFlags, value),
    translateAuditType: (value) => translateLookup(copy.data.auditTypes, value),
    translateAuditResult: (value) =>
      translateLookup(copy.data.auditResults, value)
  };
}

function translateLookup(dictionary: Record<string, string>, value: string) {
  return dictionary[normalizeLookupKey(value)] ?? value;
}

function normalizeLookupKey(value: string) {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "");
}

const I18nContext = createContext<I18nContextValue>(
  createContextValue("en", () => undefined)
);
