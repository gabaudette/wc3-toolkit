//! Multiboards: a titled board of rows and columns in the top right corner,
//! like a scoreboard. Rows are fixed or repeat once per playing player; the
//! Lua sets any cell by row and column name afterwards
//! (`UI.Scoreboard.SetValue(player, "Kills", 5)`).

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use wc3_map::lua::check_syntax;

use super::{MODULE_MARKER, SHOW_DELAY, color_code, comment_text, lua_string, module_file};

/// The text of a cell on a player's row that becomes that player's name, in
/// their color.
pub const PLAYER_NAME: &str = "{player}";
/// Columns are at most this share of the screen's width, in percent.
const MAX_WIDTH: f64 = 100.0;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MultiboardDesign {
    pub id: String,
    pub name: String,
    pub title: String,
    /// The title's color as `RRGGBB`; the game's own when `None`.
    #[serde(default)]
    pub title_color: Option<String>,
    pub columns: Vec<BoardColumn>,
    pub rows: Vec<BoardRow>,
    /// Shown shortly after the game starts.
    pub show_at_start: bool,
    /// Shown folded to its title bar; players open it with a click.
    #[serde(default)]
    pub minimized: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BoardColumn {
    #[serde(default)]
    pub id: String,
    /// How the Lua names it: `SetValue(row, "Kills", 5)`.
    pub name: String,
    /// Width in percent of the screen's width.
    pub width: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BoardRow {
    #[serde(default)]
    pub id: String,
    /// How the Lua names a fixed row; rows per player are named by the player.
    pub name: String,
    /// One row for every playing player, in player order, instead of one.
    pub per_player: bool,
    /// One per column; missing ones are empty.
    pub cells: Vec<BoardCell>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct BoardCell {
    /// On a player's row, `{player}` is that player's name in their color.
    pub text: String,
    /// The text's color as `RRGGBB`; white when `None`.
    #[serde(default)]
    pub color: Option<String>,
    /// An icon left of the text, such as `ReplaceableTextures\CommandButtons\BTNFootman.blp`.
    #[serde(default)]
    pub icon: Option<String>,
}

/// Problems that would make the board misbehave, as sentences.
fn check(board: &MultiboardDesign) -> Result<(), String> {
    let problem = |what: String| Err(format!("{}: {what}", board.name));

    if board.columns.is_empty() {
        return problem("it needs at least one column".to_owned());
    }

    let mut seen = HashSet::new();

    for (i, column) in board.columns.iter().enumerate() {
        let n = i + 1;

        if column.name.trim().is_empty() {
            return problem(format!("column {n} needs a name"));
        }

        if !seen.insert(column.name.trim().to_lowercase()) {
            return problem(format!("two columns are named {:?}", column.name.trim()));
        }

        if !(column.width > 0.0 && column.width <= MAX_WIDTH) {
            return problem(format!(
                "column {:?} must be wider than 0% and at most 100% of the screen",
                column.name
            ));
        }
    }

    let mut seen = HashSet::new();

    for (i, row) in board.rows.iter().enumerate() {
        if !row.per_player && !seen.insert(row.name.trim().to_lowercase()) {
            return problem(format!("two rows are named {:?}", row.name.trim()));
        }

        for cell in &row.cells {
            if let Some(color) = &cell.color {
                color_code(color).map_err(|e| format!("{}, row {}: {e}", board.name, i + 1))?;
            }
        }
    }

    if let Some(color) = &board.title_color {
        color_code(color).map_err(|e| format!("{}: {e}", board.name))?;
    }

    Ok(())
}

fn lua_cell(cell: &BoardCell) -> String {
    let optional = |value: &Option<String>| match value {
        Some(v) => lua_string(v),
        None => "nil".to_owned(),
    };

    format!(
        "{{ text = {}, color = {}, icon = {} }}",
        lua_string(&cell.text),
        optional(&cell.color.as_ref().map(|c| c.to_ascii_lowercase())),
        optional(&cell.icon)
    )
}

/// The Lua module for one multiboard, checked as Lua 5.3.
pub fn multiboard_lua(
    board: &MultiboardDesign,
    names: &HashMap<String, String>,
    source: &str,
) -> Result<String, String> {
    check(board)?;
    let name = names
        .get(&board.id)
        .ok_or_else(|| format!("{} has no Lua name", board.name))?;

    let columns: Vec<String> = board
        .columns
        .iter()
        .map(|c| {
            format!(
                "        {{ name = {}, width = {} }},",
                lua_string(c.name.trim()),
                c.width / 100.0
            )
        })
        .collect();
    let rows: Vec<String> = board
        .rows
        .iter()
        .map(|row| {
            let cells: Vec<String> = (0..board.columns.len())
                .map(|i| {
                    let cell = row.cells.get(i).cloned().unwrap_or_default();
                    format!("                {},", lua_cell(&cell))
                })
                .collect();
            format!(
                "        {{\n            name = {},\n            perPlayer = {},\n            cells = {{\n{}\n            }},\n        }},",
                lua_string(row.name.trim()),
                row.per_player,
                cells.join("\n")
            )
        })
        .collect();
    let title = match &board.title_color {
        Some(color) => lua_string(&format!("{}{}|r", color_code(color)?, board.title)),
        None => lua_string(&board.title),
    };
    let at_start = if board.show_at_start {
        format!(
            r#"
        local timer = CreateTimer()
        TimerStart(timer, {SHOW_DELAY}, false, function()
            DestroyTimer(timer)
            UI.{name}.Show()
        end)"#
        )
    } else {
        String::new()
    };

    let code = format!(
        r#"{MODULE_MARKER}: the multiboard {name_comment} (UI.{name}), from {source_comment}.
-- Edit it there: Tinkerbench rewrites this file. From your own code:
--   UI.{name}.SetValue(row, column, value)   -- row: a row's name, or a player
--   UI.{name}.SetColor(row, column, "ff0303") and .SetIcon(row, column, path)
--   UI.{name}.Show(), .Hide(), .Minimize(true or false)

UI = UI or {{}}

do
    local TITLE = {title}
    local MINIMIZED = {minimized}
    local COLUMNS = {{
{columns}
    }}
    -- In order; rows per player repeat once for every playing player.
    local ROWS = {{
{rows}
    }}
    -- The game's player colors, by GetHandleId(GetPlayerColor(player)).
    local PLAYER_COLORS = {{
        [0] = "ff0303", "0042ff", "1ce6b9", "540081", "fffc00", "fe8a0e", "20c000",
        "e55bb0", "959697", "7ebff1", "106246", "4a2a04", "9b0000", "0000c3", "00eaff",
        "be00fe", "ebcd87", "f8a48b", "bfff80", "dcb9eb", "282828", "ebf0ff", "00781e",
        "a46f33",
    }}

    local board = nil
    -- Board row (from 0) by row name, or by player for rows per player.
    local rowOf = {{}}
    local columnOf = {{}}
    -- What each cell shows, by board row and column, to redraw one part at a time.
    local cells = {{}}

    local function playerName(player)
        local color = PLAYER_COLORS[GetHandleId(GetPlayerColor(player))] or "ffffff"
        return "|cff" .. color .. GetPlayerName(player) .. "|r"
    end

    local function draw(row, column)
        local cell = cells[row][column]
        local item = MultiboardGetItem(board, row, column)
        local text = cell.text
        if cell.color ~= nil then
            text = "|cff" .. cell.color .. text .. "|r"
        end
        MultiboardSetItemValue(item, text)
        MultiboardSetItemStyle(item, true, cell.icon ~= nil)
        if cell.icon ~= nil then
            MultiboardSetItemIcon(item, cell.icon)
        end
        MultiboardSetItemWidth(item, COLUMNS[column + 1].width)
        MultiboardReleaseItem(item)
    end

    -- The players in the game, computers included, in player order.
    local function playingPlayers()
        local players = {{}}
        for i = 0, bj_MAX_PLAYERS - 1 do
            local player = Player(i)
            if GetPlayerSlotState(player) == PLAYER_SLOT_STATE_PLAYING then
                table.insert(players, player)
            end
        end
        return players
    end

    local function create()
        local count = 0
        local players = playingPlayers()
        for _, spec in ipairs(ROWS) do
            local owners = {{ false }}
            if spec.perPlayer then
                owners = players
            end
            for _, owner in ipairs(owners) do
                local row = count
                count = count + 1
                if owner then
                    rowOf[owner] = row
                else
                    rowOf[spec.name] = row
                end
                cells[row] = {{}}
                for column, cell in ipairs(spec.cells) do
                    local text = cell.text
                    if owner then
                        text = string.gsub(text, {player_pattern}, function() return playerName(owner) end)
                    end
                    cells[row][column - 1] = {{ text = text, color = cell.color, icon = cell.icon }}
                end
            end
        end
        for column, spec in ipairs(COLUMNS) do
            columnOf[spec.name] = column - 1
        end

        board = CreateMultiboard()
        MultiboardSetRowCount(board, count)
        MultiboardSetColumnCount(board, #COLUMNS)
        MultiboardSetTitleText(board, TITLE)
        for row = 0, count - 1 do
            for column = 0, #COLUMNS - 1 do
                draw(row, column)
            end
        end
    end

    -- The cell for a row (name or player) and a column name, or nil.
    local function find(row, column)
        local r, c = rowOf[row], columnOf[column]
        if r == nil or c == nil then
            return nil
        end
        return r, c
    end

    UI.{name} = {{
        Show = function()
            if board ~= nil then
                MultiboardDisplay(board, true)
                MultiboardMinimize(board, MINIMIZED)
            end
        end,
        Hide = function()
            if board ~= nil then MultiboardDisplay(board, false) end
        end,
        Minimize = function(minimized)
            if board ~= nil then MultiboardMinimize(board, minimized) end
        end,
        SetValue = function(row, column, value)
            local r, c = find(row, column)
            if r == nil then return end
            cells[r][c].text = tostring(value)
            draw(r, c)
        end,
        -- A color as "RRGGBB", or nil for white.
        SetColor = function(row, column, color)
            local r, c = find(row, column)
            if r == nil then return end
            cells[r][c].color = color
            draw(r, c)
        end,
        -- An icon path, or nil for none.
        SetIcon = function(row, column, path)
            local r, c = find(row, column)
            if r == nil then return end
            cells[r][c].icon = path
            draw(r, c)
        end,
    }}

    Dev.OnStart(function()
        create(){at_start}
    end)
end
"#,
        name_comment = comment_text(&board.name),
        // No pattern characters in it, so Lua matches it as plain text.
        player_pattern = lua_string(PLAYER_NAME),
        source_comment = comment_text(source),
        minimized = board.minimized,
        columns = columns.join("\n"),
        rows = rows.join("\n"),
    );

    check_syntax(&module_file(name), &code).map_err(|e| e.to_string())?;
    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::{Design, lua_names};

    fn cell(text: &str) -> BoardCell {
        BoardCell {
            text: text.to_owned(),
            color: None,
            icon: None,
        }
    }

    pub fn scoreboard() -> MultiboardDesign {
        MultiboardDesign {
            id: "m".to_owned(),
            name: "Scoreboard".to_owned(),
            title: "Kills".to_owned(),
            title_color: Some("FFCC00".to_owned()),
            columns: vec![
                BoardColumn {
                    id: String::new(),
                    name: "Player".to_owned(),
                    width: 12.0,
                },
                BoardColumn {
                    id: String::new(),
                    name: "Kills".to_owned(),
                    width: 5.0,
                },
            ],
            rows: vec![
                BoardRow {
                    id: String::new(),
                    name: "Header".to_owned(),
                    per_player: false,
                    cells: vec![
                        BoardCell {
                            color: Some("FFCC00".to_owned()),
                            ..cell("Player")
                        },
                        cell("Kills"),
                    ],
                },
                BoardRow {
                    id: String::new(),
                    name: "Players".to_owned(),
                    per_player: true,
                    cells: vec![
                        BoardCell {
                            icon: Some(
                                "ReplaceableTextures\\CommandButtons\\BTNFootman.blp".to_owned(),
                            ),
                            ..cell(PLAYER_NAME)
                        },
                        cell("0"),
                    ],
                },
            ],
            show_at_start: true,
            minimized: false,
        }
    }

    fn names(board: &MultiboardDesign) -> HashMap<String, String> {
        lua_names(&[Design::Multiboard(board.clone())])
    }

    #[test]
    fn boards_become_valid_lua_with_their_rows_and_cells() {
        let board = scoreboard();
        let code = multiboard_lua(&board, &names(&board), "test UI.json").unwrap();

        assert!(code.starts_with(MODULE_MARKER), "{code}");
        assert!(
            code.contains("local TITLE = \"|cffffcc00Kills|r\""),
            "{code}"
        );
        assert!(
            code.contains("{ name = \"Player\", width = 0.12 }"),
            "{code}"
        );
        assert!(code.contains("perPlayer = true"), "{code}");
        assert!(
            code.contains("icon = \"ReplaceableTextures\\\\CommandButtons\\\\BTNFootman.blp\""),
            "{code}"
        );
        assert!(code.contains("color = \"ffcc00\""), "{code}");
        assert!(code.contains("UI.Scoreboard.Show()"), "{code}");
    }

    #[test]
    fn short_rows_get_empty_cells() {
        let mut board = scoreboard();

        board.rows[0].cells.truncate(1);
        let code = multiboard_lua(&board, &names(&board), "s").unwrap();

        assert!(
            code.contains("{ text = \"\", color = nil, icon = nil }"),
            "{code}"
        );
    }

    #[test]
    fn bad_boards_are_reported() {
        let mut no_columns = scoreboard();

        no_columns.columns.clear();
        assert!(multiboard_lua(&no_columns, &names(&no_columns), "s").is_err());

        let mut twin_columns = scoreboard();

        twin_columns.columns[1].name = "player".to_owned();
        let err = multiboard_lua(&twin_columns, &names(&twin_columns), "s").unwrap_err();

        assert!(err.contains("two columns"), "{err}");

        let mut too_wide = scoreboard();

        too_wide.columns[0].width = 0.0;
        assert!(multiboard_lua(&too_wide, &names(&too_wide), "s").is_err());

        let mut twin_rows = scoreboard();

        twin_rows.rows[1].per_player = false;
        twin_rows.rows[1].name = "HEADER".to_owned();
        assert!(multiboard_lua(&twin_rows, &names(&twin_rows), "s").is_err());

        let mut bad_color = scoreboard();

        bad_color.rows[0].cells[0].color = Some("gold".to_owned());
        assert!(multiboard_lua(&bad_color, &names(&bad_color), "s").is_err());
    }
}
