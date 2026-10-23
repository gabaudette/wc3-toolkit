import { CODE_THEME_STORAGE_KEY, CODE_THEMES, parseCodeTheme } from "./codeThemes";

function applyTheme(value: unknown) {
  const theme = parseCodeTheme(value);
  document.documentElement.dataset.codeTheme = theme;
  for (const select of document.querySelectorAll<HTMLSelectElement>(".code-theme-select")) {
    select.value = theme;
  }
}

function installPickers() {
  for (const block of document.querySelectorAll<HTMLElement>(".expressive-code")) {
    if (block.parentElement?.classList.contains("code-snippet")) {
      continue;
    }

    const wrapper = document.createElement("div");
    wrapper.className = "code-snippet not-content";
    const toolbar = document.createElement("div");
    toolbar.className = "code-theme-toolbar";
    const label = document.createElement("label");
    const caption = document.createElement("span");
    caption.textContent = "Code theme";
    const select = document.createElement("select");
    select.className = "code-theme-select";

    for (const theme of [{ id: "auto", label: "Follow site theme" }, ...CODE_THEMES]) {
      const option = document.createElement("option");
      option.value = theme.id;
      option.textContent = theme.label;
      select.append(option);
    }

    select.value = parseCodeTheme(document.documentElement.dataset.codeTheme);
    select.addEventListener("change", () => {
      applyTheme(select.value);
      try {
        localStorage.setItem(CODE_THEME_STORAGE_KEY, select.value);
      } catch {
        // The selection still works when browser privacy settings block storage.
      }
    });

    label.append(caption, select);
    toolbar.append(label);
    block.before(wrapper);
    wrapper.append(toolbar, block);
  }
}

installPickers();
document.addEventListener("astro:page-load", installPickers);
window.addEventListener("storage", (event) => {
  if (event.key === CODE_THEME_STORAGE_KEY) {
    applyTheme(event.newValue);
  }
});
