import { startTransition, useEffect, useState } from "react";

import {
  fetchDashboardSnapshot,
  recordMarkdownExport,
  recordSoftDelete,
  type DashboardSnapshot
} from "./lib/api";
import {
  getInitialLanguage,
  getMessages,
  I18nProvider,
  LANGUAGE_STORAGE_KEY,
  type Language
} from "./lib/i18n";
import { AuditRoute } from "./routes/audit";
import { ConfigsRoute } from "./routes/configs";
import { OverviewRoute } from "./routes/index";
import { RootShell } from "./routes/__root";
import { SessionDetailRoute } from "./routes/sessions.$id";
import { SessionsRoute } from "./routes/sessions";

export function App() {
  const [snapshot, setSnapshot] = useState<DashboardSnapshot | null>(null);
  const [language, setLanguage] = useState<Language>(() => getInitialLanguage());
  const [currentPath, setCurrentPath] = useState(getCurrentPath);
  const copy = getMessages(language);

  useEffect(() => {
    fetchDashboardSnapshot().then((data) => {
      startTransition(() => {
        setSnapshot(data);
      });
    });
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

  const handleExportMarkdown = (sessionId: string) => {
    startTransition(() => {
      setSnapshot((current) =>
        current ? recordMarkdownExport(current, sessionId) : current
      );
    });
  };

  const handleSoftDelete = (sessionId: string) => {
    startTransition(() => {
      setSnapshot((current) =>
        current ? recordSoftDelete(current, sessionId) : current
      );
    });
  };

  return (
    <I18nProvider language={language} setLanguage={setLanguage}>
      <RootShell currentPath={normalizePath(currentPath)}>
        <section className="route-shell">
          {snapshot ? (
            renderRoute(
              snapshot,
              normalizePath(currentPath),
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
  onExportMarkdown: (sessionId: string) => void,
  onSoftDelete: (sessionId: string) => void
) {
  if (path === "/configs") {
    return <ConfigsRoute configs={snapshot.configs} />;
  }

  if (path === "/audit") {
    return <AuditRoute events={snapshot.auditEvents} />;
  }

  if (path.startsWith("/sessions/")) {
    const selectedSessionId = path.replace("/sessions/", "");
    const selectedSession = snapshot.sessions.find(
      (session) => session.sessionId === selectedSessionId
    );

    return (
      <SessionDetailRoute
        onExportMarkdown={onExportMarkdown}
        onSoftDelete={onSoftDelete}
        session={selectedSession ?? snapshot.sessions[0]}
      />
    );
  }

  if (path === "/sessions") {
    return (
      <SessionsRoute
        onExportMarkdown={onExportMarkdown}
        onSoftDelete={onSoftDelete}
        sessions={snapshot.sessions}
      />
    );
  }

  return (
    <>
      <OverviewRoute snapshot={snapshot} />
      <SessionsRoute
        onExportMarkdown={onExportMarkdown}
        onSoftDelete={onSoftDelete}
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
