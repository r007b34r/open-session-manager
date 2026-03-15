import type { PropsWithChildren } from "react";

type RootShellProps = PropsWithChildren<{
  currentPath: string;
}>;

const navigation = [
  { href: "#/", label: "Overview" },
  { href: "#/sessions", label: "Sessions" },
  { href: "#/configs", label: "Configs" },
  { href: "#/audit", label: "Audit" }
];

export function RootShell({ children, currentPath }: RootShellProps) {
  return (
    <main className="app-shell">
      <section className="hero-shell">
        <div className="hero-copy">
          <p className="eyebrow">Bootstrap</p>
          <h1>Agent Session Governance</h1>
          <p className="hero-text">
            Local-first control center for inspecting coding-agent sessions,
            configs, and cleanup actions before any destructive change is made.
          </p>
        </div>

        <nav aria-label="Primary" className="primary-nav">
          {navigation.map((item) => (
            <a
              className={currentPath === normalizeRoute(item.href) ? "is-active" : ""}
              href={item.href}
              key={item.href}
            >
              {item.label}
            </a>
          ))}
        </nav>
      </section>

      {children}
    </main>
  );
}

function normalizeRoute(href: string) {
  return href.replace(/^#/, "") || "/";
}
