import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import type { ReactNode } from "react";

import { I18nProvider } from "../lib/i18n";
import type { SessionDetailRecord } from "../lib/api";
import { SessionDetail } from "./session-detail";

describe("SessionDetail", () => {
  it("未知成本时明确显示 Unknown，而不伪装成 $0.00", () => {
    renderWithI18n(
      <SessionDetail
        session={buildSessionDetailRecord({
          usage: {
            model: "gpt-5-codex",
            inputTokens: 120,
            outputTokens: 80,
            cacheReadTokens: 0,
            cacheWriteTokens: 0,
            reasoningTokens: 0,
            totalTokens: 200
          }
        })}
      />
    );

    expect(screen.getByText(/cost \(usd\): unknown/i)).toBeInTheDocument();
    expect(screen.queryByText(/\$0\.00/i)).not.toBeInTheDocument();
  });

  it("真实零成本时仍然显示 $0.00", () => {
    renderWithI18n(
      <SessionDetail
        session={buildSessionDetailRecord({
          usage: {
            model: "gpt-5-codex",
            inputTokens: 120,
            outputTokens: 80,
            cacheReadTokens: 0,
            cacheWriteTokens: 0,
            reasoningTokens: 0,
            totalTokens: 200,
            costUsd: 0
          }
        })}
      />
    );

    expect(screen.getByText(/cost \(usd\): \$0\.00/i)).toBeInTheDocument();
  });

  it("展示会话控制状态，并允许发送继续提示", async () => {
    const user = userEvent.setup();
    const onContinueSession = vi.fn();

    renderWithI18n(
      <SessionDetail
        onContinueSession={onContinueSession}
        onResumeSession={vi.fn()}
        session={buildSessionDetailRecord({
          sessionControl: {
            supported: true,
            available: true,
            controller: "codex",
            command: "codex",
            attached: true,
            lastResponse: "READY from previous run"
          }
        })}
      />
    );

    expect(screen.getByRole("heading", { name: /session control/i })).toBeInTheDocument();
    expect(screen.getByText(/ready from previous run/i)).toBeInTheDocument();

    await user.type(
      screen.getByLabelText(/continue prompt/i),
      "Continue with validation"
    );
    await user.click(screen.getByRole("button", { name: /continue session/i }));

    expect(onContinueSession).toHaveBeenCalledWith(
      "ses-001",
      "Continue with validation"
    );
  });

  it("detached 会话必须先附着，继续按钮才可用", async () => {
    const user = userEvent.setup();

    renderWithI18n(
      <SessionDetail
        session={buildSessionDetailRecord({
          sessionControl: {
            supported: true,
            available: true,
            controller: "codex",
            command: "codex",
            attached: false
          }
        })}
      />
    );

    await user.type(
      screen.getByLabelText(/continue prompt/i),
      "Continue while detached"
    );

    expect(
      screen.getByRole("button", { name: /continue session/i })
    ).toBeDisabled();
    expect(
      screen.getByRole("button", { name: /attach session/i })
    ).toBeInTheDocument();
  });

  it("用 busy、waiting、idle 文案展示运行态", () => {
    const { rerender } = renderWithI18n(
      <SessionDetail
        session={buildSessionDetailRecord({
          sessionControl: {
            supported: true,
            available: true,
            controller: "codex",
            command: "codex",
            attached: true,
            runtimeState: "busy"
          } as any
        })}
      />
    );

    expect(screen.getByText(/control status: busy/i)).toBeInTheDocument();

    rerender(
      <I18nProvider language="en" setLanguage={vi.fn()}>
        <SessionDetail
          session={buildSessionDetailRecord({
            sessionControl: {
              supported: true,
              available: true,
              controller: "codex",
              command: "codex",
              attached: true,
              runtimeState: "waiting"
            } as any
          })}
        />
      </I18nProvider>
    );
    expect(screen.getByText(/control status: waiting/i)).toBeInTheDocument();

    rerender(
      <I18nProvider language="en" setLanguage={vi.fn()}>
        <SessionDetail
          session={buildSessionDetailRecord({
            sessionControl: {
              supported: true,
              available: true,
              controller: "codex",
              command: "codex",
              attached: true,
              runtimeState: "idle"
            } as any
          })}
        />
      </I18nProvider>
    );
    expect(screen.getByText(/control status: idle/i)).toBeInTheDocument();
  });

  it("busy 会话会禁用继续按钮并提示等待当前运行完成", async () => {
    const user = userEvent.setup();

    renderWithI18n(
      <SessionDetail
        onContinueSession={vi.fn()}
        session={buildSessionDetailRecord({
          sessionControl: {
            supported: true,
            available: true,
            controller: "codex",
            command: "codex",
            attached: true,
            runtimeState: "busy"
          } as any
        })}
      />
    );

    await user.type(
      screen.getByLabelText(/continue prompt/i),
      "Continue while busy"
    );

    expect(screen.getByRole("button", { name: /continue session/i })).toBeDisabled();
    expect(
      screen.getByText(/wait for the current run to report ready or go idle/i)
    ).toBeInTheDocument();
  });

  it("冷却窗口内会禁用继续按钮并提示稍后重试", async () => {
    const user = userEvent.setup();

    renderWithI18n(
      <SessionDetail
        onContinueSession={vi.fn()}
        session={buildSessionDetailRecord({
          sessionControl: {
            supported: true,
            available: true,
            controller: "codex",
            command: "codex",
            attached: true,
            runtimeState: "waiting",
            lastContinuedAt: new Date().toISOString()
          } as any
        })}
      />
    );

    await user.type(
      screen.getByLabelText(/continue prompt/i),
      "Continue too fast"
    );

    expect(screen.getByRole("button", { name: /continue session/i })).toBeDisabled();
    expect(
      screen.getByText(/wait a moment before sending another follow-up prompt/i)
    ).toBeInTheDocument();
  });

  it("要求在移入隔离区前显式确认 cleanup 审查", async () => {
    const user = userEvent.setup();
    const onSoftDelete = vi.fn();

    renderWithI18n(
      <SessionDetail
        canSoftDelete
        exportPath="C:/Users/Max/Documents/OpenSessionManager/exports/session-ses-001.md"
        onSoftDelete={onSoftDelete}
        session={buildSessionDetailRecord()}
      />
    );

    await user.click(screen.getByRole("button", { name: /move to quarantine/i }));

    expect(onSoftDelete).not.toHaveBeenCalled();
    expect(
      screen.getByRole("heading", { name: /review cleanup before quarantine/i })
    ).toBeInTheDocument();
    expect(
      screen.getByText(/session-ses-001\.md/i)
    ).toBeInTheDocument();

    const confirmButton = screen.getByRole("button", {
      name: /confirm move to quarantine/i
    });
    expect(confirmButton).toBeDisabled();

    await user.click(
      screen.getByRole("checkbox", {
        name: /i exported the valuable parts and want to continue/i
      })
    );
    await user.click(confirmButton);

    expect(onSoftDelete).toHaveBeenCalledWith("ses-001");
  });

  it("收到搜索命中的 transcript 目标后高亮对应条目", () => {
    renderWithI18n(
      <SessionDetail
        session={buildSessionDetailRecord({
          transcriptHighlights: [
            {
              role: "User",
              content: "Review the current cleanup flow."
            },
            {
              role: "Assistant",
              content: "Continue with validation once the relay override is removed."
            }
          ]
        })}
        transcriptFocus={{
          highlightIndex: 1,
          terms: ["validation", "relay override"]
        }}
      />
    );

    expect(screen.getByText(/search hit/i)).toBeInTheDocument();
    expect(screen.getByText(/continue with/i).closest("article")).toHaveClass(
      "is-search-match"
    );
    expect(screen.getByText("validation").tagName).toBe("MARK");
    expect(screen.getByText("relay override").tagName).toBe("MARK");
  });

  it("允许在详情里切换 rule 和 skill 提炼预览", async () => {
    const user = userEvent.setup();

    renderWithI18n(
      <SessionDetail
        session={buildSessionDetailRecord({
          title: "Audit Anthropic relay settings",
          assistant: "Claude Code",
          summary:
            "Proxy endpoint and permissive shell hooks were identified, but remediation steps were not applied yet.",
          tags: ["relay", "risk", "claude"],
          riskFlags: ["dangerous_permissions", "shell_hook"],
          transcriptHighlights: [
            {
              role: "Assistant",
              content:
                "Mapped ANTHROPIC_BASE_URL override and traced the permissive shell hook chain."
            }
          ],
          todoItems: [
            {
              content: "Review shell hook chain",
              completed: false
            },
            {
              content: "Export remediation summary before cleanup",
              completed: true
            }
          ]
        })}
      />
    );

    expect(screen.getByRole("heading", { name: /knowledge lift/i })).toBeInTheDocument();
    expect(screen.getByDisplayValue(/kind: osm-rule/i)).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /skill artifact/i }));

    expect(screen.getByDisplayValue(/name: audit-anthropic-relay-settings/i)).toBeInTheDocument();
    expect(screen.getByDisplayValue(/## resume cue/i)).toBeInTheDocument();
  });
});

function renderWithI18n(node: ReactNode) {
  return render(
    <I18nProvider language="en" setLanguage={vi.fn()}>
      {node}
    </I18nProvider>
  );
}

function buildSessionDetailRecord(
  overrides: Partial<SessionDetailRecord> = {}
): SessionDetailRecord {
  return {
    sessionId: "ses-001",
    title: "Unknown cost detail",
    assistant: "Codex",
    progressState: "In Progress",
    progressPercent: 65,
    lastActivityAt: "2026-03-15 12:40",
    environment: "WSL: Ubuntu",
    valueScore: 84,
    summary: "Collector still needs a stable manifest layer.",
    projectPath: "/home/max/src/open-session-manager",
    sourcePath: "C:/Users/Max/.codex/sessions/demo.jsonl",
    tags: ["wsl", "collector"],
    riskFlags: [],
    keyArtifacts: ["Defined distro handshake checkpoints"],
    transcriptHighlights: [],
    todoItems: [],
    sessionControl: {
      supported: true,
      available: true,
      controller: "codex",
      command: "codex",
      attached: false
    },
    ...overrides
  } as SessionDetailRecord;
}
