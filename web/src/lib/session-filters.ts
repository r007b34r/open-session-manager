import type { SessionDetailRecord } from "./api";

export type SessionFilterState = {
  assistant: string;
  project: string;
  risk: "all" | "at-risk" | "clean";
  export: "all" | "ready-to-quarantine" | "needs-export";
  control: "all" | "controllable" | "attached";
};

export type SessionFilterContext = {
  exportedSessionIds: ReadonlySet<string>;
};

export const DEFAULT_SESSION_FILTERS: SessionFilterState = {
  assistant: "all",
  project: "all",
  risk: "all",
  export: "all",
  control: "all"
};

export function applySessionFilters(
  sessions: SessionDetailRecord[],
  filters: SessionFilterState,
  context: SessionFilterContext
) {
  return sessions.filter((session) => matchesSessionFilters(session, filters, context));
}

export function hasActiveSessionFilters(filters: SessionFilterState) {
  return Object.entries(filters).some(([, value]) => value !== "all");
}

function matchesSessionFilters(
  session: SessionDetailRecord,
  filters: SessionFilterState,
  context: SessionFilterContext
) {
  if (filters.assistant !== "all" && session.assistant !== filters.assistant) {
    return false;
  }

  if (filters.project !== "all" && session.projectPath !== filters.project) {
    return false;
  }

  if (filters.risk === "at-risk" && session.riskFlags.length === 0) {
    return false;
  }

  if (filters.risk === "clean" && session.riskFlags.length > 0) {
    return false;
  }

  const isExported = context.exportedSessionIds.has(session.sessionId);
  if (filters.export === "ready-to-quarantine" && !isExported) {
    return false;
  }

  if (filters.export === "needs-export" && isExported) {
    return false;
  }

  const control = session.sessionControl;
  if (
    filters.control === "controllable" &&
    !(control?.supported && control.available)
  ) {
    return false;
  }

  if (filters.control === "attached" && !control?.attached) {
    return false;
  }

  return true;
}
