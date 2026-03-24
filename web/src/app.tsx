import { startTransition, useEffect, useEffectEvent, useRef, useState } from "react";

import {
  applySessionAttach,
  applyConfigWriteback,
  applyDashboardPreferences,
  applyGitProjectBranchSwitch,
  applyGitProjectCommit,
  applyGitProjectPush,
  applyMarkdownExport,
  recordLocalAuditEvent,
  applySessionContinue,
  applySessionDetach,
  applySessionResume,
  applySoftDelete,
  fetchDashboardSnapshot,
  isConfigWritebackAvailable,
  previewGitProjectFile,
  type ConfigWritebackInput,
  type DashboardSnapshot,
  type GitProjectBranchSwitchInput,
  type GitProjectCommitInput,
  type GitProjectFilePreviewInput,
  type GitProjectFilePreviewRecord,
  type GitProjectPushInput,
  type LocalAuditEventInput
} from "./lib/api";
import {
  getInitialLanguage,
  getMessages,
  I18nProvider,
  LANGUAGE_STORAGE_KEY,
  type Language
} from "./lib/i18n";
import {
  getInitialThemePreference,
  resolveTheme,
  THEME_STORAGE_KEY,
  watchSystemTheme,
  type ThemePreference
} from "./lib/theme";
import { AuditRoute } from "./routes/audit";
import { ConfigsRoute } from "./routes/configs";
import { OverviewRoute } from "./routes/index";
import { RootShell } from "./routes/__root";
import { SessionsRoute } from "./routes/sessions";

const SNAPSHOT_AUTO_REFRESH_INTERVAL_MS = 15_000;

