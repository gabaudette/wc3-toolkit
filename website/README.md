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
- `../object-editor/crates/wc3-data/data/*-fields.toml`: project-authored field explanations,
  shared with the editor. Every guide ID gets a page, including IDs absent from the snapshot.
- `data/fields.toml`: display names, types, limits, choices and applicability extracted from
  the local installation. Includes spell-specific fields beyond the authored guides.
- `data/api.json`: API declaration metadata, source hashes and the snapshot version label.
- `data/examples.json`: authored JASS and Lua examples attached to selected API entries.
- `scripts/jass-language.mjs`: project-authored JASS highlighting grammar.

Generated pages in `src/content/docs/reference/{api,fields}/` are ignored. Every build
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
