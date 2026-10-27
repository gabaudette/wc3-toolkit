use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use wc3_data::abilities::AbilityDatabase;
use wc3_data::doodads::DoodadDatabase;
use wc3_data::game::{ArtMode, GameFiles};
use wc3_data::guide::FieldGuide;
use wc3_data::units::UnitDatabase;
use wc3_data::upgrades::UpgradeDatabase;
use wc3_map::Map;
use wc3_map::abilities::MapAbilities;
use wc3_map::imports::{MapImports, candidates};
use wc3_map::leveled::{DOODADS, LeveledObjects, UPGRADES};
use wc3_map::plain::{BUFFS, DESTRUCTABLES, ITEMS, PlainObjects};
use wc3_map::units::MapUnits;

use crate::interface::{self, Design};

pub struct Session {
    pub files: Arc<GameFiles>,
    pub db: UnitDatabase,
    pub guide: FieldGuide,
    pub abilities: AbilityDatabase,
    pub ability_guide: FieldGuide,
    pub items: UnitDatabase,
    pub item_guide: FieldGuide,
    pub buffs: UnitDatabase,
    pub buff_guide: FieldGuide,
    pub destructables: UnitDatabase,
    pub destructable_guide: FieldGuide,
    pub doodads: DoodadDatabase,
    pub doodad_guide: FieldGuide,
    pub upgrades: UpgradeDatabase,
    pub upgrade_guide: FieldGuide,
    pub mode: ArtMode,
    pub map: Option<OpenMap>,
    pub icons: HashMap<(ArtMode, String), Vec<u8>>,
}

pub struct OpenMap {
    pub path: PathBuf,
    pub units: MapUnits,
    pub abilities: MapAbilities,
    pub items: PlainObjects,
    pub upgrades: LeveledObjects,
    pub buffs: PlainObjects,
    pub destructables: PlainObjects,
    pub doodads: LeveledObjects,
    pub imports: MapImports,
    pub saved_files: Vec<(&'static str, Vec<u8>)>,
    pub icons: HashMap<String, Vec<u8>>,
    pub ui: Vec<Design>,
    pub saved_ui: Vec<Design>,
}

impl OpenMap {
    pub fn load(path: PathBuf) -> wc3_map::Result<Self> {
        let archive = Map::open(&path)?;
        let ui = interface::load(&path).map_err(wc3_map::Error::Invalid)?;
        let mut map = Self {
            units: MapUnits::load(&archive)?,
            abilities: MapAbilities::load(&archive)?,
            items: PlainObjects::load(&archive, ITEMS)?,
            upgrades: LeveledObjects::load(&archive, UPGRADES)?,
            buffs: PlainObjects::load(&archive, BUFFS)?,
            destructables: PlainObjects::load(&archive, DESTRUCTABLES)?,
            doodads: LeveledObjects::load(&archive, DOODADS)?,
            imports: MapImports::load(&archive)?,
            path,
            saved_files: Vec::new(),
            icons: HashMap::new(),
            saved_ui: ui.clone(),
            ui,
        };

        map.saved_files = map.files();
        Ok(map)
    }

    pub fn files(&self) -> Vec<(&'static str, Vec<u8>)> {
        let mut files = self.units.files();

        files.extend(self.abilities.files());
        files.extend(self.items.files());
        files.extend(self.upgrades.files());
        files.extend(self.buffs.files());
        files.extend(self.destructables.files());
        files.extend(self.doodads.files());
        files
    }

    pub fn unsaved(&self) -> bool {
        self.files() != self.saved_files || self.imports.has_pending() || self.ui != self.saved_ui
    }

    pub fn reader(&self) -> impl Fn(&str) -> Option<Vec<u8>> + '_ {
        let archive = Map::open(&self.path).ok();

        move |path| {
            self.imports
                .pending(path)
                .map(<[u8]>::to_vec)
                .or_else(|| archive.as_ref()?.read_file(path).ok().flatten())
        }
    }

    pub fn art(&self, path: &str) -> Option<(String, Vec<u8>)> {
        let read = self.reader();

        candidates(path)
            .into_iter()
            .find_map(|file| read(&file).map(|bytes| (file, bytes)))
    }
}

#[derive(Default)]
pub struct AppState(Mutex<Option<Session>>);

pub const NO_GAME: &str = "Open your Warcraft III folder first.";
pub const NO_MAP: &str = "Open a map first. Changes are saved into a map.";

impl AppState {
    pub fn replace(&self, session: Session) -> Result<(), String> {
        *self.0.lock().map_err(|_| poisoned())? = Some(session);
        Ok(())
    }

    pub fn with<T>(&self, f: impl FnOnce(&mut Session) -> Result<T, String>) -> Result<T, String> {
        let mut guard = self.0.lock().map_err(|_| poisoned())?;
        let session = guard.as_mut().ok_or(NO_GAME)?;

        f(session)
    }
}

fn poisoned() -> String {
    "The editor hit an internal error. Restart it; unsaved changes are lost.".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wc3_map::imports::ImportedFile;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    const MAX_FILES: u32 = 8;

    #[test]
    fn art_comes_from_the_map_or_its_queued_imports() -> TestResult {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("map.w3x");
        let mut archive = mpq::Archive::create(&path, MAX_FILES)?;

        archive.write("Haramad.mdx", b"model")?;
        archive.write("Textures\\Skin.dds", b"texture")?;
        archive.save()?;

        let mut open = OpenMap::load(path.clone())?;

        open.imports.add(ImportedFile {
            path: "war3mapImported\\BTNHero.blp".to_owned(),
            bytes: b"icon".to_vec(),
        });

        let found = |art: &str| open.art(art).map(|(file, bytes)| (file, bytes.len()));

        assert_eq!(found("Haramad.mdl"), Some(("Haramad.mdx".to_owned(), 5)));
        assert_eq!(
            found("Textures/Skin.blp"),
            Some(("Textures\\Skin.dds".to_owned(), 7))
        );
        assert_eq!(
            found("war3mapImported\\BTNHero.blp"),
            Some(("war3mapImported\\BTNHero.blp".to_owned(), 4))
        );
        assert_eq!(found("units\\human\\Footman\\Footman"), None);
        Ok(())
    }

    #[test]
    fn imported_tga_art_is_found_and_decoded() -> TestResult {
        // A 1x1 top-down true-color TGA holding one red pixel (stored BGR).
        let mut tga = vec![0u8; 18];

        tga[2] = 2;
        tga[12] = 1;
        tga[14] = 1;
        tga[16] = 24;
        tga[17] = 0x20;
        tga.extend_from_slice(&[0, 0, 255]);

        let dir = tempfile::tempdir()?;
        let path = dir.path().join("map.w3x");
        let mut archive = mpq::Archive::create(&path, MAX_FILES)?;

        archive.write("war3mapImported\\BTNRed.tga", &tga)?;
        archive.save()?;

        let open = OpenMap::load(path)?;
        let (file, bytes) = open
            .art("war3mapImported\\BTNRed.blp")
            .ok_or("the TGA was not found")?;

        assert_eq!(file, "war3mapImported\\BTNRed.tga");
        let image = wc3_data::icon::decode_texture(&bytes)?;

        assert_eq!((image.width, image.height), (1, 1));
        assert_eq!(image.rgba, [255, 0, 0, 255]);
        Ok(())
    }
}
