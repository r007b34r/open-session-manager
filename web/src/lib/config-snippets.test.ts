import { describe, expect, it } from "vitest";

import type { ConfigWritebackInput } from "./api";
import {
  applyConfigSnippetToDraft,
  buildConfigSnippet,
  parseConfigSnippet,
  serializeConfigSnippet
} from "./config-snippets";

describe("config snippets", () => {
  it("builds a stable snippet payload, exports it, and parses it back", () => {
    const draft: ConfigWritebackInput = {
      artifactId: "cfg-003",
      assistant: "GitHub Copilot CLI",
      scope: "Global",
      path: "~/.copilot/config.json",
      provider: "openai",
      model: "gpt-5-mini",
      baseUrl: "https://api.openai.com/v1",
      secret: "should-not-be-exported"
    };

    const snippet = buildConfigSnippet(draft, {
      name: "OpenAI Direct",
      createdAt: "2026-03-23T01:30:00.000Z"
    });
    const exported = serializeConfigSnippet(snippet);

    expect(exported).toContain('"version": 1');
    expect(exported).toContain('"name": "OpenAI Direct"');
    expect(exported).toContain('"provider": "openai"');
    expect(exported).not.toContain("should-not-be-exported");
    expect(parseConfigSnippet(exported)).toEqual(snippet);
  });

  it("applies an imported snippet without overwriting the draft secret and rejects malformed payloads", () => {
    const draft: ConfigWritebackInput = {
      artifactId: "cfg-003",
      assistant: "GitHub Copilot CLI",
      scope: "Global",
      path: "~/.copilot/config.json",
      provider: "github",
      model: "gpt-5",
      baseUrl: "https://api.githubcopilot.com",
      secret: "keep-me"
    };

    const snippet = parseConfigSnippet(`{
      "version": 1,
      "name": "OpenRouter Shared",
      "provider": "openrouter",
      "model": "openrouter/anthropic/claude-sonnet-4",
      "baseUrl": "https://openrouter.ai/api/v1",
      "originAssistant": "Factory Droid",
      "createdAt": "2026-03-23T02:00:00.000Z"
    }`);
    const applied = applyConfigSnippetToDraft(draft, snippet);

    expect(applied).toMatchObject({
      provider: "openrouter",
      model: "openrouter/anthropic/claude-sonnet-4",
      baseUrl: "https://openrouter.ai/api/v1",
      secret: "keep-me"
    });
    expect(() => parseConfigSnippet(`{"version":1,"name":"broken"}`)).toThrow(
      /invalid config snippet/i
    );
  });
});
