---

title: "Troubleshooting"
description: "Fix game-folder detection, loading, read-only editing, map saving, missing models and icons, and import problems in Tinkerbench."
---

Start with the symptom you see. These checks follow the current editor’s behavior; a preview
or successful save still needs verification in Warcraft III. Use a copy of your map while
trying changes.

| What is happening?                                         | Start here                                                                |
| ---------------------------------------------------------- | ------------------------------------------------------------------------- |
| The game folder is blank or points to another installation | [Game-folder detection](#the-game-folder-is-not-detected)                 |
| Load game data fails, or the Map section never appears     | [Loading game data](#the-game-data-will-not-load)                         |
| Fields cannot be edited, or Save is unavailable            | [Read-only browsing and pending edits](#i-cannot-edit-or-save-fields)     |
| A unit, object or field seems to have disappeared          | [Search and filters](#my-unit-or-field-is-missing)                        |
| A map cannot be opened or reopened                         | [Opening a map](#my-map-will-not-open)                                    |
| Save reports an error                                      | [Saving a map](#saving-the-map-fails)                                     |
| Changes disappear or do not show in game                   | [Checking the saved result](#my-changes-disappear-or-do-not-show-in-game) |
| A model, texture or icon is missing                        | [Artwork and rendering](#a-model-or-icon-is-missing)                      |
| An imported object loses its artwork or changes raw code   | [Import previews](#an-import-loses-artwork-or-changes-raw-codes)          |

## The game folder is not detected

**Symptom:** **1. Warcraft III folder** is empty, or the filled-in path belongs to a different
installation.
1. Choose **Browse…** and select the Warcraft III installation directory. It must contain
   both **`Data/`** and **`.build.info`**. Show hidden files in your file browser if needed.
2. Do not select the Maps folder or only the game executable. On Linux with Wine or Proton,
   choose the installation inside the prefix that contains your game.
3. If you have several installations or prefixes, check the selected path yourself.
   Detection searches common locations; it does not search every possible install path.

Choose the art mode, then **Load game data**. If detection still finds nothing but manually
selecting the folder works, you can continue using that folder.

**Next:** [Connect the game](../install/#connect-the-game), or continue with
[loading checks](#the-game-data-will-not-load) if an error appears.

## The game data will not load

**Symptom:** **Load game data** reports an error, or **2. Map** has not appeared.
1. Selecting a folder does not load it: choose **Load game data** and wait for the loaded
   unit count. The Map section appears only after the game data finishes loading.
2. For **“is not a Warcraft III folder”**, check both `Data/` and `.build.info` in the selected
   directory. The current error mentions a missing Data folder even when `.build.info` is
   the missing marker.
3. If both are present, check that the game installation has finished installing or updating
   and that your user can read it. Record the complete error before retrying.

A folder passing those two checks does not guarantee its game data is complete or readable.
If loading still fails, include the exact file or table named in the error when reporting it.

**Next:** [Installation guide](../install/), or [report the unresolved error](#report-an-unresolved-problem).

## I cannot edit or save fields

**Symptom:** The top bar says **Game data (read-only)**, fields cannot be changed, or **Save**
is missing or disabled.
1. If you chose **Browse game data only**, choose **Open map…** in the top bar and open a
   test map. Browsing the installation alone does not enable editing or map import/sharing.
2. If a map name is already shown, **Save** is disabled when there are no pending changes
   or a save is in progress. That alone does not mean the map is read-only.
3. Finish a typed field edit with **Enter** or by leaving the field. In a per-level table,
   use **Enter** or **Tab** to commit it. Look for the **unsaved** marker beside the map name
   before saving with **Ctrl S** or **Save**.

**Share…** needs at least one custom unit in the open map. Changing a game unit’s values
does not make it a custom unit; create a separate unit with **New unit based on this** when
that is what you intend.

**Next:** [Open a map in the visual tour](../visual-tour/#2-open-a-map), then
[edit your first map](../first-map/).

## My unit or field is missing

**Symptom:** An object is not listed, or its page shows fewer fields than expected.
1. Check the object tab: units, abilities, items, upgrades, buffs, destructables and doodads
   have separate lists.
2. Clear the list search and check its filter chips. Expand the game-object group if it is
   folded, or use **Ctrl K** to search by the exact raw code across object types.
3. On the object’s page, clear **Filter fields**, turn off **Changed only** and expand the
   relevant section or advanced group.

Objects can share a display name. Compare the selected raw code with the one used by your
map before editing another object to compensate.

**Next:** [Find and change fields](../objects/#find-and-change-fields), or look up the
[field’s raw code](../../reference/fields/).

## My map will not open

**Symptom:** **Open a map…**, **Open map…** or **Reopen** reports an error.
1. Choose the actual `.w3m` or `.w3x` file. If the remembered file was moved or renamed, use
   **Open a map…** at startup or **Open map…** in the workbench to select its new location.
2. Check that your user can read the file. Try a separate, small World Editor test map to
   determine whether the failure affects every map or just this one.
3. Copy the full error, including any named object file or string table. A failure reading
   one of those files can prevent the map from opening.

If you can reproduce the failure with only one map, report its format and whether it is
protected. A missing import list in a protected map is not, by itself, proof that all of its
files are unreadable. Keep an unchanged copy for comparison.

**Next:** [Create a small test map](../first-map/#prepare-a-small-test-map) and read the
[protected-map discovery limits](../../formats/imports/#protected-maps).

## Saving the map fails

**Symptom:** **Save** reports an error and the map still has pending changes.
1. Keep Tinkerbench open and copy the error. Close the same map in the World Editor and
   retry the save.
2. Check that you can write to the map file **and its containing directory**, and that the
   destination has free space. Saving creates a temporary copy beside the map before
   replacing the destination.
3. After retrying **Save**, check that the **unsaved** marker disappears. An error message
   does not confirm that the file was saved.

If you need to move the map to a writable folder, record your unsaved edits first. A copy
of the file on disk will not include edits that only exist in the running editor, and
opening another map can discard those edits after confirmation. There is currently no
**Save As** control in the desktop editor.

**Next:** [Save and test](../first-map/#save-and-test), or
[report the save error](#report-an-unresolved-problem) with the failing path and error text.

## My changes disappear or do not show in game

**Symptom:** A saved value reverts, the World Editor shows an older value, or the game result
differs from what you edited.
1. Confirm the edit was committed and the **unsaved** marker cleared after saving. Hover
   the map name in Tinkerbench’s top bar to check its full path; reopen and test that file.
2. Close the map in the World Editor before saving from Tinkerbench, then reopen it there.
   A later World Editor save can overwrite Tinkerbench’s changes with its older copy.
3. Check the raw code of the edited object against the object actually placed or used in
   the map. A custom object has its own code; changing its base object is a different edit.
4. Check bonuses and triggers that affect the value in game. For example, a hero’s
   **Hit Points Maximum (Base)** excludes the life it gains from Strength.

To isolate the problem, repeat the first-map guide’s Footman `hfoo` / maximum life `uhpm`
change in a small test map. Compare the stored field, the World Editor’s value and the
in-game result separately. They help distinguish a saving problem from gameplay behavior.

**Next:** [The Footman walkthrough](../first-map/) and
[game objects versus custom objects](../objects/#game-objects-and-custom-objects).

## A model or icon is missing

**Symptom:** A preview says **No model**, an icon is a plain square, a model has missing
textures, or the inspector shows a rendering error.
1. Check the object’s model or icon field. **No model** means no model path was supplied to
   the viewer. A plain icon square can mean either an empty path or a failed load.
2. If the object uses imported art, check that the open map contains the referenced file
   and that its path matches the object field. A model also needs its referenced textures.
   Imported files take priority over installation art.
3. Compare with a standard game object. If that works but your imported object does not,
   start with the imported model, texture and icon paths. A **“texture(s) could not be
   loaded”** message means the model may still display with missing textures.
4. To compare SD and HD art, save your changes, restart Tinkerbench and change **Use HD
   (Reforged) art values and icons** before **Load game data**. The editor tries alternate
   game-art layers when art is missing; this does not guarantee every requested asset exists.

**“WebGL is not available in this window.”** and **“Could not show this model…”** are renderer
or loading failures, rather than the **No model** state. Record the full message. If standard
game objects fail too, include that comparison and your operating system and graphics
details, if known, in the report. This guide cannot confirm that a rendering failure means
the map’s artwork is broken in Warcraft III.

**Next:** [Tooltips and art](../objects/#tooltips-and-art),
[paths and previews](../../formats/imports/#paths-and-previews), or
[report the affected object](#report-an-unresolved-problem).

## An import loses artwork or changes raw codes

**Symptom:** An imported object has a missing preview, some files are absent from the
preview, or an ID changes from something like `H000` to `H001`.
1. Check the transfer format. A `W3OBJ1:` share code contains object definitions, not
   imported models, textures or icons. For those files, create a `.w3obj` bundle in the
   source map and use **Import… → Bundle file → Choose file… → Preview** in the destination.
2. Review the objects, imported files and **Not included** warnings before confirming the
   import. Asset discovery follows known references; it is not a complete archive listing,
   especially for protected maps.
3. A proposed new raw code means the old one is already taken in the destination. Review
   the assigned IDs and update handwritten scripts or text that refer to the old codes;
   those references cannot be safely rewritten automatically.
4. Choose **Import** with the displayed object count, then save the destination map. Merely
   opening **Preview** does not apply the import, and applying it does not save it to disk.

For an imported file marked **this map keeps its own**, the destination already has a
different file at that path. Inspect that conflict before assuming the source artwork
replaced it.

**Next:** [Share codes and bundles](../sharing/),
[missing assets](../sharing/#missing-assets) and
[import discovery limits](../../formats/imports/#protected-maps).

## Report an unresolved problem

After trying the relevant checks, [search existing issues](https://github.com/gabaudette/wc3-toolkit/issues)
or [open a new issue](https://github.com/gabaudette/wc3-toolkit/issues/new). Copy the exact
error from the startup screen, editor or model inspector. Include the version shown beside
Tinkerbench’s name and a short reproduction:

```text
Tinkerbench version:
Operating system:
Warcraft III version/build (if known):
HD art enabled or disabled:
Game data only, or an open map:
Object tab, name and raw code:
Field raw code or model/icon path (if relevant):
Steps to reproduce:
Expected result:
Actual result:
Exact error message:
Does a standard game object or small test map work?:
```

Attach a screenshot when it helps show the symptom. Use a minimal test map you have
permission to share; do not upload your game installation or extracted game files.
The [bug-report guide](../../contributing/#report-a-useful-bug) explains what makes a report
actionable.
