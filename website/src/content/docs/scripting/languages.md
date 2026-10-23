---

title: "Scripting basics"
description: "Understand game API calls, raw codes, callbacks and handles in your selected language."
---

Choose your map’s script language in the World Editor. This guide follows the language selected in the top bar.

The [API reference](../../reference/api/) is generated from the installed `common.j` and `blizzard.j` declarations. It documents function signatures and engine types; individual examples explain how to use selected functions.

## Read an API page

- **Native** means an engine-provided function.
- **Function** means a helper declared in Blizzard’s script library. Its body is not reproduced here.
- **Type** shows a game handle’s parent type.
- **Constant** shows a named value and its engine type. Initializers are not included.

Parameters keep the names and order from the source. Names such as `whichUnit` are placeholders: pass your own unit value. Generated call syntax shows the argument order; authored examples provide the setup and values for a complete task.

## Create a unit

Inside a trigger action or other initialized map code:

<section data-example-language="jass">

Use a four-character raw code literal for the unit type:

```jass
local unit footman = CreateUnit(Player(0), 'hfoo', 0.0, 0.0, 270.0)
```

</section>

<section data-example-language="lua">

Convert the unit’s four-character raw code to an integer with `FourCC`:

```lua
local footman = CreateUnit(Player(0), FourCC("hfoo"), 0.0, 0.0, 270.0)
```

</section>

Player indices start at zero. The raw code identifies the object type; the name shown to players is not its object ID.

## Callbacks

Callbacks let a trigger or timer run your code when an event happens.

<section data-example-language="jass">

Pass a named callback with `function CallbackName`. Declare that callback before the function that registers it, then call `StartExampleTimer()` from initialized map code:

```jass
function OnExampleTimerExpired takes nothing returns nothing
    local timer expired = GetExpiredTimer()
    call DestroyTimer(expired)
    set expired = null
endfunction

function StartExampleTimer takes nothing returns nothing
    local timer t = CreateTimer()
    call TimerStart(t, 1.0, false, function OnExampleTimerExpired)
    set t = null
endfunction
```

`nothing` means no arguments or no return value. Use `null` for an empty handle reference, and `true` or `false` for booleans.

</section>

<section data-example-language="lua">

Pass a function value or an inline closure. Run this in initialized map code, such as a trigger action or a [harness startup callback](../../editor/lua-harness/):

```lua
local t = CreateTimer()
TimerStart(t, 1.0, false, function()
    DestroyTimer(t)
end)
```

Omit arguments for a function that takes none. Use `nil` for an empty reference, `true` or `false` for booleans, and ordinary quotes for strings. Engine parameter types in the API reference describe expected values; they are not type annotations to paste into your code.

</section>

## Handles and cleanup

Units, timers, groups, locations and triggers are engine handles. Clearing a variable does not destroy its engine object. Destroy temporary timers and groups and remove locations when they are no longer needed. Do not destroy shared handles that another system owns.

<section data-example-language="jass">

Clear local handle references with `null` when appropriate after releasing the owned resource. In the timer example, clearing the local reference does not stop the timer; the expiry callback destroys it.

</section>

<section data-example-language="lua">

Keep closures and periodic callbacks from retaining resources indefinitely. Clearing a variable with `nil` does not call the engine’s destroy function; release the owned engine resource first.

</section>

## Language reference

<section data-example-language="lua">

The harness syntax-checks Lua 5.3. Use the [official Lua 5.3 manual](https://www.lua.org/manual/5.3/) for language syntax. Warcraft III supplies its own runtime environment; the presence of a library in the standard manual does not guarantee the game exposes it.

</section>

<section data-example-language="jass">

Use the [API reference](../../reference/api/) for declarations, parameter types and return types. Add custom functions in declaration order and call their setup function once from a map initialization trigger.

</section>

See [your first trigger](../../guides/triggers/) for a complete event registration example.
