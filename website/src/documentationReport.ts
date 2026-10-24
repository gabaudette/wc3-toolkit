import type { CodeLanguage } from "./codeLanguage";

type ReportContext = {
  title: string;
  canonicalUrl: string | URL;
  reference?: string;
  language?: CodeLanguage;
};

/** Only explicit page context goes to GitHub, never preview hosts or browser query strings. */
export function documentationReportUrl(context: ReportContext): string {
  const canonical = new URL(context.canonicalUrl);
  canonical.search = "";
  canonical.hash = "";
  const details = [`Page: ${context.title}`, `URL: ${canonical.href}`];
  if (context.reference) {
    details.push(context.reference);
  }
  if (context.language) {
    const labels = { lua: "Lua", jass: "JASS" };
    details.push(`Code language: ${labels[context.language]}`);
  }

  const report = new URL("https://github.com/gabaudette/wc3-toolkit/issues/new");
  report.searchParams.set("title", `Docs: ${context.title}`);
  report.searchParams.set(
    "body",
    [
      "## Page context",
      details.join("\n"),
      "## What is wrong or unclear?",
      "<!-- Quote the affected text or describe the broken link/example. Include steps and exact error text when relevant. -->",
      "## What did you expect?",
      "<!-- Describe the explanation or result you expected. -->",
      "## Relevant details (optional)",
      "<!-- Add Tinkerbench/Warcraft III versions, object or field IDs, or a minimal example when relevant. -->",
    ].join("\n\n"),
  );
  return report.href;
}
