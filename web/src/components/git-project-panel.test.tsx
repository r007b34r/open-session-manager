import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";

import type { GitProjectRecord } from "../lib/api";
import { I18nProvider } from "../lib/i18n";
import { GitProjectPanel } from "./git-project-panel";

describe("GitProjectPanel", () => {
  it("支持按 summary、author 或 sha 筛选 commit history", async () => {
    const user = userEvent.setup();

    renderWithI18n(
      <GitProjectPanel
        auditEvents={[]}
        onCommit={vi.fn()}
        onPush={vi.fn()}
        onSwitchBranch={vi.fn()}
        projects={[buildProject()]}
      />
    );

    await user.type(screen.getByLabelText(/filter history/i), "9042ddf");

    expect(
      screen.getByRole("button", { name: /show details for 9042ddf/i })
    ).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: /show details for 7fd57a6/i })
    ).not.toBeInTheDocument();
  });

  it("支持展开单条 commit 明细查看 sha、author 和 authoredAt", async () => {
    const user = userEvent.setup();

    renderWithI18n(
      <GitProjectPanel
        auditEvents={[]}
        onCommit={vi.fn()}
        onPush={vi.fn()}
        onSwitchBranch={vi.fn()}
        projects={[buildProject()]}
      />
    );

    await user.click(
      screen.getByRole("button", { name: /show details for 7fd57a6/i })
    );

    const commitRow = screen
      .getByRole("button", { name: /hide details for 7fd57a6/i })
      .closest("li");
    expect(commitRow).not.toBeNull();
    expect(commitRow).toHaveTextContent("SHA: 7fd57a6");
    expect(commitRow).toHaveTextContent("Author: r007b34r");
    expect(commitRow).toHaveTextContent("Authored at: 2026-03-23T03:00:00.000Z");
  });

  it("renders a read-only workspace explorer with relative paths and truncation hint", () => {
    const project = Object.assign(buildProject(), {
      workspaceEntries: [
        { relativePath: "README.md", kind: "file", depth: 0 },
        { relativePath: "src", kind: "directory", depth: 0 },
        { relativePath: "src/main.rs", kind: "file", depth: 1 }
      ],
      workspaceTruncated: true
    }) as GitProjectRecord;

    renderWithI18n(
      <GitProjectPanel
        auditEvents={[]}
        onCommit={vi.fn()}
        onPush={vi.fn()}
        onSwitchBranch={vi.fn()}
        projects={[project]}
      />
    );

    expect(screen.getByText(/workspace explorer/i)).toBeInTheDocument();
    expect(screen.getByText("README.md")).toBeInTheDocument();
    expect(screen.getByText("src/")).toBeInTheDocument();
    expect(screen.getByText("src/main.rs")).toBeInTheDocument();
    expect(screen.getByText(/preview capped to the first/i)).toBeInTheDocument();
  });
});

function renderWithI18n(node: ReactNode) {
  return render(
    <I18nProvider language="en" setLanguage={vi.fn()}>
      {node}
    </I18nProvider>
  );
}

function buildProject(overrides: Partial<GitProjectRecord> = {}): GitProjectRecord {
  return {
    projectPath: "C:/Projects/osm",
    repoRoot: "C:/Projects/osm",
    branch: "feat/usability-clarity",
    status: "dirty",
    sessionCount: 2,
    dirty: true,
    stagedChanges: 1,
    unstagedChanges: 2,
    untrackedFiles: 1,
    ahead: 1,
    behind: 0,
    lastCommitSummary: "feat: add cockpit",
    lastCommitAt: "2026-03-23T03:00:00.000Z",
    recentCommits: [
      {
        sha: "7fd57a6",
        summary: "feat: add cockpit",
        author: "r007b34r",
        authoredAt: "2026-03-23T03:00:00.000Z"
      },
      {
        sha: "9042ddf",
        summary: "test: widen viewport coverage",
        author: "r007b34r",
        authoredAt: "2026-03-23T02:30:00.000Z"
      }
    ],
    ...overrides
  };
}
