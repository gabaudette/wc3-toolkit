---
title: "Commonly confused concepts"
description: "Choose between Armor Type and Defense Type, RemoveUnit and KillUnit, or numeric coordinates and location handles, with examples and cleanup guidance."
sidebar:
  label: "Common confusions"
tableOfContents:
  maxHeadingLevel: 2
---

Start with the decision you need to make:

- [Which armor field changes damage?](#armor-type-versus-defense-type)
- [Should I remove or kill a unit?](#removing-versus-killing-a-unit)
- [Do I need X/Y coordinates or a location handle?](#coordinates-versus-location-handles)

The [glossary](../glossary/) defines the terms. These comparisons explain when to choose
each option and which mistakes to avoid. Follow the reference links for full parameter,
field-value and ownership notes.

**Review status:** These explanations were checked against the project's field guides,
metadata and API guidance. The scenarios and engine edge cases described here have not
been exercised in game for this page. Test the intended result in your map and game build.

## Armor Type versus Defense Type

The three unit fields below control different parts of an attack's result:

| What you want to change | Field on the Units tab |
| --- | --- |
| The material used for the sound when a unit is hit | [Armor Type (`uarm`)](../fields/unit/uarm/) |
| The defense category used by the damage table | [Defense Type (`udty`)](../fields/unit/udty/) |
| The unit's base armor amount | [Defense Base (`udef`)](../fields/unit/udef/) |

The attacker's [Attack 1 - Attack Type (`ua1t`)](../fields/unit/ua1t/) is the other side of
the attack-type/defense-type damage table. The map's damage table and other modifiers matter;
this guide does not assume a fixed multiplier for a given pairing.

### Choose the field by the intended result

For a unit that should use a metal hit material, choose **Metal** in **Armor Type**. That is
a sound-material choice; it does not select a defense category or add base armor.

For a unit that should use the **Fortified** defense category, change **Defense Type**.
If you instead want to adjust its base armor amount, change **Defense Base**. Keep the
other settings fixed while checking which change produced the intended result.

### Common mistakes

- Looking for a damage reduction setting under the label **Armor Type**. Use Defense Type
  for the damage-table category and Defense Base for the armor amount.
- Assuming a defense category is a complete damage prediction. The attacker's attack type,
  the map's settings and gameplay bonuses can also affect the result.
- Using these unit fields for a destructable or item. Select the relevant object tab and
  follow its field reference rather than matching only a similar label.

**Check in game:** Hit-sound output and numerical damage for these example choices have
not been verified here. Use a small test map and compare the same attacker and target
before and after changing one field.

Continue with [the armor task](../field-tasks/#change-armor-or-defense-type) or
[find and change fields](../../editor/objects/#find-and-change-fields).

## Removing versus killing a unit

Both APIs act on an existing unit reference. Choose according to the gameplay or cleanup
operation you intend:

| Your intention | API to consider |
| --- | --- |
| Retire a temporary unit that your system no longer needs | [RemoveUnit](../api/common/native/removeunit/) |
| Make a unit die as part of the map's gameplay | [KillUnit](../api/common/native/killunit/) |
| Remove a unit only from a group collection | [GroupRemoveUnit](../api/common/native/groupremoveunit/) |

### Choose death or removal deliberately

A system that created a temporary helper unit can retire it with `RemoveUnit` when its work
is finished. Use the existing [temporary-unit cleanup recipe](../api/common/native/removeunit/#example)
for paired JASS/Lua syntax and its ownership requirements.

If the design calls for a unit to die, choose `KillUnit` and handle that death through your
map's intended logic. The existing [unit-death event recipe](../api/common/native/gettriggerunit/#example)
shows how to register a listener and read the event unit. It is an event-response example,
not proof that removal and killing produce the same events or results.

<section data-example-language="jass">

Inside an existing JASS function, with a valid unit reference in `whichUnit`, a native with
no return value is invoked as `call KillUnit(whichUnit)` or `call RemoveUnit(whichUnit)`.
Choose one operation for the intended behavior, rather than calling both as routine cleanup.

</section>

<section data-example-language="lua">

Inside initialized Lua map code, with a valid unit reference in `whichUnit`, use
`KillUnit(whichUnit)` or `RemoveUnit(whichUnit)` for the intended operation. Choose one
operation deliberately, rather than calling both as routine cleanup.

</section>

### Ownership and common mistakes

- Removing a unit, killing it and dropping group membership are separate operations.
  `GroupRemoveUnit` changes the collection; it does not remove the unit from the map.
- A borrowed event unit is not a resource to remove merely because an action finished
  reading it. Resource ownership means cleanup responsibility, not the unit's owning player.
- After removal, stop using the old unit reference and clear your system's stored references.
  Clearing a variable by itself does not remove the unit.
- Neither call releases unrelated timers, groups or locations created by your code. Plan
  those lifetimes separately. Killing a unit is not a general handle-cleanup function.

**Unverified behavior:** This page does not establish death-event ordering, kill credit,
bounty, corpse handling, revival behavior or behavior during already-running callbacks.
Do not substitute removal for a death workflow based on those assumptions, or append
`RemoveUnit` to every death handler when your map may still need the unit.

See [RemoveUnit's ownership notes](../api/common/native/removeunit/#ownership-and-cleanup)
and [event response](../glossary/#event-response) for the surrounding context.

## Coordinates versus location handles

Both can describe an X/Y position, but their return values and cleanup requirements differ:

| What you need | API and ownership |
| --- | --- |
| Plain X/Y numbers for a calculation or coordinate-taking API | [GetUnitX](../api/common/native/getunitx/) and [GetUnitY](../api/common/native/getunity/); no location is allocated |
| A unit's position as a location handle | [GetUnitLoc](../api/common/native/getunitloc/); the caller owns the new location |
| A location from known X/Y numbers | [Location](../api/common/native/location/); the caller owns the new location |

### Prefer the form the next operation needs

To display a unit's coordinates or pass X/Y numbers to another API, use the existing
[numeric-coordinate recipe](../api/common/native/getunitx/#example). The returned numbers
do not need `RemoveLocation`, and reading them does not transfer ownership of the unit.

When the receiving API requires a `location`, keep the returned handle until its last use,
then call [RemoveLocation](../api/common/native/removelocation/). The existing
[unit-position location recipe](../api/common/native/getunitloc/#example) reads the position,
extracts X/Y and releases its temporary location. The
[known-point recipe](../api/common/native/removelocation/#example) shows the same cleanup
when the point was allocated with `Location`.

### Ownership and common mistakes

- `GetUnitLoc` creates a new location each time, including repeated calls for the same unit.
  Overwriting a variable does not remove the old location.
- A location from `GetUnitLoc` is a position snapshot; it does not follow later movement.
  Stored X/Y numbers are snapshots too. Read a fresh position when you need an update.
- `RemoveLocation` releases the point, not the unit at that point. Do not remove the source
  unit as cleanup for reading its position.
- Do not pass a coordinate number to `RemoveLocation`, or keep using a location after
  removing it. For a shared or borrowed point, agree on which system releases it.

<section data-example-language="jass">

For an owned temporary location stored in `point`, release the engine resource with
`call RemoveLocation(point)`, then clear a finished handle reference with `set point = null`.
The assignment alone does not release the point. See the linked recipes for complete functions.

</section>

<section data-example-language="lua">

For an owned temporary location stored in `point`, call `RemoveLocation(point)` after its
last use, then clear retained references with `point = nil` when finished. Lua garbage
collection does not replace the engine cleanup call. See the linked recipes for complete functions.

</section>

These ownership notes follow the existing API guides. Patch-specific behavior, invalid
handles and calls involving a unit already removed from the map have not been tested here;
use valid references and follow the linked API caveats.

Continue with [handles and cleanup](../../scripting/languages/#handles-and-cleanup) or
[ownership in the glossary](../glossary/#ownership-and-cleanup).
