import { startTransition, useEffect, useState } from "react";

import {
  fetchDashboardSnapshot,
  recordMarkdownExport,
  recordSoftDelete,
  type DashboardSnapshot
} from "./lib/api";
import { AuditRoute } from "./routes/audit";
import { ConfigsRoute } from "./routes/configs";
import { OverviewRoute } from "./routes/index";
import { RootShell } from "./routes/__root";
import { SessionDetailRoute } from "./routes/sessions.$id";
import { SessionsRoute } from "./routes/sessions";

export function App() {
  const [snapshot, setSnapshot] = useState<DashboardSnapshot | null>(null);
  const [currentPath, setCurrentPath] = useState(getCurrentPath);

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
            <p className="section-kicker">Loading</p>
            <h2>Preparing governance snapshot</h2>
            <p className="panel-copy">
              Collecting sessions, config risks, and cleanup recommendations.
            </p>
          </section>
        )}
      </section>
    </RootShell>
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
