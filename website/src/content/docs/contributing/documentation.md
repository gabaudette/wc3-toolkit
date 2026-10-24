---

title: "Documentation contributions"
description: "Preview the website, edit authored guides and shared field explanations, and validate documentation changes."
---

You can improve the guides and website without installing Warcraft III. The build uses
checked-in declaration and field metadata, alongside the project’s own explanations.

## Run the website locally

Use Node.js 24 and Bun 1.4.2. After cloning the
[repository](https://github.com/gabaudette/wc3-toolkit), run these commands from its root:

```sh
cd website
bun install --frozen-lockfile
bun run dev
```

Open the local URL printed by Astro. The development command generates the reference pages
before starting the server. Most authored Markdown changes appear automatically. After
editing a field-guide TOML file or a metadata snapshot, rerun `bun run generate` to refresh
the reference pages.

## Edit the right source

| What you want to change                            | Source                                             |
| -------------------------------------------------- | -------------------------------------------------- |
| Editor guides, tutorials or scripting explanations | `website/src/content/docs/`                        |
| Object-field explanations shared with the editor   | `object-editor/crates/wc3-data/data/*-fields.toml` |
| API declaration metadata                           | `website/data/api.json`                            |
| Field display names, types, ranges and choices     | `website/data/fields.toml`                         |
| Lifecycle diagrams and guide template              | `website/data/lifecycles.md`                       |
| Paired JASS/Lua recipes                            | `website/data/examples.json`                       |
| Application screenshots                            | `website/public/screenshots/`                      |

Pages under `website/src/content/docs/reference/{api,fields,examples,lifecycles}/` are generated and
ignored by Git. Edit their source guides, snapshots or generator instead of changing those
Markdown files directly. The lifecycle template embeds existing paired recipes through
`<!-- lifecycle-example: Name -->` markers; edit the shared recipe source to change its code.

Refreshing API and field snapshots requires a local game installation. Follow
[the extraction and refresh instructions](https://github.com/gabaudette/wc3-toolkit/blob/main/website/README.md#refresh-local-api-and-metadata)
and review both snapshot diffs. Keep game scripts, implementations, comments and standalone
game assets out of the repository.

## Write and illustrate a guide

- Describe the map-making task and the result readers should see. Use the application’s
  actual labels and mention raw codes when they help identify an object or field.
- Explain setup and cleanup for scripting examples. Keep JASS and Lua recipes equivalent;
  the global language toggle should switch both code and language-specific explanations.
- Mark behavior that has not been confirmed in game instead of presenting it as verified.
- Use real application captures for UI guides, with descriptive alt text and captions.
  Reuse existing screenshot files when showing the same view. Add “Image subject to change.”
  below each screenshot, using the existing screenshot markup.
- Use relative links to site pages and assets so the guide also works under a subpath.

For screenshot capture conventions and generator details, see the
[website README](https://github.com/gabaudette/wc3-toolkit/blob/main/website/README.md).

## Validate and preview the result

From `website/`, run these sequentially:

```sh
bun run test
bun run generate
bun run check
bun run build
bun run check:links
bun run preview
```

Generation owns its output directories, so let it finish before running Astro checks or a
build. `check` runs Biome and Astro diagnostics; `check:links` validates local links and assets
in the built site. Fix errors and warnings before opening a pull request.

Search is built during production builds. Test it with `bun run preview` rather than the
development server. Check the changed pages at desktop and mobile widths, and confirm that
language switching, links and any screenshots work.

Update the changelog and devlog, then follow the [pull request checklist](../#open-a-pull-request).
