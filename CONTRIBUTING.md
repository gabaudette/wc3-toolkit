# Contributing

Contributions are welcome. This project follows the AGE Studio engineering standards, included
in full in [docs/standards/](docs/standards/README.md). The rules below summarize how they apply
to this repository; the rule IDs refer to those documents.

## Validation

Every change must pass these checks with zero errors and zero warnings. Run them from
`object-editor/`:

```bash
cargo fmt-check
cargo lint
cargo test --workspace
```

`cargo lint` runs Clippy on all targets with warnings as errors (see `.cargo/config.toml`).
Lint levels live in `object-editor/Cargo.toml` under `[workspace.lints]`. CI runs the same
checks on every push and pull request.

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
- The TypeScript rules (**AGE-TS**: `type` over `interface`, Biome, no `enum`, no ternaries,
  no `any`, no magic numbers) apply to the Tauri frontend once it exists.

## Game files

Never commit Blizzard game files or assets (models, textures, icons, data tables). Tools read
them from the user's own installation at runtime.

## Commits

Use conventional commit prefixes: `feat:`, `fix:`, `docs:`, `style:`, `refactor:`, `test:`,
`ci:`, `chore:`.
