---

title: "Imported files and assets"
description: "Import models and textures, inspect their dependencies and uses, and rename, replace or export map files."
---

Use **Assets** for the [imported files](../../reference/glossary/#imported-file) inside an
open map. This includes models, textures and icons. Changes are written into the map when
you save it; a preview alone does not save an import.

## Import a model and its textures

1. Choose **Import files…** for individual files or **Import folder…** for a folder.
2. In **Import files**, select the files to include and review each path in the map.
   Text `.mdl` models are converted to binary `.mdx` files.
3. Check the listed textures each model needs. Keep their map paths equal to the paths
   named inside the model. The preview reports which textures are in the import, map or
   game and which are missing. Add missing textures before relying on the model's preview.
4. Review any **Replaces the file already there.** notice. Choose **Add** with the displayed
   file count **to the map**, then save the map.

<figure class="app-screenshot">
  <a href="../../screenshots/assets-import.png">
    <img src="../../screenshots/assets-import.png" alt="Import files preview for a Knight model folder, with editable map paths, text-model conversion, replacement notices and texture statuses for the import and game." width="1062" height="540" loading="lazy" />
  </a>
  <figcaption>Review each map path before adding files. The text model is converted to MDX, and its textures are accounted for in the import or game.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

The import preview lets you correct paths before adding files. A model does not search
arbitrary folders for textures. See [paths and previews](../../formats/imports/#paths-and-previews)
when a texture is present but the model still cannot find it.

## Inspect and manage files

Use **Filter by path** to find a file, then select it. Models have a 3D preview and a
**Textures** list; images have an image preview. **Used by** lists object fields that refer
to the file. Select a use to open that object.

<figure class="app-screenshot">
  <a href="../../screenshots/assets-model.png">
    <img src="../../screenshots/assets-model.png" alt="Imported Knight model selected in Assets, with a textured preview, management actions, Path in the map, texture dependencies and Sunwell Knight under Used by." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption>The selected model shows its texture paths and the Sunwell Knight object field that uses it.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

| Control                            | Result                                                                                        |
| ---------------------------------- | --------------------------------------------------------------------------------------------- |
| **Path in the map** and **Rename** | Changes the file path and updates object fields that point to it.                             |
| **Replace…**                       | Chooses a replacement file at the same map path; save to write it.                            |
| **Remove**                         | Requests removal; **Remove file** confirms it. Objects using the file can lose their artwork. |
| **Export…**                        | Saves the selected imported file to disk.                                                     |
| **Export as PNG…**                 | Exports an image as PNG.                                                                      |

The usage list concerns object fields. An **unused** file can still be referenced from your
Lua, a model or other map content; inspect those references before removing it. Renaming
object-field references does not establish that handwritten script or model texture paths
have been updated.

If a protected map has no import list, **Assets** cannot list its old imports. Newly imported
files are listed. This does not prove that an unlisted file is absent; see
[protected-map limits](../../formats/imports/#protected-maps).

## Create a unit from a model

Select an imported model and choose **Create a unit from this model…**. Search **Base unit**
by name or ID and pick a game unit whose stats and abilities suit your purpose. If imported
images are available, **Icon** can choose one or keep **The base unit's icon**. Choose
**Create unit** to make a custom unit and open its page.

The new unit copies its base's stats and abilities, uses the imported model, and uses the
chosen icon. Review its fields and name, then save. It is an object type; to place a copy
on the terrain, use [Placed units](../placed-units/).

## Export a file from the game

Choose **Export a game file…**, enter **Game path** from an object's model or icon field,
and choose **Preview**. **Export…** copies the game file to disk; images also offer
**Export as PNG…**. Game files are for your own use; do not add extracted Blizzard assets
to this repository or distribute them with documentation examples.

These workflows were checked against the source. A successful import and desktop preview
do not confirm how the model or texture renders in Warcraft III: the examples here are
**not yet confirmed in game**. Save and test the same map in game.
