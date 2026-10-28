use wc3_data::objects::{FourCC, ObjectData, Value};

use crate::abilities::MapAbilities;
use crate::imports::{candidates, same_path, split_extension};
use crate::leveled::LeveledObjects;
use crate::plain::PlainObjects;
use crate::units::MapUnits;

pub trait ObjectTables {
    fn tables(&self) -> [&ObjectData; 2];
    fn tables_mut(&mut self) -> [&mut ObjectData; 2];
}

impl ObjectTables for MapUnits {
    fn tables(&self) -> [&ObjectData; 2] {
        let (data, skin, _) = self.parts();

        [data, skin]
    }

    fn tables_mut(&mut self) -> [&mut ObjectData; 2] {
        let (data, skin, _) = self.parts_mut();

        [data, skin]
    }
}

impl ObjectTables for MapAbilities {
    fn tables(&self) -> [&ObjectData; 2] {
        let tables = MapAbilities::tables(self);

        [&tables.data, &tables.skin]
    }

    fn tables_mut(&mut self) -> [&mut ObjectData; 2] {
        let tables = MapAbilities::tables_mut(self);

        [&mut tables.data, &mut tables.skin]
    }
}

impl ObjectTables for PlainObjects {
    fn tables(&self) -> [&ObjectData; 2] {
        let tables = PlainObjects::tables(self);

        [&tables.data, &tables.skin]
    }

    fn tables_mut(&mut self) -> [&mut ObjectData; 2] {
        let tables = PlainObjects::tables_mut(self);

        [&mut tables.data, &mut tables.skin]
    }
}

impl ObjectTables for LeveledObjects {
    fn tables(&self) -> [&ObjectData; 2] {
        let tables = LeveledObjects::tables(self);

        [&tables.data, &tables.skin]
    }

    fn tables_mut(&mut self) -> [&mut ObjectData; 2] {
        let tables = LeveledObjects::tables_mut(self);

        [&mut tables.data, &mut tables.skin]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileUse {
    pub id: FourCC,
    pub field: FourCC,
}

fn names_file(item: &str, path: &str) -> bool {
    candidates(item).iter().any(|c| same_path(c, path))
}

pub fn file_uses(objects: &dyn ObjectTables, path: &str) -> Vec<FileUse> {
    let mut uses = Vec::new();

    for data in objects.tables() {
        for entry in data.original.iter().chain(&data.custom) {
            for m in entry.mods() {
                let Value::String(value) = &m.value else {
                    continue;
                };

                let found = FileUse {
                    id: entry.effective_id(),
                    field: m.field,
                };

                if value.split(',').any(|item| names_file(item, path)) && !uses.contains(&found) {
                    uses.push(found);
                }
            }
        }
    }

    uses
}

fn renamed_item(item: &str, old: &str, new: &str) -> Option<String> {
    if !names_file(item, old) {
        return None;
    }

    let (new_stem, new_extension) = split_extension(new);
    let old_extension = split_extension(old).1;
    let extension = match split_extension(item.trim()).1 {
        None => return Some(new_stem.to_owned()),
        Some(own) if Some(&own) == old_extension.as_ref() => new_extension.unwrap_or(own),
        Some(own) => own,
    };

    Some(format!("{new_stem}.{extension}"))
}

pub fn rename_file_uses(objects: &mut dyn ObjectTables, old: &str, new: &str) -> usize {
    let mut changed = 0;

    for data in objects.tables_mut() {
        for entry in data.original.iter_mut().chain(data.custom.iter_mut()) {
            for set in &mut entry.sets {
                for m in &mut set.mods {
                    let Value::String(value) = &mut m.value else {
                        continue;
                    };

                    let mut any = false;
                    let items: Vec<String> = value
                        .split(',')
                        .map(|item| {
                            let renamed = renamed_item(item, old, new);

                            any |= renamed.is_some();
                            renamed.unwrap_or_else(|| item.to_owned())
                        })
                        .collect();

                    if any {
                        *value = items.join(",");
                        changed += 1;
                    }
                }
            }
        }
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_name_files_in_any_of_their_spellings() {
        assert!(names_file(
            "war3mapImported\\Knight.mdl",
            "war3mapImported\\Knight.mdx"
        ));
        assert!(names_file(
            "war3mapImported\\BTNKnight.blp",
            "war3mapImported\\BTNKnight.dds"
        ));
        assert!(!names_file(
            "units\\human\\Footman\\Footman",
            "war3mapImported\\Knight.mdx"
        ));
    }

    #[test]
    fn renamed_values_keep_their_own_extension() {
        let old = "war3mapImported\\Knight.mdx";
        let new = "war3mapImported\\Heroes\\Paladin.mdx";

        assert_eq!(
            renamed_item("war3mapImported\\Knight.mdl", old, new).as_deref(),
            Some("war3mapImported\\Heroes\\Paladin.mdl")
        );
        assert_eq!(
            renamed_item("war3mapImported\\Knight.mdx", old, new).as_deref(),
            Some("war3mapImported\\Heroes\\Paladin.mdx")
        );
        assert_eq!(renamed_item("Other.mdl", old, new), None);
    }
}
