//! Files imported into a map: custom models, their textures, icons.
//!
//! Objects name these files in their art fields. [`files_used`] finds the ones
//! a map holds, following models to their textures, so they can travel with
//! the objects. [`MapImports`] adds them to another map together with entries
//! in its import list (`war3map.imp`), which the World Editor's Import Manager
//! shows and keeps them by.
//!
//! The files are found by name rather than by listing the archive: protected
//! maps have no list of their files, but every file still reads by its name.

use std::collections::HashSet;

use wc3_data::model::texture_paths;

use crate::{Error, Map, Result};

pub const IMPORT_LIST_FILE: &str = "war3map.imp";
const IMPORT_LIST_VERSION: u32 = 1;
/// Flag written for added entries: a full path inside the archive (13 in the
/// classic format, plus the 0x10 bit every Reforged entry carries).
/// Unverified in the World Editor.
const FULL_PATH_FLAG: u8 = 0x10 | 13;
/// Low bits of an entry's flag saying its path is under `war3mapImported\`
/// (classic 5 and 8; Reforged adds `0x10`, as in 24).
const IMPORTED_DIR_FLAGS: &[u8] = &[5, 8];
const FLAG_KIND_MASK: u8 = 0x0F;
const IMPORTED_DIR: &str = "war3mapImported\\";
const PATH_SEPARATOR: char = '\\';
const MODEL_EXTENSIONS: &[&str] = &["mdx", "mdl"];
const MODEL_EXTENSION: &str = "mdx";
const TEXTURE_EXTENSIONS: &[&str] = &["blp", "dds", "tga"];
/// Reforged looks for a unit's portrait next to its model under this suffix.
const PORTRAIT_SUFFIX: &str = "_portrait";
const ICON_PREFIXES: &[&str] = &["BTN", "PASBTN", "ATC"];
/// The game greys out unusable buttons with a matching `DIS` icon from here.
const DISABLED_ICONS_DIR: &str = "ReplaceableTextures\\CommandButtonsDisabled\\DIS";
/// The map's own files (`war3map.w3u`, `war3mapSkin.w3a`, ...) sit at the archive
/// root under this prefix; imports in `war3mapImported\` are in a folder.
const MAP_FILE_PREFIX: &str = "war3map";

/// A file from a map, at its path inside the archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedFile {
    pub path: String,
    pub bytes: Vec<u8>,
}

/// `war3map.imp`: the files the World Editor's Import Manager lists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportList {
    version: u32,
    /// Flags and path of each entry, as stored.
    entries: Vec<(u8, String)>,
}

impl ImportList {
    fn new() -> Self {
        Self {
            version: IMPORT_LIST_VERSION,
            entries: Vec::new(),
        }
    }

    pub fn parse(data: &[u8]) -> Result<Self> {
        let invalid = || Error::Invalid(format!("{IMPORT_LIST_FILE} is damaged"));
        let word = |pos: usize| -> Result<u32> {
            let bytes = data.get(pos..pos + 4).ok_or_else(invalid)?;
            Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        };

        let version = word(0)?;
        let count = word(4)?;
        let mut pos = 8;
        let mut entries = Vec::new();
        for _ in 0..count {
            let flag = *data.get(pos).ok_or_else(invalid)?;
            let rest = data.get(pos + 1..).ok_or_else(invalid)?;
            let len = rest.iter().position(|b| *b == 0).ok_or_else(invalid)?;
            entries.push((flag, String::from_utf8_lossy(&rest[..len]).into_owned()));
            pos += 1 + len + 1;
        }

        if pos != data.len() {
            return Err(invalid());
        }

        Ok(Self { version, entries })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = self.version.to_le_bytes().to_vec();
        // Import lists hold at most a few thousand entries.
        let count = u32::try_from(self.entries.len()).unwrap_or(u32::MAX);
        out.extend_from_slice(&count.to_le_bytes());
        for (flag, path) in &self.entries {
            out.push(*flag);
            out.extend_from_slice(path.as_bytes());
            out.push(0);
        }
        out
    }

