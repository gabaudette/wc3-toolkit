import starlight from "@astrojs/starlight";
import { defineConfig } from "astro/config";
import jass from "./scripts/jass-language.mjs";

export default defineConfig({
  site: process.env.DOCS_SITE || "https://tinkerbench-docs.vercel.app",
  base: process.env.DOCS_BASE || "/",
  integrations: [
    starlight({
      title: "Tinkerbench · Field guide",
      logo: { src: "./src/assets/tinkerbench.png", alt: "Tinkerbench gnome" },
      favicon: "/favicon.ico",
      disable404Route: true,
      expressiveCode: { shiki: { langs: [jass] } },
      description: "Warcraft III map making: Tinkerbench, JASS and Lua, and object data explained.",
      customCss: ["./src/styles/custom.css"],
      components: { ThemeProvider: "./src/components/ThemeProvider.astro" },
      social: [
        { icon: "github", label: "GitHub", href: "https://github.com/gabaudette/wc3-toolkit" },
      ],
      sidebar: [
        {
          label: "Start here",
          items: [{ slug: "index" }, { slug: "editor/install" }, { slug: "editor/first-map" }],
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
          label: "JASS & Lua",
          items: [{ slug: "scripting/languages" }, { slug: "reference/api" }],
        },
        { label: "Object data", items: [{ slug: "reference/fields" }] },
        { label: "Map formats", items: [{ autogenerate: { directory: "formats" } }] },
      ],
    }),
  ],
});
