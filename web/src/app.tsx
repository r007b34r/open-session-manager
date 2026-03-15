import { startTransition, useEffect, useState } from "react";

import {
  applyDashboardPreferences,
  applyMarkdownExport,
  applySoftDelete,
  fetchDashboardSnapshot,
  type DashboardSnapshot
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

export function App() {
  const [snapshot, setSnapshot] = useState<DashboardSnapshot | null>(null);
  const [language, setLanguage] = useState<Language>(() => getInitialLanguage());
  const [themePreference, setThemePreference] = useState<ThemePreference>(() =>
    getInitialThemePreference()
  );
  const [currentPath, setCurrentPath] = useState(getCurrentPath);
  const copy = getMessages(language);

  useEffect(() => {
    let cancelled = false;

    void fetchDashboardSnapshot().then((data) => {
      if (cancelled) {
        return;
      }

      startTransition(() => {
        setSnapshot(data);
      });
    });

    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    const handleHashChange = () => {
      startTransition(() => {
        setCurrentPath(getCurrentPath());
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

  const handleSelectSession = (sessionId: string) => {
    const nextPath = `/sessions/${encodeURIComponent(sessionId)}`;

    startTransition(() => {
      setCurrentPath(nextPath);
    });

    if (window.location.hash !== `#${nextPath}`) {
      window.location.hash = nextPath;
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
              handleSelectSession,
              handleSaveExportRoot,
              handleResetExportRoot,
              handleExportMarkdown,
              handleSoftDelete
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
  onSelectSession: (sessionId: string) => void,
  onSaveExportRoot: (exportRoot: string) => void,
  onResetExportRoot: () => void,
  onExportMarkdown: (sessionId: string) => void,
  onSoftDelete: (sessionId: string) => void
) {
  const exportedSessionIds = new Set(
    snapshot.auditEvents
      .filter((event) => event.type === "export_markdown" && event.result === "success")
      .map((event) => event.target)
  );
  const latestMarkdownExportPaths = buildLatestMarkdownExportPaths(snapshot);

  if (path === "/configs") {
    return <ConfigsRoute configs={snapshot.configs} />;
  }

  if (path === "/audit") {
    return <AuditRoute events={snapshot.auditEvents} />;
  }

  if (path === "/sessions" || path.startsWith("/sessions/")) {
    const selectedSessionId = getSelectedSessionId(path);

    return (
      <SessionsRoute
        exportedSessionIds={exportedSessionIds}
        latestMarkdownExportPaths={latestMarkdownExportPaths}
        onExportMarkdown={onExportMarkdown}
        onResetExportRoot={onResetExportRoot}
        onSaveExportRoot={onSaveExportRoot}
        onSelectSession={onSelectSession}
        onSoftDelete={onSoftDelete}
        runtime={snapshot.runtime}
        selectedSessionId={selectedSessionId}
        sessions={snapshot.sessions}
      />
    );
  }

  return (
    <>
      <OverviewRoute snapshot={snapshot} />
      <SessionsRoute
        exportedSessionIds={exportedSessionIds}
        latestMarkdownExportPaths={latestMarkdownExportPaths}
        onExportMarkdown={onExportMarkdown}
        onResetExportRoot={onResetExportRoot}
        onSaveExportRoot={onSaveExportRoot}
        onSelectSession={onSelectSession}
        onSoftDelete={onSoftDelete}
        runtime={snapshot.runtime}
        selectedSessionId={snapshot.sessions[0]?.sessionId}
        sessions={snapshot.sessions}
      />
      <ConfigsRoute configs={snapshot.configs} />
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