    fn contains(&self, path: &str) -> bool {
        self.entries.iter().any(|(_, p)| same_path(p, path))
    }

    fn add(&mut self, path: &str) {
        if !self.contains(path) {
            self.entries.push((FULL_PATH_FLAG, path.to_owned()));
        }
    }
}

/// Archive paths ignore case and treat `/` like `\`.
fn same_path(a: &str, b: &str) -> bool {
    normalized(a) == normalized(b)
}

fn normalized(path: &str) -> String {
    path.replace('/', "\\").to_ascii_lowercase()
}

/// A map's import list and the files waiting to be written into it.
pub struct MapImports {
    /// `None` when the map has no import list yet.
    list: Option<ImportList>,
    pending: Vec<ImportedFile>,
}

impl MapImports {
    pub fn load(map: &Map) -> Result<Self> {
        let list = match map.read_file(IMPORT_LIST_FILE)? {
            Some(bytes) => Some(ImportList::parse(&bytes)?),
            None => None,
        };
        Ok(Self {
            list,
            pending: Vec::new(),
        })
    }

    /// A file waiting to be written, by its path.
    pub fn pending(&self, path: &str) -> Option<&[u8]> {
        self.pending
            .iter()
            .find(|f| same_path(&f.path, path))
            .map(|f| f.bytes.as_slice())
    }

    /// Every file the map imports, by its path in the archive: the import
    /// list's entries and the files queued for the next save. Protected maps
    /// often have no list, so their imports are not known.
    pub fn paths(&self) -> Vec<String> {
        let listed = self.list.iter().flat_map(|list| {
            list.entries.iter().map(|(flag, path)| {
                if IMPORTED_DIR_FLAGS.contains(&(flag & FLAG_KIND_MASK)) {
                    format!("{IMPORTED_DIR}{path}")
                } else {
                    path.clone()
                }
            })
        });
        let mut paths: Vec<String> = Vec::new();
        for path in listed.chain(self.pending.iter().map(|f| f.path.clone())) {
            if !paths.iter().any(|p| same_path(p, &path)) {
                paths.push(path);
            }
        }
        paths
    }

    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Queues a file for the next save, replacing a queued one at the same path.
    pub fn add(&mut self, file: ImportedFile) {
        self.pending.retain(|f| !same_path(&f.path, &file.path));
        self.pending.push(file);
    }

    /// The files to write on save: the queued files and the import list with
    /// their entries. Nothing when no file is queued.
    pub fn files(&self) -> Vec<(String, Vec<u8>)> {
        if self.pending.is_empty() {
            return Vec::new();
        }

        let mut list = self.list.clone().unwrap_or_else(ImportList::new);
        let mut files = Vec::new();
        for file in &self.pending {
            list.add(&file.path);
            files.push((file.path.clone(), file.bytes.clone()));
        }
        files.push((IMPORT_LIST_FILE.to_owned(), list.to_bytes()));
        files
    }

    /// After a save: the queued files are now in the map and its list.
    pub fn saved(&mut self) {
        let mut list = self.list.take().unwrap_or_else(ImportList::new);
        for file in self.pending.drain(..) {
            list.add(&file.path);
        }
        self.list = Some(list);
    }
}

/// File name and extension of a path, the extension lowercased.
fn split_extension(path: &str) -> (&str, Option<String>) {
    let name_start = path.rfind(PATH_SEPARATOR).map_or(0, |i| i + 1);
    match path[name_start..].rfind('.') {
        Some(dot) => (
            &path[..name_start + dot],
            Some(path[name_start + dot + 1..].to_ascii_lowercase()),
        ),
        None => (path, None),
    }
}

