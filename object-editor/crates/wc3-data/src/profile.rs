//! "Profile" text files: INI-like `[ID]` sections with `key=value` lines
//! (`HumanUnitFunc.txt`, `UnitSkin.txt`, `WorldEditStrings.txt`, ...).
//!
//! Several files are merged into one [`Profile`]; later files override earlier
//! ones key by key. Section IDs are case-sensitive (`hfoo` ≠ `Hfoo`), keys are not.
//! Like the game, the parser ignores lines that are neither a section header nor
//! a `key=value` pair.

use std::collections::HashMap;

const BYTE_ORDER_MARK: char = '\u{feff}';

#[derive(Debug, Default, Clone)]
pub struct Profile {
    sections: HashMap<String, HashMap<String, String>>,
}

impl Profile {
    pub fn new() -> Self {
        Self::default()
    }

    /// Merges a file into this profile, overriding existing keys.
    pub fn merge_text(&mut self, text: &str) {
        let mut current: Option<String> = None;

        for raw in text.lines() {
            let line = raw.trim_start_matches(BYTE_ORDER_MARK).trim();

            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
                current = Some(name.to_owned());
                continue;
            }

            let (Some(section), Some((key, value))) = (&current, line.split_once('=')) else {
                continue;
            };

            self.sections
                .entry(section.clone())
                .or_default()
                .insert(key.trim().to_ascii_lowercase(), value.to_owned());
        }
    }

    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.sections
            .get(section)?
            .get(&key.to_ascii_lowercase())
            .map(String::as_str)
    }

    pub fn section(&self, section: &str) -> Option<&HashMap<String, String>> {
        self.sections.get(section)
    }

    pub fn section_names(&self) -> impl Iterator<Item = &str> {
        self.sections.keys().map(String::as_str)
    }
}

/// Splits a comma-separated profile value, keeping commas inside quotes and
/// stripping the quotes: `"a, b",c` → `["a, b", "c"]`.
pub fn split_list(value: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut quoted = false;

    for ch in value.chars() {
        match ch {
            '"' => quoted = !quoted,
            ',' if !quoted => items.push(std::mem::take(&mut current)),
            _ => current.push(ch),
        }
    }

    items.push(current);
    items
}

/// Strips surrounding quotes from a single profile value.
pub fn unquote(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .unwrap_or(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merges_and_overrides() {
        let mut p = Profile::new();

        p.merge_text("// comment\n[Hpal]\nButtonpos=2,2\nName=Paladin\n");
        p.merge_text("[Hpal]\nname=Better Paladin\n");

        assert_eq!(p.get("Hpal", "buttonpos"), Some("2,2"));
        assert_eq!(p.get("Hpal", "Name"), Some("Better Paladin"));
        assert_eq!(p.get("hpal", "Name"), None);
    }

    #[test]
    fn splits_quoted_lists() {
        assert_eq!(split_list("\"a, b\",c"), ["a, b", "c"]);
        assert_eq!(split_list("AHhb,AHds"), ["AHhb", "AHds"]);
        assert_eq!(unquote("\"hi\""), "hi");
    }
}
