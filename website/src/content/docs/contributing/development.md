---

title: "Local development"
description: "Set up Tinkerbench, understand the repository and run the editor and Rust checks."
---

Use this guide for editor and toolkit changes. If you only want to change the website, start
with [documentation contributions](../documentation/).

## Prepare your tools

Install Git, Bun, Rust through rustup, a C/C++ compiler and CMake. The Rust workspace’s
`rust-toolchain.toml` pins the compiler, rustfmt and Clippy versions; rustup installs them
when you run a Cargo command in `object-editor/`.

The desktop app also needs [Tauri’s system prerequisites](https://v2.tauri.app/start/prerequisites/).
On Linux, these include the WebKitGTK libraries. Use your own legal Warcraft III: Reforged
installation to load game data and try the editor.

## Clone and run the editor

```sh
git clone https://github.com/gabaudette/wc3-toolkit.git
cd wc3-toolkit/object-editor/app
bun install
bun run tauri dev
```

`bun install` also installs the Git hooks. The Tauri development command starts the frontend
and desktop app. Choose your game folder and open a test map as shown in the
[visual tour](../../editor/visual-tour/).

## Find the relevant code

| Directory                               | What belongs here                                                 |
| --------------------------------------- | ----------------------------------------------------------------- |
| `object-editor/app/src/`                | React interface, field controls, previews and frontend tests.     |
| `object-editor/app/src-tauri/`          | Tauri commands, application state and the bridge to Rust data.    |
| `object-editor/crates/wc3-data/`        | Game data readers, field guides, object formats and art decoding. |
| `object-editor/crates/wc3-map/`         | Map archives, object edits, sharing, bundles and the Lua harness. |
| `object-editor/crates/casc/` and `mpq/` | Game storage and map archive wrappers.                            |
| `website/`                              | The documentation site, generators and declaration snapshots.     |

## Check frontend changes

From `object-editor/app/`, run:

```sh
bun run check
bun run test
bun run build
```

Biome owns formatting and linting; `bun run lint:fix` applies its fixes. The TypeScript check
and tests still need to pass after formatting.

## Check Rust changes

Build the frontend first with `bun run build` from `object-editor/app/`: the Rust app embeds
that output. Then, from `object-editor/`, run:

```sh
cargo fmt-check
cargo lint
cargo test --workspace
```

Use `cargo fmt --all` to fix Rust formatting. `cargo lint` runs Clippy on all targets with
warnings treated as errors. The Git hooks run the checks that apply to staged editor and
Rust files; run the website checks separately for documentation changes.

## Test real game data and maps

Tests that need a game installation or real maps are ignored by default. Set these variables
in your shell before running them:

| Variable       | Value                                               |
| -------------- | --------------------------------------------------- |
| `WC3_GAME_DIR` | Your Warcraft III installation directory.           |
| `WC3_MAPS_DIR` | A directory containing test `.w3m` and `.w3x` maps. |

Then, from `object-editor/`, run:

```sh
cargo test --workspace -- --ignored
```

The real-map tests check that object data files and string tables round-trip byte for byte.
Use copies for manual tests, and close a map in the World Editor before saving from Tinkerbench.
Confirm gameplay changes in Warcraft III; a parser or UI test does not establish in-game
behavior.

Keep game files and local extraction output outside the repository. Follow the
[pull request checklist](../#open-a-pull-request) when the change is ready.
