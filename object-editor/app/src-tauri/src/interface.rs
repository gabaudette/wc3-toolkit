//! The Interface tab: in-game UI designed visually (dialogs, multiboards,
//! leaderboards and timer windows), kept in
//! `<map> UI.json` next to the map, and the Lua that builds it in game. The
//! modules go in the map's Lua folder with the triggered spells, for the
//! harness's `[DEV]` copy.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::State;
use wc3_map::lua::check_syntax;

use crate::state::{AppState, NO_MAP};

mod leaderboard;
mod multiboard;
mod timer;

pub use leaderboard::LeaderboardDesign;
use leaderboard::leaderboard_lua;
pub use multiboard::MultiboardDesign;
use multiboard::multiboard_lua;
pub use timer::TimerWindowDesign;
use timer::timer_lua;

/// Bumped when the file changes in a way older versions cannot read.
pub const FORMAT_VERSION: u32 = 1;
/// The first line of every module this tab writes: how it finds its own
/// files again, to replace them, and to remove those of deleted designs.
const MODULE_MARKER: &str = "-- Made in Tinkerbench's Interface tab";
const MODULE_PREFIX: &str = "ui_";
/// Seconds after the game starts before dialogs shown at start appear: shown
/// during map init they never do.
const SHOW_DELAY: &str = "1.0";
const LUA_KEYWORDS: &[&str] = &[
    "and", "break", "do", "else", "elseif", "end", "false", "for", "function", "goto", "if", "in",
    "local", "nil", "not", "or", "repeat", "return", "then", "true", "until", "while",
];

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct UiFile {
    pub version: u32,
    pub designs: Vec<Design>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Design {
    Dialog(DialogDesign),
    Multiboard(MultiboardDesign),
    Leaderboard(LeaderboardDesign),
    TimerWindow(TimerWindowDesign),
}

impl Design {
    fn id(&self) -> &str {
        match self {
            Design::Dialog(d) => &d.id,
            Design::Multiboard(m) => &m.id,
            Design::Leaderboard(l) => &l.id,
            Design::TimerWindow(t) => &t.id,
        }
    }

    fn name(&self) -> &str {
        match self {
            Design::Dialog(d) => &d.name,
            Design::Multiboard(m) => &m.name,
            Design::Leaderboard(l) => &l.name,
            Design::TimerWindow(t) => &t.name,
        }
    }
}

/// A dialog: a message and a column of buttons, shown to chosen players.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DialogDesign {
    /// Stable, for references between designs; never shown.
    pub id: String,
    /// The name in the tab; the Lua name (`UI.ChoosePath`) comes from it.
    pub name: String,
    /// The message at the top, as plain text (color codes still work).
    pub title: String,
    /// The message's color as `RRGGBB`; the game's own gold when `None`.
    #[serde(default)]
    pub title_color: Option<String>,
    pub buttons: Vec<DialogButton>,
    /// Shown to every player shortly after the game starts.
    pub show_at_start: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DialogButton {
    pub text: String,
    /// One letter or digit, or empty for none.
    pub hotkey: String,
    /// The hotkey's letter in the text turns gold, as in the game's own dialogs.
    #[serde(default = "yes")]
    pub highlight_hotkey: bool,
    pub action: ButtonAction,
}

fn yes() -> bool {
    true
}

/// The gold the game highlights hotkeys with.
const HOTKEY_COLOR: &str = "ffcc00";

fn color_code(color: &str) -> Result<String, String> {
    if color.len() == 6 && color.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(format!("|cff{}", color.to_ascii_lowercase()))
    } else {
        Err(format!(
            "{color:?} is not a color (six hex digits, like ffcc00)"
        ))
    }
}

/// The message as the game gets it: in its color, when it has one.
pub fn title_text(dialog: &DialogDesign) -> Result<String, String> {
    match &dialog.title_color {
        Some(color) => Ok(format!("{}{}|r", color_code(color)?, dialog.title)),
        None => Ok(dialog.title.clone()),
    }
}

/// A button's text as the game gets it: the first letter matching its
/// hotkey in gold (`|cffffcc00H|rard`), when highlighted. Text without that
/// letter stays as it is.
pub fn button_text(button: &DialogButton) -> String {
    let key = button.hotkey.trim();
    if !button.highlight_hotkey || key.chars().count() != 1 {
        return button.text.clone();
    }

    let Some((at, letter)) = button
        .text
        .char_indices()
        .find(|(_, c)| c.to_string().eq_ignore_ascii_case(key))
    else {
        return button.text.clone();
    };
    let end = at + letter.len_utf8();
    format!(
        "{}|cff{HOTKEY_COLOR}{letter}|r{}",
        &button.text[..at],
        &button.text[end..]
    )
}

