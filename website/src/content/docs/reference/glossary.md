---
title: "Map-making glossary"
description: "Definitions and examples for raw codes, base and custom objects, fields, dependencies, handles, natives, Blizzard helpers, JASS, Lua and map-file terms."
tableOfContents:
  maxHeadingLevel: 2
---

Use this glossary when a field explanation or scripting example introduces an unfamiliar
term. Each definition links to a guide or reference for the next step. The entries follow
the terminology used by the project's existing guides and supported map formats.

Jump to [A–C](#ac), [D–H](#dh), [J–P](#jp) or [R–T](#rt). Every term heading has its own link,
so you can share a definition directly. To search this page from anywhere on the site, use
**Ctrl K** (or **⌘ K**) and choose **Guides** under **Search in**.

The **Lua/JASS** toggle changes examples and language-specific notes here. Choose the map's
actual scripting language in the World Editor; changing the website toggle does not convert a map.

## A–C

### API

An **application programming interface**: the functions, types and named values your script
uses to work with the game. The [API reference](../api/) lists engine-provided natives and
Blizzard script helpers. For example, [CreateUnit](../api/common/native/createunit/) creates
a unit and returns a handle to it. An API declaration describes a call, not a complete map script.

### Argument and parameter

A **parameter** is a named input slot in a function declaration; an **argument** is the value
you supply in that slot. In `CreateUnit`, `unitid` is a parameter. The Footman type ID is one
possible argument. Keep the declared order and engine types when supplying values.
See [reading an API page](../../scripting/languages/#read-an-api-page) and [FourCC](#fourcc).

### Base object

The object type from which a [custom object](#custom-object) starts. A custom unit based on
Footman inherits the base's values except where the map stores overrides. **Base value** in
the editor is the value shown for comparison with an edited field; it is not the custom
object's new raw code. See [game objects and custom objects](../../editor/objects/#game-objects-and-custom-objects).

### Blizzard helper

A function implemented in Blizzard's script library rather than a native engine call.
Helpers are often called **BJ functions**, after the `BJ` suffix used by many of them.
[TriggerRegisterAnyUnitEventBJ](../api/blizzard/function/triggerregisteranyuniteventbj/), for
example, registers a player-unit event across player slots. Check the API page's **function**
kind and source file; the suffix alone is not the declaration kind. See [scripting basics](../../scripting/languages/#read-an-api-page).

### Bundle file

A `.w3obj` file carrying shared object definitions and the imported files the toolkit can
discover for them. A bundle is a transfer package, not an entire map. It can include model
textures as well as the model itself; undiscoverable references can still leave files missing.
See [saving and importing a bundle](../../editor/sharing/#save-a-bundle-file) and [dependency](#dependency).

### Callback

A function you pass to another function so it can run your code at the relevant moment.
For example, [TimerStart](../api/common/native/timerstart/) takes a callback to run when its
timer expires. Passing a callback is different from calling it immediately.

<section data-example-language="jass">

In JASS, after declaring a named callback, pass it as `function OnTimerExpired`.

</section>

<section data-example-language="lua">

In Lua, pass the function value, such as `OnTimerExpired`, or an inline function. Writing
`OnTimerExpired()` calls it now instead.

</section>

See the complete [callback examples](../../scripting/languages/#callbacks).

### Constant

A named value provided by the scripting API, such as `EVENT_PLAYER_UNIT_DEATH`. Use its name
where the matching engine type is expected. A constant is not a function to call. The
[constant reference entry](../api/common/constant/event_player_unit_death/) records its type;
this site's declaration snapshot does not include its initialization value.

### Custom object

An object type defined by the map with its own [raw code](#raw-code), based on an existing
object. Editing a game object's fields does not by itself create a custom object. For
example, the sharing preview can contain a custom unit with code `H000`, distinct from its
base. See [creating a separate object type](../../editor/objects/#game-objects-and-custom-objects)
and [raw-code collisions during import](../../editor/sharing/#raw-code-collisions).

## D–H

### Dependency

Another object or file needed by the object you are transferring. A custom shop can refer
to a custom item in [Items Sold (`usei`)](../fields/unit/usei/); a model can refer to textures.
These are object dependencies and asset dependencies respectively. The sharing workflow
follows supported references, but does not rewrite IDs embedded in arbitrary scripts or
text. See [sharing objects](../../editor/sharing/) and [import discovery limits](../../formats/imports/#protected-maps).

### Destructable

A map object such as a tree, gate or crate that can be destroyed. **Destructables** is the
application's tab name; you may also encounter the spelling **destructible** in ordinary
text. Its [Hit Points (`bhps`)](../fields/destructable/bhps/) field differs from a unit's
maximum-life field. Browse [destructable fields](../fields/destructable/) or [find a field by goal](../field-tasks/).

### Doodad

A decorative map object such as a prop, plant or light, edited in the **Doodads** tab.
Its [Model File (`dfil`)](../fields/doodad/dfil/) and [Pathing Texture (`dptx`)](../fields/doodad/dptx/)
are separate fields. Doodad color fields are edited per model variation. See the
[other-tabs notes](../../editor/objects/#other-tabs), including the current variation-writing limitation.

### Event response

A value read from the event currently being handled. For example,
[GetTriggerUnit](../api/common/native/gettriggerunit/) reads the unit from a matching
unit-event context. It does not find whichever unit is selected. Capture the needed value
before deferring work to a timer; the original event context is not a general lookup service.
See [GetTriggerUnit's usage and caveats](../api/common/native/gettriggerunit/#how-to-use-it).

### Field

An individual setting on an object type, identified by a four-character **field ID** and a
display label. For example, the unit field `uhpm` is
[Hit Points Maximum (Base)](../fields/unit/uhpm/). A field ID and an object's raw code have
different roles: `uhpm` names a setting, while `hfoo` names the Footman type. Use
[Which field do I change?](../field-tasks/) to start with a goal instead of an ID.

### Field override

A value stored by the map in place of an object's inherited value. Setting the Footman's
`uhpm` field to `1000` in the [first-map walkthrough](../../editor/first-map/#change-one-field)
stores an override for that field. Resetting a field removes its override so the base value
applies again. See [finding, changing and resetting fields](../../editor/objects/#find-and-change-fields).

### FourCC

Short for **four-character code**. In scripting, a raw code must be supplied in the form
expected by the API, often an integer rather than a display-name string.

<section data-example-language="jass">

In JASS, `'hfoo'` is a raw-code literal you can pass as the integer `unitid` argument to
`CreateUnit`. It is different from the string `"hfoo"`.

</section>

<section data-example-language="lua">

In Lua, `FourCC("hfoo")` converts the four-character string to the integer type ID expected
by `CreateUnit`. Passing just `"hfoo"` does not perform that conversion.

</section>

See the [CreateUnit example](../api/common/native/createunit/#example) and [raw code](#raw-code).

### Game object

An object type supplied by the Warcraft III installation, such as Footman `hfoo`. Tinkerbench
can browse these objects read-only without a map. In an open map, changes to a game object
are stored as map overrides. This term does not mean a particular Footman standing on the
map; that is a [placed instance](#placed-instance). See [game objects and custom objects](../../editor/objects/#game-objects-and-custom-objects).

### Handle

An engine value referring to a runtime object, such as a unit, timer, group, trigger or
location. A raw code identifies an object type; the unit handle returned by `CreateUnit`
refers to the created unit. A handle is also not a plain table you construct yourself.
See [handles and cleanup](../../scripting/languages/#handles-and-cleanup) and [ownership](#ownership-and-cleanup)
before releasing an engine resource.

For a concrete example, [compare coordinates and location handles](../common-confusions/#coordinates-versus-location-handles).

## J–P

### JASS

One of Warcraft III's map scripting languages. Its declarations explicitly name parameter
and return types, and its callback syntax uses `function CallbackName`. The JASS view in the
[API reference](../api/) shows declarations from the recorded script snapshot.
See [scripting basics](../../scripting/languages/) for complete usage examples.

### Lua

The other map scripting language covered by this guide. The Lua view shows call syntax
using the same API names and argument order, without JASS type annotations. Engine calls
come from Warcraft III; a standard Lua library being documented does not establish that the
game exposes it. See the [language-reference notes](../../scripting/languages/#language-reference).

### Lua harness

The toolkit's development-map builder for external Lua modules. It appends modules to the
World Editor's generated Lua script in a development copy of a map. `Dev.OnStart` is a
helper supplied by that harness, not a game native available in every map. See
[the Lua harness](../../editor/lua-harness/) for its loading order, commands and limits.

### Map archive

The container holding a map's named files. In the **MPQ** maps handled by this toolkit,
object changes, strings, scripts and imported assets are separate archive members. A map
file such as `.w3x` is not the same thing as a `.w3obj` bundle. See
[object-data files](../../formats/object-data/#files-and-object-kinds) and [imports and protected maps](../../formats/imports/).

### Native

An engine-provided function, declared with the `native` kind in the API snapshot.
[CreateTimer](../api/common/native/createtimer/) is one example. A native's declaration tells
you its inputs and return type; its engine implementation is not reproduced in this
reference. Compare [Blizzard helper](#blizzard-helper), or read [an API page](../../scripting/languages/#read-an-api-page).

### Object type

The reusable definition identified by an object raw code. Editing a type changes the
settings the map uses for that type; it does not select one unit's position on the terrain.
Both game objects and custom objects are types. See [placed instance](#placed-instance)
and the [first-map guide](../../editor/first-map/) for the distinction in an editing workflow.

### Ownership and cleanup

The responsibility your code takes for releasing a resource it creates or retains. This
is different from a unit's owning player. A system that creates a temporary timer normally
arranges [DestroyTimer](../api/common/native/destroytimer/) when finished; simply clearing
a variable does not destroy the engine timer. A borrowed event or enumeration result does
not automatically become yours to destroy. See [handles and cleanup](../../scripting/languages/#handles-and-cleanup)
and the [practical API guides](../api/practical/).

The [visual scripting lifecycles](../lifecycles/) trace those responsibilities for timers,
groups and locations, including the difference between retaining and releasing a handle.

### Per-level field

A field with a separate value for each ability or upgrade level, rather than one shared
value. The editor presents these in a level table. In the doodad editor, the repeated color
values instead refer to model variations. See [ability levels](../../editor/objects/#ability-levels)
and the [stored level/column layout](../../formats/object-data/#versions-and-levels).

### Placed instance

One particular copy of an object type on a map, such as one of several Footmen placed in
the World Editor. Tinkerbench edits object-type data; terrain and placement remain World
Editor work. At runtime, a unit handle refers to an individual unit, including units created
by scripts. See [what the editor saves](../../editor/first-map/#what-the-editor-saves) and [object type](#object-type).

## R–T

### Raw code

A four-character identifier, also called a **rawcode** or **object ID** for an object type.
For example, `hfoo` identifies the Footman type, while a custom object has its own code.
Field IDs also use four-character codes, but identify settings rather than object types.
Keep the exact spelling and case when copying a code. Display names and `TRIGSTR` references
are not substitutes for object IDs. See [FourCC](#fourcc), [field](#field), and
[import collisions](../../editor/sharing/#raw-code-collisions).

### Return value

The result a function gives back to its caller. For example,
[CreateGroup](../api/common/native/creategroup/) returns a new group handle; `DestroyGroup`
has no return value. A returned handle can be newly allocated or borrowed, so check that
API page's ownership notes rather than assuming every result needs destruction.
See [API parameter and return types](../../scripting/languages/#read-an-api-page).

### Share code

A `W3OBJ1:` text package containing object definitions for the sharing workflow. Unlike a
bundle file, it does not include imported models, textures or icons. Pasting a code opens an
import preview; adding the objects and saving the destination map are separate steps. See
[sharing and importing a code](../../editor/sharing/#share-a-code) and [bundle file](#bundle-file).

### Skin file

A separate map object-data file, such as `war3mapSkin.w3u`, used for some cosmetic overrides
in Reforged. The editor reads gameplay and skin data together when reconstructing an object.
Which fields belong in skin data follows metadata; do not assume every art-looking field
has the same storage rule. See [the supported skin-file behavior](../../formats/object-data/#skin-files).

### String table

The map's **WTS** file, `war3map.wts`, which maps numeric IDs to text. An object name or tooltip
can store a token such as `TRIGSTR_007`; entry `7` supplies its displayed text. This is a text
reference, not an object's raw code. Transfers must resolve source text and allocate destination
entries rather than blindly copying the token. See [map strings](../../formats/strings/).

### Tooltip placeholder

A token such as `<ID,Column>` in tooltip text that asks for an object-data value. Tinkerbench's
preview resolves supported placeholders from game and map data. It is different from a
`TRIGSTR` string-table reference, and changing tooltip wording does not change the gameplay
field being described. See [tooltips and art](../../editor/objects/#tooltips-and-art).

### Trigger

An engine object to which you can attach event registrations, conditions and actions. An
event can cause its action callbacks to run; creating a trigger alone does not register an
event. A map-lifetime listener and a temporary listener need different cleanup plans.
See [CreateTrigger](../api/common/native/createtrigger/),
[TriggerAddAction](../api/common/native/triggeraddaction/), and the
[unit-death event example](../api/common/native/gettriggerunit/#example).
