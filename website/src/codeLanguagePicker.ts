import { CODE_LANGUAGE_STORAGE_KEY, parseCodeLanguage } from "./codeLanguage";

function applyLanguage(value: unknown) {
  const language = parseCodeLanguage(value);
  document.documentElement.dataset.codeLanguage = language;
  for (const button of document.querySelectorAll<HTMLButtonElement>(
    "[data-code-language-option]",
  )) {
    button.setAttribute("aria-pressed", String(button.dataset.codeLanguageOption === language));
  }
}

function initializeLanguagePickers() {
  // The table of contents is built from both variants. Keep it in sync with visible sections.
  for (const section of document.querySelectorAll<HTMLElement>("[data-example-language]")) {
    for (const heading of section.querySelectorAll<HTMLElement>("h2[id], h3[id], h4[id]")) {
      for (const link of document.querySelectorAll<HTMLAnchorElement>("nav a[href]")) {
        if (link.hash === `#${heading.id}`) {
          const item = link.closest("li");
          if (item) {
            item.dataset.exampleLanguage = section.dataset.exampleLanguage;
          }
        }
      }
    }
  }

  applyLanguage(document.documentElement.dataset.codeLanguage);
  for (const button of document.querySelectorAll<HTMLButtonElement>(
    "[data-code-language-option]",
  )) {
    if (button.dataset.languagePickerReady) {
      continue;
    }
    button.dataset.languagePickerReady = "true";
    button.addEventListener("click", () => {
      const language = parseCodeLanguage(button.dataset.codeLanguageOption);
      applyLanguage(language);
      try {
        localStorage.setItem(CODE_LANGUAGE_STORAGE_KEY, language);
      } catch {
        // The current page still switches when browser privacy settings block storage.
      }
    });
  }
}

initializeLanguagePickers();
document.addEventListener("astro:page-load", initializeLanguagePickers);
window.addEventListener("storage", (event) => {
  if (event.key === CODE_LANGUAGE_STORAGE_KEY) {
    applyLanguage(event.newValue);
  }
});