/// What a button does once clicked. The game hides the dialog first.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ButtonAction {
    /// Nothing more: the dialog is already hidden.
    Close,
    /// Shows another dialog to the player who clicked.
    ShowDialog { dialog: String },
    /// Calls a global Lua function with the player who clicked, if the map
    /// defines one: `function OnHardMode(player) ... end`.
    RunFunction { function: String },
}

/// `test.w3m` → `test UI.json`, beside it.
pub fn file_for(map: &Path) -> PathBuf {
    let stem = map.file_stem().unwrap_or_default().to_string_lossy();
    map.with_file_name(format!("{stem} UI.json"))
}

/// The map's designs; none when it has no file yet.
pub fn load(map: &Path) -> Result<Vec<Design>, String> {
    let path = file_for(map);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("Cannot read {}: {e}", path.display())),
    };
    let file: UiFile = serde_json::from_str(&text).map_err(|e| {
        format!(
            "{} is not a design file Tinkerbench can read: {e}",
            path.display()
        )
    })?;
    if file.version > FORMAT_VERSION {
        return Err(format!(
            "{} was made by a newer Tinkerbench (format {}); update to edit it.",
            path.display(),
            file.version
        ));
    }

    Ok(file.designs)
}

pub fn save(map: &Path, designs: &[Design]) -> Result<(), String> {
    let path = file_for(map);
    let file = UiFile {
        version: FORMAT_VERSION,
        designs: designs.to_vec(),
    };
    let text = serde_json::to_string_pretty(&file).map_err(|e| e.to_string())?;
    std::fs::write(&path, text + "\n").map_err(|e| format!("Cannot write {}: {e}", path.display()))
}

fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !LUA_KEYWORDS.contains(&name)
}

/// `Choose your path!` → `ChooseYourPath`; `Design` when nothing is left.
fn pascal_case(name: &str) -> String {
    let mut out = String::new();
    for word in name.split(|c: char| !c.is_ascii_alphanumeric()) {
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            out.push(first.to_ascii_uppercase());
            out.extend(chars);
        }
    }
    if out.is_empty() || out.starts_with(|c: char| c.is_ascii_digit()) {
        out.insert_str(0, "Design");
    }

    out
}

/// Each design's Lua name, by ID: from its name, numbered when two match.
pub fn lua_names(designs: &[Design]) -> HashMap<String, String> {
    let mut taken = HashSet::new();
    designs
        .iter()
        .map(|design| {
            let base = pascal_case(design.name());
            let mut name = base.clone();
            let mut n = 2;
            while !taken.insert(name.to_ascii_lowercase()) {
                name = format!("{base}{n}");
                n += 1;
            }
            (design.id().to_owned(), name)
        })
        .collect()
}

