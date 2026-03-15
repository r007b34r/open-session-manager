import type { PropsWithChildren } from "react";

import { useI18n, type Language } from "../lib/i18n";
import type { ThemePreference } from "../lib/theme";

type RootShellProps = PropsWithChildren<{
  currentPath: string;
  themePreference: ThemePreference;
  onThemeChange: (theme: ThemePreference) => void;
}>;

export function RootShell({
  children,
  currentPath,
  themePreference,
  onThemeChange
}: RootShellProps) {
  const { copy, language, setLanguage } = useI18n();
  const navigation = [
    { href: "#/", label: copy.root.nav.overview },
    { href: "#/sessions", label: copy.root.nav.sessions },
    { href: "#/configs", label: copy.root.nav.configs },
    { href: "#/audit", label: copy.root.nav.audit }
  ];

  return (
    <main className="app-shell">
      <section className="hero-shell">
        <div className="hero-copy">
          <p className="eyebrow">{copy.root.eyebrow}</p>
          <h1>{copy.root.title}</h1>
          <p className="hero-text">{copy.root.description}</p>
        </div>

        <div className="hero-aside">
          <div
            aria-label={copy.root.languageLabel}
            className="language-switcher"
            role="group"
          >
            {(["zh-CN", "en"] as Language[]).map((option) => (
              <button
                className={
                  language === option ? "language-button is-active" : "language-button"
                }
                key={option}
                onClick={() => setLanguage(option)}
                type="button"
              >
                {copy.root.languageNames[option]}
              </button>
            ))}
          </div>

          <div
            aria-label={copy.root.themeLabel}
            className="theme-switcher"
            role="group"
          >
            {(["system", "light", "dark"] as ThemePreference[]).map((option) => (
              <button
                className={
                  themePreference === option ? "theme-button is-active" : "theme-button"
                }
                key={option}
                onClick={() => onThemeChange(option)}
                type="button"
              >
                {copy.root.themeNames[option]}
              </button>
            ))}
          </div>

          <nav aria-label={copy.root.navLabel} className="primary-nav">
            {navigation.map((item) => (
              <a
                className={
                  currentPath === normalizeRoute(item.href) ? "is-active" : ""
                }
                href={item.href}
                key={item.href}
              >
                {item.label}
              </a>
            ))}
          </nav>
        </div>
      </section>

      {children}
    </main>
  );
}

function normalizeRoute(href: string) {
  return href.replace(/^#/, "") || "/";
}
