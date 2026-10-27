# Tinkerbench field guide

Astro Starlight documentation with the Workshop design (graphite, mint, DM Sans).
The gnome header logo and favicon are the app's existing icons. Fonts are bundled locally.

## Run and validate

Use Node.js 24 and Bun 1.4.2. From this directory:

```sh
bun install --frozen-lockfile
bun run dev
```

`dev` generates the references first. After editing a TOML field guide or a snapshot, rerun
`bun run generate` to refresh the running preview. Search is generated during a production
build; test it with `bun run preview`, not the development server.

```sh
bun run test
bun run generate
bun run check
bun run build
bun run check:links
bun run preview
```

Run these sequentially: generation owns its output directories and must finish before Astro
reads them. `check:links` validates local links and assets throughout the built site. CI runs
the same checks and uploads a static site artifact. No Warcraft installation is needed in CI.

## Author content

- `src/content/docs/`: editor guides, tutorials, language introduction and map format notes.
  The troubleshooting hub in `editor/troubleshooting.md` follows the current setup, field,
  save and import flows and preview messages. Recheck its advice when those controls or
  errors change; a source-level check does not establish in-game behavior.
- `src/content/docs/guides/custom-spell.md`: the current spell-wizard walkthrough, with
  nine real dialog captures in `public/screenshots/spell-*.png`. Recheck against
  `SpellWizard.tsx`, `SpellBuffStep.tsx`, `SpellButtonStep.tsx`, `SpellUnits.tsx` and
  `SpellTriggered.tsx` when the workflow changes. Refresh on disposable Lua maps with a
  Paladin-based hero holding four hero abilities: one map for Sunwell Bolt (Storm Bolt,
  three levels, its own buff, hotkey B), another for Attribute Strike (Strength defaults,
  hotkey G). Keep the spell and learn buttons in free slots and use the IDs actually
  assigned, which need not match the captures. Capture the dialog from the app window;
  these images are 640 or 1024 pixels wide, with their real heights in the markup.
  Read saved object data and the generated test-copy script back before describing them
  as checked. The guide labels in-game casting as unverified; recording an actual play
  test is required before changing that statement.
- `src/content/docs/guides/hero.md` and `item.md`: illustrated object-editing walkthroughs
  using existing game abilities. Refresh `public/screenshots/tutorial-hero-*.png` and
  `tutorial-item-*.png` on a disposable map: Sunwell Warden copies `Hpal`, with starting
  attributes 24/15/20 and gains 2/1/3; Wayfarer Draught copies `phea`, has three charges,
  costs 125 gold and retains Item Healing. Use the current assigned custom IDs; the
  captures use `H001` and `I000`. Keep the hero's four game spells and normal inventory.
  These full-window captures are 1440 × 900. Check saved values, captions and notices,
  then play-test level progression/learning and pickup, dropping, healing uses, final
  consumption and selling. Allow for missing health and the real item cooldown.
  The pages record the tested game build; update that evidence when the examples change.
- `src/content/docs/reference/field-tasks.md`: authored goal-to-field index. It sits outside
  the generated reference subdirectories. Recheck linked field labels, object types and
  companion settings against `data/fields.toml` and the shared TOML explanations when
  metadata changes. Its task combinations are source-reviewed, not in-game verification.
- `src/content/docs/reference/glossary.md`: authored, alphabetically grouped definitions with
  links from introductory guides. Keep term heading anchors stable; incoming term links and
  production search can point directly to them. Language-specific notes use the same paired
  sections as other guides. Recheck definitions against the linked guides and supported
  formats when app or reference terminology changes. This page is not generated.
- `src/content/docs/reference/common-confusions.md`: authored decision guidance for armor
  fields, unit removal/death and coordinates/location handles. Reuse the existing paired
  API recipes by linking to them. The generator's `conceptComparisons` map adds backlinks
  from selected API and unit-field pages; keep its anchors aligned with the authored
  headings. Recheck field labels and choices against metadata and retain explicit notes
  for game behavior that has not been tested. The page is not generated.
