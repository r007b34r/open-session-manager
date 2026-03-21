import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { App } from "./app";
import { LANGUAGE_STORAGE_KEY } from "./lib/i18n";
import { THEME_STORAGE_KEY } from "./lib/theme";

describe("App", () => {
  beforeEach(() => {
    window.localStorage.clear();
    window.localStorage.setItem("open-session-manager.enable-demo-data", "1");
    window.location.hash = "";
    mockNavigatorLanguage("en-US", ["en-US"]);
    mockMatchMedia(false);
  });

  it("renders the governance dashboard shell", async () => {
    render(<App />);

    expect(
      await screen.findByRole("heading", { name: /open session manager/i })
    ).toBeInTheDocument();
    expect(
      await screen.findByText(/inspect local coding-agent sessions/i)
    ).toBeInTheDocument();
  });

  it("根据浏览器语言自动切换到中文界面", async () => {
    mockNavigatorLanguage("zh-CN", ["zh-CN", "en-US"]);

    render(<App />);

    expect(
      await screen.findByRole("heading", { name: /开放会话管理器/i })
    ).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "总览" })).toBeInTheDocument();
  });

  it("允许手动切换语言并保存用户选择", async () => {
    const user = userEvent.setup();

    render(<App />);

    expect(
      await screen.findByRole("heading", { name: /open session manager/i })
    ).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "中文" }));

    expect(
      await screen.findByRole("heading", { name: /开放会话管理器/i })
    ).toBeInTheDocument();
    expect(window.localStorage.getItem(LANGUAGE_STORAGE_KEY)).toBe("zh-CN");
  });

  it("跟随系统深色偏好自动切换主题", async () => {
    mockMatchMedia(true);

    render(<App />);

    await screen.findByRole("heading", { name: /open session manager/i });

    expect(document.documentElement.dataset.theme).toBe("dark");
  });

  it("允许手动切换深色主题并保存用户选择", async () => {
    const user = userEvent.setup();

    render(<App />);

    await screen.findByRole("heading", { name: /open session manager/i });
    await user.click(screen.getByRole("button", { name: /dark/i }));

    expect(document.documentElement.dataset.theme).toBe("dark");
    expect(window.localStorage.getItem(THEME_STORAGE_KEY)).toBe("dark");
  });

  it("要求先导出 Markdown 才允许移入隔离区", async () => {
    const user = userEvent.setup();

    render(<App />);

    const moveButton = await screen.findByRole("button", {
      name: /move to quarantine/i
    });
    expect(moveButton).toBeDisabled();

    await user.click(
      screen.getByRole("button", { name: /export markdown/i })
    );

    expect(
      await screen.findByRole("button", { name: /move to quarantine/i })
    ).toBeEnabled();
  });

  it("导出后明确显示 Markdown 保存路径", async () => {
    const user = userEvent.setup();
    window.location.hash = "#/sessions";

    render(<App />);

    await screen.findByRole("heading", { name: /retention-first queue/i });
    await user.click(screen.getByRole("button", { name: /export markdown/i }));

    expect(
      await screen.findByText(/session-ses-001\.md/i)
    ).toBeInTheDocument();
    expect(
      screen.getByDisplayValue(/documents\/opensessionmanager\/exports/i)
    ).toBeInTheDocument();
  });

  it("允许修改 Markdown 导出目录并应用到后续导出", async () => {
    const user = userEvent.setup();
    window.location.hash = "#/sessions";

    render(<App />);

    const exportRootInput = await screen.findByLabelText(/markdown export folder/i);
    await user.clear(exportRootInput);
    await user.type(exportRootInput, "D:/OSM/exports");
    await user.click(screen.getByRole("button", { name: /save export folder/i }));
    await user.click(screen.getByRole("button", { name: /export markdown/i }));

    expect(
      await screen.findByDisplayValue("D:/OSM/exports")
    ).toBeInTheDocument();
    expect(screen.getByText(/d:\/osm\/exports\/session-ses-001\.md/i)).toBeInTheDocument();
  });

  it("在 Sessions 页里切换会话时保留列表并展示目标详情", async () => {
    const user = userEvent.setup();
    window.location.hash = "#/sessions";

    render(<App />);

    await screen.findByRole("heading", { name: /retention-first queue/i });
    await user.click(
      screen.getByRole("button", { name: /audit anthropic relay settings/i })
    );

    expect(window.location.hash).toBe("#/sessions/ses-002");
    expect(
      await screen.findByRole("heading", { name: /audit anthropic relay settings/i })
    ).toBeInTheDocument();
    expect(
      screen.getByRole("searchbox", { name: /search sessions/i })
    ).toBeInTheDocument();
  });

  it("在 Sessions 页里点击会话行的非标题区域也会切换详情", async () => {
    const user = userEvent.setup();
    window.location.hash = "#/sessions";

    render(<App />);

    await screen.findByRole("heading", { name: /retention-first queue/i });
    await user.click(screen.getByText("47"));

    expect(window.location.hash).toBe("#/sessions/ses-002");
    expect(
      await screen.findByRole("heading", { name: /audit anthropic relay settings/i })
    ).toBeInTheDocument();
  });

  it("搜索结果为空时不会继续展示不匹配的旧详情", async () => {
    const user = userEvent.setup();
    window.location.hash = "#/sessions/ses-002";

    render(<App />);

    await screen.findByRole("heading", { name: /audit anthropic relay settings/i });
    await user.clear(
      screen.getByRole("searchbox", { name: /search sessions/i })
    );
    await user.type(
      screen.getByRole("searchbox", { name: /search sessions/i }),
      "definitely-no-match"
    );

    await waitFor(() => {
      expect(
        screen.queryByRole("heading", {
          name: /audit anthropic relay settings/i
        })
      ).not.toBeInTheDocument();
    });
    expect(
      screen.getByRole("heading", { name: /select a session/i })
    ).toBeInTheDocument();
  });

  it("支持按项目路径关键词搜索会话", async () => {
    const user = userEvent.setup();

    render(<App />);

    await screen.findByRole("searchbox", { name: /search sessions/i });
    await user.type(
      screen.getByRole("searchbox", { name: /search sessions/i }),
      "ops"
    );

    await waitFor(() => {
      expect(
        screen.queryByRole("button", { name: /refactor wsl collector handshake/i })
      ).not.toBeInTheDocument();
    });
    expect(
      screen.getByRole("button", { name: /audit anthropic relay settings/i })
    ).toBeInTheDocument();
  });

  it("搜索结果在列表里显示命中片段和来源标签", async () => {
    const user = userEvent.setup();
    window.location.hash = "#/sessions";

    render(<App />);

    await screen.findByRole("searchbox", { name: /search sessions/i });
    await user.type(
      screen.getByRole("searchbox", { name: /search sessions/i }),
      "manifest framing"
    );

    const sessionButton = await screen.findByRole("button", {
      name: /refactor wsl collector handshake/i
    });
    const row = sessionButton.closest("tr");

    expect(row).not.toBeNull();
    expect(within(row as HTMLElement).getByText(/Finalize manifest framing/i)).toBeInTheDocument();
    expect(within(row as HTMLElement).getByText(/To-do/i)).toBeInTheDocument();
  });

  it("在详情面板中展示 transcript 高亮和待办快照", async () => {
    const user = userEvent.setup();
    window.location.hash = "#/sessions";

    render(<App />);

    await screen.findByRole("heading", { name: /retention-first queue/i });
    await user.click(
      screen.getByRole("button", { name: /audit anthropic relay settings/i })
    );

    expect(
      await screen.findByRole("heading", { name: /transcript highlights/i })
    ).toBeInTheDocument();
    expect(screen.getByText(/mapped anthropic_base_url override/i)).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: /todo snapshot/i })
    ).toBeInTheDocument();
    expect(screen.getByText(/review shell hook chain/i)).toBeInTheDocument();
  });

  it("在首页直接展示已吸收的上游能力", async () => {
    render(<App />);

    expect(
      await screen.findByRole("heading", { name: /open session manager/i })
    ).toBeInTheDocument();
    expect(
      await screen.findByText(/daaain\/claude-code-log/i)
    ).toBeInTheDocument();
    expect(
      await screen.findByText(/viewer-style transcript detail/i)
    ).toBeInTheDocument();
  });

  it("在总览里展示 usage analytics 面板和助手级汇总", async () => {
    render(<App />);

    const heading = await screen.findByRole("heading", {
      name: /usage analytics/i
    });
    const panel = heading.closest("section");

    expect(heading).toBeInTheDocument();
    expect(panel).not.toBeNull();
    const openCodeCard = within(panel as HTMLElement)
      .getByText(/^OpenCode$/)
      .closest("article");

    expect(openCodeCard).not.toBeNull();
    expect(within(openCodeCard as HTMLElement).getByText(/\$0\.02/i)).toBeInTheDocument();
  });

  it("在首页嵌入式会话区切换详情时不应强制跳转到 sessions 路由", async () => {
    const user = userEvent.setup();

    render(<App />);

    await screen.findByRole("heading", { name: /open session manager/i });
    await screen.findByRole("button", { name: /audit anthropic relay settings/i });
    await user.click(
      screen.getByRole("button", { name: /audit anthropic relay settings/i })
    );

    expect(window.location.hash).toBe("");
    expect(
      await screen.findAllByRole("heading", { name: /audit anthropic relay settings/i })
    ).not.toHaveLength(0);
  });

  it("浏览器模式拿不到真实快照时不应展示不存在的示例配置", async () => {
    window.localStorage.removeItem("open-session-manager.enable-demo-data");
    vi.spyOn(globalThis, "fetch").mockRejectedValue(new Error("offline"));

    render(<App />);

    expect(
      await screen.findByRole("heading", { name: /open session manager/i })
    ).toBeInTheDocument();
    await screen.findByRole("heading", { name: /config risk center/i });

    expect(screen.queryByText(/github copilot cli/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/factory droid/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/copilot\.enterprise-relay\.example/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/factory-relay\.example/i)).not.toBeInTheDocument();
  });

  it("在会话详情里展示 token 和成本细节", async () => {
    const user = userEvent.setup();
    window.location.hash = "#/sessions";

    render(<App />);

    await screen.findByRole("heading", { name: /retention-first queue/i });
    await user.click(
      screen.getByRole("button", {
        name: /package export and quarantine workflow/i
      })
    );

    expect(
      await screen.findByRole("heading", { name: /^usage$/i })
    ).toBeInTheDocument();
    expect(screen.getByText(/gpt-5/i)).toBeInTheDocument();
    expect(screen.getByText(/\$0\.02/i)).toBeInTheDocument();
  });
});

function mockNavigatorLanguage(language: string, languages: string[]) {
  Object.defineProperty(window.navigator, "language", {
    configurable: true,
    value: language
  });

  Object.defineProperty(window.navigator, "languages", {
    configurable: true,
    value: languages
  });
}

function mockMatchMedia(prefersDark: boolean) {
  Object.defineProperty(window, "matchMedia", {
    configurable: true,
    writable: true,
    value: (query: string) => ({
      matches: query === "(prefers-color-scheme: dark)" ? prefersDark : false,
      media: query,
      onchange: null,
      addEventListener: () => undefined,
      removeEventListener: () => undefined,
      addListener: () => undefined,
      removeListener: () => undefined,
      dispatchEvent: () => false
    })
  });
}
