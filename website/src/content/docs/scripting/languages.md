---

title: "Scripting basics"
description: "Read the same game API in two languages and understand raw codes, callbacks and handles."
---

A map chooses its script language in the World Editor. The game exposes many of the same API names in JASS and Lua, but the calling syntax differs. The [API reference](../../reference/api/) is generated from the installed `common.j` and `blizzard.j` declarations; it is a declaration reference, not a complete explanation of every function’s behavior.

## Read an API page

- **Native** means an engine-provided function.
- **Function** means a helper declared in Blizzard’s script library. Its body is not reproduced here.
- **Type** shows a game handle’s parent type.
- **Constant** shows a named value and its JASS type. Initializers are not included.

Parameters keep the names and order from the source. Names such as `whichUnit` are placeholders: pass your own unit value. Generated Lua call syntax describes how to call the API; authored examples provide real context for selected functions.

## Create a unit

Inside a trigger action or other initialized map code:

<section data-example-language="jass">

```jass
local unit footman = CreateUnit(Player(0), 'hfoo', 0.0, 0.0, 270.0)
```

</section>

<section data-example-language="lua">

```lua
local footman = CreateUnit(Player(0), FourCC("hfoo"), 0.0, 0.0, 270.0)
```

</section>

Player indices start at zero. The four-character raw code becomes an integer: JASS uses `'hfoo'`, Lua uses `FourCC("hfoo")`. Names shown to players are not object IDs.

## Callbacks

JASS passes a named callback with `function CallbackName`. Declare that callback before the code that uses it. Lua can pass a function value or an inline closure.

<section data-example-language="lua">

```lua
local t = CreateTimer()
TimerStart(t, 1.0, false, function()
    DestroyTimer(t)
end)
```

</section>

`nothing` means no arguments or no return value in a JASS declaration. It is not an argument to pass in Lua. Lua booleans use `true` and `false`; strings use ordinary quotes.

## Handles and cleanup

Units, timers, groups, locations and triggers are engine handles. Releasing a Lua variable does not necessarily destroy its engine object. Destroy temporary timers and groups and remove locations when they are no longer needed. Do not destroy shared handles that another system owns.

In JASS, clear local handle references when appropriate after releasing the owned resource. In Lua, keep closures and periodic callbacks from retaining resources indefinitely.

## Language reference

The harness syntax-checks Lua 5.3. Use the [official Lua 5.3 manual](https://www.lua.org/manual/5.3/) for language syntax. Warcraft III supplies its own runtime environment; the presence of a library in the standard manual does not guarantee the game exposes it.

See [your first trigger](../../guides/triggers/) for complete JASS and Lua registration examples.
