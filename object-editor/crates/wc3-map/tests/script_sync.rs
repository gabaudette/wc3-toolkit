//! Rewriting the script of small World Editor maps when their data changes.
//!
//! `fixtures/lua-map` and `fixtures/jass-map` are one-player maps saved by the
//! World Editor 2.0: their `war3map.w3i`, script and `war3mapUnits.doo` (a start
//! location, no units). The real-map tests check the same against many maps,
//! but need maps and an install that CI does not have.

use std::path::PathBuf;

use wc3_map::FourCC;
use wc3_map::info::MapInfo;
use wc3_map::placements::UnitPlacements;
use wc3_map::script::{Language, UnitKinds};
use wc3_map::sync::{MapData, ScriptState};

type TestResult = Result<(), Box<dyn std::error::Error>>;

/// Map options: players' race and start fixed in the lobby; terrain fog.
const FIXED_PLAYER_SETTINGS: u32 = 0x0020;
const TERRAIN_FOG: u32 = 0x2000;
const ORC: u32 = 2;
const RAIN: FourCC = FourCC(*b"RAhr");
const FOOTMAN: FourCC = FourCC(*b"hfoo");
const BOUNDS: [f32; 4] = [-3328.0, -3584.0, 3328.0, 3072.0];

/// The Footman is a plain unit: not a building, hero, item or gold mine.
struct Units;

impl UnitKinds for Units {
    fn is_building(&self, _: FourCC) -> bool {
        false
    }

    fn is_hero(&self, _: FourCC) -> bool {
        false
    }

    fn is_item(&self, _: FourCC) -> bool {
        false
    }

    fn is_gold_mine(&self, _: FourCC) -> bool {
        false
    }

    fn team_color(&self, _: FourCC) -> Option<i32> {
        None
    }

    fn autocast_orders(&self, _: FourCC) -> Option<(String, String)> {
        None
    }
}

struct Fixture {
    file: &'static str,
    language: Language,
    script: Vec<u8>,
    info_bytes: Vec<u8>,
    info: MapInfo,
    placements: UnitPlacements,
}

fn read(dir: &str, file: &str) -> std::io::Result<Vec<u8>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(dir)
        .join(file);

    std::fs::read(path)
}

fn fixture(
    dir: &str,
    file: &'static str,
    language: Language,
) -> Result<Fixture, Box<dyn std::error::Error>> {
    let info_bytes = read(dir, "war3map.w3i")?;

    Ok(Fixture {
        file,
        language,
        script: read(dir, file)?,
        info: MapInfo::parse(&info_bytes)?,
        info_bytes,
        placements: UnitPlacements::parse(&read(dir, "war3mapUnits.doo")?)?,
    })
}

fn fixtures() -> Result<Vec<Fixture>, Box<dyn std::error::Error>> {
    Ok(vec![
        fixture("lua-map", "war3map.lua", Language::Lua)?,
        fixture("jass-map", "war3map.j", Language::Jass)?,
    ])
}

fn data<'a>(info: &'a MapInfo, placements: &'a UnitPlacements) -> MapData<'a> {
    MapData {
        placements: Some(placements),
        tables: info.tables(),
        properties: info.properties(),
        bounds: Some(BOUNDS),
    }
}

fn state(f: &Fixture, script: &[u8], info: &MapInfo, placements: &UnitPlacements) -> ScriptState {
    ScriptState::new(f.file, f.language, script, &data(info, placements), &Units)
}

/// The map's info with its properties changed.
fn changed(info: &MapInfo, change: impl FnOnce(&mut wc3_map::info::Properties)) -> MapInfo {
    let mut info = info.clone();

    if let Some(properties) = info.properties_mut() {
        change(properties);
    }

    info
}

#[test]
fn the_world_editor_s_map_info_round_trips() -> TestResult {
    for f in fixtures()? {
        assert_eq!(f.info.to_bytes(), f.info_bytes, "{}", f.file);
    }

    Ok(())
}

#[test]
fn world_editor_scripts_can_be_rewritten_and_stay_as_they_are() -> TestResult {
    for f in fixtures()? {
        let state = state(&f, &f.script, &f.info, &f.placements);

        assert!(state.rewrites_units(), "{}: units", f.file);
        assert!(state.rewrites_setup(), "{}: setup", f.file);
        assert_eq!(
            state.updated(&data(&f.info, &f.placements), &Units)?,
            None,
            "{}: nothing changed",
            f.file
        );
        assert_eq!(
            state.bytes(&state.text),
            f.script,
            "{}: line endings",
            f.file
        );
    }

    Ok(())
}