/// A Lua string literal for any text.
fn lua_string(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            c if c.is_control() => out.push_str(&format!("\\{}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The code `DialogAddButton` takes for a hotkey: the letter's character code.
fn hotkey_code(hotkey: &str) -> Result<u32, String> {
    let mut chars = hotkey.trim().chars();
    match (chars.next(), chars.next()) {
        (None, _) => Ok(0),
        (Some(c), None) if c.is_ascii_alphanumeric() => Ok(u32::from(c.to_ascii_uppercase())),
        _ => Err(format!("hotkey {hotkey:?} is not one letter or digit")),
    }
}

/// The Lua module for one dialog, checked as Lua 5.3.
pub fn dialog_lua(
    dialog: &DialogDesign,
    names: &HashMap<String, String>,
    source: &str,
) -> Result<String, String> {
    let name = names
        .get(&dialog.id)
        .ok_or_else(|| format!("{} has no Lua name", dialog.name))?;
    let mut buttons = String::new();
    for (i, button) in dialog.buttons.iter().enumerate() {
        let n = i + 1;
        let problem = |what: String| format!("{}, button {n}: {what}", dialog.name);
        let hotkey = hotkey_code(&button.hotkey).map_err(problem)?;
        let action = match &button.action {
            ButtonAction::Close => "function(player) end".to_owned(),
            ButtonAction::ShowDialog { dialog: target } => {
                let other = names
                    .get(target)
                    .ok_or_else(|| problem("it shows a dialog that no longer exists".to_owned()))?;
                format!(
                    "function(player)\n            if UI.{other} then UI.{other}.Show(player) end\n        end"
                )
            }
            ButtonAction::RunFunction { function } => {
                if !is_identifier(function) {
                    return Err(problem(format!(
                        "{function:?} is not a Lua function name (letters, digits and _, not starting with a digit)"
                    )));
                }
                format!(
                    "function(player)\n            if type({function}) == \"function\" then {function}(player) end\n        end"
                )
            }
        };
        buttons.push_str(&format!(
            "        buttons[{n}] = DialogAddButton(dialog, {}, {hotkey})\n        actions[{n}] = {action}\n",
            lua_string(&button_text(button))
        ));
    }

    let at_start = if dialog.show_at_start {
        format!(
            r#"
        local timer = CreateTimer()
        TimerStart(timer, {SHOW_DELAY}, false, function()
            DestroyTimer(timer)
            UI.{name}.ShowAll()
        end)"#
        )
    } else {
        String::new()
    };

    let code = format!(
        r#"{MODULE_MARKER}: the dialog {title_comment} (UI.{name}), from {source_comment}.
-- Edit it there: Tinkerbench rewrites this file. UI.{name}.Show(player),
-- .Hide(player) and .ShowAll() show it from your own code.

UI = UI or {{}}

do
    local dialog = nil
    local buttons = {{}}
    -- What each button does once clicked; the game hides the dialog first.
    local actions = {{}}

    local function display(player, show)
        if dialog ~= nil then
            DialogDisplay(player, dialog, show)
        end
    end

    UI.{name} = {{
        Show = function(player) display(player, true) end,
        Hide = function(player) display(player, false) end,
        -- Every player at the keyboard (not computers or empty slots).
        ShowAll = function()
            for i = 0, bj_MAX_PLAYERS - 1 do
                local player = Player(i)
                if GetPlayerSlotState(player) == PLAYER_SLOT_STATE_PLAYING
                    and GetPlayerController(player) == MAP_CONTROL_USER then
                    display(player, true)
                end
            end
        end,
    }}

    local function create()
        dialog = DialogCreate()
        DialogSetMessage(dialog, {title})
{buttons}
        local trigger = CreateTrigger()
        TriggerRegisterDialogEvent(trigger, dialog)
        TriggerAddAction(trigger, function()
            local clicked = GetClickedButton()
            for i, button in ipairs(buttons) do
                if button == clicked then
                    actions[i](GetTriggerPlayer())
                end
            end
        end)
    end

    Dev.OnStart(function()
        create(){at_start}
    end)
end
"#,
        title_comment = comment_text(&dialog.name),
        source_comment = comment_text(source),
        title = lua_string(&title_text(dialog)?),
    );
    check_syntax(&module_file(name), &code).map_err(|e| e.to_string())?;
    Ok(code)
}

/// Text safe inside a `--` comment: one line.
fn comment_text(text: &str) -> String {
    format!(
        "\"{}\"",
        text.chars()
            .map(|c| if c.is_control() { ' ' } else { c })
            .collect::<String>()
    )
}

fn module_file(lua_name: &str) -> String {
    format!("{MODULE_PREFIX}{}.lua", lua_name.to_ascii_lowercase())
}

/// Every design's module by file name.
fn modules(designs: &[Design], source: &str) -> Result<Vec<(String, String)>, String> {
    let names = lua_names(designs);
    designs
        .iter()
        .map(|design| match design {
            Design::Dialog(dialog) => {
                let code = dialog_lua(dialog, &names, source)?;
                Ok((module_file(&names[&dialog.id]), code))
            }
            Design::Multiboard(board) => {
                let code = multiboard_lua(board, &names, source)?;
                Ok((module_file(&names[&board.id]), code))
            }
            Design::Leaderboard(board) => {
                let code = leaderboard_lua(board, &names, source)?;
                Ok((module_file(&names[&board.id]), code))
            }
            Design::TimerWindow(timer) => {
                let code = timer_lua(timer, &names, source)?;
                Ok((module_file(&names[&timer.id]), code))
            }
        })
        .collect()
}

/// Writes every design's module into `dir` and removes modules this tab
/// wrote for designs that are gone. Returns the files written.
pub fn write_modules(dir: &Path, designs: &[Design], source: &str) -> Result<Vec<String>, String> {
    let modules = modules(designs, source)?;
    std::fs::create_dir_all(dir).map_err(|e| format!("Cannot create {}: {e}", dir.display()))?;

    let current: HashSet<&str> = modules.iter().map(|(file, _)| file.as_str()).collect();
    let entries =
        std::fs::read_dir(dir).map_err(|e| format!("Cannot read {}: {e}", dir.display()))?;
    for entry in entries.flatten() {
        let file = entry.file_name().to_string_lossy().into_owned();
        let ours = file.starts_with(MODULE_PREFIX)
            && file.ends_with(".lua")
            && !current.contains(file.as_str())
            && std::fs::read_to_string(entry.path()).is_ok_and(|t| t.starts_with(MODULE_MARKER));
        if ours {
            std::fs::remove_file(entry.path())
                .map_err(|e| format!("Cannot remove {}: {e}", entry.path().display()))?;
        }
    }

    modules
        .into_iter()
        .map(|(file, code)| {
            let path = dir.join(&file);
            std::fs::write(&path, code)
                .map_err(|e| format!("Cannot write {}: {e}", path.display()))?;
            Ok(file)
        })
        .collect()
}

fn source_name(map: &Path) -> String {
    file_for(map)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

#[tauri::command]
pub fn ui_designs(state: State<'_, AppState>) -> Result<Vec<Design>, String> {
    state.with(|session| Ok(session.map.as_ref().ok_or(NO_MAP)?.ui.clone()))
}

/// Replaces the map's designs; they are written with the map on Save.
#[tauri::command]
pub fn set_ui_designs(state: State<'_, AppState>, designs: Vec<Design>) -> Result<(), String> {
    state.with(|session| {
        session.map.as_mut().ok_or(NO_MAP)?.ui = designs;
        Ok(())
    })
}

/// One design's module, for the tab to show.
#[tauri::command]
pub fn ui_lua(state: State<'_, AppState>, id: String) -> Result<String, String> {
    state.with(|session| {
        let map = session.map.as_ref().ok_or(NO_MAP)?;
        let names = lua_names(&map.ui);
        let source = source_name(&map.path);
        match map.ui.iter().find(|d| d.id() == id) {
            Some(Design::Dialog(dialog)) => dialog_lua(dialog, &names, &source),
            Some(Design::Multiboard(board)) => multiboard_lua(board, &names, &source),
            Some(Design::Leaderboard(board)) => leaderboard_lua(board, &names, &source),
            Some(Design::TimerWindow(timer)) => timer_lua(timer, &names, &source),
            None => Err(format!("No design {id}.")),
        }
    })
}

/// Writes every design's module into the Lua folder; see [`write_modules`].
#[tauri::command]
pub fn write_ui_lua(state: State<'_, AppState>, dir: String) -> Result<Vec<String>, String> {
    state.with(|session| {
        let map = session.map.as_ref().ok_or(NO_MAP)?;
        write_modules(Path::new(&dir), &map.ui, &source_name(&map.path))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dialog(id: &str, name: &str, buttons: Vec<DialogButton>) -> DialogDesign {
        DialogDesign {
            id: id.to_owned(),
            name: name.to_owned(),
            title: "Choose \"wisely\"".to_owned(),
            title_color: Some("FF0303".to_owned()),
            buttons,
            show_at_start: true,
        }
    }

    fn button(text: &str, hotkey: &str, action: ButtonAction) -> DialogButton {
        DialogButton {
            text: text.to_owned(),
            hotkey: hotkey.to_owned(),
            highlight_hotkey: true,
            action,
        }
    }

    #[test]
    fn lua_names_come_from_names_and_stay_unique() {
        let designs = vec![
            Design::Dialog(dialog("a", "Choose your path!", vec![])),
            Design::Dialog(dialog("b", "choose  YOUR path", vec![])),
            Design::Dialog(dialog("c", "3 lives", vec![])),
            Design::Dialog(dialog("d", "", vec![])),
        ];
        let names = lua_names(&designs);
        assert_eq!(names["a"], "ChooseYourPath");
        assert_eq!(names["b"], "ChooseYOURPath2");
        assert_eq!(names["c"], "Design3Lives");
        assert_eq!(names["d"], "Design");
    }

    #[test]
    fn dialogs_become_valid_lua_with_their_buttons() {
        let main = dialog(
            "a",
            "Difficulty",
            vec![
                button("Easy", "e", ButtonAction::Close),
                button(
                    "More…",
                    "",
                    ButtonAction::ShowDialog {
                        dialog: "b".to_owned(),
                    },
                ),
                button(
                    "Hard",
                    "H",
                    ButtonAction::RunFunction {
                        function: "OnHardMode".to_owned(),
                    },
                ),
            ],
        );
        let other = dialog("b", "Details", vec![]);
        let designs = vec![Design::Dialog(main.clone()), Design::Dialog(other)];
        let code = dialog_lua(&main, &lua_names(&designs), "test UI.json").unwrap();

        assert!(code.starts_with(MODULE_MARKER), "{code}");
        assert!(
            code.contains("DialogAddButton(dialog, \"|cffffcc00E|rasy\", 69)"),
            "{code}"
        );
        assert!(
            code.contains("DialogAddButton(dialog, \"More…\", 0)"),
            "{code}"
        );
        assert!(code.contains("UI.Details.Show(player)"), "{code}");
        assert!(code.contains("OnHardMode(player)"), "{code}");
        assert!(
            code.contains("\"|cffff0303Choose \\\"wisely\\\"|r\""),
            "{code}"
        );
        assert!(code.contains("UI.Difficulty.ShowAll()"), "{code}");
    }

    #[test]
    fn hotkey_letters_turn_gold_when_highlighted() {
        let mut hard = button("Hard", "h", ButtonAction::Close);
        assert_eq!(button_text(&hard), "|cffffcc00H|rard");
        hard.text = "Go Ahead".to_owned();
        assert_eq!(button_text(&hard), "Go A|cffffcc00h|read");
        hard.text = "Normal".to_owned();
        assert_eq!(button_text(&hard), "Normal");
        hard.highlight_hotkey = false;
        hard.text = "Hard".to_owned();
        assert_eq!(button_text(&hard), "Hard");
    }

    #[test]
    fn bad_buttons_are_reported() {
        let names = |d: &DialogDesign| lua_names(&[Design::Dialog(d.clone())]);
        let two_keys = dialog("a", "A", vec![button("x", "ab", ButtonAction::Close)]);
        assert!(dialog_lua(&two_keys, &names(&two_keys), "s").is_err());

        let bad_function = dialog(
            "a",
            "A",
            vec![button(
                "x",
                "",
                ButtonAction::RunFunction {
                    function: "end".to_owned(),
                },
            )],
        );
        assert!(dialog_lua(&bad_function, &names(&bad_function), "s").is_err());

        let gone = dialog(
            "a",
            "A",
            vec![button(
                "x",
                "",
                ButtonAction::ShowDialog {
                    dialog: "zz".to_owned(),
                },
            )],
        );
        let err = dialog_lua(&gone, &names(&gone), "s").unwrap_err();
        assert!(err.contains("no longer exists"), "{err}");
    }

    #[test]
    fn modules_replace_old_ones_and_leave_other_files_alone() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("ui_old.lua"),
            format!("{MODULE_MARKER}: old\n"),
        )
        .unwrap();
        std::fs::write(dir.path().join("ui_mine.lua"), "-- written by hand\n").unwrap();
        std::fs::write(dir.path().join("spell.lua"), "-- a spell\n").unwrap();

        let designs = vec![Design::Dialog(dialog("a", "Welcome", vec![]))];
        let written = write_modules(dir.path(), &designs, "s").unwrap();

        assert_eq!(written, vec!["ui_welcome.lua"]);
        assert!(!dir.path().join("ui_old.lua").exists());
        assert!(dir.path().join("ui_mine.lua").exists());
        assert!(dir.path().join("spell.lua").exists());
    }

    #[test]
    fn design_files_round_trip_and_refuse_newer_formats() {
        let dir = tempfile::tempdir().unwrap();
        let map = dir.path().join("test.w3m");
        assert!(load(&map).unwrap().is_empty());

        let designs = vec![Design::Dialog(dialog(
            "a",
            "Welcome",
            vec![button("Go", "G", ButtonAction::Close)],
        ))];
        save(&map, &designs).unwrap();
        assert!(dir.path().join("test UI.json").exists());
        assert_eq!(load(&map).unwrap(), designs);

        std::fs::write(file_for(&map), "{\"version\": 99, \"designs\": []}").unwrap();
        assert!(load(&map).unwrap_err().contains("newer"));
    }
}
