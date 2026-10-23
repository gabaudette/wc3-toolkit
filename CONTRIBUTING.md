# Contributing

Contributions are welcome. This project follows the AGE Studio engineering standards, included
in full in [docs/standards/](docs/standards/README.md). The rules below summarize how they apply
to this repository; the rule IDs refer to those documents.

## Validation

Every change must pass these checks with zero errors and zero warnings.

Every check runs before each commit, through [lefthook](lefthook.yml): rustfmt, Clippy and
`cargo test --workspace` when Rust files are staged; Biome, the TypeScript check, the frontend
tests and the build when frontend files are. After a small change the whole hook takes a few
seconds. `bun install` in `object-editor/app` sets it up. CI does not lint; it builds and runs
the tests on a clean machine: on Windows, and on Linux for every crate but the app (which needs
system libraries that take minutes to install there). Fix what the hook reports with
`cargo fmt --all` and `bun run lint:fix`, and read the Clippy and test output for the rest.
Skip the hook only for work in progress that you will fix before pushing
(`git commit --no-verify`).

Run the checks by hand from `object-editor/`:

```bash
cargo fmt-check
cargo lint
cargo test --workspace
```

`cargo lint` runs Clippy on all targets with warnings as errors (see `.cargo/config.toml`).
Lint levels live in `object-editor/Cargo.toml` under `[workspace.lints]`.

Some tests read a real Warcraft III install and real maps. They are marked `#[ignore]` and
run with:

```bash
WC3_GAME_DIR="/path/to/Warcraft III" WC3_MAPS_DIR="/path/to/Maps" cargo test --workspace -- --ignored
```

The editor's frontend (`object-editor/app`) follows the AGE TypeScript rules through Biome:

```bash
cd object-editor/app
bun run check
bun run test
bun run build
```

The Rust app crate embeds the built frontend, so run `bun run build` once before
`cargo lint` or `cargo test`.

`WC3_MAPS_DIR` can be any folder of `.w3m`/`.w3x` maps; every object data file and string table
in them must rewrite byte for byte. Building also needs CMake (for the bundled StormLib).

## Rules

- **AGE-QUAL-001/002** — Zero errors, zero warnings. Warnings fail validation.
- **AGE-QUAL-003** — Fix the cause instead of silencing a diagnostic. Every `unsafe` block has a
  `// SAFETY:` comment. Any suppression is narrow and documented; today these are the
  compiler-warning suppression for the vendored CascLib and StormLib sources, and the CLI
  binaries' use of `eprintln!` for errors.
- **AGE-QUAL-004** — Do not relax lint or formatter configuration to make code pass.
- **AGE-QUAL-006** — `rustfmt` owns Rust formatting (default settings, 100 columns).
- **AGE-ERR-001/003** — Fallible code returns `Result`. Parsers report malformed data with
  line or row context instead of skipping it. No `unwrap`, `expect` or `panic!` outside tests.
- **AGE-WS-001/002** — One blank line after a completed block and between groups of
  declarations with a different purpose, where `rustfmt` does not decide otherwise.
- **AGE-TECH-005** — Platform-agnostic by default: no hardcoded OS or install paths in
  libraries, and cargo aliases or Rust tools instead of shell scripts.
- Library crates do not print to stdout or stderr; binaries may print intentionally.
- Frontend tests (`*.test.ts`) may use literal numbers: expected values are the point of a
  test. That is the only Biome override (`biome.json`).
- The TypeScript rules (**AGE-TS**: `type` over `interface`, Biome, no `enum`, no ternaries,
  no `any`, no magic numbers) apply to the Tauri frontend once it exists.

## Game files

Never commit Blizzard game files or assets (models, textures, icons, data tables). Tools read
them from the user's own installation at runtime.

## Changelog and devlog

Every pull request updates both, and the pre-push hook refuses a branch that changes neither
compared with `main`:

- [CHANGELOG.md](CHANGELOG.md) is for the people who use Tinkerbench, and the app shows it under
  "What's new" (click the version next to the name). Add plain-language entries under
  `## Unreleased`, in `### Added`, `### Changed` or `### Fixed`. A pull request that is only
  about speed is one line, "Performance improvements."; one that is only technical (CI,
  refactoring, tests, tooling) is "Internal improvements." Do not repeat a line already there.
- [docs/devlog.md](docs/devlog.md) is the technical story, newest first: one entry per pull
  request with the date, its number, what changed, why, and what was learned.

The changelog dialog understands only `##`, `###`, `- ` items (wrapped lines indented) and
inline code, bold and links.

## Releases

The version lives in one place: `version` in `object-editor/Cargo.toml` (the app, its
installers and the version shown in the editor all read it). To release:

1. Set the new version in `object-editor/Cargo.toml`, rename `## Unreleased` in
   `CHANGELOG.md` to `## 0.2.0 - <date>`, add a devlog entry for the release, and commit them on
   `main`. The next change starts a new `## Unreleased` section (an empty one would show in the
   app).
2. Tag that commit and push the tag: `git tag v0.2.0 && git push origin v0.2.0`.
3. The Release workflow checks that the tag matches the version, builds the Windows
   installer (NSIS) and the Linux packages (`.deb`, AppImage), and attaches them to a
   **draft** release.
4. Install and try them on Windows and Linux, then publish the draft on GitHub.

The Windows installer is not signed: Windows shows a SmartScreen warning ("Windows protected
your PC") until a code signing certificate is added. The install notes explain how to get past
it. The release notes come from `.github/release-notes.md`.

## Commits

Use conventional commit prefixes: `feat:`, `fix:`, `docs:`, `style:`, `refactor:`, `test:`,
`ci:`, `chore:`.
