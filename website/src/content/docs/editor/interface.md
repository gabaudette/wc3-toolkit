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
These instructions and Lua signatures were checked against the editor and its generators.
The previews and generated code do not establish game behavior. The designs and examples
on this page are **not yet confirmed in game**.
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

<!-- Screenshot placeholder: Interface tab, Difficulty dialog selected; Message, Message color,
two buttons with hotkeys, When clicked: Run a Lua function with OnHardMode, and live preview visible.
Use an app-screenshot figure, descriptive alt text and an Image subject to change. notice. -->

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

<!-- Screenshot placeholder: Interface tab, Scoreboard multiboard selected; Player, Kills and
Deaths columns, repeating player row, a selected column's Style controls and live preview visible.
Use an app-screenshot figure, descriptive alt text and an Image subject to change. notice. -->

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

<!-- Screenshot placeholder: Interface tab, Kills leaderboard selected; player starting value,
one own row, Order set to Highest first and live preview visible. Use an app-screenshot figure,
descriptive alt text and an Image subject to change. notice. -->

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

<!-- Screenshot placeholder: Interface tab, Next wave timer selected; Length, repeat and
start options, SpawnWave callback and countdown preview visible. Use an app-screenshot figure,
descriptive alt text and an Image subject to change. notice. -->

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

<!-- Screenshot placeholder: Interface tab with Show Lua expanded and Lua folder filled;
show the success message after Save and build test copy. Use an app-screenshot figure,
descriptive alt text and an Image subject to change. notice. -->

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

<!-- Screenshot placeholder: Interface tab with Import designs open; show a numbered name
collision, a brought-along dialog, imported icon and required Lua callback list. Use an
app-screenshot figure, descriptive alt text and an Image subject to change. notice. -->

Existing designs remain. An imported design with a taken name receives a number, such as
`Scoreboard 2`; icons the destination lacks are added. The design file lists required Lua
function names but does not carry your callback implementations. Add those modules to the
destination's **Lua folder** and check any handwritten `UI.Name` references after renaming.
