import { SEARCH_CONTENT_KINDS } from "./searchMetadata";

function initializeContentChooser() {
  const root = document.querySelector<HTMLElement>("#starlight__search");
  if (!root || root.dataset.contentChooserReady) {
    return;
  }
  root.dataset.contentChooserReady = "true";

  // Pagefind mounts lazily on the first query. Retain its result renderer and modal.
  const install = () => {
    const panel = root.querySelector<HTMLElement>(".pagefind-ui__filter-panel");
    const drawer = root.querySelector<HTMLElement>(".pagefind-ui__drawer");
    if (!panel || !drawer) {
      return false;
    }
    const label = document.createElement("label");
    label.className = "search-kind";
    label.append("Search in");
    const select = document.createElement("select");
    for (const [value, title] of [
      ["", "All content"],
      ...SEARCH_CONTENT_KINDS.map((kind) => [kind, kind]),
    ]) {
      const option = document.createElement("option");
      option.value = value;
      option.textContent = title;
      select.append(option);
    }
    select.addEventListener("change", () => {
      // The default UI combines checkboxes with AND. A single choice avoids impossible
      // intersections between disjoint content kinds; clearing the choice searches all.
      for (const checkbox of panel.querySelectorAll<HTMLInputElement>("input[name=Content]")) {
        const checked = checkbox.value === select.value;
        if (checkbox.checked !== checked) {
          checkbox.checked = checked;
          checkbox.dispatchEvent(new Event("change", { bubbles: true }));
        }
      }
    });
    label.append(select);
    drawer.before(label);
    panel.hidden = true;
    return true;
  };

  if (install()) {
    return;
  }
  const observer = new MutationObserver(() => {
    if (install()) {
      observer.disconnect();
    }
  });
  observer.observe(root, { childList: true, subtree: true });
}

initializeContentChooser();
document.addEventListener("astro:page-load", initializeContentChooser);
