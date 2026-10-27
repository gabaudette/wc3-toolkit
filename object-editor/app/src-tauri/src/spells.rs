use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tauri::State;

use wc3_data::spells::{
    SpellTemplate, fill_tooltip, render_template, spell_kinds as kinds,
    spell_templates as templates,
};
use wc3_map::lua::{build_dev_map, check_syntax, default_output};

use wc3_map::FourCC;

use crate::abilities::{AbilityCard, AbilitySummary};
use crate::commands::{fourcc, view, with_abilities};
use crate::hotkeys::{self, ABILITY_LIST, HERO_ABILITY_LIST, SpellFit};
use crate::state::{AppState, NO_MAP, Session};

/// Images a map can import as icons.
const IMAGE_EXTENSIONS: &[&str] = &["blp", "dds", "tga"];
const HERO_KIND: &str = "hero";
/// Channel: the base of every triggered spell, which does nothing by itself.
const CHANNEL: &str = "ANcl";
/// Base orders for triggered spells, from abilities custom units rarely have:
/// two spells on one unit with the same order would cast each other. Each new
/// spell takes the first one no other Channel copy in the map uses.
const CHANNEL_ORDERS: &[&str] = &[
    "channel",
    "acidbomb",
    "ambush",
    "breathoffire",
    "carrionswarm",
    "drunkenhaze",
    "forkedlightning",
    "howlofterror",
    "monsoon",
    "soulburn",
    "stampede",
    "volcano",
    "doom",
    "rainoffire",
    "deathpact",
    "spiritlink",
    "locustswarm",
    "massteleport",
    "tranquility",
];
const CHANNEL_ORDER: &str = "Ncl6";
/// Channel's options: the button shows, and point spells show the area.
const CHANNEL_VISIBLE: u32 = 1;
const CHANNEL_TARGET_IMAGE: u32 = 2;
const TOOLTIP_TEXT: &str = "aub1";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpellKind {
    pub id: String,
    pub label: String,
    pub description: String,
    pub bases: Vec<SpellBase>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpellBase {
    pub card: AbilityCard,
    pub why: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IconChoice {
    pub path: String,
    pub name: String,
    pub from_map: bool,
}

fn file_stem(path: &str) -> &str {
    let name = path.rsplit(['\\', '/']).next().unwrap_or(path);

    name.rsplit_once('.').map_or(name, |(stem, _)| stem)
}

fn icon_choices(session: &Session) -> Vec<IconChoice> {
    let mut seen = std::collections::HashSet::new();
    let mut choices = Vec::new();
    let mut add = |path: String, from_map: bool| {
        if seen.insert(path.to_ascii_lowercase().replace('/', "\\")) {
            choices.push(IconChoice {
                name: file_stem(&path).to_owned(),
                path,
                from_map,
            });
        }
    };

    if let Some(map) = &session.map {
        for path in map.imports.paths() {
            let extension = path.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase());

            if extension.is_some_and(|e| IMAGE_EXTENSIONS.contains(&e.as_str())) {
                add(path, true);
            }
        }
    }

    let mode = session.mode;

    for id in session.abilities.ids() {
        if let Some(icon) = session.abilities.info(id).and_then(|info| info.icon) {
            add(icon, false);
        }
    }

    for db in [&session.db, &session.items, &session.buffs] {
        for id in db.ids() {
            if let Some(icon) = db.icon_path(id, mode) {
                add(icon, false);
            }
        }
    }

    choices.sort_by(|a, b| {
        b.from_map.cmp(&a.from_map).then_with(|| {
            a.name
                .to_ascii_lowercase()
                .cmp(&b.name.to_ascii_lowercase())
        })
    });

    choices
}

#[tauri::command]
pub fn spell_icons(state: State<'_, AppState>) -> Result<Vec<IconChoice>, String> {
    state.with(|session| Ok(icon_choices(session)))
}

#[tauri::command]
pub fn spell_kinds(state: State<'_, AppState>) -> Result<Vec<SpellKind>, String> {
    let kinds = kinds().map_err(|e| e.to_string())?;

    state.with(|session| {
        Ok(with_abilities(session, |abilities| {
            kinds
                .into_iter()
                .map(|kind| SpellKind {
                    id: kind.id,
                    label: kind.label,
                    description: kind.description,
                    bases: kind
                        .bases
                        .into_iter()
                        .map(|base| SpellBase {
                            card: abilities.card(&base.code),
                            why: base.why,
                        })
                        .collect(),
                })
                .collect()
        }))
    })
}

fn fit(session: &Session, spell: FourCC, unit: FourCC) -> Result<SpellFit, String> {
    let view = view(session);
    let item = view.item(unit).ok_or(format!("Unknown unit {unit}."))?;

    Ok(with_abilities(session, |abilities| {
        hotkeys::spell_fit(
            &view,
            &|ability| abilities.info(ability),
            unit,
            spell,
            item.kind == HERO_KIND,
        )
    }))
}

fn set_list(
    session: &mut Session,
    unit: FourCC,
    field_id: &str,
    list: &[FourCC],
) -> Result<(), String> {
    let field = session
        .db
        .field(field_id)
        .ok_or(format!("Unknown field {field_id}."))?;
    let map = session.map.as_mut().ok_or(NO_MAP)?;
    let text: Vec<String> = list.iter().map(ToString::to_string).collect();

    map.units
        .set(&session.db, unit, field, &text.join(","))
        .map_err(|e| e.to_string())
}

fn take(session: &mut Session, spell: FourCC, unit: FourCC) -> Result<(), String> {
    let (normal, learned) = hotkeys::ability_lists(&view(session), unit);

    for (field_id, list) in [(ABILITY_LIST, normal), (HERO_ABILITY_LIST, learned)] {
        if list.contains(&spell) {
            let kept: Vec<FourCC> = list.into_iter().filter(|a| *a != spell).collect();

            set_list(session, unit, field_id, &kept)?;
        }
    }

    Ok(())
}

fn holders(session: &Session, spell: FourCC) -> Vec<FourCC> {
    let view = view(session);

    view.ids()
        .into_iter()
        .filter(|id| {
            let (normal, learned) = hotkeys::ability_lists(&view, *id);

            normal.contains(&spell) || learned.contains(&spell)
        })
        .collect()
}

#[tauri::command]
pub fn spell_units(state: State<'_, AppState>, spell: String) -> Result<Vec<String>, String> {
    let spell = fourcc(&spell)?;

    state.with(|session| {
        Ok(holders(session, spell)
            .iter()
            .map(ToString::to_string)
            .collect())
    })
}

#[tauri::command]
pub fn spell_fit(
    state: State<'_, AppState>,
    spell: String,
    unit: String,
) -> Result<SpellFit, String> {
    let (spell, unit) = (fourcc(&spell)?, fourcc(&unit)?);

    state.with(|session| fit(session, spell, unit))
}

#[tauri::command]
pub fn give_spell(
    state: State<'_, AppState>,
    spell: String,
    unit: String,
) -> Result<SpellFit, String> {
    let (spell, unit) = (fourcc(&spell)?, fourcc(&unit)?);

    state.with(|session| {
        let before = fit(session, spell, unit)?;

        if let Some(reason) = before.blocked {
            return Err(reason);
        }

        if !before.has {
            let (mut normal, mut learned) = hotkeys::ability_lists(&view(session), unit);

            if before.learned {
                learned.push(spell);
                set_list(session, unit, HERO_ABILITY_LIST, &learned)?;
            } else {
                normal.push(spell);
                set_list(session, unit, ABILITY_LIST, &normal)?;
            }
        }

        fit(session, spell, unit)
    })
}

#[tauri::command]
pub fn take_spell(
    state: State<'_, AppState>,
    spell: String,
    unit: String,
) -> Result<SpellFit, String> {
    let (spell, unit) = (fourcc(&spell)?, fourcc(&unit)?);

    state.with(|session| {
        take(session, spell, unit)?;
        fit(session, spell, unit)
    })
}

fn template(id: &str) -> Result<SpellTemplate, String> {
    templates()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|t| t.id == id)
        .ok_or(format!("No spell template {id}."))
}

