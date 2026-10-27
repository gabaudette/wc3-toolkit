//! Design files (`.w3ui`): designs taken from one map's Interface tab to
//! another's. A file holds the chosen designs, the dialogs their buttons
//! show, and the files imported into the source map that they use (icons on
//! multiboard cells), the way object bundles carry models and icons.
//! Importing gives each design an ID and a name the map does not have yet.

use std::collections::HashSet;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::{Deserialize, Serialize};
use wc3_map::imports::{ImportedFile, files_used};

use super::{ButtonAction, Design, fill_missing_ids, pascal_case};

/// What `format` says in every design file, so other JSON files are refused.
pub const FILE_FORMAT: &str = "tinkerbench-ui";
/// Bumped when the file changes in a way older versions cannot read.
pub const FILE_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct DesignFile {
    pub format: String,
    pub version: u32,
    pub designs: Vec<Design>,
    /// Files imported into the source map that the designs use.
    #[serde(default)]
    pub files: Vec<SharedFile>,
}

/// A file from the source map, at its path inside the archive.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SharedFile {
    pub path: String,
    /// The bytes, base64-encoded.
    pub data: String,
}

impl DesignFile {
    pub fn to_text(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self)
            .map(|text| text + "\n")
            .map_err(|e| e.to_string())
    }

    pub fn parse(text: &str) -> Result<Self, String> {
        let invalid = |reason: String| format!("Not a design file Tinkerbench can read: {reason}.");
        let mut file: Self = serde_json::from_str(text).map_err(|e| invalid(e.to_string()))?;

        if file.format != FILE_FORMAT {
            return Err(invalid("it is not a .w3ui file".to_owned()));
        }

        if file.version > FILE_VERSION {
            return Err(format!(
                "This design file was made by a newer Tinkerbench (format {}); update to import it.",
                file.version
            ));
        }

        fill_missing_ids(&mut file.designs);
        Ok(file)
    }

    /// The imported files, decoded.
    pub fn imported_files(&self) -> Result<Vec<ImportedFile>, String> {
        self.files
            .iter()
            .map(|file| {
                let bytes = BASE64
                    .decode(&file.data)
                    .map_err(|_| format!("The design file's copy of {} is damaged.", file.path))?;

                Ok(ImportedFile {
                    path: file.path.clone(),
                    bytes,
                })
            })
            .collect()
    }
}

/// A design in an export or import, as the window lists it.
#[derive(Clone, Debug, PartialEq)]
pub struct Listed {
    pub design: Design,
    /// Why it is there when nobody chose it: the dialog whose button shows it.
    pub shown_by: Option<String>,
    /// Its name in the file, when importing renamed it.
    pub renamed_from: Option<String>,
}

/// The chosen designs, then every dialog their buttons show (and the dialogs
/// those show), each once, in the map's order within each group.
pub fn with_dependencies(all: &[Design], chosen: &[String]) -> Vec<Listed> {
    let mut listed: Vec<Listed> = all
        .iter()
        .filter(|d| chosen.iter().any(|id| id == d.id()))
        .map(|design| Listed {
            design: design.clone(),
            shown_by: None,
            renamed_from: None,
        })
        .collect();

    let mut next = 0;

    while next < listed.len() {
        let current = listed[next].design.clone();

        next += 1;
        for target in shown_dialogs(&current) {
            let known = listed.iter().any(|l| l.design.id() == target);

            if let Some(design) = all.iter().find(|d| d.id() == target).filter(|_| !known) {
                listed.push(Listed {
                    design: design.clone(),
                    shown_by: Some(current.name().to_owned()),
                    renamed_from: None,
                });
            }
        }
    }

    listed
}

/// The dialogs a design's buttons show, by ID.
fn shown_dialogs(design: &Design) -> Vec<String> {
    let Design::Dialog(dialog) = design else {
        return Vec::new();
    };

    dialog
        .buttons
        .iter()
        .filter_map(|b| match &b.action {
            ButtonAction::ShowDialog { dialog } => Some(dialog.clone()),
            _ => None,
        })
        .collect()
}

/// The icon paths the designs name: multiboard cells' icons.
pub fn icon_paths(designs: &[Design]) -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();

    for design in designs {
        let Design::Multiboard(board) = design else {
            continue;
        };

        for icon in board
            .rows
            .iter()
            .flat_map(|r| &r.cells)
            .filter_map(|c| c.icon.as_ref())
        {
            if !paths.contains(icon) {
                paths.push(icon.clone());
            }
        }
    }

    paths
}

