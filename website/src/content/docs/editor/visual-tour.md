---

title: "Visual tour"
description: "Tour Tinkerbench's startup, object fields, map tools, interface designs, saving and sharing."
---

Get to know the workbench before [editing your first map](../first-map/). These are screenshots
of the running desktop application, with a test map and a local Warcraft III: Reforged
installation. Select any screenshot to open the full-size image.

If you have not installed Tinkerbench yet, start with [installation](../install/).

## 1. Load the game

After [Tinkerbench Setup](../install/#run-the-setup), Tinkerbench loads the game you chose there
and goes on to **2. Map**. After a `.deb` or AppImage install, or when the game has moved, it asks
for your Warcraft III installation. It may already fill in the folder from detection or your
previous session, and lists the installations under **Found on this computer** when it finds
several. Otherwise choose **Browse…** under **1. Warcraft III folder**, then select the game
directory containing `Data/` and `.build.info`; you can also type or paste the folder’s path.

<figure class="app-screenshot">
  <a href="../../screenshots/load-game.png">
    <img src="../../screenshots/load-game.png" alt="First-launch Tinkerbench setup with the detected Warcraft III folder, HD art checkbox, Load game data and Prompt for an AI assistant buttons." width="600" height="480" loading="lazy" />
  </a>
  <figcaption><span class="screenshot-notice">Image subject to change.</span>On the first launch, connect the game installation before opening a map.</figcaption>
</figure>

Enable **Use HD (Reforged) art values and icons** if you want HD art, then select
**Load game data**. Wait for the loaded unit count and the **2. Map** section to appear.
The path in these screenshots is a local Linux/Proton installation; choose your own game
folder. Your Maps folder is a separate location.

On later launches, Tinkerbench loads that folder and art choice, then reopens the last map
directly. While game data loads, **Stop and choose** or Escape stops before the map opens.
With no remembered map, it stays on setup. If loading fails, setup shows the reason.

<figure class="app-screenshot">
  <a href="../../screenshots/auto-open.png">
    <img src="../../screenshots/auto-open.png" alt="Tinkerbench loading the remembered game folder, with Stop and choose and the Escape hint visible." width="600" height="400" loading="lazy" />
  </a>
  <figcaption><span class="screenshot-notice">Image subject to change.</span>During automatic loading, Stop and choose or Escape lets you choose another map.</figcaption>
</figure>

## 2. Open a map

After the game data loads, choose **Open a map…** and select a copy of a `.w3m` or `.w3x`
map. If you previously opened a map, **Reopen** offers that file directly.

<figure class="app-screenshot">
  <a href="../../screenshots/open-map.png">
    <img src="../../screenshots/open-map.png" alt="Tinkerbench setup after loading 928 units, offering Reopen Field guide.w3m, Open a map, read-only browsing and the checked automatic-startup setting." width="600" height="740" loading="lazy" />
  </a>
  <figcaption><span class="screenshot-notice">Image subject to change.</span>After game data loads, open a map or browse read-only; the checkbox controls later automatic launches.</figcaption>
</figure>

**Browse game data only** opens the workbench without a map. You can inspect objects, fields
and previews, but editing requires an open map. Use **Open map…** in the workbench’s top bar
when you are ready.

Clear **At start, load this game data and reopen the last map** on setup to choose manually
each time. From the workbench, **Ctrl K → Change game folder…** returns to setup after
handling any unsaved changes.

Close the map in the World Editor before saving changes from Tinkerbench. For a small test
map to follow along with, see [your first map](../first-map/).

## 3. Find your way around

<figure class="app-screenshot">
  <a href="../../screenshots/unit-editor.png">
    <img src="../../screenshots/unit-editor.png" alt="Paladin selected in the Units tab, with object search on the left, stats and grouped fields in the center, and its model and hit point explanation in the inspector." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption><span class="screenshot-notice">Image subject to change.</span>The Paladin’s page brings its identity, values, model and field help into one view.</figcaption>
</figure>

| Area            | What you can do                                                                                                                       |
| --------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| Top bar         | See the open map, search across objects and commands, import, share and save.                                                         |
| Object tabs     | Switch between units, abilities, items, upgrades, buffs, destructables and doodads.                                                   |
| Map tools       | Edit [Placed units](../placed-units/), [Map settings](../map-settings/), [Interface designs](../interface/) and [Assets](../assets/). |
| Left column     | Search the current object type and use its filter chips. Expand the game-object group when it is folded.                              |
| Center column   | Check the selected object’s raw code and summary, then browse or filter its grouped fields.                                           |
| Right inspector | Read the selected field’s explanation, limits and base value. Objects with models also have a 3D preview.                             |

Tabs keep their search, filters, list and selected object when you return, making switching faster.

For a separate unit type, select a base object and choose **New unit based on this**. The new
object gets its own raw code. Editing a game object directly changes that type wherever your
map uses it.

## 4. Jump straight to an object

Press **Ctrl K**, or select the search box in the top bar. Type a name such as `holy light`
or a raw code such as `AHhb`, then select the matching result. This search reaches across
object types, so you can find an ability while browsing units.

<figure class="app-screenshot">
  <a href="../../screenshots/object-search.png">
    <img src="../../screenshots/object-search.png" alt="The Ctrl K search overlay showing Holy Light ability results, their icons and raw codes above the unit editor." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption><span class="screenshot-notice">Image subject to change.</span>Use the raw code on the right to distinguish objects with the same name.</figcaption>
</figure>

The left column’s search stays within the current tab. It also matches field values, which
helps find objects that reference a particular model or ability. Search still finds game
objects when their group is folded.

## 5. Understand a field before changing it

On the Paladin’s page, type `hit points` in **Filter fields**. Select the information button
beside **Hit Points Maximum (Base)** (`uhpm`) to read its explanation in the inspector.

<figure class="app-screenshot">
  <a href="../../screenshots/field-inspector.png">
    <img src="../../screenshots/field-inspector.png" alt="The Paladin’s fields filtered to hit points, showing a base maximum of 100, regeneration fields, and an inspector explaining that heroes gain additional life from Strength." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption><span class="screenshot-notice">Image subject to change.</span>The inspector explains why a hero’s base hit points are only part of its in-game life.</figcaption>
</figure>

Change a value in the center column. The editor highlights overrides and keeps the inherited
value alongside them. **Changed only** narrows the page to your overrides; a changed field’s
inspector offers **Restore the base value**.

The screenshot shows unchanged game data. The Paladin’s `100` base hit points exclude the
life gained from Strength. For a first experiment with fewer moving parts, follow the
[Footman walkthrough](../first-map/).

## 6. Compare ability levels side by side

Open **Abilities**, find **Holy Light** (`AHhb`), and select it. The page starts with its
resolved tooltip and puts per-level numbers in a table. Select the information button beside
**Mana Cost** to see the help shown here.

<figure class="app-screenshot">
  <a href="../../screenshots/ability-levels.png">
    <img src="../../screenshots/ability-levels.png" alt="Holy Light’s ability editor showing its colored tooltip, three level columns for mana cost, cooldown and cast range, and mana cost help in the inspector." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption><span class="screenshot-notice">Image subject to change.</span>Holy Light has three levels. Its tooltip preview and level table are visible together.</figcaption>
</figure>

Use the table’s keyboard controls:

| Key                        | Action                                          |
| -------------------------- | ----------------------------------------------- |
| Arrow keys                 | Move between cells.                             |
| Enter                      | Start editing a cell.                           |
| Enter or Tab while editing | Commit the value and move to the next level.    |
| Ctrl Enter while editing   | Apply the value to this level and later levels. |
| Delete on a selected cell  | Reset that level’s override.                    |
| Esc while editing          | Cancel the cell edit.                           |

Committing a cell updates the map in the editor. Save the map to write those changes to disk.
For a complete spell workflow, continue with [create a custom spell](../../guides/custom-spell/).

## 7. Save, test and share

Select **Save**, or press **Ctrl S**, to save the open map. Close that map in the World Editor
before saving from Tinkerbench, then reopen it and test the result in game.

To move custom units to another map, choose **Share…** in the top bar. Share codes carry
object data; `.w3obj` bundles can carry imported models, textures and icons too. The receiving
map uses **Import…**. See [sharing objects](../sharing/) for the full workflow and dependency
handling.

For spreadsheets or tools, use [Export objects as CSV or JSON](../exporting/).
[Work with an AI assistant](../ai-assistant/) explains the copyable prompt and map summary.

Ready to try it? [Edit your first map](../first-map/), or browse the
[object field reference](../../reference/fields/) when you need a field’s exact meaning.