- `public/screenshots/`: real desktop captures shared by the visual tour, startup guides,
  homepage and root README. Editor views are 1440 × 900; startup captures focus on their
  panels (600 × 420 and 600 × 600) to keep the controls readable. Sharing and import dialogs
  use 680 × 540 and 680 × 420 captures. Capture only the Tinkerbench
  window with `import -window <window-id>`;
  `xdotool search --name '^Tinkerbench$'` finds it on X11. Use a test map, let data and models
  finish loading, and move the pointer away from the content before capturing. Keep filenames
  stable when refreshing the UI. Screenshots show game art in context; do not export or add
  standalone game assets. The guide’s captions and alt text should match each refreshed image.
  Include “Image subject to change.” below every screenshot, including reused images.
- `../object-editor/crates/wc3-data/data/*-fields.toml`: project-authored field explanations,
  shared with the editor. Every guide ID gets a page, including IDs absent from the snapshot.
- `data/fields.toml`: display names, types, limits, choices and applicability extracted from
  the local installation. Includes spell-specific fields beyond the authored guides.
- `data/api.json`: API declaration metadata, source hashes and the snapshot version label.
- `scripts/api-categories.ts`: authored browse topics, ordered symbol-name rules and
  engine-type ownership. The generator creates a topic overview, one page per topic and a
  complete alphabetical index without changing existing symbol URLs. Topic rules run before
  callable parameter/return-type fallbacks so, for example, saving a unit handle stays under
  storage. Tests cover cross-topic operations and new symbols. Update these rules when new
  API families need a more specific home than Runtime and general utilities.
- `data/api-guides.json`: project-authored practical API explanations, separate from the
  extracted snapshot and preserved by `refresh:api`. The practical index lists the initial
  26 APIs. Match each symbol and source, document every parameter by its exact name, explain
  returns and ownership, and include usage, mistakes, related APIs and a recipe key. Reuse
  recipes where possible; the selected recipe must call the API in both languages. Optional
  `jassNotes` and `luaNotes` switch with the language picker; `unverified` records specific
  engine behavior that has not been exercised. Keep the source-review status honest: parsing
  or syntax checking does not establish game behavior. To claim in-game verification, record
  the exact game build, test setup and observed result before changing the status text.
  Validation rejects unresolved or ambiguous symbols/links, stale parameter names, duplicate
  guides, blank guidance and missing or unrelated examples before replacing generated pages.
- `data/lifecycles.md`: authored lifecycle-guide template. Diagrams use semantic HTML and
  the theme-aware flow styles in `src/styles/custom.css`; each has a visible text-equivalent
  caption, while decorative diagram nodes are hidden from assistive technology. The
  `<!-- lifecycle-example: Name -->` markers expand recipes from `data/examples.json` into
  native, keyboard-accessible Code disclosures. Both languages remain in the generated
  page and follow the existing toggle. Duplicate folded code is excluded from Pagefind;
  the canonical API recipes remain searchable. The generator rejects missing, incomplete
  or malformed recipe markers before replacing generated pages. Its `lifecycleGuides`
  mapping adds API backlinks; keep resource anchors aligned with the template headings.
- `data/examples.json`: paired, complete JASS and Lua recipes with titles, categories and
  usage instructions. The API pages and generated examples cookbook use this same source.
  `examples.test.ts` checks Lua 5.3 syntax, named entry points, API references and duplicate
  JASS function names. The 48 current JASS recipes were also checked with PJass against the
  local installation scripts. Game execution still needs a test map.
- `src/searchMetadata.ts`: derives Pagefind Content filters and result type labels from
  collection IDs, independent of the deployment base path. Authored pages are Guides; the
  generated API and field trees are API and Object fields. The generator marks its curated
  API explanations and practical index with `practicalGuide: true`, making them discoverable
  in both Guides and API. `MarkdownContent.astro` emits filter/metadata attributes and gives
  guide body text a ranking boost. `astro.config.mjs` reduces Pagefind’s extra title-metadata
  multiplier while retaining normal heading weights, balancing long guides against short
  reference pages. The Search override wraps the existing modal. `searchFilters.ts` adds
  an explicit
  All content/Guides/API/Object fields chooser and synchronizes Pagefind’s underlying
  checkboxes via change events. One choice avoids its default AND semantics for multiple
  values. Recheck this bridge when upgrading Pagefind. Search tests cover
  route boundaries, curated flags and classification; verify actual ranking and UI against
  the production build for a guide query, `CreateUnit`, `uarm` and a no-match query.
