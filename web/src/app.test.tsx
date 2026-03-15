import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { App } from "./app";
import { LANGUAGE_STORAGE_KEY } from "./lib/i18n";

describe("App", () => {
  beforeEach(() => {
    window.localStorage.clear();
    window.location.hash = "";
    mockNavigatorLanguage("en-US", ["en-US"]);
  });

  it("renders the governance dashboard shell", async () => {
    render(<App />);

    expect(
      await screen.findByRole("heading", { name: /open session manager/i })
    ).toBeInTheDocument();
    expect(
      await screen.findByText(/local-first control center/i)
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
