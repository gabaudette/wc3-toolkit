---

title: "Interface designs"
description: "Design dialogs, multiboards, leaderboards and timer windows, build their Lua, and reuse them in another map."
---

Use **Interface** to design a difficulty chooser, scoreboard or countdown for a Lua map.
Choose **Dialog**, **Multiboard**, **Leaderboard** or **Timer** on the left, then edit the
design beside its live preview. **Name** identifies the design in Tinkerbench; **Message**
or **Title** supplies the text players see.

Designs are saved beside the map in `<map> UI.json`. Keep that file with your source map.
Saving the designs and building their Lua are separate steps: the playable interface comes
from the modules added by the [Lua harness](../lua-harness/).

:::note[Verification]
The [recorded play test](../verification/#interface-results-and-known-failures) confirmed
hotkeys, a player callback, navigation, multiboard native updates and timer controls on one
Lua map. It also found startup failures: blank dialog messages, missing leaderboard rows
and a default timer title; repeated dialogs can lose their message. Those bugs remain open.
Read the results and limits before relying on the designs in a playable map.
:::

## Make a dialog

1. Choose **Dialog** and give it a **Name**, such as `Difficulty`.
2. Set **Message** and **Message color**.
3. Use **Add button** to add choices. Enter each button's text and a one-letter or digit
   hotkey. **Gold hotkey letter** highlights its matching letter in the preview.
4. Under **When clicked:**, choose **Close the dialog**, **Show another dialog** or
   **Run a Lua function**. For another dialog, select its design; for a function, enter its
   name, such as `OnHardMode`, without parentheses.
5. Enable **Show to every player when the game starts** if this is the opening choice.

<figure class="app-screenshot">
  <a href="../../screenshots/interface-dialog.png">
    <img src="../../screenshots/interface-dialog.png" alt="Difficulty dialog in the Interface tab, with a white message, Normal and Hard hotkeys, an OnHardMode callback and the live dialog preview." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption>Set the message, hotkeys and button actions beside the dialog preview; Hard calls OnHardMode with the clicking player.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

Your own module must define the named function. The generated action passes the player
who clicked: `function OnHardMode(player)`. Function names accept letters, digits and `_`,
cannot start with a digit, and cannot be Lua keywords. An undefined callback does nothing.
Keep at least one button that lets the player leave the dialog.

For a design whose displayed Lua name is `Difficulty`:

| Call                         | What the generated module does                                             |
| ---------------------------- | -------------------------------------------------------------------------- |
| `UI.Difficulty.Show(player)` | Displays the dialog to that player.                                        |
| `UI.Difficulty.Hide(player)` | Hides it from that player.                                                 |
| `UI.Difficulty.ShowAll()`    | Displays it to playing human players, excluding computers and empty slots. |

## Make a multiboard

A [multiboard](../../reference/glossary/#multiboard) is a table with several columns, such
as player, kills and deaths. Choose **Multiboard**, then set **Title** and **Title color**.
Add columns with **Column** and rows with **Row**. Give them names you can use from Lua and
set each column's width as a percentage of the screen. Enable **Each player** to repeat a
row for each player; `{player}` in its text becomes
the player's name in their color. The generated rows include playing computer slots too.

Enter cell text in the table. Select a cell, or a row or column's **Style** button, to apply **Text color** and
**Icon** in the **Style:** section; **No icon** clears the selected cells' icons. Choose
**Show when the game starts**, and optionally **Start folded to its title (players open it
with a click)**.

<figure class="app-screenshot">
  <a href="../../screenshots/interface-multiboard.png">
    <img src="../../screenshots/interface-multiboard.png" alt="Scoreboard multiboard with Player, Kills and Deaths columns, a repeating player row and the live corner-board preview." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption>The multiboard&#x27;s fixed header and per-player row are edited in the same table, with a live preview above.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

<figure class="app-screenshot">
  <a href="../../screenshots/interface-style.png">
    <img src="../../screenshots/interface-style.png" alt="The Kills column selected for styling, showing multiboard cells, text-color swatches, No icon and the icon picker." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption>Scroll below the table to style the selected cell, row or column; here Style selects the whole Kills column.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

For a design whose displayed Lua name is `Scoreboard`:

| Call                                                 | What the generated module does                   |
| ---------------------------------------------------- | ------------------------------------------------ |
| `UI.Scoreboard.SetValue(player, "Kills", 5)`         | Sets a playing player's Kills cell to `5`.       |
| `UI.Scoreboard.SetValue("Header", "Kills", "Kills")` | Sets a fixed row's cell, using its row name.     |
| `UI.Scoreboard.SetColor(player, "Kills", "00ff00")`  | Sets the cell's text color; use `nil` for white. |
| `UI.Scoreboard.SetIcon(player, "Kills", path)`       | Sets an icon path; use `nil` for no icon.        |
| `UI.Scoreboard.Show()` / `UI.Scoreboard.Hide()`      | Shows or hides the board.                        |
| `UI.Scoreboard.Minimize(true)`                       | Folds it to its title; `false` expands it.       |

Row and column strings must match the design's names, including case. Calls with an unknown
row or column do nothing. These functions change the display; your Lua tracks kills or
other game events and calls them with the new values.

## Make a leaderboard

A [leaderboard](../../reference/glossary/#leaderboard) ranks rows by a single number.
Choose **Leaderboard**, then set **Title** and **Title color**. Enable **A row for each
player, starting at** and set its initial value. **Rows of your own** supports up to four
extra rows, such as team totals, through **Add row**. Set each row's name, starting value
and color.

Choose **Order**: **Highest first**, **Lowest first** or **As listed**. Enable **Show when
the game starts** if needed. Player rows use the player's name and color; the generated
module creates them for playing slots, including computers.

<figure class="app-screenshot">
  <a href="../../screenshots/interface-leaderboard.png">
    <img src="../../screenshots/interface-leaderboard.png" alt="Kills leaderboard with player rows starting at zero, a blue Waves row, Highest first order and the live ranking preview." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption>A leaderboard combines player scores with named rows of your own and previews the selected order.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

For a design whose displayed Lua name is `Kills`:

| Call                                  | What the generated module does                            |
| ------------------------------------- | --------------------------------------------------------- |
| `UI.Kills.SetValue(player, 5)`        | Replaces a player's value and reapplies the chosen order. |
| `UI.Kills.AddValue(player, 1)`        | Adds to a player's value and reapplies the order.         |
| `UI.Kills.GetValue(player)`           | Returns the tracked value, or `nil` for an unknown row.   |
| `UI.Kills.SetValue("Alliance", 10)`   | Updates an own row by its exact name.                     |
| `UI.Kills.Show()` / `UI.Kills.Hide()` | Shows or hides the board.                                 |

The same row argument works with `SetValue`, `AddValue` and `GetValue`. Your Lua supplies
the scores; the design does not count kills automatically.

## Make a timer window

Choose **Timer** to design a [timer window](../../reference/glossary/#timer-window).
Set **Title**, **Title color** and **Length** in minutes and seconds. Choose whether to
**Start counting when the game starts** and **Start over each time it runs out**.
For a timer that runs once, **Hide the window when the time runs out** is also available.

To run your code on expiry, fill **When the time runs out, run this Lua function** with
a function name, such as `SpawnWave`. Define `function SpawnWave()` in your own module;
no player argument is passed. A repeating timer calls it each time it expires.

<figure class="app-screenshot">
  <a href="../../screenshots/interface-timer.png">
    <img src="../../screenshots/interface-timer.png" alt="Next wave timer design set to one minute thirty seconds, starting with the game, repeating and calling SpawnWave on expiry." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption>The timer preview reflects its title and length; the callback field names SpawnWave without arguments.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

For a design whose displayed Lua name is `NextWave`:

| Call                                        | What the generated module does                                      |
| ------------------------------------------- | ------------------------------------------------------------------- |
| `UI.NextWave.Start()`                       | Starts or restarts with the designed duration and shows the window. |
| `UI.NextWave.Start(30)`                     | Starts or restarts with a 30-second duration.                       |
| `UI.NextWave.Stop()`                        | Pauses the timer and hides the window.                              |
| `UI.NextWave.Show()` / `UI.NextWave.Hide()` | Changes window visibility without starting or stopping the timer.   |
| `UI.NextWave.Remaining()`                   | Returns seconds remaining, or `0` before the timer exists.          |

## Save and build the Lua

1. Set **Lua folder**, or choose **Browse…**, to select the folder containing your modules.
2. Choose **Show Lua** to inspect the selected design's generated code. Use the `UI.Name`
   shown by the editor; spaces and punctuation in design names are converted, and colliding
   Lua names are numbered.
3. Choose **Save and build test copy**. It writes one `ui_<name>.lua` module per design,
   saves the map and `<map> UI.json`, then builds a `[DEV]` copy beside the map with every
   module in the folder. The source map must use Lua.
4. Open that development copy in Warcraft III and test your choices, scores and timers.

<figure class="app-screenshot">
  <a href="../../screenshots/interface-lua-build.png">
    <img src="../../screenshots/interface-lua-build.png" alt="Interface tab with Show Lua expanded, the Lua folder set to a temporary directory and a success message for five modules and Field guide_dev.w3m." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption>Save and build test copy wrote five generated modules and built the development map; this is a build result, not an in-game test.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

Tinkerbench rewrites its generated modules on the next build. Keep your callbacks and game
logic in separate files. The modules create their game handles through `Dev.OnStart`;
make calls from your own startup callback after the generated modules' callbacks, or from
later game events. The harness loads files alphabetically, so a name such as
`zz_score_updates.lua` puts your startup registration after `ui_*.lua`.
See [loading order and iteration](../lua-harness/) before combining modules.

## Reuse designs in another map

**Duplicate** copies the selected design as a starting point in the same map.
To transfer designs:
1. Choose **Export…** in **Interface**. In **Export designs**, select the designs to include.
   The preview also includes dialogs their buttons show and imported icons they use.
2. Review the listed Lua callbacks, then choose **Save design file…** to write a `.w3ui` file.
3. Open the destination map and choose **Import…** in its **Interface** tab. Select that file.
4. Review **Import designs**, including the resulting `UI.Name` values, then choose
   **Import** with the displayed design count. Save the destination map and build its test copy.

<figure class="app-screenshot">
  <a href="../../screenshots/interface-export.png">
    <img src="../../screenshots/interface-export.png" alt="Export designs with Difficulty and Scoreboard selected, Details brought along by Difficulty, an imported icon and the required OnHardMode callback." width="678" height="577" loading="lazy" />
  </a>
  <figcaption>The export preview brings Details along because a Difficulty button shows it, and lists the imported icon and required callback.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

<figure class="app-screenshot">
  <a href="../../screenshots/interface-import.png">
    <img src="../../screenshots/interface-import.png" alt="Import designs preview with Difficulty 2, Scoreboard 2 and Details 2 name conflicts, their Lua names, an existing imported icon and the OnHardMode requirement." width="678" height="410" loading="lazy" />
  </a>
  <figcaption>The import preview numbers taken design names and shows the resulting Lua names before you add anything.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

Existing designs remain. An imported design with a taken name receives a number, such as
`Scoreboard 2`; icons the destination lacks are added. The design file lists required Lua
function names but does not carry your callback implementations. Add those modules to the
destination's **Lua folder** and check any handwritten `UI.Name` references after renaming.