#[test]
fn player_changes_reach_the_script_as_the_world_editor_writes_them() -> TestResult {
    for f in fixtures()? {
        let state = state(&f, &f.script, &f.info, &f.placements);
        let info = changed(&f.info, |p| {
            p.players[0].race = ORC;
            p.flags |= FIXED_PLAYER_SETTINGS;
        });
        let text = state
            .updated(&data(&info, &f.placements), &Units)?
            .ok_or("the script did not change")?;
        let selectable = text
            .lines()
            .find(|l| l.contains("SetPlayerRaceSelectable"))
            .ok_or("no SetPlayerRaceSelectable")?;

        assert!(text.contains("RACE_PREF_ORC"), "{}", f.file);
        assert!(!text.contains("RACE_PREF_HUMAN"), "{}", f.file);
        assert!(selectable.contains("false"), "{}: {selectable}", f.file);

        let again = self::state(&f, text.as_bytes(), &info, &f.placements);

        assert!(
            again.rewrites_setup(),
            "{}: rewritten text reproduces",
            f.file
        );
    }

    Ok(())
}

#[test]
fn fog_and_weather_reach_main() -> TestResult {
    for f in fixtures()? {
        let state = state(&f, &f.script, &f.info, &f.placements);
        let info = changed(&f.info, |p| {
            p.flags |= TERRAIN_FOG;
            p.weather = RAIN;
        });
        let text = state
            .updated(&data(&info, &f.placements), &Units)?
            .ok_or("the script did not change")?;

        assert!(text.contains("SetTerrainFog"), "{}", f.file);
        assert!(text.contains("AddWeatherEffect"), "{}", f.file);
        assert!(text.contains("RAhr"), "{}", f.file);
        assert!(
            self::state(&f, text.as_bytes(), &info, &f.placements).rewrites_setup(),
            "{}: rewritten text reproduces",
            f.file
        );
    }

    Ok(())
}

#[test]
fn placed_units_reach_the_script() -> TestResult {
    for f in fixtures()? {
        let state = state(&f, &f.script, &f.info, &f.placements);
        let mut placements = f.placements.clone();

        placements.add(FOOTMAN, 0, 100.0, 200.0, 0.0);

        let text = state
            .updated(&data(&f.info, &placements), &Units)?
            .ok_or("the script did not change")?;

        assert!(text.contains("CreateAllUnits"), "{}", f.file);
        assert!(text.contains("hfoo"), "{}", f.file);
        assert!(
            self::state(&f, text.as_bytes(), &f.info, &placements).rewrites_units(),
            "{}: rewritten text reproduces",
            f.file
        );
    }

    Ok(())
}

#[test]
fn a_script_changed_by_hand_is_left_as_it_is() -> TestResult {
    for f in fixtures()? {
        let (header, custom) = match f.language {
            Language::Lua => (
                "function InitCustomPlayerSlots()\r\n",
                "print(\"mine\")\r\n",
            ),
            Language::Jass => (
                "function InitCustomPlayerSlots takes nothing returns nothing\r\n",
                "    call DoNothing(  )\r\n",
            ),
        };
        let original = String::from_utf8(f.script.clone())?;
        let edited = original.replacen(header, &format!("{header}{custom}"), 1);

        assert_ne!(edited, original, "{}: the edit applies", f.file);

        let state = state(&f, edited.as_bytes(), &f.info, &f.placements);
        let info = changed(&f.info, |p| p.players[0].race = ORC);

        assert!(!state.rewrites_setup(), "{}", f.file);
        assert!(
            state.rewrites_units(),
            "{}: units are still rewritten",
            f.file
        );
        assert_eq!(
            state.updated(&data(&info, &f.placements), &Units)?,
            None,
            "{}: left byte for byte",
            f.file
        );
    }

    Ok(())
}

#[test]
fn a_save_records_the_data_the_script_now_follows() -> TestResult {
    for f in fixtures()? {
        let mut state = state(&f, &f.script, &f.info, &f.placements);
        let info = changed(&f.info, |p| p.players[0].race = ORC);
        let text = state.updated(&data(&info, &f.placements), &Units)?;

        assert!(text.is_some(), "{}", f.file);
        state.saved(text, &data(&info, &f.placements));
        assert_eq!(
            state.updated(&data(&info, &f.placements), &Units)?,
            None,
            "{}: saved",
            f.file
        );
    }

    Ok(())
}
