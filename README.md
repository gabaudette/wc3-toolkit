# wc3-toolkit

Open-source tools for Warcraft III: Reforged map creation.

> **Status: early development.** Nothing here is ready for everyday use yet.

## Tools

### Object Editor (`object-editor/`, in progress)

An external replacement for the World Editor's Object Editor, built with Rust and Tauri. It
reads and writes the same object data as the World Editor, with a better interface:

- one search box for names, raw codes and field values;
- fields grouped by topic with full names and a description of what each one does;
- edit in place, with changed fields highlighted and a reset per field;
- ability cards showing icon, tooltip and per-level values wherever an ability is referenced;
- live tooltip preview with color codes;
- a 3D model viewer;
- share codes to copy objects between maps.

The first version covers the Units tab (units, heroes, buildings). See
[docs/pet-peeves.md](docs/pet-peeves.md) for the full list of goals.

### Map script harness (`map/`)

Injects Lua modules from `map/src/` into a development copy of a map, so custom code and the
World Editor never overwrite each other. Temporary and Linux-only for now.

## Requirements

- A legal copy of Warcraft III: Reforged. The tools read game data from your installation;
  this repository contains no Blizzard files.
- Rust (stable), a C/C++ compiler and CMake (for the bundled CascLib and StormLib).

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