export function App() {
  const [snapshot, setSnapshot] = useState<DashboardSnapshot | null>(null);
  const [isRefreshingSnapshot, setIsRefreshingSnapshot] = useState(false);
  const [language, setLanguage] = useState<Language>(() => getInitialLanguage());
  const [themePreference, setThemePreference] = useState<ThemePreference>(() =>
    getInitialThemePreference()
  );
  const [currentPath, setCurrentPath] = useState(getCurrentPath);
  const [selectedSessionId, setSelectedSessionId] = useState<string | undefined>(() =>
    getSelectedSessionId(getCurrentPath())
  );
  const isMountedRef = useRef(true);
  const isSnapshotFetchInFlightRef = useRef(false);
  const copy = getMessages(language);

  const loadSnapshot = useEffectEvent(async (options: { refresh: boolean }) => {
    if (isSnapshotFetchInFlightRef.current) {
      return;
    }

    isSnapshotFetchInFlightRef.current = true;

    if (options.refresh) {
      startTransition(() => {
        setIsRefreshingSnapshot(true);
      });
    }

    try {
      const data = await fetchDashboardSnapshot();
      if (isMountedRef.current) {
        startTransition(() => {
          setSnapshot(data);
        });
      }
    } finally {
      isSnapshotFetchInFlightRef.current = false;
      if (isMountedRef.current && options.refresh) {
        startTransition(() => {
          setIsRefreshingSnapshot(false);
        });
      }
    }
  });

  useEffect(() => {
    return () => {
      isMountedRef.current = false;
    };
  }, []);

  useEffect(() => {
    void loadSnapshot({ refresh: false });

    const refreshIfVisible = () => {
      if (document.visibilityState === "hidden") {
        return;
      }

      void loadSnapshot({ refresh: false });
    };

    const intervalId = window.setInterval(
      refreshIfVisible,
      SNAPSHOT_AUTO_REFRESH_INTERVAL_MS
    );

    window.addEventListener("focus", refreshIfVisible);
    document.addEventListener("visibilitychange", refreshIfVisible);

    return () => {
      window.clearInterval(intervalId);
      window.removeEventListener("focus", refreshIfVisible);
      document.removeEventListener("visibilitychange", refreshIfVisible);
    };
  }, []);

  useEffect(() => {
    const handleHashChange = () => {
      const nextPath = getCurrentPath();
      const nextSelectedSessionId = getSelectedSessionId(nextPath);

      startTransition(() => {
        setCurrentPath(nextPath);
        setSelectedSessionId(nextSelectedSessionId);
      });
    };

    window.addEventListener("hashchange", handleHashChange);
    return () => window.removeEventListener("hashchange", handleHashChange);
  }, []);

  useEffect(() => {
    try {
      window.localStorage.setItem(LANGUAGE_STORAGE_KEY, language);
    } catch {
      // Ignore storage failures and keep the in-memory language choice.
    }

    document.documentElement.lang = language;
  }, [language]);

  useEffect(() => {
    const applyResolvedTheme = () => {
      const resolvedTheme = resolveTheme(themePreference);
      document.documentElement.dataset.theme = resolvedTheme;
      document.documentElement.style.colorScheme = resolvedTheme;
    };

    try {
      window.localStorage.setItem(THEME_STORAGE_KEY, themePreference);
    } catch {
      // Ignore storage failures and keep the in-memory theme choice.
    }

    applyResolvedTheme();

    if (themePreference !== "system") {
      return undefined;
    }

    return watchSystemTheme(applyResolvedTheme);
  }, [themePreference]);

  const handleExportMarkdown = (sessionId: string) => {
    if (!snapshot) {
      return;
    }

    void applyMarkdownExport(snapshot, sessionId).then((nextSnapshot) => {
      startTransition(() => {
        setSnapshot(nextSnapshot);
      });
    });
  };

  const handleSoftDelete = (sessionId: string) => {
    if (!snapshot) {
      return;
    }

    void applySoftDelete(snapshot, sessionId).then((nextSnapshot) => {
      startTransition(() => {
        setSnapshot(nextSnapshot);
      });
    });
  };

  const handleSessionResume = (sessionId: string) => {
    if (!snapshot) {
      return;
    }

    void applySessionResume(snapshot, sessionId).then((nextSnapshot) => {
      startTransition(() => {
        setSnapshot(nextSnapshot);
      });
    });
  };

  const handleSessionAttach = (sessionId: string) => {
    if (!snapshot) {
      return;
    }

    void applySessionAttach(snapshot, sessionId).then((nextSnapshot) => {
      startTransition(() => {
        setSnapshot(nextSnapshot);
      });
    });
  };

  const handleSessionContinue = (sessionId: string, prompt: string) => {
    if (!snapshot) {
      return;
    }

    void applySessionContinue(snapshot, { sessionId, prompt }).then((nextSnapshot) => {
      startTransition(() => {
        setSnapshot(nextSnapshot);
      });
    });
  };

  const handleSessionDetach = (sessionId: string) => {
    if (!snapshot) {
      return;
    }

    void applySessionDetach(snapshot, sessionId).then((nextSnapshot) => {
      startTransition(() => {
        setSnapshot(nextSnapshot);
      });
    });
  };

  const handleSaveExportRoot = (exportRoot: string) => {
    if (!snapshot) {
      return;
    }

    void applyDashboardPreferences(snapshot, { exportRoot }).then((nextSnapshot) => {
      startTransition(() => {
        setSnapshot(nextSnapshot);
      });
    });
  };

  const handleResetExportRoot = () => {
    if (!snapshot) {
      return;
    }

    void applyDashboardPreferences(snapshot, { exportRoot: null }).then(
      (nextSnapshot) => {
        startTransition(() => {
          setSnapshot(nextSnapshot);
        });
      }
    );
  };

  const handleSaveConfig = (input: ConfigWritebackInput) => {
    if (!snapshot) {
      return;
    }

    void applyConfigWriteback(snapshot, input).then((nextSnapshot) => {
      startTransition(() => {
        setSnapshot(nextSnapshot);
      });
    });
  };

  const handleAuditEvent = (input: LocalAuditEventInput) => {
    if (!snapshot) {
      return;
    }

    startTransition(() => {
      setSnapshot(recordLocalAuditEvent(snapshot, input));
    });
  };

  const handleGitProjectCommit = (input: GitProjectCommitInput) => {
    if (!snapshot) {
      return;
    }

    void applyGitProjectCommit(snapshot, input).then((nextSnapshot) => {
      startTransition(() => {
        setSnapshot(nextSnapshot);
      });
    });
  };

  const handleGitProjectBranchSwitch = (input: GitProjectBranchSwitchInput) => {
    if (!snapshot) {
      return;
    }

    void applyGitProjectBranchSwitch(snapshot, input).then((nextSnapshot) => {
      startTransition(() => {
        setSnapshot(nextSnapshot);
      });
    });
  };

  const handleGitProjectPush = (input: GitProjectPushInput) => {
    if (!snapshot) {
      return;
    }

    void applyGitProjectPush(snapshot, input).then((nextSnapshot) => {
      startTransition(() => {
        setSnapshot(nextSnapshot);
      });
    });
  };

  const handleGitProjectPreview = (
    input: GitProjectFilePreviewInput,
  ): Promise<GitProjectFilePreviewRecord> => {
    if (!snapshot) {
      return Promise.reject(new Error("Snapshot not loaded."));
    }

    return previewGitProjectFile(snapshot, input);
  };

  const handleRefreshSnapshot = () => {
    loadSnapshot({ refresh: true });
  };

  const handleSelectSession = (sessionId: string) => {
    startTransition(() => {
      setSelectedSessionId(sessionId);
    });

    if (!currentPath.startsWith("/sessions")) {
      return;
    }

    const nextPath = `/sessions/${encodeURIComponent(sessionId)}`;

    startTransition(() => {
      setCurrentPath(nextPath);
    });

    if (window.location.hash !== `#${nextPath}`) {
      window.history.replaceState(null, "", `#${nextPath}`);
    }
  };

  return (
    <I18nProvider language={language} setLanguage={setLanguage}>
      <RootShell
        currentPath={normalizePath(currentPath)}
        onThemeChange={setThemePreference}
        themePreference={themePreference}
      >
        <section className="route-shell">
          {snapshot ? (
            renderRoute(
              snapshot,
              normalizePath(currentPath),
              selectedSessionId,
              handleSelectSession,
              handleSaveExportRoot,
              handleResetExportRoot,
              handleExportMarkdown,
              handleSessionResume,
              handleSessionAttach,
              handleSessionContinue,
              handleSessionDetach,
              handleSoftDelete,
              handleSaveConfig,
              handleAuditEvent,
              handleGitProjectCommit,
              handleGitProjectBranchSwitch,
              handleGitProjectPreview,
              handleGitProjectPush,
              handleRefreshSnapshot,
              isRefreshingSnapshot
            )
          ) : (
            <section className="panel empty-state">
              <p className="section-kicker">{copy.app.loadingKicker}</p>
              <h2>{copy.app.loadingTitle}</h2>
              <p className="panel-copy">{copy.app.loadingBody}</p>
            </section>
          )}
        </section>
      </RootShell>
    </I18nProvider>
  );
}

