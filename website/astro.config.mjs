import { createHash } from "node:crypto";
import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";
import jass from "./scripts/jass-language.mjs";
import { CODE_THEMES } from "./src/codeThemes.ts";

const HIGHLIGHT_CACHE_KEY_LENGTH = 12;

/** @param {{ name: string }} theme */
function codeThemeSelector(theme) {
  const selected = `[data-code-theme='${theme.name}']`;
  if (theme.name === "github-dark") {
    return `${selected}, [data-code-theme='auto'][data-theme='dark']`;
  }
  if (theme.name === "github-light") {
    return `${selected}, [data-code-theme='auto'][data-theme='light']`;
  }
  return selected;
}

// Markdown caches include generated stylesheet URLs, so palette changes need a fresh cache.
const highlightCacheKey = createHash("sha256")
  .update(
    JSON.stringify({ themes: CODE_THEMES, grammar: jass, selector: codeThemeSelector.toString() }),
  )
  .digest("hex")
  .slice(0, HIGHLIGHT_CACHE_KEY_LENGTH);

export default defineConfig({
  cacheDir: `./node_modules/.astro/highlighting-${highlightCacheKey}/`,
  site: process.env.DOCS_SITE || "https://tinkerbench-docs.vercel.app",
  base: process.env.DOCS_BASE || "/",
  integrations: [
    starlight({
      title: "Tinkerbench",
      logo: { src: "./src/assets/tinkerbench.png", alt: "Tinkerbench gnome" },
      favicon: "/icon-128.png",
      disable404Route: true,
      pagefind: {
        ranking: {
          // Keep normal heading weights without an extra multiplier for short titles.
          metaWeights: { title: 1 },
        },
      },
      expressiveCode: {
        shiki: { langs: [jass] },
        themes: CODE_THEMES.map((theme) => theme.id),
        useDarkModeMediaQuery: false,
        themeCssSelector: codeThemeSelector,
      },
      description: "Warcraft III map making: Tinkerbench, JASS and Lua, and object data explained.",
      customCss: ["./src/styles/custom.css"],
      components: {
        Footer: "./src/components/Footer.astro",
        Search: "./src/components/Search.astro",
        ThemeProvider: "./src/components/ThemeProvider.astro",
        MarkdownContent: "./src/components/MarkdownContent.astro",
        ThemeSelect: "./src/components/ThemeSelect.astro",
      },
      social: [
        { icon: "github", label: "GitHub", href: "https://github.com/gabaudette/wc3-toolkit" },
      ],
      sidebar: [
        {
          label: "Start here",
          items: [
            { slug: "index" },
            { slug: "editor/install" },
            { slug: "editor/visual-tour" },
            { slug: "editor/first-map" },
            { slug: "reference/glossary" },
            { slug: "editor/troubleshooting" },
          ],
        },
        {
          label: "Tinkerbench",
          items: [
            { slug: "editor/objects" },
            { slug: "editor/sharing" },
            { slug: "editor/lua-harness" },
          ],
        },
        { label: "Make something", items: [{ autogenerate: { directory: "guides" } }] },
        {
          label: "Scripting",
          items: [
            { slug: "scripting/languages" },
            { slug: "reference/examples" },
            { slug: "reference/api" },
            { slug: "reference/api/practical" },
          ],
        },
        {
          label: "Object data",
          items: [
            { slug: "reference/field-tasks" },
            { slug: "reference/common-confusions" },
            { slug: "reference/fields" },
          ],
        },
        { label: "Map formats", items: [{ autogenerate: { directory: "formats" } }] },
        {
          label: "Contributing",
          items: [
            { slug: "contributing" },
            { slug: "contributing/development" },
            { slug: "contributing/documentation" },
          ],
        },
      ],
    }),
  ],
});
