---

title: "Share codes and bundles"
description: "Move custom units and their dependencies between maps."
---

Sharing starts from custom units. The transfer code follows their referenced custom units, abilities, buffs, items and upgrades so that a unit does not arrive with missing object definitions.

## Share a code

1. Open the source map and choose **Share units…** from the command palette.
2. Select the custom units to transfer.
3. Review the dependency preview and create or copy the share code.
4. In the destination map, choose **Import…**, then **Share code**.
5. Paste the `W3OBJ1:` code and review the objects before importing.
6. Save the destination map and test it.

A share code carries object definitions. It does not carry imported models, textures or icons.

## Save a bundle file

Choose **Save bundle file…** in the sharing dialog to write a `.w3obj` bundle. Bundles include discovered imported files used by the objects, including textures referenced by models.

In the destination map, choose **Import… → Bundle file**, select the bundle and review the import. The **Another map** tab can also import from a source map.

## Raw code collisions

If the destination already uses an incoming custom raw code, the importer assigns a free code and updates object references it recognizes through field metadata. Review the assigned IDs after import.

References embedded in handwritten scripts or arbitrary text cannot be safely rewritten this way. Update those yourself. Trigger systems, terrain and placed objects are not part of unit sharing.

## Missing assets

Dependency discovery starts from art fields and follows model texture references. Protected maps may not list their contents, so the importer tries known names. An asset that cannot be discovered by reference may be missing from the bundle. Check the model, icon, tooltips and abilities in the destination map and in game.
