export const CODE_LANGUAGE_STORAGE_KEY = "tinkerbench-code-language";
export type CodeLanguage = "lua" | "jass";

export function parseCodeLanguage(value: unknown): CodeLanguage {
  if (value === "jass") {
    return "jass";
  }
  return "lua";
}