fn set_ability(
    session: &mut Session,
    id: FourCC,
    field_id: &str,
    levels: u32,
    text: &str,
) -> Result<(), String> {
    let field = session
        .abilities
        .field(field_id)
        .ok_or(format!("Unknown field {field_id}."))?;
    let map = session.map.as_mut().ok_or(NO_MAP)?;
    let levels = if field.per_level { 1..=levels } else { 0..=0 };

    for level in levels {
        map.abilities
            .set(
                &session.abilities,
                map.units.strings_mut(),
                id,
                field,
                level,
                text,
            )
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn free_order(session: &Session, spell: FourCC) -> String {
    let Some(map) = &session.map else {
        return CHANNEL_ORDERS[0].to_owned();
    };
    let Some(field) = session.abilities.field(CHANNEL_ORDER) else {
        return CHANNEL_ORDERS[0].to_owned();
    };
    let strings = map.units.strings();
    let used: Vec<String> = map
        .abilities
        .abilities()
        .iter()
        .filter(|a| a.id != spell)
        .filter(|a| map.abilities.code(&session.abilities, a.id).as_deref() == Some(CHANNEL))
        .filter_map(|a| {
            map.abilities
                .raw_value(&session.abilities, strings, a.id, field, 1, session.mode)
        })
        .map(|order| order.trim().to_ascii_lowercase())
        .collect();

    CHANNEL_ORDERS
        .iter()
        .find(|order| !used.iter().any(|u| u == *order))
        .unwrap_or(&CHANNEL_ORDERS[0])
        .to_string()
}

#[tauri::command]
pub fn spell_templates() -> Result<Vec<SpellTemplate>, String> {
    templates().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_triggered_spell(
    state: State<'_, AppState>,
    template: String,
) -> Result<AbilitySummary, String> {
    let template = self::template(&template)?;
    let base = fourcc(CHANNEL)?;

    state.with(|session| {
        let map = session.map.as_mut().ok_or(NO_MAP)?;
        let id = map
            .abilities
            .create(&session.abilities, base)
            .map_err(|e| e.to_string())?;
        let levels = template
            .params
            .iter()
            .map(|p| p.values.len())
            .max()
            .unwrap_or(1)
            .max(1) as u32;
        let (target_type, options) = match template.target.as_str() {
            "unit" => (1, CHANNEL_VISIBLE),
            "point" => (2, CHANNEL_VISIBLE | CHANNEL_TARGET_IMAGE),
            _ => (0, CHANNEL_VISIBLE),
        };
        let order = free_order(session, id);
        let number = |v: f64| v.to_string();

        // The hero flag and levels first: which fields apply depends on them.
        let mut fields: Vec<(&str, String)> = vec![
            ("aher", "1".to_owned()),
            ("alev", levels.to_string()),
            ("anam", template.label.clone()),
            ("atp1", template.label.clone()),
            (
                "aret",
                format!("Learn {} - [|cffffcc00Level %d|r]", template.label),
            ),
            ("arut", template.description.clone()),
            ("aart", template.icon.clone()),
            ("arar", template.icon.clone()),
            ("ahky", template.hotkey.clone()),
            // Channel's own spot hides Hold Position; the bottom row is free on most units.
            ("abpx", "0".to_owned()),
            ("abpy", "2".to_owned()),
            // Channel has no learn menu spot; heroes' own spells fill the top row.
            ("arpx", "0".to_owned()),
            ("arpy", "1".to_owned()),
            ("arhk", template.hotkey.clone()),
            ("aran", number(template.range)),
            ("aare", number(template.area)),
            ("acdn", number(template.cooldown)),
            ("amcs", number(template.mana)),
            // Channel: no follow-through time or art, other abilities stay usable.
            ("Ncl1", "0".to_owned()),
            ("Ncl2", target_type.to_string()),
            ("Ncl3", options.to_string()),
            ("Ncl4", "0".to_owned()),
            ("Ncl5", "0".to_owned()),
            (CHANNEL_ORDER, order),
        ];

        if !template.targets.is_empty() {
            fields.push(("atar", template.targets.clone()));
        }

        for (field, text) in &fields {
            set_ability(session, id, field, levels, text)?;
        }

        set_tooltips(session, id, &template, &HashMap::new())?;

        with_abilities(session, |abilities| abilities.summary(id))
            .ok_or(format!("Created {id} but cannot show it."))
    })
}

fn set_tooltips(
    session: &mut Session,
    spell: FourCC,
    template: &SpellTemplate,
    values: &HashMap<String, Vec<f64>>,
) -> Result<(), String> {
    let field = session
        .abilities
        .field(TOOLTIP_TEXT)
        .ok_or(format!("Unknown field {TOOLTIP_TEXT}."))?;
    let map = session.map.as_mut().ok_or(NO_MAP)?;
    let levels = map.abilities.levels(&session.abilities, spell);

    for level in 1..=levels {
        let text = fill_tooltip(template, &spell.to_string(), level, values);

        map.abilities
            .set(
                &session.abilities,
                map.units.strings_mut(),
                spell,
                field,
                level,
                &text,
            )
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub fn set_triggered_values(
    state: State<'_, AppState>,
    spell: String,
    template: String,
    values: HashMap<String, Vec<f64>>,
) -> Result<crate::abilities::AbilityDetail, String> {
    let template = self::template(&template)?;
    let spell = fourcc(&spell)?;

    state.with(|session| {
        set_tooltips(session, spell, &template, &values)?;
        with_abilities(session, |abilities| abilities.detail(spell))
            .ok_or(format!("Unknown ability {spell}."))
    })
}

fn module_source(
    session: &Session,
    spell: FourCC,
    template: &SpellTemplate,
    values: &HashMap<String, Vec<f64>>,
) -> Result<String, String> {
    let name = with_abilities(session, |abilities| abilities.summary(spell))
        .map(|s| without_color_codes(&s.name))
        .ok_or(format!("Unknown ability {spell}."))?;
    let source =
        render_template(template, &spell.to_string(), &name, values).map_err(|e| e.to_string())?;

    check_syntax(&template.file, &source).map_err(|e| e.to_string())?;
    Ok(source)
}

fn without_color_codes(text: &str) -> String {
    const COLOR_LENGTH: usize = 10;

    let mut plain = String::new();
    let mut rest = text;

    while let Some(at) = rest.find('|') {
        plain.push_str(&rest[..at]);
        let code = &rest[at..];
        let skip = match code.as_bytes().get(1).map(u8::to_ascii_lowercase) {
            Some(b'c') if code.len() >= COLOR_LENGTH => COLOR_LENGTH,
            Some(b'r') => 2,
            Some(b'n') => {
                plain.push(' ');
                2
            }
            _ => {
                plain.push('|');
                1
            }
        };

        rest = &code[skip.min(code.len())..];
    }

    plain.push_str(rest);
    plain
}

fn module_file_name(name: &str, spell: FourCC) -> String {
    let mut slug = String::new();

    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
        } else if !slug.is_empty() && !slug.ends_with('_') {
            slug.push('_');
        }
    }

    let slug = slug.trim_end_matches('_');
    let id = spell.to_string().to_ascii_lowercase();

    if slug.is_empty() {
        format!("spell_{id}.lua")
    } else {
        format!("{slug}_{id}.lua")
    }
}

#[tauri::command]
pub fn triggered_spell_lua(
    state: State<'_, AppState>,
    spell: String,
    template: String,
    values: HashMap<String, Vec<f64>>,
) -> Result<String, String> {
    let template = self::template(&template)?;
    let spell = fourcc(&spell)?;

    state.with(|session| module_source(session, spell, &template, &values))
}

#[tauri::command]
pub fn write_triggered_spell_lua(
    state: State<'_, AppState>,
    spell: String,
    template: String,
    values: HashMap<String, Vec<f64>>,
    dir: String,
) -> Result<String, String> {
    let template = self::template(&template)?;
    let spell = fourcc(&spell)?;
    let dir = PathBuf::from(dir);

    state.with(|session| {
        let source = module_source(session, spell, &template, &values)?;
        let name = with_abilities(session, |abilities| abilities.summary(spell))
            .map(|s| without_color_codes(&s.name))
            .unwrap_or_default();
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Cannot create {}: {e}", dir.display()))?;
        let path = dir.join(module_file_name(&name, spell));
        std::fs::write(&path, source)
            .map_err(|e| format!("Cannot write {}: {e}", path.display()))?;
        Ok(path.display().to_string())
    })
}

#[tauri::command]
pub fn default_lua_folder(state: State<'_, AppState>) -> Result<Option<String>, String> {
    state.with(|session| {
        Ok(session.map.as_ref().map(|map| {
            let stem = map.path.file_stem().unwrap_or_default().to_string_lossy();

            map.path
                .with_file_name(format!("{stem} Lua"))
                .display()
                .to_string()
        }))
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestCopy {
    pub path: String,
    pub modules: Vec<String>,
}

#[tauri::command]
pub fn build_test_copy(state: State<'_, AppState>, dir: String) -> Result<TestCopy, String> {
    state.with(|session| {
        let map = session.map.as_ref().ok_or(NO_MAP)?;

        if map.unsaved() {
            return Err(
                "Save the map first: the test copy is made from the saved file.".to_owned(),
            );
        }

        let output = default_output(&map.path);
        let report =
            build_dev_map(&map.path, Path::new(&dir), &output).map_err(|e| e.to_string())?;

        Ok(TestCopy {
            path: report.output.display().to_string(),
            modules: report.modules,
        })
    })
}

#[tauri::command]
pub fn discard_spell(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let id = fourcc(&id)?;

    state.with(|session| {
        for unit in holders(session, id) {
            take(session, id, unit)?;
        }

        let map = session.map.as_mut().ok_or(NO_MAP)?;

        if map.abilities.remove(map.units.strings_mut(), id) {
            Ok(())
        } else {
            Err(format!("{id} is not a custom ability of this map."))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{file_stem, module_file_name, without_color_codes};
    use wc3_map::FourCC;

    #[test]
    fn icon_names_drop_folders_and_extensions() {
        assert_eq!(
            file_stem("ReplaceableTextures\\CommandButtons\\BTNStormBolt.blp"),
            "BTNStormBolt"
        );
        assert_eq!(file_stem("war3mapImported/BTNFire.v2.dds"), "BTNFire.v2");
        assert_eq!(file_stem("NoFolder"), "NoFolder");
    }

    #[test]
    fn color_codes_are_dropped_from_names() {
        assert_eq!(without_color_codes("|cffff0000Fire|r Bolt"), "Fire Bolt");
        assert_eq!(without_color_codes("A|nB"), "A B");
        assert_eq!(without_color_codes("a | b"), "a | b");
    }

    #[test]
    fn module_files_are_named_after_the_spell_and_its_id() {
        let id = FourCC::parse("A00B").unwrap();

        assert_eq!(
            module_file_name("Knockback Wave", id),
            "knockback_wave_a00b.lua"
        );
        assert_eq!(module_file_name("  Arc -- Bolt! ", id), "arc_bolt_a00b.lua");
        assert_eq!(module_file_name("Éclair", id), "clair_a00b.lua");
        assert_eq!(module_file_name("!!!", id), "spell_a00b.lua");
    }
}
