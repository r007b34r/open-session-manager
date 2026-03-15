import { render, screen } from "@testing-library/react";
import { App } from "./app";

describe("App", () => {
  it("renders the governance dashboard shell", () => {
    render(<App />);

    expect(
      screen.getByRole("heading", { name: /agent session governance/i })
    ).toBeInTheDocument();
    expect(
      screen.getByText(/local-first control center/i)
    ).toBeInTheDocument();
  });
});
