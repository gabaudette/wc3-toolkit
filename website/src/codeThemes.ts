export const CODE_THEMES = [
  { id: "github-dark", label: "GitHub Dark" },
  { id: "github-light", label: "GitHub Light" },
  { id: "dracula", label: "Dracula" },
  { id: "one-dark-pro", label: "One Dark Pro" },
  { id: "nord", label: "Nord" },
  { id: "catppuccin-mocha", label: "Catppuccin Mocha" },
] as const;

export const CODE_THEME_STORAGE_KEY = "tinkerbench-code-theme";
export type CodeTheme = "auto" | (typeof CODE_THEMES)[number]["id"];

export function parseCodeTheme(value: unknown): CodeTheme {
  if (CODE_THEMES.some((theme) => theme.id === value)) {
    return value as CodeTheme;
  }
  return "auto";
}
