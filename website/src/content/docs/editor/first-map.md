---

title: "Your first map"
description: "Change a unit, save the map, and verify the result in game."
---

For this first pass, change an existing unit’s maximum life. Tinkerbench edits object data
and [placed units](../placed-units/); use the World Editor to prepare this example's terrain
and starting player.

## Prepare a small test map

1. In the World Editor, create a map and place a Footman for a player you can control.
2. Save it as a separate test map. Close the map in the World Editor before saving changes from Tinkerbench.
3. Start Tinkerbench. After [Tinkerbench Setup](../install/#run-the-setup), it loads your game
   data on its own. After a `.deb` or AppImage install, choose your Warcraft III folder on the
   first launch and select **Load game data**; see
   [connect the game](../install/#connect-the-game). Later launches load the last folder and
   reopen the last map directly.
4. If setup is showing, choose **Open a map…** under **2. Map** and select your test map.
   If the workbench opened another map, use **Open map…** in its top bar. You can also press
   Escape or **Stop and choose** while game data loads to stay on setup.

<figure class="app-screenshot">
  <a href="../../screenshots/open-map.png">
    <img src="../../screenshots/open-map.png" alt="Tinkerbench setup after loading 928 units, offering Reopen Field guide.w3m, Open a map, read-only browsing and the checked automatic-startup setting." width="600" height="740" loading="lazy" />
  </a>
  <figcaption>After game data loads, open a map or browse read-only; the checkbox controls later automatic launches.<span class="screenshot-notice">Image subject to change.</span></figcaption>
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

Tinkerbench writes object changes and the map’s string table, [imported files](../assets/),
[map properties and constants](../map-settings/), and [placed units and loot](../placed-units/).
For placed units and map setup, it also updates the relevant generated script when it can
reproduce the World Editor's parts exactly. If a tab reports that it cannot, open and save
the result in the World Editor before testing in game.

[Interface designs](../interface/) and cascade tiers also have files beside the map:
`<map> UI.json` and `<map> Loot.json`. Keep them with your source map; Interface Lua is
built separately into a development copy.

Use the World Editor for terrain and GUI triggers. Avoid keeping the same map open in two
editors: a later save can overwrite the other tool's work.

If [saving fails](../troubleshooting/#saving-the-map-fails) or
[the result differs in game](../troubleshooting/#my-changes-disappear-or-do-not-show-in-game),
follow the troubleshooting checks before repeating the edit.