/// What the designs expect the map's own Lua to define: the functions their
/// buttons and timers call. The designs work without them; those calls do
/// nothing.
pub fn needs(designs: &[Design]) -> Vec<String> {
    let mut functions: Vec<String> = Vec::new();

    for design in designs {
        let called = match design {
            Design::Dialog(dialog) => dialog
                .buttons
                .iter()
                .filter_map(|b| match &b.action {
                    ButtonAction::RunFunction { function } => Some(function.clone()),
                    _ => None,
                })
                .collect(),
            Design::TimerWindow(timer) if !timer.on_done.trim().is_empty() => {
                vec![timer.on_done.trim().to_owned()]
            }
            _ => Vec::new(),
        };

        for function in called {
            if !functions.contains(&function) {
                functions.push(function);
            }
        }
    }

    functions
}

/// The design file for the chosen designs, the dialogs they show, and the
/// files `read` finds in the source map for their icons.
pub fn export(
    all: &[Design],
    chosen: &[String],
    read: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> (DesignFile, Vec<Listed>, Vec<ImportedFile>) {
    let listed = with_dependencies(all, chosen);
    let designs: Vec<Design> = listed.iter().map(|l| l.design.clone()).collect();
    let icons = icon_paths(&designs);
    let files = files_used(icons.iter().map(String::as_str), read);
    let file = DesignFile {
        format: FILE_FORMAT.to_owned(),
        version: FILE_VERSION,
        designs,
        files: files
            .iter()
            .map(|f| SharedFile {
                path: f.path.clone(),
                data: BASE64.encode(&f.bytes),
            })
            .collect(),
    };

    (file, listed, files)
}

/// The file's designs as they would join a map holding `existing`: each with
/// an ID the map does not use (buttons follow their dialog to its new ID) and
/// a name whose Lua name (`UI.Scoreboard`) is free, numbered when taken.
pub fn planned(incoming: &[Design], existing: &[Design]) -> Vec<Listed> {
    let mut ids: HashSet<String> = existing.iter().map(|d| d.id().to_owned()).collect();
    let mut lua_names: HashSet<String> = existing
        .iter()
        .map(|d| pascal_case(d.name()).to_ascii_lowercase())
        .collect();

    let mut moved: Vec<(String, String)> = Vec::new();
    let mut listed: Vec<Listed> = incoming
        .iter()
        .map(|design| {
            let id = free(design.id(), &ids);
            ids.insert(id.clone());
            moved.push((design.id().to_owned(), id.clone()));

            let mut name = design.name().to_owned();
            let mut n = 2;
            while lua_names.contains(&pascal_case(&name).to_ascii_lowercase()) {
                name = format!("{} {n}", design.name());
                n += 1;
            }
            lua_names.insert(pascal_case(&name).to_ascii_lowercase());

            let renamed_from = Some(design.name().to_owned()).filter(|old| *old != name);
            let mut design = design.clone();
            design.rename(id, name);
            Listed {
                design,
                shown_by: None,
                renamed_from,
            }
        })
        .collect();

    for item in &mut listed {
        if let Design::Dialog(dialog) = &mut item.design {
            for button in &mut dialog.buttons {
                if let ButtonAction::ShowDialog { dialog: target } = &mut button.action
                    && let Some((_, new)) = moved.iter().find(|(old, _)| old == target)
                {
                    target.clone_from(new);
                }
            }
        }
    }

    listed
}

/// `id`, or `id-2`, `id-3`... when taken.
fn free(id: &str, taken: &HashSet<String>) -> String {
    let mut candidate = id.to_owned();
    let mut n = 2;

    while taken.contains(&candidate) {
        candidate = format!("{id}-{n}");
        n += 1;
    }

    candidate
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::multiboard::{BoardCell, BoardColumn, BoardRow};
    use crate::interface::{DialogButton, DialogDesign, MultiboardDesign};

    fn dialog(id: &str, name: &str, shows: Option<&str>) -> Design {
        let action = match shows {
            Some(target) => ButtonAction::ShowDialog {
                dialog: target.to_owned(),
            },
            None => ButtonAction::RunFunction {
                function: "OnPick".to_owned(),
            },
        };
        Design::Dialog(DialogDesign {
            id: id.to_owned(),
            name: name.to_owned(),
            title: "Pick".to_owned(),
            title_color: None,
            buttons: vec![DialogButton {
                id: "go".to_owned(),
                text: "Go".to_owned(),
                hotkey: String::new(),
                highlight_hotkey: true,
                action,
            }],
            show_at_start: false,
        })
    }

    fn board(id: &str, name: &str, icon: &str) -> Design {
        Design::Multiboard(MultiboardDesign {
            id: id.to_owned(),
            name: name.to_owned(),
            title: "Scores".to_owned(),
            title_color: None,
            columns: vec![BoardColumn {
                id: "player".to_owned(),
                name: "Player".to_owned(),
                width: 10.0,
            }],
            rows: vec![BoardRow {
                id: "players".to_owned(),
                name: "Players".to_owned(),
                per_player: true,
                cells: vec![BoardCell {
                    text: "{player}".to_owned(),
                    color: None,
                    icon: Some(icon.to_owned()),
                }],
            }],
            show_at_start: true,
            minimized: false,
        })
    }

    #[test]
    fn exports_bring_the_dialogs_shown_and_the_map_s_icons() {
        let all = vec![
            dialog("a", "Mode", Some("b")),
            dialog("b", "Difficulty", Some("c")),
            dialog("c", "Confirm", None),
            board("d", "Scoreboard", "war3mapImported\\BTNCrown.blp"),
            board(
                "e",
                "Other",
                "ReplaceableTextures\\CommandButtons\\BTNFootman.blp",
            ),
        ];
        let read = |path: &str| (path == "war3mapImported\\BTNCrown.blp").then(|| vec![1, 2, 3]);
        let (file, listed, files) = export(&all, &["a".to_owned(), "d".to_owned()], &read);

        let names: Vec<&str> = listed.iter().map(|l| l.design.name()).collect();

        assert_eq!(names, ["Mode", "Scoreboard", "Difficulty", "Confirm"]);
        assert_eq!(listed[2].shown_by.as_deref(), Some("Mode"));
        assert_eq!(listed[3].shown_by.as_deref(), Some("Difficulty"));
        assert_eq!(needs(&file.designs), ["OnPick"]);
        // The game's own footman icon stays behind; the map's crown comes along.
        assert_eq!(files.len(), 1);
        assert_eq!(file.files[0].path, "war3mapImported\\BTNCrown.blp");

        let read_back = DesignFile::parse(&file.to_text().unwrap()).unwrap();

        assert_eq!(read_back, file);
        assert_eq!(read_back.imported_files().unwrap()[0].bytes, [1, 2, 3]);
    }

    #[test]
    fn imports_rename_what_the_map_already_has_and_keep_buttons_linked() {
        let existing = vec![board("d", "Scoreboard", "x.blp"), dialog("a", "Mode", None)];
        let incoming = vec![
            board("d", "Scoreboard", "x.blp"),
            dialog("a", "Mode", Some("b")),
            dialog("b", "Difficulty", None),
        ];
        let plan = planned(&incoming, &existing);

        assert_eq!(plan[0].design.name(), "Scoreboard 2");
        assert_eq!(plan[0].renamed_from.as_deref(), Some("Scoreboard"));
        assert_eq!(plan[0].design.id(), "d-2");
        assert_eq!(plan[1].design.name(), "Mode 2");
        assert_eq!(plan[2].design.name(), "Difficulty");
        assert_eq!(plan[2].renamed_from, None);
        let Design::Dialog(mode) = &plan[1].design else {
            panic!("not a dialog");
        };

        assert_eq!(
            mode.buttons[0].action,
            ButtonAction::ShowDialog {
                dialog: "b".to_owned()
            }
        );

        // Importing the same file twice numbers on.
        let twice: Vec<Design> = existing
            .iter()
            .cloned()
            .chain(plan.into_iter().map(|l| l.design))
            .collect();

        assert_eq!(planned(&incoming, &twice)[0].design.name(), "Scoreboard 3");
    }

    #[test]
    fn other_files_are_refused() {
        assert!(DesignFile::parse("{\"version\": 1, \"designs\": []}").is_err());
        assert!(DesignFile::parse("not json").is_err());
        let newer = format!("{{\"format\": \"{FILE_FORMAT}\", \"version\": 99, \"designs\": []}}");

        assert!(DesignFile::parse(&newer).unwrap_err().contains("newer"));
    }
}
