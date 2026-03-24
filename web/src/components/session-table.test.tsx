import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

describe("SessionTable", () => {
  it("renders title, assistant, progress, and last activity for each session", async () => {
    const { SessionTable } = await import("./session-table");

    render(
      <SessionTable
        sessions={[
          {
            sessionId: "ses-001",
            title: "Refactor WSL collector handshake",
            assistant: "Codex",
            progressState: "In Progress",
            progressPercent: 65,
            lastActivityAt: "2026-03-15 12:40",
            environment: "WSL: Ubuntu",
            valueScore: 84
          },
          {
            sessionId: "ses-002",
            title: "Audit Anthropic relay settings",
            assistant: "Claude Code",
            progressState: "Blocked",
            progressPercent: 15,
            lastActivityAt: "2026-03-14 22:10",
            environment: "Windows 11",
            valueScore: 47
          }
        ]}
        selectedSessionId="ses-001"
      />
    );

    expect(
      screen.getAllByRole("columnheader", { name: /session/i })
    ).toHaveLength(2);
    const primarySessionButton = screen.getByRole("button", {
      name: /refactor wsl collector handshake/i
    });

    expect(primarySessionButton).toBeInTheDocument();
    expect(primarySessionButton).toHaveTextContent(/wsl: ubuntu/i);
    expect(primarySessionButton).toHaveTextContent(/ses-001/i);
    expect(screen.getByText(/codex/i)).toBeInTheDocument();
    expect(screen.getByText(/65%/i)).toBeInTheDocument();
    expect(screen.getByText(/2026-03-15 12:40/i)).toBeInTheDocument();
  });

  it("allows selecting a session row through an explicit workspace control", async () => {
    const user = userEvent.setup();
    const onSelectSession = vi.fn();
    const { SessionTable } = await import("./session-table");

    render(
      <SessionTable
        onSelectSession={onSelectSession}
        selectedSessionId="ses-001"
        sessions={[
          {
            sessionId: "ses-001",
            title: "Refactor WSL collector handshake",
            assistant: "Codex",
            progressState: "In Progress",
            progressPercent: 65,
            lastActivityAt: "2026-03-15 12:40",
            environment: "WSL: Ubuntu",
            valueScore: 84
          },
          {
            sessionId: "ses-002",
            title: "Audit Anthropic relay settings",
            assistant: "Claude Code",
            progressState: "Blocked",
            progressPercent: 15,
            lastActivityAt: "2026-03-14 22:10",
            environment: "Windows 11",
            valueScore: 47
          }
        ]}
      />
    );

    await user.click(
      screen.getByRole("button", { name: /audit anthropic relay settings/i })
    );

    expect(onSelectSession).toHaveBeenCalledWith("ses-002");
  });

  it("allows selecting a session by clicking a non-title cell in the row", async () => {
    const user = userEvent.setup();
    const onSelectSession = vi.fn();
    const { SessionTable } = await import("./session-table");

    render(
      <SessionTable
        onSelectSession={onSelectSession}
        selectedSessionId="ses-001"
        sessions={[
          {
            sessionId: "ses-001",
            title: "Refactor WSL collector handshake",
            assistant: "Codex",
            progressState: "In Progress",
            progressPercent: 65,
            lastActivityAt: "2026-03-15 12:40",
            environment: "WSL: Ubuntu",
            valueScore: 84
          },
          {
            sessionId: "ses-002",
            title: "Audit Anthropic relay settings",
            assistant: "Claude Code",
            progressState: "Blocked",
            progressPercent: 15,
            lastActivityAt: "2026-03-14 22:10",
            environment: "Windows 11",
            valueScore: 47
          }
        ]}
      />
    );

    await user.click(screen.getByText("47"));

    expect(onSelectSession).toHaveBeenCalledWith("ses-002");
  });
});
