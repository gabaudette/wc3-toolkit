use serde::Serialize;
use tauri::State;
use wc3_data::strings::MapStrings;
use wc3_map::FourCC;
use wc3_map::info::{Force, Player};

use crate::loot::player_name;
use crate::state::{AppState, OpenMap, Session};

const FORCE_ALLIED: u32 = 0x01;
const START_LOCATION: FourCC = FourCC(*b"sloc");

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantContext {
    pub balance: Option<String>,
    pub map: Option<MapContext>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapContext {
    pub name: String,
    pub script: Option<&'static str>,
    pub players: Vec<String>,
    pub forces: Vec<String>,
    pub objects: Vec<ObjectCount>,
    pub placed: usize,
    pub imported_files: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectCount {
    pub kind: &'static str,
    pub custom: usize,
    pub changed: usize,
}

fn controller(value: u32) -> &'static str {
    match value {
        1 => "user",
        2 => "computer",
        3 => "neutral",
        4 => "rescuable",
        _ => "none",
    }
}

fn race(value: u32) -> &'static str {
    match value {
        1 => "Human",
        2 => "Orc",
        3 => "Undead",
        4 => "Night Elf",
        _ => "any race",
    }
}

fn player_line(player: &Player, strings: &MapStrings) -> String {
    format!(
        "{} \"{}\": {}, {}",
        player_name(player.number),
        strings.resolve(&player.name),
        controller(player.controller),
        race(player.race)
    )
}

fn force_line(force: &Force, players: &[Player], strings: &MapStrings) -> String {
    let members: Vec<String> = players
        .iter()
        .map(|p| p.number)
        .filter(|n| {
            force
                .players
                .checked_shr(*n)
                .is_some_and(|rest| rest & 1 != 0)
        })
        .map(player_name)
        .collect();
    let allied = match force.flags & FORCE_ALLIED != 0 {
        true => ", allied",
        false => "",
    };

    format!(
        "\"{}\": {}{allied}",
        strings.resolve(&force.name),
        members.join(", ")
    )
}

fn count(kind: &'static str, objects: impl Iterator<Item = bool>) -> ObjectCount {
    let flags: Vec<bool> = objects.collect();
    let custom = flags.iter().filter(|c| **c).count();

    ObjectCount {
        kind,
        custom,
        changed: flags.len() - custom,
    }
}

fn map_context(map: &OpenMap) -> MapContext {
    let strings = map.units.strings();
    let properties = map.info.as_ref().and_then(|i| i.properties());
    let script = properties.map(|p| match p.uses_lua() {
        true => "Lua",
        false => "JASS",
    });

    MapContext {
        name: map
            .path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default(),
        script,
        players: properties
            .map(|p| {
                p.players
                    .iter()
                    .map(|pl| player_line(pl, strings))
                    .collect()
            })
            .unwrap_or_default(),
        forces: properties
            .map(|p| {
                p.forces
                    .iter()
                    .map(|f| force_line(f, &p.players, strings))
                    .collect()
            })
            .unwrap_or_default(),
        objects: vec![
            count("units", map.units.units().into_iter().map(|u| u.is_custom)),
            count(
                "abilities",
                map.abilities.abilities().into_iter().map(|a| a.is_custom),
            ),
            count(
                "items",
                map.items.objects().into_iter().map(|o| o.is_custom),
            ),
            count(
                "upgrades",
                map.upgrades.objects().into_iter().map(|o| o.is_custom),
            ),
            count(
                "buffs",
                map.buffs.objects().into_iter().map(|o| o.is_custom),
            ),
            count(
                "destructables",
                map.destructables.objects().into_iter().map(|o| o.is_custom),
            ),
            count(
                "doodads",
                map.doodads.objects().into_iter().map(|o| o.is_custom),
            ),
        ],
        placed: map.placements.as_ref().map_or(0, |p| {
            p.units().iter().filter(|u| u.id != START_LOCATION).count()
        }),
        imported_files: map.imports.paths().len(),
    }
}

fn context(session: &Session) -> AssistantContext {
    AssistantContext {
        balance: session.files.options().balance.clone(),
        map: session.map.as_ref().map(map_context),
    }
}

#[tauri::command]
pub fn assistant_context(state: State<'_, AppState>) -> Result<AssistantContext, String> {
    state.with(|session| Ok(context(session)))
}
