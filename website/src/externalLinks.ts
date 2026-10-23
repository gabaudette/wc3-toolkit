function configureExternalLink(link: HTMLAnchorElement) {
  if (
    (link.protocol === "https:" || link.protocol === "http:") &&
    link.origin !== window.location.origin
  ) {
    link.target = "_blank";
    link.relList.add("noopener", "noreferrer");
  }
}

function configureExternalLinks() {
  for (const link of document.querySelectorAll<HTMLAnchorElement>("a[href]")) {
    configureExternalLink(link);
  }
}

configureExternalLinks();
document.addEventListener("astro:page-load", configureExternalLinks);

// Apply the same behavior to links inserted after the page has loaded.
function configureClickedLink(event: MouseEvent) {
  if (event.target instanceof Element) {
    const link = event.target.closest<HTMLAnchorElement>("a[href]");
    if (link) {
      configureExternalLink(link);
    }
  }
}

document.addEventListener("click", configureClickedLink, { capture: true });
document.addEventListener("auxclick", configureClickedLink, { capture: true });
