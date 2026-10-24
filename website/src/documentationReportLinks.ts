import { type CodeLanguage, parseCodeLanguage } from "./codeLanguage";
import { documentationReportUrl } from "./documentationReport";

function updateReportLinks() {
  const pairedExamples = document.querySelector("[data-example-language]");
  for (const link of document.querySelectorAll<HTMLAnchorElement>("[data-documentation-report]")) {
    const { reportTitle, reportUrl, reportReference } = link.dataset;
    if (!reportTitle || !reportUrl) {
      continue;
    }
    let language: CodeLanguage | undefined;
    if (pairedExamples) {
      language = parseCodeLanguage(document.documentElement.dataset.codeLanguage);
    }
    link.href = documentationReportUrl({
      title: reportTitle,
      canonicalUrl: reportUrl,
      reference: reportReference,
      language,
    });
  }
}

updateReportLinks();
document.addEventListener("astro:page-load", updateReportLinks);
// Keep normal clicks, keyboard activation, copying and opening in a new tab consistent.
new MutationObserver(updateReportLinks).observe(document.documentElement, {
  attributes: true,
  attributeFilter: ["data-code-language"],
});