function renderRoute(
  snapshot: DashboardSnapshot,
  path: string,
  selectedSessionId: string | undefined,
  onSelectSession: (sessionId: string) => void,
  onSaveExportRoot: (exportRoot: string) => void,
  onResetExportRoot: () => void,
  onExportMarkdown: (sessionId: string) => void,
  onResumeSession: (sessionId: string) => void,
  onAttachSession: (sessionId: string) => void,
  onContinueSession: (sessionId: string, prompt: string) => void,
  onDetachSession: (sessionId: string) => void,
  onSoftDelete: (sessionId: string) => void,
  onSaveConfig: (input: ConfigWritebackInput) => void,
  onAuditEvent: (input: LocalAuditEventInput) => void,
  onGitProjectCommit: (input: GitProjectCommitInput) => void,
  onGitProjectBranchSwitch: (input: GitProjectBranchSwitchInput) => void,
  onGitProjectPreview: (
    input: GitProjectFilePreviewInput,
  ) => Promise<GitProjectFilePreviewRecord>,
  onGitProjectPush: (input: GitProjectPushInput) => void,
  onRefreshSnapshot: () => void,
  isRefreshingSnapshot: boolean
) {
  const canEditConfigs = isConfigWritebackAvailable();
  const exportedSessionIds = new Set(
    snapshot.auditEvents
      .filter((event) => event.type === "export_markdown" && event.result === "success")
      .map((event) => event.target)
  );
  const latestMarkdownExportPaths = buildLatestMarkdownExportPaths(snapshot);

  if (path === "/configs") {
    return (
      <ConfigsRoute
        canEditConfigs={canEditConfigs}
        configs={snapshot.configs}
        onAuditEvent={onAuditEvent}
        onSaveConfig={onSaveConfig}
      />
    );
  }

  if (path === "/audit") {
    return <AuditRoute events={snapshot.auditEvents} />;
  }

  if (path === "/sessions" || path.startsWith("/sessions/")) {
    return (
      <SessionsRoute
        exportedSessionIds={exportedSessionIds}
        latestMarkdownExportPaths={latestMarkdownExportPaths}
        onAttachSession={onAttachSession}
        onExportMarkdown={onExportMarkdown}
        onResetExportRoot={onResetExportRoot}
        onSaveExportRoot={onSaveExportRoot}
        onSelectSession={onSelectSession}
        onContinueSession={onContinueSession}
        onDetachSession={onDetachSession}
        onSoftDelete={onSoftDelete}
        onResumeSession={onResumeSession}
        runtime={snapshot.runtime}
        selectedSessionId={selectedSessionId}
        sessions={snapshot.sessions}
      />
    );
  }

  return (
    <>
      <OverviewRoute
        isRefreshing={isRefreshingSnapshot}
        onGitProjectBranchSwitch={onGitProjectBranchSwitch}
        onGitProjectCommit={onGitProjectCommit}
        onGitProjectPreview={onGitProjectPreview}
        onGitProjectPush={onGitProjectPush}
        onRefreshSnapshot={onRefreshSnapshot}
        snapshot={snapshot}
      />
      <SessionsRoute
        exportedSessionIds={exportedSessionIds}
        latestMarkdownExportPaths={latestMarkdownExportPaths}
        onAttachSession={onAttachSession}
        onExportMarkdown={onExportMarkdown}
        onResetExportRoot={onResetExportRoot}
        onSaveExportRoot={onSaveExportRoot}
        onSelectSession={onSelectSession}
        onContinueSession={onContinueSession}
        onDetachSession={onDetachSession}
        onSoftDelete={onSoftDelete}
        onResumeSession={onResumeSession}
        runtime={snapshot.runtime}
        selectedSessionId={selectedSessionId ?? snapshot.sessions[0]?.sessionId}
        sessions={snapshot.sessions}
      />
      <ConfigsRoute
        canEditConfigs={canEditConfigs}
        configs={snapshot.configs}
        onAuditEvent={onAuditEvent}
        onSaveConfig={onSaveConfig}
      />
    </>
  );
}

function getCurrentPath() {
  return normalizePath(window.location.hash.replace(/^#/, ""));
}

function normalizePath(value: string) {
  return value || "/";
}

function getSelectedSessionId(path: string) {
  if (!path.startsWith("/sessions/")) {
    return undefined;
  }

  const encodedSessionId = path.slice("/sessions/".length);
  if (!encodedSessionId) {
    return undefined;
  }

  try {
    return decodeURIComponent(encodedSessionId);
  } catch {
    return encodedSessionId;
  }
}

function buildLatestMarkdownExportPaths(snapshot: DashboardSnapshot) {
  const paths = new Map<string, string>();

  for (const event of snapshot.auditEvents) {
    if (
      event.type === "export_markdown" &&
      event.result === "success" &&
      typeof event.outputPath === "string" &&
      !paths.has(event.target)
    ) {
      paths.set(event.target, event.outputPath);
    }
  }

  return paths;
}
