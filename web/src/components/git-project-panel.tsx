import { useState } from "react";

import type {
  AuditEventRecord,
  GitProjectBranchSwitchInput,
  GitProjectCommitInput,
  GitProjectPushInput,
  GitProjectRecord
} from "../lib/api";
import { useI18n } from "../lib/i18n";

type GitProjectPanelProps = {
  projects: GitProjectRecord[];
  auditEvents: AuditEventRecord[];
  onCommit: (input: GitProjectCommitInput) => void;
  onSwitchBranch: (input: GitProjectBranchSwitchInput) => void;
  onPush: (input: GitProjectPushInput) => void;
};

export function GitProjectPanel({
  projects,
  auditEvents,
  onCommit,
  onSwitchBranch,
  onPush
}: GitProjectPanelProps) {
  const { copy } = useI18n();
  const [commitDrafts, setCommitDrafts] = useState<Record<string, string>>({});
  const [branchDrafts, setBranchDrafts] = useState<Record<string, string>>({});
  const [historyFilters, setHistoryFilters] = useState<Record<string, string>>({});
  const [expandedCommits, setExpandedCommits] = useState<Record<string, string | undefined>>({});
  const latestGitEvents = buildLatestGitEventMap(auditEvents);

  return (
    <section className="panel">
      <div className="panel-header">
        <div>
          <p className="section-kicker">{copy.overview.git.kicker}</p>
          <h2>{copy.overview.git.title}</h2>
        </div>
        <p className="panel-copy">{copy.overview.git.description}</p>
      </div>

      {projects.length === 0 ? (
        <p className="panel-copy">{copy.overview.git.empty}</p>
      ) : (
        <div className="config-grid">
          {projects.map((project) => {
            const commitMessage = commitDrafts[project.repoRoot] ?? "";
            const branchDraft = branchDrafts[project.repoRoot] ?? "";
            const historyFilter = historyFilters[project.repoRoot] ?? "";
            const latestEvent = latestGitEvents.get(project.repoRoot);
            const branchSwitchBlocked = project.dirty;
            const pushDisabled = project.dirty || project.ahead === 0;
            const filteredCommits = project.recentCommits.filter((commit) =>
              matchesCommitFilter(commit, historyFilter)
            );

            return (
              <article className="config-card" key={project.repoRoot}>
                <div className="config-card-topline">
                  <strong>{project.repoRoot}</strong>
                  <div className="badge-row">
                    <span className={`badge ${project.dirty ? "badge-risk" : "badge-safe"}`}>
                      {translateGitStatus(copy.overview.git.statuses, project.status)}
                    </span>
                  </div>
                </div>

                <dl className="config-meta">
                  <div>
                    <dt>{copy.overview.git.fields.branch}</dt>
                    <dd>{project.branch}</dd>
                  </div>
                  <div>
                    <dt>{copy.overview.git.fields.sessions}</dt>
                    <dd>{project.sessionCount}</dd>
                  </div>
                  <div>
                    <dt>{copy.overview.git.fields.staged}</dt>
                    <dd>{project.stagedChanges}</dd>
                  </div>
                  <div>
                    <dt>{copy.overview.git.fields.unstaged}</dt>
                    <dd>{project.unstagedChanges}</dd>
                  </div>
                  <div>
                    <dt>{copy.overview.git.fields.untracked}</dt>
                    <dd>{project.untrackedFiles}</dd>
                  </div>
                  <div>
                    <dt>{copy.overview.git.fields.aheadBehind}</dt>
                    <dd>
                      {project.ahead}/{project.behind}
                    </dd>
                  </div>
                  {project.lastCommitSummary ? (
                    <div>
                      <dt>{copy.overview.git.fields.latestCommit}</dt>
                      <dd>{project.lastCommitSummary}</dd>
                    </div>
                  ) : null}
                </dl>

                {project.recentCommits.length > 0 ? (
                  <>
                    <strong>{copy.overview.git.fields.recentCommits}</strong>
                    <label className="git-action-group">
                      <span>{copy.overview.git.actions.historyFilter}</span>
                      <input
                        type="text"
                        value={historyFilter}
                        onChange={(event) =>
                          setHistoryFilters((current) => ({
                            ...current,
                            [project.repoRoot]: event.target.value
                          }))
                        }
                      />
                    </label>
                    <ul className="detail-list">
                      {filteredCommits.map((commit) => {
                        const expandedCommit = expandedCommits[project.repoRoot];
                        const isExpanded = expandedCommit === commit.sha;

                        return (
                        <li key={`${project.repoRoot}:${commit.sha}`}>
                          <div className="config-card-topline">
                            <span>{commit.summary}</span>
                            <button
                              type="button"
                              aria-expanded={isExpanded}
                              onClick={() =>
                                setExpandedCommits((current) => ({
                                  ...current,
                                  [project.repoRoot]: isExpanded ? undefined : commit.sha
                                }))
                              }
                            >
                              {isExpanded
                                ? `${copy.overview.git.actions.hideDetails} ${commit.sha}`
                                : `${copy.overview.git.actions.showDetails} ${commit.sha}`}
                            </button>
                          </div>
                          {isExpanded ? (
                            <div className="git-action-note">
                              <p>
                                <strong>{copy.overview.git.fields.commitSha}</strong>: {commit.sha}
                              </p>
                              <p>
                                <strong>{copy.overview.git.fields.commitAuthor}</strong>: {commit.author}
                              </p>
                              <p>
                                <strong>{copy.overview.git.fields.commitAuthoredAt}</strong>: {commit.authoredAt}
                              </p>
                            </div>
                          ) : null}
                        </li>
                        );
                      })}
                    </ul>
                  </>
                ) : null}

                {latestEvent ? (
                  <div className="git-action-note">
                    <strong>{copy.overview.git.fields.lastAction}</strong>
                    <p>{latestEvent.detail}</p>
                  </div>
                ) : null}

                <div className="git-action-grid">
                  <label className="git-action-group">
                    <span>{copy.overview.git.actions.commitMessage}</span>
                    <input
                      type="text"
                      value={commitMessage}
                      onChange={(event) =>
                        setCommitDrafts((current) => ({
                          ...current,
                          [project.repoRoot]: event.target.value
                        }))
                      }
                    />
                  </label>
                  <button
                    type="button"
                    onClick={() => {
                      onCommit({
                        repoRoot: project.repoRoot,
                        message: commitMessage
                      });
                      setCommitDrafts((current) => ({
                        ...current,
                        [project.repoRoot]: ""
                      }));
                    }}
                    disabled={!project.dirty || !commitMessage.trim()}
                  >
                    {copy.overview.git.actions.commitButton}
                  </button>
                </div>

                <div className="git-action-grid">
                  <label className="git-action-group">
                    <span>{copy.overview.git.actions.branchName}</span>
                    <input
                      type="text"
                      value={branchDraft}
                      onChange={(event) =>
                        setBranchDrafts((current) => ({
                          ...current,
                          [project.repoRoot]: event.target.value
                        }))
                      }
                    />
                  </label>
                  <button
                    type="button"
                    onClick={() => onSwitchBranch({ repoRoot: project.repoRoot, branch: branchDraft })}
                    disabled={branchSwitchBlocked || !branchDraft.trim()}
                  >
                    {copy.overview.git.actions.switchButton}
                  </button>
                </div>

                {branchSwitchBlocked ? (
                  <p className="panel-copy">{copy.overview.git.guardrails.cleanBeforeSwitch}</p>
                ) : null}

                <div className="git-action-grid">
                  <button
                    type="button"
                    onClick={() => onPush({ repoRoot: project.repoRoot })}
                    disabled={pushDisabled}
                  >
                    {copy.overview.git.actions.pushButton}
                  </button>
                </div>

                {project.dirty ? (
                  <p className="panel-copy">{copy.overview.git.guardrails.cleanBeforePush}</p>
                ) : null}
                {!project.dirty && project.ahead === 0 ? (
                  <p className="panel-copy">{copy.overview.git.guardrails.nothingToPush}</p>
                ) : null}
              </article>
            );
          })}
        </div>
      )}
    </section>
  );
}

function buildLatestGitEventMap(auditEvents: AuditEventRecord[]) {
  const events = new Map<string, AuditEventRecord>();

  for (const event of auditEvents) {
    if (!event.type.startsWith("git_") || events.has(event.target)) {
      continue;
    }

    events.set(event.target, event);
  }

  return events;
}

function translateGitStatus(
  statuses: Record<"clean" | "dirty" | "diverged", string>,
  value: string
) {
  return statuses[value as keyof typeof statuses] ?? value;
}

function matchesCommitFilter(
  commit: GitProjectRecord["recentCommits"][number],
  filterValue: string
) {
  const normalizedFilter = filterValue.trim().toLowerCase();
  if (!normalizedFilter) {
    return true;
  }

  return [commit.summary, commit.author, commit.sha].some((value) =>
    value.toLowerCase().includes(normalizedFilter)
  );
}