/// The archive paths a field value or a model's texture may name, most likely
/// first. Models are stored as `.mdx` whatever the data says; textures may be
/// any of the image formats.
pub fn candidates(value: &str) -> Vec<String> {
    let path = value.trim().replace('/', "\\");
    if path.is_empty() || !(path.contains('.') || path.contains(PATH_SEPARATOR)) {
        return Vec::new();
    }

    let (stem, extension) = split_extension(&path);
    let mut found = match extension.as_deref() {
        Some(ext) if MODEL_EXTENSIONS.contains(&ext) => vec![format!("{stem}.{MODEL_EXTENSION}")],
        Some(ext) if TEXTURE_EXTENSIONS.contains(&ext) => {
            let mut list = vec![path.clone()];
            list.extend(TEXTURE_EXTENSIONS.iter().map(|e| format!("{stem}.{e}")));
            list
        }
        Some(_) => vec![path.clone()],
        None => vec![format!("{stem}.{MODEL_EXTENSION}")],
    };
    let mut seen = HashSet::new();
    found.retain(|p| seen.insert(normalized(p)));
    found
}

/// Files that go with a found file: a model's portrait and textures, an
/// icon's greyed-out version.
fn companions(file: &ImportedFile) -> Vec<String> {
    let (stem, extension) = split_extension(&file.path);
    match extension.as_deref() {
        Some(MODEL_EXTENSION) => {
            let mut list = vec![format!("{stem}{PORTRAIT_SUFFIX}.{MODEL_EXTENSION}")];
            list.extend(
                texture_paths(&file.bytes)
                    .iter()
                    .flat_map(|t| candidates(t)),
            );
            list
        }
        Some(ext) => {
            let name_start = file.path.rfind(PATH_SEPARATOR).map_or(0, |i| i + 1);
            let name = &file.path[name_start..];
            let upper = name.to_ascii_uppercase();
            if ICON_PREFIXES.iter().any(|p| upper.starts_with(p))
                && TEXTURE_EXTENSIONS.contains(&ext)
            {
                vec![format!("{DISABLED_ICONS_DIR}{name}")]
            } else {
                Vec::new()
            }
        }
        None => Vec::new(),
    }
}

/// The files a map holds for the given field values (each may be a
/// comma-separated list), with the textures of the models among them.
pub fn files_used<'a>(
    values: impl IntoIterator<Item = &'a str>,
    read: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Vec<ImportedFile> {
    let mut queue: Vec<String> = values
        .into_iter()
        .flat_map(|v| v.split(','))
        .flat_map(candidates)
        .collect();
    let mut tried = HashSet::new();
    let mut found = Vec::new();

    while let Some(path) = queue.pop() {
        let key = normalized(&path);
        let map_file = key.starts_with(MAP_FILE_PREFIX) && !key.contains(PATH_SEPARATOR);
        if map_file || !tried.insert(key) {
            continue;
        }

        if let Some(bytes) = read(&path) {
            let file = ImportedFile { path, bytes };
            queue.extend(companions(&file));
            found.push(file);
        }
    }

    found.sort_by_key(|f| normalized(&f.path));
    found
}

/// What importing a bundled file does to the target map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    /// The map does not have it: it is added.
    New,
    /// The map already has the same file.
    Same,
    /// The map has a different file at that path; the map's own stays.
    Different,
}

