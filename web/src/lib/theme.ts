export const THEME_STORAGE_KEY = "open-session-manager.theme";

export type ThemePreference = "system" | "light" | "dark";
export type ResolvedTheme = "light" | "dark";

type MatchMediaLike = Pick<
  MediaQueryList,
  "matches" | "addEventListener" | "removeEventListener" | "addListener" | "removeListener"
>;

export function getInitialThemePreference(): ThemePreference {
  if (typeof window === "undefined") {
    return "system";
  }

  try {
    const stored = coerceThemePreference(
      window.localStorage.getItem(THEME_STORAGE_KEY)
    );
    if (stored) {
      return stored;
    }
  } catch {
    return "system";
  }

  return "system";
}

export function resolveTheme(
  preference: ThemePreference,
  matchMediaFactory: ((query: string) => MatchMediaLike) | undefined =
    typeof window === "undefined" ? undefined : window.matchMedia
): ResolvedTheme {
  if (preference === "light" || preference === "dark") {
    return preference;
  }

  return matchMediaFactory?.("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

export function watchSystemTheme(
  onChange: () => void
): (() => void) | undefined {
  if (typeof window === "undefined" || typeof window.matchMedia !== "function") {
    return undefined;
  }

  const query = window.matchMedia("(prefers-color-scheme: dark)");

  if (typeof query.addEventListener === "function") {
    query.addEventListener("change", onChange);
    return () => query.removeEventListener("change", onChange);
  }

  if (typeof query.addListener === "function") {
    query.addListener(onChange);
    return () => query.removeListener(onChange);
  }

  return undefined;
}

function coerceThemePreference(
  value?: string | null
): ThemePreference | null {
  switch (value?.trim().toLowerCase()) {
    case "system":
    case "light":
    case "dark":
      return value.trim().toLowerCase() as ThemePreference;
    default:
      return null;
  }
}
