---

title: "Work with an AI assistant"
description: "Copy a prompt with Tinkerbench's workflow and your map summary, then use the response to plan, edit and test."
---

**Prompt for an AI assistant…** prepares context to paste into an assistant such as Claude
or ChatGPT. It explains Tinkerbench, the World Editor's part and the [Lua harness](../lua-harness/),
so you can ask for a plan and code suited to these tools.

## Copy the prompt

1. Open your map if you want its summary included. The prompt is also available before
   opening a map.
2. Choose **Prompt for an AI assistant…** on the setup screen, or from **Ctrl K** in the editor.
3. Fill **The map you want to make** with your goal: players, objectives, systems and what
   you want to build first. Review **Prompt** below it.
4. Choose **Copy prompt**, then paste into your assistant's chat. If the clipboard is
   unavailable, select and copy the prompt text by hand.

<figure class="app-screenshot">
  <a href="../../screenshots/assistant-prompt.png">
    <img src="../../screenshots/assistant-prompt.png" alt="Prompt for an AI assistant dialog with a cooperative Sunwell-defense goal, the open Lua map&#x27;s player, force, object and import summary, and Copy prompt." width="1062" height="623" loading="lazy" />
  </a>
  <figcaption>The copied prompt combines your goal with a summary of the open map; it does not include every object&#x27;s values or your scripts.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

## What it includes

The prompt describes the editor's tabs, raw-code assignment, object fields, text formatting,
and the harness's Lua 5.3 modules and `Dev.OnStart` callback. It explains which tasks remain
in the World Editor, including terrain, regions, GUI triggers, start locations and adding
players. It asks the assistant for:
- A step-by-step build plan, with each step assigned to World Editor, Tinkerbench or Lua.
- Object specifications with base objects, field IDs, names and values.
- Complete Lua modules with filenames.
- Instructions to test each step in the `[DEV]` map copy.

For an open map it adds the filename, script language and balance data when available,
players and forces, counts of custom and changed objects by type, a count of placed units
and items, and the imported-file count. This is a summary, not an export of every object
ID, field value, design or script. With no map open, the prompt says so.

## Turn the answer into a small test

Ask for one testable step at a time. Check the proposed field IDs in the
[field reference](../../reference/fields/) and provide the actual raw codes assigned to
your custom objects. An [object export](../exporting/) can give the assistant exact values
that the summary omits.

For generated interface calls, provide the functions shown in **Show Lua** or the
[per-design function tables](../interface/). Dialogs, boards and timer windows have different
signatures; do not assume every design supports `ShowAll()` or accepts a player argument.

Keep your game logic in ordinary `.lua` files, separate from Tinkerbench's generated modules.
Build with the [Lua harness](../lua-harness/#build-the-development-copy) or **Save and build
test copy** in **Interface**, then play the `[DEV]` copy. A successful syntax check does not
confirm object IDs, game API use, multiplayer behavior or the assistant's gameplay claims.
Code produced from the prompt is **not yet confirmed in game** until you test it.
