import { render, screen } from "@testing-library/react";
import { App } from "./app";

describe("App", () => {
  it("renders the governance dashboard shell", async () => {
    render(<App />);

    expect(
      await screen.findByRole("heading", { name: /agent session governance/i })
    ).toBeInTheDocument();
    expect(
      await screen.findByText(/local-first control center/i)
    ).toBeInTheDocument();
  });
});