/// Compares bundled files with the target map's files at the same paths.
pub fn file_statuses(
    files: &[ImportedFile],
    target: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Vec<FileStatus> {
    files
        .iter()
        .map(|file| match target(&file.path) {
            None => FileStatus::New,
            Some(bytes) if bytes == file.bytes => FileStatus::Same,
            Some(_) => FileStatus::Different,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn model_with_texture(texture: &str) -> Vec<u8> {
        let mut entry = vec![0u8; 268];
        entry[4..4 + texture.len()].copy_from_slice(texture.as_bytes());
        let mut out = b"MDLX".to_vec();
        out.extend_from_slice(b"TEXS");
        out.extend_from_slice(&u32::try_from(entry.len()).unwrap().to_le_bytes());
        out.extend(entry);
        out
    }

    fn archive(files: &[(&str, Vec<u8>)]) -> HashMap<String, Vec<u8>> {
        files
            .iter()
            .map(|(p, b)| (normalized(p), b.clone()))
            .collect()
    }

    #[test]
    fn finds_models_textures_portraits_and_icons() {
        let map = archive(&[
            ("Hero.mdx", model_with_texture("Textures\\HeroSkin.blp")),
            ("Hero_portrait.mdx", vec![1]),
            ("Textures\\HeroSkin.blp", vec![2]),
            ("war3mapImported\\BTNHero.blp", vec![3]),
            (
                "ReplaceableTextures\\CommandButtonsDisabled\\DISBTNHero.blp",
                vec![4],
            ),
            ("Unrelated.blp", vec![5]),
        ]);
        let read = |p: &str| map.get(&normalized(p)).cloned();

        let values = [
            "Hero.mdl",
            "war3mapImported/BTNHero.blp",
            "Units\\Human\\Footman\\Footman,1.5,None",
            "war3map.w3u",
        ];
        let found: Vec<String> = files_used(values, &read)
            .into_iter()
            .map(|f| f.path)
            .collect();

        assert_eq!(
            found,
            [
                "Hero.mdx",
                "Hero_portrait.mdx",
                "ReplaceableTextures\\CommandButtonsDisabled\\DISBTNHero.blp",
                "Textures\\HeroSkin.blp",
                "war3mapImported\\BTNHero.blp",
            ]
        );
    }

    #[test]
    fn import_paths_include_the_imported_folder_and_queued_files() {
        let mut list = ImportList::new();
        list.entries.push((24, "KoboldDigger.mdx".to_owned()));
        list.entries.push((5, "BTNOld.blp".to_owned()));
        list.entries
            .push((FULL_PATH_FLAG, "Textures\\Skin.dds".to_owned()));
        let mut imports = MapImports {
            list: Some(list),
            pending: Vec::new(),
        };
        imports.add(ImportedFile {
            path: "war3mapImported\\BTNNew.blp".into(),
            bytes: vec![1],
        });
        imports.add(ImportedFile {
            path: "textures\\skin.DDS".into(),
            bytes: vec![2],
        });

        assert_eq!(
            imports.paths(),
            [
                "war3mapImported\\KoboldDigger.mdx",
                "war3mapImported\\BTNOld.blp",
                "Textures\\Skin.dds",
                "war3mapImported\\BTNNew.blp",
            ]
        );
    }

    #[test]
    fn import_lists_round_trip_and_gain_entries_once() {
        let mut list = ImportList::new();
        list.entries.push((24, "KoboldDigger.mdx".to_owned()));
        let bytes = list.to_bytes();
        assert_eq!(ImportList::parse(&bytes).unwrap(), list);
        assert!(ImportList::parse(&bytes[..bytes.len() - 1]).is_err());

        let mut imports = MapImports {
            list: Some(list),
            pending: Vec::new(),
        };
        assert!(imports.files().is_empty());
        imports.add(ImportedFile {
            path: "Hero.mdx".into(),
            bytes: vec![1],
        });
        imports.add(ImportedFile {
            path: "hero.MDX".into(),
            bytes: vec![2],
        });

        let files = imports.files();
        assert_eq!(files.len(), 2);
        assert_eq!(files[0], ("hero.MDX".to_owned(), vec![2]));
        let written = ImportList::parse(&files[1].1).unwrap();
        assert_eq!(written.entries.len(), 2);
        assert_eq!(written.entries[1], (FULL_PATH_FLAG, "hero.MDX".to_owned()));

        imports.saved();
        assert!(!imports.has_pending());
        assert!(imports.list.as_ref().unwrap().contains("HERO.mdx"));
    }

    #[test]
    fn compares_files_with_the_target() {
        let files = [
            ImportedFile {
                path: "a.blp".into(),
                bytes: vec![1],
            },
            ImportedFile {
                path: "b.blp".into(),
                bytes: vec![1],
            },
            ImportedFile {
                path: "c.blp".into(),
                bytes: vec![1],
            },
        ];
        let target = archive(&[("A.BLP", vec![1]), ("b.blp", vec![9])]);
        let read = |p: &str| target.get(&normalized(p)).cloned();

        assert_eq!(
            file_statuses(&files, &read),
            [FileStatus::Same, FileStatus::Different, FileStatus::New]
        );
    }
}
