use std::collections::HashMap;
use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::{Deserialize, Serialize};
use tauri::State;
use wc3_data::icon::{dds_file, decode_texture};
use wc3_data::model::{model_file, texture_paths};
use wc3_map::imports::ImportedFile;
use wc3_map::references::{FileUse, ObjectTables, file_uses, rename_file_uses};

use crate::commands::{view, with_abilities};
use crate::state::{AppState, NO_MAP, OpenMap, Session};
use crate::{leveled, plain};

// Warcraft III loads `.mdx` models; `.mdl` is the text form the World Editor
// names in fields, which the importer converts.
const MODEL_EXTENSIONS: &[&str] = &["mdx", "mdl"];
const IMAGE_EXTENSIONS: &[&str] = &["blp", "dds", "tga"];
const TEXT_MODEL: &str = "mdl";
const BINARY_MODEL: &str = "mdx";
const IMPORTED_DIR: &str = "war3mapImported\\";
const MAP_FILE_PREFIX: &str = "war3map";
const MDL_IMAGE_KEY: &str = "Image \"";

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetUse {
    pub kind: &'static str,
    pub id: String,
    pub name: String,
    pub field: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetItem {
    pub path: String,
    pub kind: &'static str,
    pub size: usize,
    pub queued: bool,
    pub uses: Vec<AssetUse>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureStatus {
    pub path: String,
    pub status: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetList {
    pub assets: Vec<AssetItem>,
    pub listed: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportCandidate {
    pub disk_path: String,
    pub import_path: String,
    pub kind: &'static str,
    pub size: u64,
    pub convert: bool,
    pub exists: bool,
    pub textures: Vec<TextureStatus>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportRequest {
    pub disk_path: String,
    pub import_path: String,
    #[serde(default)]
    pub converted: Option<String>,
}

fn extension(path: &str) -> String {
    Path::new(&path.replace('\\', "/"))
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default()
}

fn kind_of(path: &str) -> &'static str {
    let extension = extension(path);

    if MODEL_EXTENSIONS.contains(&extension.as_str()) {
        return "model";
    }

    if IMAGE_EXTENSIONS.contains(&extension.as_str()) {
        return "image";
    }

    "other"
}

fn file_name(path: &str) -> &str {
    path.rsplit(['\\', '/']).next().unwrap_or(path)
}

fn stem(path: &str) -> String {
    let name = file_name(path);
    let stem = name.rsplit_once('.').map_or(name, |(stem, _)| stem);

    stem.to_ascii_lowercase()
}

fn with_extension(path: &str, extension: &str) -> String {
    match path.rsplit_once('.') {
        Some((stem, _)) if !stem.ends_with(['\\', '/']) => format!("{stem}.{extension}"),
        _ => format!("{path}.{extension}"),
    }
}

fn map_tables(map: &OpenMap) -> [(&'static str, &dyn ObjectTables); 7] {
    [
        ("unit", &map.units),
        ("ability", &map.abilities),
        ("item", &map.items),
        ("buff", &map.buffs),
        ("destructable", &map.destructables),
        ("upgrade", &map.upgrades),
        ("doodad", &map.doodads),
    ]
}

fn names_of(session: &Session, kind: &str) -> HashMap<String, String> {
    let pairs: Vec<(String, String)> = match kind {
        "unit" => view(session)
            .items()
            .into_iter()
            .map(|u| (u.id, u.name))
            .collect(),
        "ability" => with_abilities(session, |abilities| abilities.list())
            .into_iter()
            .map(|a| (a.id, a.name))
            .collect(),
        "item" => plain::items_of(session, plain::Catalog::Items),
        "buff" => plain::items_of(session, plain::Catalog::Buffs),
        "destructable" => plain::items_of(session, plain::Catalog::Destructables),
        "upgrade" => leveled::items_of(session, leveled::Catalog::Upgrades),
        _ => leveled::items_of(session, leveled::Catalog::Doodads),
    };

    pairs.into_iter().collect()
}

fn uses_by_path(session: &Session, paths: &[String]) -> Vec<Vec<AssetUse>> {
    let Some(map) = &session.map else {
        return paths.iter().map(|_| Vec::new()).collect();
    };

    let found: Vec<Vec<(&'static str, FileUse)>> = paths
        .iter()
        .map(|path| {
            map_tables(map)
                .into_iter()
                .flat_map(|(kind, tables)| {
                    file_uses(tables, path).into_iter().map(move |u| (kind, u))
                })
                .collect()
        })
        .collect();
    let mut names: HashMap<&'static str, HashMap<String, String>> = HashMap::new();

    found
        .into_iter()
        .map(|uses| {
            uses.into_iter()
                .map(|(kind, found)| {
                    let id = found.id.to_string();
                    let name = names
                        .entry(kind)
                        .or_insert_with(|| names_of(session, kind))
                        .get(&id)
                        .cloned()
                        .unwrap_or_else(|| id.clone());

                    AssetUse {
                        kind,
                        id,
                        name,
                        field: found.field.to_string(),
                    }
                })
                .collect()
        })
        .collect()
}

#[tauri::command]
pub fn list_assets(state: State<'_, AppState>) -> Result<AssetList, String> {
    state.with(|session| {
        let map = session.map.as_ref().ok_or(NO_MAP)?;
        let paths = map.imports.paths();
        let read = map.reader();
        let uses = uses_by_path(session, &paths);
        let assets = paths
            .iter()
            .zip(uses)
            .map(|(path, uses)| AssetItem {
                kind: kind_of(path),
                size: read(path).map_or(0, |bytes| bytes.len()),
                queued: map.imports.pending(path).is_some(),
                path: path.clone(),
                uses,
            })
            .collect();

        Ok(AssetList {
            assets,
            listed: map.imports.listed() || map.imports.has_pending(),
        })
    })
}

fn texture_status(session: &Session, batch: &[String], texture: &str) -> &'static str {
    if batch.iter().any(|path| stem(path) == stem(texture)) {
        return "batch";
    }

    if session
        .map
        .as_ref()
        .and_then(|map| map.art(texture))
        .is_some()
    {
        return "map";
    }

    if session
        .files
        .read_art(&dds_file(texture), session.mode)
        .is_ok()
    {
        return "game";
    }

    "missing"
}

#[tauri::command]
pub fn asset_textures(
    state: State<'_, AppState>,
    path: String,
) -> Result<Vec<TextureStatus>, String> {
    state.with(|session| {
        let map = session.map.as_ref().ok_or(NO_MAP)?;
        let bytes = map.reader()(&path).ok_or(format!("{path} is not in the map."))?;

        Ok(texture_paths(&bytes)
            .into_iter()
            .map(|texture| TextureStatus {
                status: texture_status(session, &[], &texture),
                path: texture,
            })
            .collect())
    })
}

fn mdl_textures(text: &str) -> Vec<String> {
    text.split(MDL_IMAGE_KEY)
        .skip(1)
        .filter_map(|rest| {
            rest.split_once('"')
                .map(|(path, _)| path.replace("\\\\", "\\"))
        })
        .filter(|path| !path.is_empty())
        .collect()
}

fn model_textures(disk_path: &Path) -> Vec<String> {
    let Ok(bytes) = std::fs::read(disk_path) else {
        return Vec::new();
    };

    if extension(&disk_path.to_string_lossy()) == TEXT_MODEL {
        return mdl_textures(&String::from_utf8_lossy(&bytes));
    }

    texture_paths(&bytes)
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let entries =
        std::fs::read_dir(dir).map_err(|e| format!("Cannot read {}: {e}", dir.display()))?;

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            collect_files(&path, out)?;
        } else if kind_of(&path.to_string_lossy()) != "other" {
            out.push(path);
        }
    }

    Ok(())
}

fn default_import_path(file: &Path, root: Option<&Path>) -> String {
    let relative = root
        .and_then(|root| file.strip_prefix(root).ok())
        .unwrap_or_else(|| Path::new(file.file_name().unwrap_or_default()));
    let path = format!(
        "{IMPORTED_DIR}{}",
        relative.to_string_lossy().replace('/', "\\")
    );

    if extension(&path) == TEXT_MODEL {
        return with_extension(&path, BINARY_MODEL);
    }

    path
}

#[tauri::command]
pub fn plan_import(
    state: State<'_, AppState>,
    files: Vec<String>,
    folder: Option<String>,
) -> Result<Vec<ImportCandidate>, String> {
    let mut disk: Vec<PathBuf> = files.iter().map(PathBuf::from).collect();
    let root = folder.as_ref().map(PathBuf::from);

    if let Some(root) = &root {
        collect_files(root, &mut disk)?;
    }

    disk.sort();
    let mut candidates: Vec<ImportCandidate> = disk
        .iter()
        .map(|file| {
            let disk_path = file.to_string_lossy().into_owned();

            ImportCandidate {
                import_path: default_import_path(file, root.as_deref()),
                kind: kind_of(&disk_path),
                size: std::fs::metadata(file).map_or(0, |m| m.len()),
                convert: extension(&disk_path) == TEXT_MODEL,
                exists: false,
                textures: Vec::new(),
                disk_path,
            }
        })
        .collect();
    let model_textures: Vec<Vec<String>> = disk
        .iter()
        .zip(&candidates)
        .map(|(file, candidate)| {
            if candidate.kind == "model" {
                return model_textures(file);
            }

            Vec::new()
        })
        .collect();

    for texture in model_textures.iter().flatten() {
        let matching = candidates
            .iter_mut()
            .find(|c| c.kind == "image" && stem(&c.disk_path) == stem(texture));

        if let Some(image) = matching {
            image.import_path = with_extension(texture, &extension(&image.disk_path));
        }
    }

    state.with(|session| {
        let map = session.map.as_ref().ok_or(NO_MAP)?;
        let existing = map.imports.paths();
        let batch: Vec<String> = candidates
            .iter()
            .filter(|c| c.kind == "image")
            .map(|c| c.import_path.clone())
            .collect();

        for (candidate, textures) in candidates.iter_mut().zip(model_textures) {
            candidate.exists = existing
                .iter()
                .any(|p| p.eq_ignore_ascii_case(&candidate.import_path));
            candidate.textures = textures
                .into_iter()
                .map(|texture| TextureStatus {
                    status: texture_status(session, &batch, &texture),
                    path: texture,
                })
                .collect();
        }

        Ok(candidates)
    })
}

fn check_import_path(path: &str) -> Result<(), String> {
    let lower = path.to_ascii_lowercase();
    let at_root = !path.contains(['\\', '/']);

    if path.trim().is_empty() || path.contains("..") || path.starts_with(['\\', '/']) {
        return Err(format!("{path:?} is not a path inside the map."));
    }

    if at_root && lower.starts_with(MAP_FILE_PREFIX) {
        return Err(format!(
            "{path} would replace one of the map's own files; put it in a folder."
        ));
    }

    Ok(())
}

#[tauri::command]
pub fn read_text_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| format!("Cannot read {path}: {e}"))
}

#[tauri::command]
pub fn import_assets(state: State<'_, AppState>, files: Vec<ImportRequest>) -> Result<(), String> {
    let mut read: Vec<ImportedFile> = Vec::new();

    for file in &files {
        check_import_path(&file.import_path)?;
        let bytes = match &file.converted {
            Some(encoded) => BASE64
                .decode(encoded)
                .map_err(|_| format!("The converted {} is damaged.", file.disk_path))?,
            None => std::fs::read(&file.disk_path)
                .map_err(|e| format!("Cannot read {}: {e}", file.disk_path))?,
        };

        read.push(ImportedFile {
            path: file.import_path.replace('/', "\\"),
            bytes,
        });
    }

    state.with(|session| {
        let map = session.map.as_mut().ok_or(NO_MAP)?;

        for file in read {
            map.imports.add(file);
        }

        map.icons.clear();
        Ok(())
    })
}

#[tauri::command]
pub fn remove_asset(state: State<'_, AppState>, path: String) -> Result<(), String> {
    state.with(|session| {
        let map = session.map.as_mut().ok_or(NO_MAP)?;

        map.imports.remove(&path);
        map.icons.clear();
        Ok(())
    })
}

#[tauri::command]
pub fn replace_asset(
    state: State<'_, AppState>,
    path: String,
    disk_path: String,
) -> Result<(), String> {
    let bytes = std::fs::read(&disk_path).map_err(|e| format!("Cannot read {disk_path}: {e}"))?;

    state.with(|session| {
        let map = session.map.as_mut().ok_or(NO_MAP)?;

        map.imports.add(ImportedFile { path, bytes });
        map.icons.clear();
        Ok(())
    })
}

#[tauri::command]
pub fn rename_asset(
    state: State<'_, AppState>,
    path: String,
    new_path: String,
) -> Result<usize, String> {
    check_import_path(&new_path)?;
    let new_path = new_path.replace('/', "\\");

    state.with(|session| {
        let map = session.map.as_mut().ok_or(NO_MAP)?;
        let bytes = map.reader()(&path).ok_or(format!("{path} is not in the map."))?;

        map.imports.remove(&path);
        map.imports.add(ImportedFile {
            path: new_path.clone(),
            bytes,
        });

        map.icons.clear();
        let renamed = rename_file_uses(&mut map.units, &path, &new_path)
            + rename_file_uses(&mut map.abilities, &path, &new_path)
            + rename_file_uses(&mut map.items, &path, &new_path)
            + rename_file_uses(&mut map.buffs, &path, &new_path)
            + rename_file_uses(&mut map.destructables, &path, &new_path)
            + rename_file_uses(&mut map.upgrades, &path, &new_path)
            + rename_file_uses(&mut map.doodads, &path, &new_path);

        Ok(renamed)
    })
}

fn write_export(bytes: &[u8], dest: &str, png: bool) -> Result<(), String> {
    let mut out = bytes.to_vec();

    if png {
        out = decode_texture(bytes)
            .and_then(|image| image.to_png())
            .map_err(|e| e.to_string())?;
    }

    std::fs::write(dest, out).map_err(|e| format!("Cannot write {dest}: {e}"))
}

#[tauri::command]
pub fn export_asset(
    state: State<'_, AppState>,
    path: String,
    dest: String,
    png: bool,
) -> Result<(), String> {
    let bytes = state.with(|session| {
        let map = session.map.as_ref().ok_or(NO_MAP)?;
        map.reader()(&path).ok_or(format!("{path} is not in the map."))
    })?;

    write_export(&bytes, &dest, png)
}

#[tauri::command]
pub fn export_game_file(
    state: State<'_, AppState>,
    path: String,
    dest: String,
    png: bool,
) -> Result<(), String> {
    let (files, mode) = state.with(|session| Ok((session.files.clone(), session.mode)))?;
    let file = match kind_of(&path) {
        "image" => dds_file(&path),
        _ => model_file(&path),
    };
    let bytes = files
        .read_art(&file, mode)
        .map_err(|e| format!("{file}: {e}"))?;

    write_export(&bytes, &dest, png)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_models_name_their_textures_in_image_lines() {
        let mdl = "Textures 2 {\n\tBitmap {\n\t\tImage \"Textures\\\\Knight.blp\",\n\t}\n\tBitmap {\n\t\tImage \"\",\n\t\tReplaceableId 1,\n\t}\n}";

        assert_eq!(mdl_textures(mdl), ["Textures\\Knight.blp"]);
    }

    #[test]
    fn imports_go_under_war3map_imported_keeping_folders() {
        let root = Path::new("/home/me/Knight");
        let file = Path::new("/home/me/Knight/Textures/Knight.blp");

        assert_eq!(
            default_import_path(file, Some(root)),
            "war3mapImported\\Textures\\Knight.blp"
        );
        assert_eq!(
            default_import_path(Path::new("/tmp/Knight.mdl"), None),
            "war3mapImported\\Knight.mdx"
        );
    }

    #[test]
    fn map_files_and_paths_outside_the_map_are_refused() {
        assert!(check_import_path("war3map.w3u").is_err());
        assert!(check_import_path("..\\escape.blp").is_err());
        assert!(check_import_path("").is_err());
        assert!(check_import_path("war3mapImported\\war3map.blp").is_ok());
    }
}
