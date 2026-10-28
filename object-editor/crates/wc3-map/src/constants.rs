use std::collections::BTreeMap;

use crate::{Map, Result};

pub const CONSTANTS_FILE: &str = "war3mapMisc.txt";

const BYTE_ORDER_MARK: char = '\u{feff}';
// The World Editor writes only the constants a map changes: each section with
// its keys in alphabetical order, CRLF line ends, a blank line between sections.
const LINE_END: &str = "\r\n";

/// `war3mapMisc.txt`: the gameplay constants a map changes. Without the file,
/// the map uses the game's. Untouched files are written back as they were.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapConstants {
    sections: BTreeMap<String, BTreeMap<String, String>>,
    original: Option<Vec<u8>>,
    custom: bool,
    changed: bool,
}

fn section_key(
    sections: &BTreeMap<String, BTreeMap<String, String>>,
    name: &str,
) -> Option<String> {
    sections
        .keys()
        .find(|s| s.eq_ignore_ascii_case(name))
        .cloned()
}

impl MapConstants {
    pub fn load(map: &Map) -> Result<Self> {
        Ok(Self::parse(map.read_file(CONSTANTS_FILE)?))
    }

    pub fn parse(data: Option<Vec<u8>>) -> Self {
        let mut sections: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
        let mut current: Option<String> = None;

        if let Some(bytes) = &data {
            for raw in String::from_utf8_lossy(bytes).lines() {
                let line = raw.trim_start_matches(BYTE_ORDER_MARK).trim();

                if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                    current = Some(name.to_owned());
                    sections.entry(name.to_owned()).or_default();
                    continue;
                }

                let (Some(section), Some((key, value))) = (&current, line.split_once('=')) else {
                    continue;
                };

                sections
                    .entry(section.clone())
                    .or_default()
                    .insert(key.trim().to_owned(), value.trim().to_owned());
            }
        }

        Self {
            sections,
            custom: data.is_some(),
            original: data,
            changed: false,
        }
    }

    pub fn is_custom(&self) -> bool {
        self.custom
    }

    pub fn set_custom(&mut self, custom: bool) {
        if self.custom != custom {
            self.custom = custom;
            self.changed = true;
        }
    }

    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        let name = section_key(&self.sections, section)?;

        self.sections
            .get(&name)?
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
    }

    pub fn set(&mut self, section: &str, key: &str, value: Option<&str>) {
        let name = section_key(&self.sections, section).unwrap_or_else(|| section.to_owned());
        let entries = self.sections.entry(name).or_default();
        let existing = entries
            .keys()
            .find(|k| k.eq_ignore_ascii_case(key))
            .cloned();

        if let Some(existing) = &existing {
            entries.remove(existing);
        }

        if let Some(value) = value {
            entries.insert(existing.unwrap_or_else(|| key.to_owned()), value.to_owned());
        }

        self.custom = true;
        self.changed = true;
    }

    /// The file to write, or `None` when the map uses the game's constants.
    pub fn to_bytes(&self) -> Option<Vec<u8>> {
        if !self.custom {
            return None;
        }

        if !self.changed {
            return self.original.clone();
        }

        let blocks: Vec<String> = self
            .sections
            .iter()
            .filter(|(name, entries)| !entries.is_empty() || name.eq_ignore_ascii_case("Misc"))
            .map(|(name, entries)| {
                let mut block = format!("[{name}]{LINE_END}");

                for (key, value) in entries {
                    block.push_str(&format!("{key}={value}{LINE_END}"));
                }

                block
            })
            .collect();
        let mut text = blocks.join(LINE_END);

        if text.is_empty() {
            text = format!("[Misc]{LINE_END}");
        }

        Some(text.into_bytes())
    }

    pub fn saved(&mut self) {
        self.original = self.to_bytes();
        self.changed = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LEGACY: &str = "[Misc]\nBoneDecayTime=2\nCallForHelp=1500\n[TWN1]\nName=\nXPFactor=\n";

    #[test]
    fn untouched_files_are_written_back_as_they_were() {
        let constants = MapConstants::parse(Some(LEGACY.as_bytes().to_vec()));

        assert_eq!(constants.to_bytes(), Some(LEGACY.as_bytes().to_vec()));
        assert_eq!(constants.get("misc", "bonedecaytime"), Some("2"));
    }

    #[test]
    fn changed_files_list_sorted_keys_with_crlf() {
        let mut constants = MapConstants::parse(Some(LEGACY.as_bytes().to_vec()));

        constants.set("Misc", "AgiDefenseBonus", Some("0.14"));
        constants.set("Misc", "CallForHelp", None);
        constants.set("HERO", "DependencyOr", Some("Hpal,Hamg"));

        assert_eq!(
            String::from_utf8(constants.to_bytes().unwrap_or_default()).unwrap_or_default(),
            "[HERO]\r\nDependencyOr=Hpal,Hamg\r\n\r\n[Misc]\r\nAgiDefenseBonus=0.14\r\nBoneDecayTime=2\r\n\r\n[TWN1]\r\nName=\r\nXPFactor=\r\n"
        );
    }

    #[test]
    fn maps_without_the_file_use_the_game_constants_until_one_changes() {
        let mut constants = MapConstants::parse(None);

        assert_eq!(constants.to_bytes(), None);
        constants.set("Misc", "MaxHeroLevel", Some("20"));
        assert_eq!(
            constants.to_bytes(),
            Some(b"[Misc]\r\nMaxHeroLevel=20\r\n".to_vec())
        );
        constants.set_custom(false);
        assert_eq!(constants.to_bytes(), None);
    }
}
