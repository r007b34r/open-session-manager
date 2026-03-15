import {
  createContext,
  useContext,
  type Dispatch,
  type PropsWithChildren,
  type SetStateAction
} from "react";

export const LANGUAGE_STORAGE_KEY = "agent-session-governance.language";

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
    nav: {
      overview: string;
      sessions: string;
      configs: string;
      audit: string;
    };
    languageNames: Record<Language, string>;
  };
  sessions: {
    searchLabel: string;
    searchPlaceholder: string;
  };
  sessionTable: {
    kicker: string;
    title: string;
    description: string;
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
    actions: {
      exportMarkdown: string;
      moveToQuarantine: string;
    };
    sections: {
      context: string;
      signals: string;
      keyArtifacts: string;
      riskFlags: string;
      topicLabels: string;
    };
    fields: {
      assistant: string;
      environment: string;
      project: string;
      source: string;
      progress: string;
      completion: string;
      valueScore: string;
      lastActive: string;
    };
    noRiskFlags: string;
  };
  configRisk: {
    kicker: string;
    title: string;
    description: string;
    fields: {
      scope: string;
      provider: string;
      endpoint: string;
      maskedKey: string;
    };
  };
  audit: {
    kicker: string;
    title: string;
    description: string;
  };
  data: {
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
      title: "Agent Session Governance",
      description:
        "Local-first control center for inspecting coding-agent sessions, configs, and cleanup actions before any destructive change is made.",
      navLabel: "Primary",
      languageLabel: "Language",
      nav: {
        overview: "Overview",
        sessions: "Sessions",
        configs: "Configs",
        audit: "Audit"
      },
      languageNames: {
        en: "English",
        "zh-CN": "中文"
      }
    },
    sessions: {
      searchLabel: "Search sessions",
      searchPlaceholder: "topic, project, assistant, risk"
    },
    sessionTable: {
      kicker: "Session Explorer",
      title: "Retention-first queue",
      description:
        "Review title quality, progress, and recency before exporting or deleting anything.",
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
      actions: {
        exportMarkdown: "Export Markdown",
        moveToQuarantine: "Move to Quarantine"
      },
      sections: {
        context: "Context",
        signals: "Signals",
        keyArtifacts: "Key Artifacts",
        riskFlags: "Risk Flags",
        topicLabels: "Topic Labels"
      },
      fields: {
        assistant: "Assistant",
        environment: "Environment",
        project: "Project",
        source: "Source",
        progress: "Progress",
        completion: "Completion",
        valueScore: "Value score",
        lastActive: "Last active"
      },
      noRiskFlags: "no active risk flags"
    },
    configRisk: {
      kicker: "Config Center",
      title: "Config Risk Center",
      description:
        "Secrets stay masked by default while endpoints, providers, and risk posture remain visible.",
      fields: {
        scope: "Scope",
        provider: "Provider",
        endpoint: "Endpoint",
        maskedKey: "Masked Key"
      }
    },
    audit: {
      kicker: "Audit Center",
      title: "Trace every destructive operation",
      description:
        "Export, quarantine, and restore actions stay attached to an actor, timestamp, and target."
    },
    data: {
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
        third_party_provider: "third_party_provider"
      },
      auditTypes: {
        export_markdown: "export_markdown",
        soft_delete: "soft_delete",
        restore: "restore"
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
      title: "会话治理平台",
      description:
        "在执行任何破坏性操作之前，本地优先检查终端编程助手的会话、配置和清理动作。",
      navLabel: "主导航",
      languageLabel: "语言",
      nav: {
        overview: "总览",
        sessions: "会话",
        configs: "配置",
        audit: "审计"
      },
      languageNames: {
        en: "English",
        "zh-CN": "中文"
      }
    },
    sessions: {
      searchLabel: "搜索会话",
      searchPlaceholder: "主题、项目、助手、风险"
    },
    sessionTable: {
      kicker: "会话浏览",
      title: "保留优先队列",
      description: "先判断标题质量、进度和最近活跃度，再决定导出或删除。",
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
      actions: {
        exportMarkdown: "导出为 Markdown",
        moveToQuarantine: "移入隔离区"
      },
      sections: {
        context: "上下文",
        signals: "信号",
        keyArtifacts: "关键产物",
        riskFlags: "风险标记",
        topicLabels: "主题标签"
      },
      fields: {
        assistant: "助手",
        environment: "环境",
        project: "项目",
        source: "来源",
        progress: "进度",
        completion: "完成度",
        valueScore: "价值分",
        lastActive: "最后活跃"
      },
      noRiskFlags: "当前没有风险标记"
    },
    configRisk: {
      kicker: "配置中心",
      title: "配置风险中心",
      description: "默认隐藏密钥明文，同时保留端点、提供商和风险态势可见。",
      fields: {
        scope: "范围",
        provider: "提供商",
        endpoint: "端点",
        maskedKey: "脱敏密钥"
      }
    },
    audit: {
      kicker: "审计中心",
      title: "追踪每一次破坏性操作",
      description: "导出、隔离和恢复动作都绑定到操作者、时间戳和目标。"
    },
    data: {
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
        third_party_provider: "第三方提供商"
      },
      auditTypes: {
        export_markdown: "导出 Markdown",
        soft_delete: "软删除",
        restore: "恢复"
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
