use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::State;
use wc3_map::info::ItemTable;
use wc3_map::placements::{DropChance, Drops, InventorySlot};

use crate::commands::{MapSummary, fourcc, map_summary, view};
use crate::plain;
use crate::state::{AppState, NO_MAP, OpenMap, Session};

const FORMAT_VERSION: u32 = 1;
const PERCENT: f64 = 100.0;
const HERO_SLOTS: usize = 6;
const START_LOCATION: &str = "sloc";
// Reforged's 24 player slots come first; the neutral players follow.
const NEUTRAL_PLAYERS: &[&str] = &[
    "Neutral Hostile",
    "Neutral Victim",
    "Neutral Extra",
    "Neutral Passive",
];
const PLAYER_COLORS: &[&str] = &[
    "Red",
    "Blue",
    "Teal",
    "Purple",
    "Yellow",
    "Orange",
    "Green",
    "Pink",
    "Gray",
    "Light Blue",
    "Dark Green",
    "Brown",
    "Maroon",
    "Navy",
    "Turquoise",
    "Violet",
    "Wheat",
    "Peach",
    "Mint",
    "Lavender",
    "Coal",
    "Snow",
    "Emerald",
    "Peanut",
];
const NO_PLACEMENTS: &str = "This map has no placed units Tinkerbench can read.";
const NO_TABLES: &str =
    "This map's info file has no item tables (maps older than The Frozen Throne).";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TierItem {
    pub item: String,
    pub weight: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Tier {
    pub name: String,
    pub chance: f64,
    pub items: Vec<TierItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Cascade {
    pub table: u32,
    pub tiers: Vec<Tier>,
}

#[derive(Serialize, Deserialize)]
struct LootFile {
    version: u32,
    cascades: Vec<Cascade>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FlatDrop {
    pub item: String,
    pub tier: usize,
    pub exact: f64,
    pub chance: u32,
}

pub fn file_for(map: &Path) -> PathBuf {
    let stem = map.file_stem().unwrap_or_default().to_string_lossy();

    map.with_file_name(format!("{stem} Loot.json"))
}

pub fn load(map: &Path) -> Result<Vec<Cascade>, String> {
    let path = file_for(map);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("Cannot read {}: {e}", path.display())),
    };
    let file: LootFile = serde_json::from_str(&text).map_err(|e| {
        format!(
            "{} is not a loot file Tinkerbench can read: {e}",
            path.display()
        )
    })?;

    Ok(file.cascades)
}

pub fn save(map: &Path, cascades: &[Cascade]) -> Result<(), String> {
    let path = file_for(map);
    let file = LootFile {
        version: FORMAT_VERSION,
        cascades: cascades.to_vec(),
    };
    let text = serde_json::to_string_pretty(&file).map_err(|e| e.to_string())?;

    std::fs::write(&path, text + "\n").map_err(|e| format!("Cannot write {}: {e}", path.display()))
}

// Each tier is tried in turn from the first; a hit picks one of its items by
// weight, a miss moves on. So a tier's share is its chance times the misses of
// every tier before it. The World Editor stores whole percents, so the shares
// are rounded by largest remainder to keep their total.
pub fn flatten(tiers: &[Tier]) -> Vec<FlatDrop> {
    let mut missed = 1.0;
    let mut drops = Vec::new();

    for (tier_index, tier) in tiers.iter().enumerate() {
        let chance = (tier.chance / PERCENT).clamp(0.0, 1.0);
        let share = missed * chance;
        let weights: u32 = tier.items.iter().map(|i| i.weight).sum();

        missed *= 1.0 - chance;
        if weights == 0 {
            continue;
        }

        for item in &tier.items {
            drops.push(FlatDrop {
                item: item.item.clone(),
                tier: tier_index,
                exact: share * f64::from(item.weight) / f64::from(weights) * PERCENT,
                chance: 0,
            });
        }
    }

    let total: f64 = drops.iter().map(|d| d.exact).sum();
    let mut left =
        whole(total.round()).saturating_sub(drops.iter().map(|d| whole(d.exact.floor())).sum());
    let mut by_remainder: Vec<usize> = (0..drops.len()).collect();

    for drop in &mut drops {
        drop.chance = whole(drop.exact.floor());
    }

    by_remainder.sort_by(|&a, &b| {
        let fraction = |i: usize| drops[i].exact - drops[i].exact.floor();

        fraction(b).total_cmp(&fraction(a))
    });

    for index in by_remainder {
        if left == 0 {
            break;
        }

        drops[index].chance += 1;
        left -= 1;
    }

    drops
}

fn whole(value: f64) -> u32 {
    value.clamp(0.0, PERCENT) as u32
}

fn flattened_set(tiers: &[Tier]) -> Result<Vec<DropChance>, String> {
    flatten(tiers)
        .into_iter()
        .filter(|d| d.chance > 0)
        .map(|d| {
            Ok(DropChance {
                item: fourcc(&d.item)?,
                chance: d.chance,
            })
        })
        .collect()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DropView {
    pub item: String,
    pub chance: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum DropsView {
    None,
    Table { table: i32 },
    Sets { sets: Vec<Vec<DropView>> },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacedUnit {
    pub index: usize,
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub hero: bool,
    pub player: u32,
    pub player_name: String,
    pub level: u32,
    pub x: f32,
    pub y: f32,
    pub inventory: Vec<Option<String>>,
    pub drops: DropsView,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlacedList {
    pub units: Vec<PlacedUnit>,
    pub error: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableView {
    pub number: u32,
    pub name: String,
    pub sets: Vec<Vec<DropView>>,
    pub tiers: Option<Vec<Tier>>,
    pub flattened: Vec<FlatDrop>,
    pub users: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TableList {
    pub tables: Vec<TableView>,
    pub error: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TableInput {
    pub number: Option<u32>,
    pub name: String,
    pub sets: Vec<Vec<DropView>>,
    pub tiers: Option<Vec<Tier>>,
}

fn drop_views(set: &[DropChance]) -> Vec<DropView> {
    set.iter()
        .map(|d| DropView {
            item: d.item.to_string(),
            chance: d.chance,
        })
        .collect()
}

fn drop_set(set: &[DropView]) -> Result<Vec<DropChance>, String> {
    set.iter()
        .map(|d| {
            Ok(DropChance {
                item: fourcc(&d.item)?,
                chance: d.chance,
            })
        })
        .collect()
}

pub fn player_name(player: u32) -> String {
    let index = player as usize;

    if let Some(color) = PLAYER_COLORS.get(index) {
        return format!("Player {} ({color})", index + 1);
    }

    NEUTRAL_PLAYERS
        .get(index - PLAYER_COLORS.len())
        .map_or_else(
            || format!("Player {}", index + 1),
            |name| (*name).to_owned(),
        )
}

fn inventory_slots(slots: &[InventorySlot]) -> Vec<Option<String>> {
    let mut inventory = vec![None; HERO_SLOTS];

    for slot in slots {
        if let Some(entry) = inventory.get_mut(slot.slot as usize) {
            *entry = Some(slot.item.to_string());
        }
    }

    inventory
}

fn drops_view(drops: Drops) -> DropsView {
    match drops {
        Drops::None => DropsView::None,
        Drops::Table(table) => DropsView::Table { table },
        Drops::Sets(sets) => DropsView::Sets {
            sets: sets.iter().map(|s| drop_views(s)).collect(),
        },
    }
}

fn placed_units(session: &Session) -> Result<PlacedList, String> {
    let map = session.map.as_ref().ok_or(NO_MAP)?;
    let Some(placements) = &map.placements else {
        return Ok(PlacedList {
            units: Vec::new(),
            error: Some(map.loot_error.clone().unwrap_or(NO_PLACEMENTS.to_owned())),
        });
    };
    let units: HashMap<String, _> = view(session)
        .items()
        .into_iter()
        .map(|u| (u.id.clone(), u))
        .collect();
    let list = placements
        .units()
        .iter()
        .enumerate()
        .filter_map(|(index, placed)| {
            let id = placed.id.to_string();
            let unit = units.get(&id)?;

            (id != START_LOCATION).then(|| PlacedUnit {
                index,
                name: unit.name.clone(),
                icon: unit.icon.clone(),
                hero: unit.kind == "hero",
                player: placed.player,
                player_name: player_name(placed.player),
                level: placed.level,
                x: placed.x,
                y: placed.y,
                inventory: inventory_slots(placed.inventory()),
                drops: drops_view(placed.drops()),
                id,
            })
        })
        .collect();

    Ok(PlacedList {
        units: list,
        error: None,
    })
}

fn table_views(map: &OpenMap) -> TableList {
    let Some(info) = &map.info else {
        return TableList {
            tables: Vec::new(),
            error: Some(map.loot_error.clone().unwrap_or(NO_TABLES.to_owned())),
        };
    };

    if !info.supports_tables() {
        return TableList {
            tables: Vec::new(),
            error: Some(NO_TABLES.to_owned()),
        };
    }

    let tables = info
        .tables()
        .iter()
        .map(|table| {
            let tiers = map
                .cascades
                .iter()
                .find(|c| c.table == table.number)
                .map(|c| c.tiers.clone());
            let users = map.placements.as_ref().map_or(0, |p| {
                p.units()
                    .iter()
                    .filter(|u| {
                        i32::try_from(table.number)
                            .is_ok_and(|number| u.drops() == Drops::Table(number))
                    })
                    .count()
            });

            TableView {
                number: table.number,
                name: table.name.clone(),
                sets: table.sets.iter().map(|s| drop_views(s)).collect(),
                flattened: tiers.as_deref().map(flatten).unwrap_or_default(),
                tiers,
                users,
            }
        })
        .collect();

    TableList {
        tables,
        error: None,
    }
}

#[tauri::command]
pub fn list_placed_units(state: State<'_, AppState>) -> Result<PlacedList, String> {
    state.with(|session| placed_units(session))
}

#[tauri::command]
pub fn set_placed_inventory(
    state: State<'_, AppState>,
    index: usize,
    items: Vec<Option<String>>,
) -> Result<MapSummary, String> {
    let mut slots = Vec::new();

    for (slot, item) in items.iter().enumerate().take(HERO_SLOTS) {
        if let Some(item) = item {
            slots.push(InventorySlot {
                slot: u32::try_from(slot).unwrap_or_default(),
                item: fourcc(item)?,
            });
        }
    }

    state.with(|session| {
        let map = session.map.as_mut().ok_or(NO_MAP)?;
        let placements = map.placements.as_mut().ok_or(NO_PLACEMENTS)?;

        placements
            .set_inventory(index, slots)
            .map_err(|e| e.to_string())?;
        Ok(map_summary(session))
    })
}

#[tauri::command]
pub fn set_placed_drops(
    state: State<'_, AppState>,
    index: usize,
    drops: DropsView,
) -> Result<MapSummary, String> {
    let drops = match drops {
        DropsView::None => Drops::None,
        DropsView::Table { table } => Drops::Table(table),
        DropsView::Sets { sets } => {
            Drops::Sets(sets.iter().map(|s| drop_set(s)).collect::<Result<_, _>>()?)
        }
    };

    state.with(|session| {
        let map = session.map.as_mut().ok_or(NO_MAP)?;
        let placements = map.placements.as_mut().ok_or(NO_PLACEMENTS)?;

        placements
            .set_drops(index, drops)
            .map_err(|e| e.to_string())?;
        Ok(map_summary(session))
    })
}

#[tauri::command]
pub fn list_item_tables(state: State<'_, AppState>) -> Result<TableList, String> {
    state.with(|session| Ok(table_views(session.map.as_ref().ok_or(NO_MAP)?)))
}

#[tauri::command]
pub fn save_item_table(state: State<'_, AppState>, table: TableInput) -> Result<u32, String> {
    let sets = match &table.tiers {
        Some(tiers) => vec![flattened_set(tiers)?],
        None => table
            .sets
            .iter()
            .map(|s| drop_set(s))
            .collect::<Result<_, _>>()?,
    };

    state.with(|session| {
        let map = session.map.as_mut().ok_or(NO_MAP)?;
        let info = map.info.as_mut().ok_or(NO_TABLES)?;
        let mut tables = info.tables().to_vec();
        let number = table.number.unwrap_or_else(|| {
            tables
                .iter()
                .map(|t| t.number + 1)
                .max()
                .unwrap_or_default()
        });
        let updated = ItemTable {
            number,
            name: table.name.clone(),
            sets,
        };

        match tables.iter_mut().find(|t| t.number == number) {
            Some(existing) => *existing = updated,
            None => tables.push(updated),
        }

        info.set_tables(tables);
        map.cascades.retain(|c| c.table != number);
        if let Some(tiers) = table.tiers {
            map.cascades.push(Cascade {
                table: number,
                tiers,
            });
        }

        Ok(number)
    })
}

#[tauri::command]
pub fn remove_item_table(state: State<'_, AppState>, number: u32) -> Result<(), String> {
    state.with(|session| {
        let map = session.map.as_mut().ok_or(NO_MAP)?;
        let info = map.info.as_mut().ok_or(NO_TABLES)?;
        let tables = info
            .tables()
            .iter()
            .filter(|t| t.number != number)
            .cloned()
            .collect();

        info.set_tables(tables);
        map.cascades.retain(|c| c.table != number);
        if let (Some(placements), Ok(number)) = (map.placements.as_mut(), i32::try_from(number)) {
            placements.tables_removed(number);
        }

        Ok(())
    })
}

#[tauri::command]
pub fn item_choices(state: State<'_, AppState>) -> Result<Vec<plain::ItemChoice>, String> {
    state.with(|session| Ok(plain::item_choices(session)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tier(name: &str, chance: f64, items: &[(&str, u32)]) -> Tier {
        Tier {
            name: name.to_owned(),
            chance,
            items: items
                .iter()
                .map(|(item, weight)| TierItem {
                    item: (*item).to_owned(),
                    weight: *weight,
                })
                .collect(),
        }
    }

    #[test]
    fn cascades_roll_each_tier_after_the_misses_before_it() {
        let drops = flatten(&[
            tier("Legendary", 2.0, &[("ofro", 1)]),
            tier("Epic", 5.0, &[("ratc", 1)]),
            tier("Common", 100.0, &[("phea", 1), ("pman", 1)]),
        ]);
        let exact: Vec<f64> = drops
            .iter()
            .map(|d| (d.exact * 100.0).round() / 100.0)
            .collect();

        assert_eq!(exact, [2.0, 4.9, 46.55, 46.55]);
        assert_eq!(drops.iter().map(|d| d.chance).sum::<u32>(), 100);
        assert_eq!(
            drops.iter().map(|d| d.chance).collect::<Vec<_>>()[..2],
            [2, 5]
        );
    }

    #[test]
    fn weights_split_a_tier_and_a_missed_last_tier_drops_nothing() {
        let drops = flatten(&[tier("Rare", 25.0, &[("ofro", 3), ("ratc", 1)])]);

        assert_eq!(drops.iter().map(|d| d.chance).collect::<Vec<_>>(), [19, 6]);
    }

    #[test]
    fn neutral_players_follow_the_24_player_slots() {
        assert_eq!(player_name(0), "Player 1 (Red)");
        assert_eq!(player_name(24), "Neutral Hostile");
        assert_eq!(player_name(27), "Neutral Passive");
    }
}
