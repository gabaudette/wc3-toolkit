---

title: "Your first map"
description: "Change a unit, save the map, and verify the result in game."
---

For this first pass, change an existing unit’s maximum life. Tinkerbench edits object data; use the World Editor for terrain, placement and triggers.

## Prepare a small test map

1. In the World Editor, create a map and place a Footman for a player you can control.
2. Save it as a separate test map. Close the map in the World Editor before saving changes from Tinkerbench.
3. Open Tinkerbench and choose **Open map…**, then select that map.

## Change one field

1. Open **Units**.
2. Search for **Footman**, or its raw code `hfoo`.
3. Select the Footman and search its fields for **Hit Points Maximum** (`uhpm`).
4. Set the value to `1000`.
5. Notice the changed field highlight and its base value. Use the reset control if you want to undo that field.

The change affects that object type wherever the map uses it. To make a separate unit type, create a custom object instead of changing the game object. See [units and abilities](../objects/).

## Save and test

Save with **Ctrl S**. The editor saves through a temporary map copy and replaces the destination after a complete write. Keep your own map backup as well.

Reopen the saved map in the World Editor and test it in Warcraft III. The Footman should show `1000` maximum life before any bonuses or trigger changes. If it does not, check that you launched the same map you saved.

## What the editor saves

Tinkerbench writes object changes and the map’s string table. It does not become your terrain or trigger editor. Avoid keeping the same map open in two editors: a later save can overwrite the other tool’s work.
