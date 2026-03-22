import { render, screen } from "@testing-library/react";
import type { ReactNode } from "react";

import { I18nProvider } from "../lib/i18n";
import { DiffViewer } from "./diff-viewer";

describe("DiffViewer", () => {
  it("renders localized field labels, before/after values, and severity badges", () => {
    renderWithI18n(
      <DiffViewer
        entries={[
          {
            field: "baseUrl",
            before: "https://api.openai.com/v1",
            after: "https://relay.example/v1",
            severity: "warning"
          },
          {
            field: "model",
            before: "gpt-5",
            after: "gpt-5-mini",
            severity: "safe"
          }
        ]}
      />
    );

    expect(screen.getByText("Endpoint")).toBeInTheDocument();
    expect(screen.getByText("https://api.openai.com/v1")).toBeInTheDocument();
    expect(screen.getByText("https://relay.example/v1")).toBeInTheDocument();
    expect(screen.getByText("Warning")).toBeInTheDocument();
    expect(screen.getByText("Safe")).toBeInTheDocument();
  });

  it("shows an explicit empty state when no field changed", () => {
    renderWithI18n(<DiffViewer entries={[]} />);

    expect(screen.getByText(/no field changes detected/i)).toBeInTheDocument();
  });
});

function renderWithI18n(node: ReactNode) {
  return render(
    <I18nProvider language="en" setLanguage={vi.fn()}>
      {node}
    </I18nProvider>
  );
}
