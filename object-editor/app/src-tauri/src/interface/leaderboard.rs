//! Leaderboards: a titled list of names and numbers in the top right corner,
//! one row per playing player and up to four rows of the map's own, kept
//! sorted by value if asked. The Lua changes a row's value by player or name
//! (`UI.Kills.AddValue(player, 1)`).

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use wc3_map::lua::check_syntax;

use super::{MODULE_MARKER, SHOW_DELAY, color_code, comment_text, lua_string, module_file};

/// The game ties every leaderboard row to a player; rows of the map's own
/// use the four neutral players, so there can be at most four.
pub const MAX_LABELS: usize = 4;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardDesign {
    pub id: String,
    pub name: String,
    pub title: String,
    /// The title's color as `RRGGBB`; the game's own when `None`.
    #[serde(default)]
    pub title_color: Option<String>,
    /// A row for every playing player, with their name in their color.
    pub players: bool,
    /// What the players' rows start at.
    #[serde(default)]
    pub player_value: i32,
    /// Rows of the map's own, such as teams.
    #[serde(default)]
    pub labels: Vec<LeaderboardLabel>,
    pub sort: LeaderboardSort,
    /// Shown shortly after the game starts.
    pub show_at_start: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardLabel {
    /// Shown, and how the Lua names the row: `SetValue("Alliance", 3)`.
    pub name: String,
    /// `RRGGBB`; white when `None`.
    #[serde(default)]
    pub color: Option<String>,
    pub value: i32,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum LeaderboardSort {
    /// In the order the rows were made: players, then labels.
    None,
    HighFirst,
    LowFirst,
}

fn check(board: &LeaderboardDesign) -> Result<(), String> {
    let problem = |what: String| Err(format!("{}: {what}", board.name));

    if !board.players && board.labels.is_empty() {
        return problem("it shows no rows: add players or a row of your own".to_owned());
    }

    if board.labels.len() > MAX_LABELS {
        return problem(format!(
            "at most {MAX_LABELS} rows of your own (the game gives each row a player, and these use the neutral ones)"
        ));
    }

    let mut seen = HashSet::new();

    for (i, label) in board.labels.iter().enumerate() {
        if label.name.trim().is_empty() {
            return problem(format!("row {} needs a name", i + 1));
        }

        if !seen.insert(label.name.trim().to_lowercase()) {
            return problem(format!("two rows are named {:?}", label.name.trim()));
        }

        if let Some(color) = &label.color {
            color_code(color).map_err(|e| format!("{}: {e}", board.name))?;
        }
    }

    if let Some(color) = &board.title_color {
        color_code(color).map_err(|e| format!("{}: {e}", board.name))?;
    }

    Ok(())
}

/// The Lua module for one leaderboard, checked as Lua 5.3.
pub fn leaderboard_lua(
    board: &LeaderboardDesign,
    names: &HashMap<String, String>,
    source: &str,
) -> Result<String, String> {
    check(board)?;
    let name = names
        .get(&board.id)
        .ok_or_else(|| format!("{} has no Lua name", board.name))?;

    let labels: Vec<String> = board
        .labels
        .iter()
        .map(|label| {
            let color = match &label.color {
                Some(c) => lua_string(&c.to_ascii_lowercase()),
                None => "nil".to_owned(),
            };
            format!(
                "        {{ name = {}, color = {color}, value = {} }},",
                lua_string(label.name.trim()),
                label.value
            )
        })
        .collect();
    let title = match &board.title_color {
        Some(color) => lua_string(&format!("{}{}|r", color_code(color)?, board.title)),
        None => lua_string(&board.title),
    };
    let sort = match board.sort {
        LeaderboardSort::None => "nil",
        LeaderboardSort::HighFirst => "false",
        LeaderboardSort::LowFirst => "true",
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
        r#"{MODULE_MARKER}: the leaderboard {name_comment} (UI.{name}), from {source_comment}.
-- Edit it there: Tinkerbench rewrites this file. From your own code:
--   UI.{name}.SetValue(row, value) and .AddValue(row, amount)   -- row: a player, or a row's name
--   UI.{name}.GetValue(row), .Show(), .Hide()

UI = UI or {{}}

do
    local TITLE = {title}
    local PLAYERS = {players}
    local PLAYER_VALUE = {player_value}
    local LABELS = {{
{labels}
    }}
    -- Sorted after each change: nil keeps the order, true lowest first, false highest first.
    local LOWEST_FIRST = {sort}
    -- Rows of the map's own belong to the neutral players: the game gives each row one.
    local NEUTRALS = {{
        PLAYER_NEUTRAL_PASSIVE, PLAYER_NEUTRAL_AGGRESSIVE, bj_PLAYER_NEUTRAL_VICTIM,
        bj_PLAYER_NEUTRAL_EXTRA,
    }}
    -- The game's player colors, by GetHandleId(GetPlayerColor(player)).
    local PLAYER_COLORS = {{
        [0] = "ff0303", "0042ff", "1ce6b9", "540081", "fffc00", "fe8a0e", "20c000",
        "e55bb0", "959697", "7ebff1", "106246", "4a2a04", "9b0000", "0000c3", "00eaff",
        "be00fe", "ebcd87", "f8a48b", "bfff80", "dcb9eb", "282828", "ebf0ff", "00781e",
        "a46f33",
    }}

    local board = nil
    -- The player each row's item belongs to, by player or row name.
    local ownerOf = {{}}
    local values = {{}}

    local function playerName(player)
        local color = PLAYER_COLORS[GetHandleId(GetPlayerColor(player))] or "ffffff"
        return "|cff" .. color .. GetPlayerName(player) .. "|r"
    end

    local function sort()
        if LOWEST_FIRST ~= nil then
            LeaderboardSortItemsByValue(board, LOWEST_FIRST)
        end
    end

    local function add(row, owner, label, value)
        LeaderboardAddItem(board, label, value, owner)
        ownerOf[row] = owner
        values[row] = value
    end

    local function create()
        board = CreateLeaderboard()
        LeaderboardSetLabel(board, TITLE)
        if PLAYERS then
            for i = 0, bj_MAX_PLAYERS - 1 do
                local player = Player(i)
                if GetPlayerSlotState(player) == PLAYER_SLOT_STATE_PLAYING then
                    add(player, player, playerName(player), PLAYER_VALUE)
                end
            end
        end
        for i, label in ipairs(LABELS) do
            local text = label.name
            if label.color ~= nil then
                text = "|cff" .. label.color .. text .. "|r"
            end
            add(label.name, Player(NEUTRALS[i]), text, label.value)
        end
        LeaderboardSetSizeByItemCount(board, LeaderboardGetItemCount(board))
        sort()
        for i = 0, bj_MAX_PLAYERS - 1 do
            PlayerSetLeaderboard(Player(i), board)
        end
    end

    UI.{name} = {{
        Show = function()
            if board ~= nil then LeaderboardDisplay(board, true) end
        end,
        Hide = function()
            if board ~= nil then LeaderboardDisplay(board, false) end
        end,
        SetValue = function(row, value)
            local owner = ownerOf[row]
            if board == nil or owner == nil then return end
            values[row] = value
            LeaderboardSetItemValue(board, LeaderboardGetPlayerIndex(board, owner), value)
            sort()
        end,
        AddValue = function(row, amount)
            if values[row] ~= nil then
                UI.{name}.SetValue(row, values[row] + amount)
            end
        end,
        GetValue = function(row)
            return values[row]
        end,
    }}

    Dev.OnStart(function()
        create(){at_start}
    end)
end
"#,
        name_comment = comment_text(&board.name),
        source_comment = comment_text(source),
        players = board.players,
        player_value = board.player_value,
        labels = labels.join("\n"),
    );

    check_syntax(&module_file(name), &code).map_err(|e| e.to_string())?;
    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::{Design, lua_names};

    fn label(name: &str, value: i32) -> LeaderboardLabel {
        LeaderboardLabel {
            name: name.to_owned(),
            color: Some("FF0303".to_owned()),
            value,
        }
    }

    fn kills() -> LeaderboardDesign {
        LeaderboardDesign {
            id: "l".to_owned(),
            name: "Kills".to_owned(),
            title: "Kills".to_owned(),
            title_color: None,
            players: true,
            player_value: 0,
            labels: vec![label("Creeps", 10)],
            sort: LeaderboardSort::HighFirst,
            show_at_start: true,
        }
    }

    fn names(board: &LeaderboardDesign) -> HashMap<String, String> {
        lua_names(&[Design::Leaderboard(board.clone())])
    }

    #[test]
    fn leaderboards_become_valid_lua_with_their_rows_and_order() {
        let board = kills();
        let code = leaderboard_lua(&board, &names(&board), "test UI.json").unwrap();

        assert!(code.starts_with(MODULE_MARKER), "{code}");
        assert!(code.contains("local PLAYERS = true"), "{code}");
        assert!(
            code.contains("{ name = \"Creeps\", color = \"ff0303\", value = 10 }"),
            "{code}"
        );
        assert!(code.contains("local LOWEST_FIRST = false"), "{code}");
        assert!(code.contains("UI.Kills.Show()"), "{code}");

        let mut unsorted = kills();

        unsorted.sort = LeaderboardSort::None;
        let code = leaderboard_lua(&unsorted, &names(&unsorted), "s").unwrap();

        assert!(code.contains("local LOWEST_FIRST = nil"), "{code}");
    }

    #[test]
    fn bad_leaderboards_are_reported() {
        let mut empty = kills();

        empty.players = false;
        empty.labels.clear();
        assert!(leaderboard_lua(&empty, &names(&empty), "s").is_err());

        let mut crowded = kills();

        crowded.labels = (0..5).map(|i| label(&format!("Team {i}"), 0)).collect();
        let err = leaderboard_lua(&crowded, &names(&crowded), "s").unwrap_err();

        assert!(err.contains("at most 4"), "{err}");

        let mut twins = kills();

        twins.labels.push(label("creeps", 1));
        assert!(leaderboard_lua(&twins, &names(&twins), "s").is_err());
    }
}
