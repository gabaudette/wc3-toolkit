use serde::{Deserialize, Serialize};
use tauri::State;
use wc3_data::constants::Constant;
use wc3_data::strings::{MapStrings, reference_id};
use wc3_map::FourCC;
use wc3_map::info::{Force, Properties};

use crate::commands::{MapSummary, map_summary};
use crate::state::{AppState, NO_MAP, Session};

const NO_INFO: &str = "This map's info file (war3map.w3i) cannot be edited here.";
const LIST_SEPARATOR: char = ',';
const TRUE: &str = "1";
const FALSE: &str = "0";
const NO_WEATHER: [u8; 4] = [0; 4];
// Map options (`war3map.w3i` flags) shown in the editor; the others are kept as
// they are: custom techtree, abilities and upgrades need data this editor does
// not write yet, and some bits are the World Editor's own bookkeeping.
const OPTIONS: &[(u32, &str)] = &[
    (0x0001, "Hide the minimap in preview screens"),
    (0x0004, "Melee map"),
    (0x0010, "Masked areas are partially visible"),
    (0x0800, "Show water waves on cliff shores"),
    (0x1000, "Show water waves on rolling shores"),
    (0x8000, "Use the item classification system"),
    (0x2_0000, "Use accurate probability for calculations"),
    (0x8_0000, "Disable the deny icon"),
    (0x10_0000, "Force the default camera zoom"),
    (0x20_0000, "Force the maximum camera zoom"),
    (0x40_0000, "Force the minimum camera zoom"),
];
const CONSTANT_GROUPS: &[(&str, &str)] = &[
    ("experience", "Hero experience"),
    ("attributes", "Hero attributes"),
    ("revival", "Hero revival"),
    ("combat", "Combat"),
    ("economy", "Food, upkeep and gold"),
    ("items", "Items"),
    ("movement", "Movement and ranges"),
    ("creeps", "Creeps"),
    ("time", "Time and decay"),
    ("display", "Display and visibility"),
    ("spells", "Spells"),
    ("illusions", "Illusions and auras"),
    ("techtree", "Techtree equivalents"),
];
const TERRAIN_FOG: u32 = 0x2000;
const WATER_TINTING: u32 = 0x1_0000;
const FIXED_PLAYER_SETTINGS: u32 = 0x0020;
const CUSTOM_FORCES: u32 = 0x0040;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScreenView {
    pub model: String,
    pub title: String,
    pub subtitle: String,
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FogView {
    pub enabled: bool,
    pub style: u32,
    pub start: f32,
    pub end: f32,
    pub density: f32,
    pub color: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionView {
    pub flag: u32,
    pub label: String,
    pub on: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerView {
    pub index: usize,
    pub number: u32,
    pub name: String,
    pub controller: u32,
    pub race: u32,
    pub fixed_start: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ForceView {
    pub name: String,
    pub flags: u32,
    pub players: Vec<u32>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PropertiesView {
    pub name: String,
    pub author: String,
    pub description: String,
    pub suggested_players: String,
    pub options: Vec<OptionView>,
    pub loading_screen: ScreenView,
    pub prologue: ScreenView,
    pub fog: FogView,
    pub weather: String,
    pub sound_environment: String,
    pub water_tinting: bool,
    pub water_color: String,
    pub fixed_player_settings: bool,
    pub custom_forces: bool,
    pub players: Vec<PlayerView>,
    pub forces: Vec<ForceView>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Choice {
    pub id: String,
    pub name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapInfoView {
    pub version: u32,
    pub lua: bool,
    pub properties: PropertiesView,
    pub weather: Vec<Choice>,
}

fn color_text(color: [u8; 4]) -> String {
    let [red, green, blue, _] = color;

    format!("{red:02x}{green:02x}{blue:02x}")
}

fn color_bytes(text: &str, alpha: u8) -> Result<[u8; 4], String> {
    let channel = |at: usize| {
        text.get(at..at + 2)
            .and_then(|hex| u8::from_str_radix(hex, 16).ok())
            .ok_or(format!("“{text}” is not a color like ff8000."))
    };

    Ok([channel(0)?, channel(2)?, channel(4)?, alpha])
}

fn shown(strings: &MapStrings, value: &str) -> String {
    strings.resolve(value).to_owned()
}

fn set_text(strings: &mut MapStrings, field: &mut String, text: &str) {
    if strings.resolve(field) == text {
        return;
    }

    match reference_id(field) {
        Some(id) if strings.set(id, text) => {}
        _ => *field = strings.add(text, None),
    }
}

fn screen_view(strings: &MapStrings, screen: &wc3_map::info::Screen) -> ScreenView {
    ScreenView {
        model: screen.model.clone(),
        title: shown(strings, &screen.title),
        subtitle: shown(strings, &screen.subtitle),
        text: shown(strings, &screen.text),
    }
}

fn set_screen(strings: &mut MapStrings, screen: &mut wc3_map::info::Screen, view: &ScreenView) {
    screen.model.clone_from(&view.model);
    set_text(strings, &mut screen.title, &view.title);
    set_text(strings, &mut screen.subtitle, &view.subtitle);
    set_text(strings, &mut screen.text, &view.text);
}

fn player_bits(players: &[u32]) -> u32 {
    players
        .iter()
        .filter(|p| **p < u32::BITS)
        .fold(0, |mask, p| mask | (1 << p))
}

fn properties_view(p: &Properties, strings: &MapStrings) -> PropertiesView {
    PropertiesView {
        name: shown(strings, &p.name),
        author: shown(strings, &p.author),
        description: shown(strings, &p.description),
        suggested_players: shown(strings, &p.suggested_players),
        options: OPTIONS
            .iter()
            .map(|(flag, label)| OptionView {
                flag: *flag,
                label: (*label).to_owned(),
                on: p.flags & flag != 0,
            })
            .collect(),
        loading_screen: screen_view(strings, &p.loading_screen),
        prologue: screen_view(strings, &p.prologue),
        fog: FogView {
            enabled: p.flags & TERRAIN_FOG != 0,
            style: p.fog.style,
            start: p.fog.start,
            end: p.fog.end,
            density: p.fog.density,
            color: color_text(p.fog.color),
        },
        weather: weather_text(p.weather),
        sound_environment: p.sound_environment.clone(),
        water_tinting: p.flags & WATER_TINTING != 0,
        water_color: color_text(p.water_color),
        fixed_player_settings: p.flags & FIXED_PLAYER_SETTINGS != 0,
        custom_forces: p.flags & CUSTOM_FORCES != 0,
        players: p
            .players
            .iter()
            .enumerate()
            .map(|(index, player)| PlayerView {
                index,
                number: player.number,
                name: shown(strings, &player.name),
                controller: player.controller,
                race: player.race,
                fixed_start: player.fixed_start,
            })
            .collect(),
        forces: p
            .forces
            .iter()
            .map(|force| ForceView {
                name: shown(strings, &force.name),
                flags: force.flags,
                players: (0..u32::BITS)
                    .filter(|b| force.players & (1 << b) != 0)
                    .collect(),
            })
            .collect(),
    }
}

fn weather_text(weather: FourCC) -> String {
    if weather.0 == NO_WEATHER {
        return String::new();
    }

    weather.to_string()
}

fn with_flag(flags: u32, flag: u32, on: bool) -> u32 {
    if on {
        return flags | flag;
    }

    flags & !flag
}

fn apply(
    p: &mut Properties,
    strings: &mut MapStrings,
    view: &PropertiesView,
) -> Result<(), String> {
    set_text(strings, &mut p.name, &view.name);
    set_text(strings, &mut p.author, &view.author);
    set_text(strings, &mut p.description, &view.description);
    set_text(strings, &mut p.suggested_players, &view.suggested_players);
    for option in &view.options {
        if OPTIONS.iter().any(|(flag, _)| *flag == option.flag) {
            p.flags = with_flag(p.flags, option.flag, option.on);
        }
    }

    p.flags = with_flag(p.flags, TERRAIN_FOG, view.fog.enabled);
    p.flags = with_flag(p.flags, WATER_TINTING, view.water_tinting);
    p.flags = with_flag(p.flags, FIXED_PLAYER_SETTINGS, view.fixed_player_settings);
    p.flags = with_flag(p.flags, CUSTOM_FORCES, view.custom_forces);
    set_screen(strings, &mut p.loading_screen, &view.loading_screen);
    set_screen(strings, &mut p.prologue, &view.prologue);
    p.fog.style = view.fog.style;
    p.fog.start = view.fog.start;
    p.fog.end = view.fog.end;
    p.fog.density = view.fog.density;
    p.fog.color = color_bytes(&view.fog.color, p.fog.color[3])?;
    p.weather = match view.weather.trim() {
        "" => FourCC(NO_WEATHER),
        code => FourCC::parse(code).ok_or(format!("“{code}” is not a weather effect ID."))?,
    };

    p.sound_environment.clone_from(&view.sound_environment);
    p.water_color = color_bytes(&view.water_color, p.water_color[3])?;
    for player in &view.players {
        let Some(target) = p.players.get_mut(player.index) else {
            continue;
        };

        set_text(strings, &mut target.name, &player.name);
        target.controller = player.controller;
        target.race = player.race;
        target.fixed_start = player.fixed_start;
    }

    let mut forces = Vec::with_capacity(view.forces.len());

    for (index, force) in view.forces.iter().enumerate() {
        let mut name = p
            .forces
            .get(index)
            .map(|f| f.name.clone())
            .unwrap_or_default();

        set_text(strings, &mut name, &force.name);
        forces.push(Force {
            flags: force.flags,
            players: player_bits(&force.players),
            name,
        });
    }

    p.forces = forces;
    Ok(())
}

#[tauri::command]
pub fn map_info(state: State<'_, AppState>) -> Result<MapInfoView, String> {
    state.with(|session| {
        let map = session.map.as_ref().ok_or(NO_MAP)?;
        let properties = map
            .info
            .as_ref()
            .and_then(|i| i.properties())
            .ok_or(NO_INFO)?;

        Ok(MapInfoView {
            version: properties.version(),
            lua: properties.uses_lua(),
            properties: properties_view(properties, map.units.strings()),
            weather: session
                .weather
                .iter()
                .map(|w| Choice {
                    id: w.id.clone(),
                    name: w.name.clone(),
                })
                .collect(),
        })
    })
}

#[tauri::command]
pub fn set_map_info(
    state: State<'_, AppState>,
    properties: PropertiesView,
) -> Result<MapSummary, String> {
    state.with(|session| {
        let map = session.map.as_mut().ok_or(NO_MAP)?;
        let target = map
            .info
            .as_mut()
            .and_then(|i| i.properties_mut())
            .ok_or(NO_INFO)?;

        apply(target, map.units.strings_mut(), &properties)?;
        Ok(map_summary(session))
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConstantView {
    pub id: String,
    pub label: String,
    pub description: String,
    pub unverified: bool,
    pub advanced: bool,
    pub value_type: String,
    pub min: Option<String>,
    pub max: Option<String>,
    pub default: String,
    pub value: String,
    pub changed: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConstantGroup {
    pub id: String,
    pub label: String,
    pub constants: Vec<ConstantView>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConstantsView {
    pub custom: bool,
    pub groups: Vec<ConstantGroup>,
}

fn constant_view(session: &Session, constant: &Constant) -> ConstantView {
    let stored = session
        .map
        .as_ref()
        .filter(|m| m.constants.is_custom())
        .and_then(|m| {
            m.constants
                .get(&constant.meta.section, &constant.meta.field)
        });
    let entry = session.constant_guide.entry(&constant.meta.id);

    ConstantView {
        id: constant.meta.id.clone(),
        label: constant.label.clone(),
        description: entry.map(|e| e.description.clone()).unwrap_or_default(),
        unverified: entry.is_some_and(|e| e.unverified),
        advanced: entry.is_some_and(|e| e.advanced),
        value_type: constant.meta.value_type.clone(),
        min: constant.meta.min.clone(),
        max: constant.meta.max.clone(),
        default: constant.default.clone(),
        value: stored.unwrap_or(&constant.default).to_owned(),
        changed: stored.is_some_and(|v| !same_value(v, &constant.default)),
    }
}

fn numbers(value: &str) -> Option<Vec<f64>> {
    value
        .split(LIST_SEPARATOR)
        .map(|item| item.trim().parse::<f64>().ok())
        .collect()
}

fn same_value(a: &str, b: &str) -> bool {
    match (numbers(a), numbers(b)) {
        (Some(x), Some(y)) => x == y,
        _ => a.trim() == b.trim(),
    }
}

fn is_numeric(value_type: &str) -> bool {
    matches!(
        value_type,
        "int" | "real" | "unreal" | "intList" | "unrealList" | "defenseTable" | "attackTable"
    )
}

fn is_whole(value_type: &str) -> bool {
    matches!(value_type, "int" | "intList")
}

// The World Editor writes decimals with at least one digit after the point
// (`BoneDecayTime=43.0`) and whole numbers as they are.
fn written_number(item: &str, whole: bool) -> Result<String, String> {
    let item = item.trim();

    if whole {
        return item
            .parse::<i64>()
            .map(|n| n.to_string())
            .map_err(|_| format!("“{item}” is not a whole number."));
    }

    let number: f64 = item
        .parse()
        .map_err(|_| format!("“{item}” is not a number."))?;

    if item.contains('.') {
        return Ok(item.to_owned());
    }

    Ok(format!("{number:.1}"))
}

fn written_value(constant: &Constant, value: &str) -> Result<String, String> {
    let value_type = constant.meta.value_type.as_str();

    if value_type == "bool" {
        return match value.trim() {
            TRUE | "true" => Ok(TRUE.to_owned()),
            FALSE | "false" => Ok(FALSE.to_owned()),
            other => Err(format!("“{other}” is not on (1) or off (0).")),
        };
    }

    if !is_numeric(value_type) {
        return Ok(value.trim().to_owned());
    }

    let items: Result<Vec<String>, String> = value
        .split(LIST_SEPARATOR)
        .map(|item| written_number(item, is_whole(value_type)))
        .collect();

    Ok(items?.join(","))
}

fn constants_view(session: &Session) -> ConstantsView {
    let mut groups: Vec<ConstantGroup> = CONSTANT_GROUPS
        .iter()
        .map(|(id, label)| ConstantGroup {
            id: (*id).to_owned(),
            label: (*label).to_owned(),
            constants: Vec::new(),
        })
        .collect();

    for (id, category) in session.constant_guide.ordered() {
        let (Some(constant), Some(group)) = (
            session.constants.find(id),
            groups.iter_mut().find(|g| g.id == category),
        ) else {
            continue;
        };

        group.constants.push(constant_view(session, constant));
    }

    for group in &mut groups {
        group
            .constants
            .sort_by_key(|c| session.constant_guide.rank(&c.id).unwrap_or(usize::MAX));
    }

    ConstantsView {
        custom: session
            .map
            .as_ref()
            .is_some_and(|m| m.constants.is_custom()),
        groups,
    }
}

#[tauri::command]
pub fn gameplay_constants(state: State<'_, AppState>) -> Result<ConstantsView, String> {
    state.with(|session| Ok(constants_view(session)))
}

#[tauri::command]
pub fn set_gameplay_constant(
    state: State<'_, AppState>,
    id: String,
    value: Option<String>,
) -> Result<MapSummary, String> {
    state.with(|session| {
        let constant = session
            .constants
            .find(&id)
            .ok_or(format!("Unknown gameplay constant {id}."))?
            .clone();
        let written = value
            .map(|v| written_value(&constant, &v))
            .transpose()?
            .filter(|v| !same_value(v, &constant.default));
        let map = session.map.as_mut().ok_or(NO_MAP)?;

        map.constants.set(
            &constant.meta.section,
            &constant.meta.field,
            written.as_deref(),
        );
        Ok(map_summary(session))
    })
}

#[tauri::command]
pub fn set_custom_constants(
    state: State<'_, AppState>,
    custom: bool,
) -> Result<MapSummary, String> {
    state.with(|session| {
        let map = session.map.as_mut().ok_or(NO_MAP)?;

        map.constants.set_custom(custom);
        Ok(map_summary(session))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimals_keep_a_digit_after_the_point_like_the_world_editor() {
        assert_eq!(written_number("43", false).ok(), Some("43.0".to_owned()));
        assert_eq!(
            written_number(" 0.14 ", false).ok(),
            Some("0.14".to_owned())
        );
        assert_eq!(written_number("20", true).ok(), Some("20".to_owned()));
        assert!(written_number("2.5", true).is_err());
    }

    #[test]
    fn values_equal_to_the_game_are_the_same_whatever_their_spelling() {
        assert!(same_value("1.0", "1"));
        assert!(same_value("1.00,0.5", "1,0.50"));
        assert!(!same_value("80,70", "80,70,60"));
    }

    #[test]
    fn colors_are_stored_red_green_blue_alpha() {
        assert_eq!(color_text([0x10, 0x20, 0x30, 0xff]), "102030");
        assert_eq!(
            color_bytes("102030", 0xff).ok(),
            Some([0x10, 0x20, 0x30, 0xff])
        );
    }

    #[test]
    fn forces_list_their_players_as_bits() {
        assert_eq!(player_bits(&[0, 2]), 0b101);
    }
}
