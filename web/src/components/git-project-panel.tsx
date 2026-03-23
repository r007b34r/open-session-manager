import type { GitProjectRecord } from "../lib/api";
import { useI18n } from "../lib/i18n";

type GitProjectPanelProps = {
  projects: GitProjectRecord[];
};

export function GitProjectPanel({ projects }: GitProjectPanelProps) {
  const { copy } = useI18n();

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
          {projects.map((project) => (
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
                  <ul className="detail-list">
                    {project.recentCommits.map((commit) => (
                      <li key={`${project.repoRoot}:${commit.sha}`}>
                        {commit.summary}
                      </li>
                    ))}
                  </ul>
                </>
              ) : null}
            </article>
          ))}
        </div>
      )}
    </section>
  );
}

function translateGitStatus(
  statuses: Record<"clean" | "dirty" | "diverged", string>,
  value: string
) {
  return statuses[value as keyof typeof statuses] ?? value;
}
