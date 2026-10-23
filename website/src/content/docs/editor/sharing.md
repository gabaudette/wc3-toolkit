---

title: "Share codes and bundles"
description: "Move custom units and their dependencies between maps."
---

Sharing starts from custom units. The transfer code follows their referenced custom units, abilities, buffs, items and upgrades so that a unit does not arrive with missing object definitions.

Select either screenshot to open it at full size.

## Share a code

1. Open the source map and choose **Share…** in the top bar, or **Share units…** from the command palette.
2. Select the custom units to transfer.
3. Choose **Create share code** to generate the code and object preview.
4. Review the objects and any warnings. Custom dependencies are marked **brought along**.
5. Choose **Copy code**.

<figure class="app-screenshot">
  <a href="../../screenshots/share-code.png">
    <img src="../../screenshots/share-code.png" alt="Share units dialog with test_hero selected, its generated W3OBJ1 code and object preview, plus Save bundle file and Copy code buttons." width="680" height="540" />
  </a>
  <figcaption>The generated code and included objects appear together. Copy the code, or use Save bundle file… to include imported assets.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

A share code carries object definitions. It does not carry imported models, textures or icons.

## Import a code

1. Open the destination map and choose **Import…**, then **Share code**.
2. Paste the `W3OBJ1:` code and choose **Preview**.
3. Review the object list, proposed raw codes and any warnings.
4. Choose **Import** with the displayed object count to add the objects to the open map.
5. Save the destination map and test it.

<figure class="app-screenshot">
  <a href="../../screenshots/import-preview.png">
    <img src="../../screenshots/import-preview.png" alt="Import objects preview on the Share code tab, showing test_hero with raw code H000 reassigned to H001, and Cancel and Import 1 object buttons." width="680" height="420" loading="lazy" />
  </a>
  <figcaption>The preview proposes H000 → H001 because H000 already exists in this map. Review the new code before choosing Import 1 object.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

**Preview** lets you review the import before applying it. **Cancel** closes the dialog.
Imported changes are written to disk when you save the map.

## Save a bundle file

After creating a share code, choose **Save bundle file…** in the sharing dialog shown above
to write a `.w3obj` bundle. Bundles include discovered imported files used by the objects,
including textures referenced by models.

In the destination map, choose **Import… → Bundle file**, choose **Choose file…**, select the
bundle and choose **Preview**. Review the objects and imported files before confirming the
import. The **Another map** tab can also import from a source map.

## Raw code collisions

If the destination already uses an incoming custom raw code, the importer assigns a free code and updates object references it recognizes through field metadata. Review the assigned IDs after import.

References embedded in handwritten scripts or arbitrary text cannot be safely rewritten this way. Update those yourself. Trigger systems, terrain and placed objects are not part of unit sharing.

## Missing assets

Dependency discovery starts from art fields and follows model texture references. Protected maps may not list their contents, so the importer tries known names. An asset that cannot be discovered by reference may be missing from the bundle. Check the model, icon, tooltips and abilities in the destination map and in game.

If an import [loses artwork or changes raw codes](../troubleshooting/#an-import-loses-artwork-or-changes-raw-codes),
check the transfer format, preview warnings and destination-file conflicts before retrying.
