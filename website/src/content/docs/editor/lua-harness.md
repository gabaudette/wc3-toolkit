---

title: "The Lua harness"
description: "Build a development map with external Lua modules while preserving World Editor code."
---

The World Editor regenerates `war3map.lua` on save. The harness keeps your modules in ordinary files and appends them to the editor’s script in a **development copy** of the map.

## Prepare the map

Use a map configured for Lua scripting in the World Editor. Save it before running the command. A JASS map without `war3map.lua` cannot use this harness.

Create a folder of `.lua` modules. Only files directly in that folder are loaded; they run in alphabetical filename order, in a shared global environment. This is not a `require` package loader.

## Add a module

Save this as `map/src/hello.lua`, or use the repository’s `welcome.lua` example:

```lua
Dev.OnStart(function()
    local t = CreateTimer()
    TimerStart(t, 1.0, false, function()
        DestroyTimer(t)
        DisplayTimedTextToPlayer(Player(0), 0, 0, 10,
            "Hello from the Lua harness!")
    end)
end)
```

`Dev.OnStart` registers work to run after the editor’s `main()`. The timer delays the text until after the loading screen. The `Dev` helper exists only in maps built by this harness.

## Build the development copy

From `object-editor/` in the checkout:

```sh
cargo run --release -p wc3-map --bin wc3map -- path/to/test.w3m lua ../map/src
```

This syntax-checks all modules as Lua 5.3 before writing anything. The original map stays unchanged. The default output is `test_dev.w3m` (or `test_dev.w3x`), and the map name gets a `[DEV]` prefix when the name can be resolved through its string table.

To choose a destination and inspect the final script:

```sh
cargo run --release -p wc3-map --bin wc3map -- path/to/test.w3m lua ../map/src -o path/to/test_dev.w3m --script path/to/combined.lua
```

## Iterate

Edit the original map in the World Editor and modules in your text editor. Save, rebuild the development copy, then launch that copy in Warcraft III. Do not save changes to the generated development copy and expect them to become part of the original.

Syntax checking catches parsing errors, not missing game APIs, invalid raw codes or runtime mistakes. Test those in game. The [JASS & Lua API](../../reference/api/) helps check parameter order and types.