- `scripts/jass-language.mjs`: project-authored JASS highlighting grammar.
- `src/components/DocumentationReport.astro`: shared footer report link, including authored
  and generated pages. `src/documentationReport.ts` constructs a draft using GitHub's
  [title/body query parameters](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/creating-an-issue#creating-an-issue-from-a-url-query).
  The server supplies the title and canonical URL from the configured site and page path,
  so preview hosts and browser query strings are not report context. The generator writes
  optional `reportReference` frontmatter from exact API symbols/source/kind or field raw
  codes/object types; do not infer case-sensitive raw codes from lowercase route slugs.
  `src/documentationReportLinks.ts` adds the current language only on pages with paired
  sections and refreshes the normal anchor when the global selection changes. Without
  JavaScript, the server-rendered link still opens a draft with page/reference context.
  Report controls are excluded from Pagefind; reporting guidance is authored in the
  documentation contribution guide. GitHub handles sign-in and reader submission.
- `src/codeThemes.ts`: bundled snippet themes, shared by highlighting and the native pickers.
  Snippet colors are independent of the site theme; “Follow site theme” selects GitHub Dark
  or Light. The preference is restored before paint and synchronized across snippets/tabs.

A global Lua/JASS toggle in `ThemeSelect.astro` stores its choice before paint. Paired
sections use `data-example-language`; code, explanatory prose, headings and table-of-contents
items switch together. Shared API tables describe engine types. Single-language instructions
such as the Lua harness stay readable.

The content cache is keyed by the theme configuration and JASS grammar, so Markdown cannot
retain stylesheet references from an older syntax-highlighting configuration.

Generated pages in `src/content/docs/reference/{api,fields,examples,lifecycles}/` are ignored. Every build
regenerates them. Edit the source guide or snapshot, rather than generated Markdown.
URLs include the script filename and declaration kind so `rect` (type) and `Rect` (native)
remain distinct on case-insensitive hosts. Generation rejects duplicate URL paths and guide IDs.

## Refresh local API and metadata

Never commit Blizzard game scripts, implementations, comments or assets. Export inputs to a
temporary folder outside the repository. From `object-editor/`:

```sh
cargo run -p wc3-data --example docs_sources -- /path/to/Warcraft-III /path/to/local-exports
```

Then from `website/`:

```sh
bun run refresh:api /path/to/local-exports "your installed game version"
bun run test
bun run build
bun run check:links
```

The refresh command retains declarations (types, natives, helper signatures and constant
names/types) and field metadata. Function bodies, constant initializers and comments stay in
the local exports. Review both snapshot diffs before committing. The source hashes identify
the exact script input; the label records which installation it came from. The initial
snapshot is explicitly labeled with its local extraction date because no patch version was
confirmed. It is not advertised as the latest patch.

## Hosting

The site is static. `bun run package:vercel` packages `website/dist` into
`.vercel/output/static` with explicit static routing (Vercel's Build Output API); the manifest
prevents a successful build from deploying an empty output. Field guides stay outside the
website directory, so the Vercel project's root is the repository root.

### Deployed from GitHub Actions, only when the documentation changes

Vercel's own Git deployments are off (`"git": { "deploymentEnabled": false }` in the root
`vercel.json`), so pull requests about the editor get no Vercel build, comment or check.
`.github/workflows/docs.yml` runs only when one of these changes:
- `website/**`
- `object-editor/crates/wc3-data/data/**` (the shared field guides)
- `object-editor/crates/wc3-data/examples/docs_sources.rs`
- `.github/workflows/docs.yml` and `vercel.json`

It tests, checks and builds the site, then its `deploy` job uploads the package with
`vercel deploy --prebuilt`: a preview for a pull request, production for `main`. The URL is in
the job summary. The job needs a `VERCEL_TOKEN` repository secret (a Vercel token for the
team that owns `wc3-toolkit-docs`); without it the site is built but not deployed, with a
warning. Pull requests from forks are built but not deployed, since they get no secrets. The
team and project IDs are in the workflow. To redeploy by hand, run the workflow from the
Actions tab (`workflow_dispatch`).

Default canonical URL: `https://tinkerbench-docs.vercel.app`. Set `DOCS_SITE` to change it.
`DOCS_BASE` supports a subpath such as `/wc3-toolkit` for a static host like GitHub Pages;
set it consistently for the build and `check:links`. The default is `/`.

The four original design studies are at `/designs/`; their search and navigation are
illustrative. The real documentation uses Starlight navigation, accessible search, a mobile
menu, a theme switcher and highlighted code.
