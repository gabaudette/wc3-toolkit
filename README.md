# wc3-toolkit

Open-source tools for Warcraft III: Reforged map creation.

> **Status: early development.** Nothing here is ready for everyday use yet.

Read the **[Tinkerbench field guide](https://wc3-toolkit-docs.vercel.app)** for installation,
first-map tutorials, JASS/Lua API declarations and the object field reference.
The documentation source and local build instructions are in [website/](website/README.md).

## Tools

### Tinkerbench, the Object Editor (`object-editor/`, in progress)

An external replacement for the World Editor's Object Editor, built with Rust and Tauri. It
reads and writes the same object data as the World Editor, with a better interface:

- one search box for names, raw codes and field values;
- fields grouped by topic with full names and a description of what each one does;
- edit in place, with changed fields highlighted and a reset per field;
- ability cards showing icon, tooltip and per-level values wherever an ability is referenced;
- live tooltip preview with color codes;
- a 3D model viewer;
- share codes and bundle files to copy objects between maps, with what they depend on.

It covers units (with heroes and buildings), abilities, items and upgrades. See
[docs/pet-peeves.md](docs/pet-peeves.md) for the full list of goals.

#### Install

Download the latest version from the
[Releases page](https://github.com/gabaudette/wc3-toolkit/releases):

- **Windows:** the `-setup.exe` file. It installs for your user only. The installer is not
  signed yet, so Windows may show "Windows protected your PC": click **More info**, then
  **Run anyway**.
- **Linux (Debian, Ubuntu, Mint):** the `.deb` file, installed with
  `sudo apt install ./<file>.deb`.
- **Linux (other):** the `.AppImage` file: make it executable (`chmod +x <file>.AppImage`)
  and run it.

On first start, point the editor at your Warcraft III folder.

#### Run from source

You need [Bun](https://bun.sh) and, on Linux, the WebKitGTK libraries Tauri uses
([Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)):

```bash
cd object-editor/app
bun install
bun run tauri dev
```

### Lua script harness (`wc3map lua`)

Builds a development copy of a map with Lua modules from a folder added to the World Editor's
script, so custom code and the editor never overwrite each other. The copy is named
`<map>_dev.w3m` and shows up as `[DEV] <map name>` in the game. Every module is syntax-checked
as Lua 5.3 before anything is written, and the original map is never modified.
`map/src/` holds example modules. Works on Windows, Linux and macOS.

```bash
cargo run --release -p wc3-map --bin wc3map -- path/to/test.w3m lua ../map/src
```

Modules register code to run at game start with `Dev.OnStart(function() ... end)`.

## Requirements

- A legal copy of Warcraft III: Reforged. The tools read game data from your installation;
  this repository contains no Blizzard files.
- Rust (rustup installs the version pinned in `object-editor/rust-toolchain.toml`), a C/C++
  compiler and CMake (for the bundled CascLib and StormLib).

## Building

```bash
cd object-editor
cargo build
```

To inspect the game data the editor works with, point the `wc3data` tool at your install:

```bash
export WC3_GAME_DIR="/path/to/Warcraft III"
cargo run --release -p wc3-data --bin wc3data -- heroes
cargo run --release -p wc3-data --bin wc3data -- unit Hpal
cargo run --release -p wc3-data --bin wc3data -- --hd icon Hpal paladin.png
```

To inspect or edit a map's units (changes are always saved through a temporary copy):

```bash
cargo run --release -p wc3-map --bin wc3map -- my_map.w3m units
cargo run --release -p wc3-map --bin wc3map -- my_map.w3m unit H000
cargo run --release -p wc3-map --bin wc3map -- my_map.w3m set H000 uhpm 1000 -o my_map_edited.w3m
cargo run --release -p wc3-map --bin wc3map -- my_map.w3m check
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the checks every change must pass.

## License

MIT, see [LICENSE](LICENSE). Bundled third-party code keeps its own license
(`object-editor/vendor/CascLib`, MIT).

Warcraft and Blizzard Entertainment are trademarks or registered trademarks of Blizzard
Entertainment, Inc. This project is not affiliated with or endorsed by Blizzard Entertainment.
