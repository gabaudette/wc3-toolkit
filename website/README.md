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
- `src/content/docs/reference/field-tasks.md`: authored goal-to-field index. It sits outside
  the generated reference subdirectories. Recheck linked field labels, object types and
  companion settings against `data/fields.toml` and the shared TOML explanations when
  metadata changes. Its task combinations are source-reviewed, not in-game verification.
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
- `src/codeThemes.ts`: bundled snippet themes, shared by highlighting and the native pickers.
  Snippet colors are independent of the site theme; “Follow site theme” selects GitHub Dark
  or Light. The preference is restored before paint and synchronized across snippets/tabs.

A global Lua/JASS toggle in `ThemeSelect.astro` stores its choice before paint. Paired
sections use `data-example-language`; code, explanatory prose, headings and table-of-contents
items switch together. Shared API tables describe engine types. Single-language instructions
such as the Lua harness stay readable.

The content cache is keyed by the theme configuration and JASS grammar, so Markdown cannot
retain stylesheet references from an older syntax-highlighting configuration.

Generated pages in `src/content/docs/reference/{api,fields,examples}/` are ignored. Every build
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

The repository root's `vercel.json` installs and builds this directory and packages
`website/dist` into `.vercel/output/static`, with explicit static routing. The Build Output
API manifest prevents a successful build from deploying an empty output. Install and build commands explicitly select Bun 1.4.2 because older build
images cannot read Bun 1.4 lockfiles. The Vercel project `wc3-toolkit-docs` connects to this repository: main pushes
produce production deployments and pull requests produce previews. Field guides remain
outside the website directory, so use the repository root as the Vercel project root.
No adapter, game files, runtime backend or deployment token in GitHub Actions is needed.

### Build only when documentation inputs change

`vercel.json` runs `scripts/ignore-build.mjs` before installation. It compares the current
commit with `VERCEL_GIT_PREVIOUS_SHA` (the last successful deployment for that branch), so a
multi-commit push cannot hide an earlier documentation edit. Only these inputs trigger a
build:
- `website/**`
- `object-editor/crates/wc3-data/data/**` (the shared field guides)
- `vercel.json` and `.vercelignore`

Other editor, map, changelog and root README changes skip the build. Vercel can still show a
skipped deployment/check; it does not install dependencies, build or replace the live site.
An initial deployment, unavailable Git history or `DOCS_FORCE_BUILD=1` builds conservatively.

The project-level Ignored Build Step also skips branches where the docs script is absent.
This protects the live site while the documentation PR has not yet been merged into main.

Default canonical URL: `https://tinkerbench-docs.vercel.app`. Set `DOCS_SITE` to change it.
`DOCS_BASE` supports a subpath such as `/wc3-toolkit` for a static host like GitHub Pages;
set it consistently for the build and `check:links`. The default is `/`.

The four original design studies are at `/designs/`; their search and navigation are
illustrative. The real documentation uses Starlight navigation, accessible search, a mobile
menu, a theme switcher and highlighted code.
