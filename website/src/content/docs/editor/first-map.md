---

title: "Your first map"
description: "Change a unit, save the map, and verify the result in game."
---

For this first pass, change an existing unit’s maximum life. Tinkerbench edits object data; use the World Editor for terrain, placement and triggers.

## Prepare a small test map

1. In the World Editor, create a map and place a Footman for a player you can control.
2. Save it as a separate test map. Close the map in the World Editor before saving changes from Tinkerbench.
3. Start Tinkerbench, choose your Warcraft III folder and select **Load game data**. See
   [connect the game](../install/#connect-the-game) if this is your first start.
4. Under **2. Map**, choose **Open a map…**, then select your test map. If you are already in
   the workbench, use **Open map…** in its top bar instead.

<figure class="app-screenshot">
  <a href="../../screenshots/open-map.png">
    <img src="../../screenshots/open-map.png" alt="The startup Map section after game data loads, with buttons to reopen the last map, open a map or browse game data only." width="600" height="600" loading="lazy" />
  </a>
  <figcaption>Choose Open a map… to select your test map. Reopen appears when a previous map is remembered.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

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
