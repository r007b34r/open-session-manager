import { render, screen } from "@testing-library/react";
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
