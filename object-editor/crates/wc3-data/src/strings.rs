//! Map string tables (`war3map.wts`).
//!
//! Text in a map's data refers to these strings as `TRIGSTR_007`. The file is
//! a list of blocks:
//!
//! ```text
//! STRING 7
//! // Units: H000 (test_hero), Name (Name)
//! {
//! test_hero
//! }
//! ```
//!
//! Parsing keeps the byte order mark, line endings, comments and spacing so an
//! unchanged table is written back byte for byte.

use crate::{Error, Result};

const BYTE_ORDER_MARK: &str = "\u{feff}";
const ENTRY_PREFIX: &str = "STRING ";
const REFERENCE_PREFIX: &str = "TRIGSTR_";
const CRLF: &str = "\r\n";
const LF: &str = "\n";

#[derive(Debug, Clone, PartialEq)]
pub struct StringEntry {
    pub id: u32,
    /// `//` lines between the header and the opening brace, without line endings.
    pub comments: Vec<String>,
    /// The string itself; multi-line strings use the file's line ending.
    pub text: String,
    /// Blank lines after the closing brace.
    pub blank_lines_after: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MapStrings {
    pub entries: Vec<StringEntry>,
    has_bom: bool,
    newline: &'static str,
}

impl Default for MapStrings {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            has_bom: true,
            newline: CRLF,
        }
    }
}

impl MapStrings {
    pub fn parse(text: &str) -> Result<Self> {
        let has_bom = text.starts_with(BYTE_ORDER_MARK);
        let body = text.strip_prefix(BYTE_ORDER_MARK).unwrap_or(text);
        let newline = if body.contains(CRLF) { CRLF } else { LF };
        let lines: Vec<&str> = body.split(newline).collect();
        let error = |line: usize, reason: &str| Error::Parse {
            line: line + 1,
            reason: reason.to_owned(),
        };

        let mut entries: Vec<StringEntry> = Vec::new();
        let mut i = 0;

        let last = lines.len().saturating_sub(1);

        while i < last {
            if lines[i].is_empty() {
                match entries.last_mut() {
                    Some(entry) => entry.blank_lines_after += 1,
                    None => return Err(error(i, "blank line before the first string")),
                }

                i += 1;
                continue;
            }

            let id = lines[i]
                .strip_prefix(ENTRY_PREFIX)
                .and_then(|n| n.trim().parse::<u32>().ok())
                .ok_or_else(|| error(i, "expected `STRING <number>`"))?;

            i += 1;

            let mut comments = Vec::new();

            while i < last && lines[i].starts_with("//") {
                comments.push(lines[i].to_owned());
                i += 1;
            }

            if lines.get(i) != Some(&"{") {
                return Err(error(i, "expected `{`"));
            }

            let start = i + 1;
            let end = (start..last)
                .find(|&j| lines[j] == "}")
                .ok_or_else(|| error(start, "string is never closed with `}`"))?;

            entries.push(StringEntry {
                id,
                comments,
                text: lines[start..end].join(newline),
                blank_lines_after: 0,
            });

            i = end + 1;
        }

        if lines.get(last).is_some_and(|l| !l.is_empty()) {
            return Err(error(last, "file does not end with a line break"));
        }

        Ok(Self {
            entries,
            has_bom,
            newline,
        })
    }

    pub fn to_text(&self) -> String {
        let nl = self.newline;
        let mut out = String::new();

        if self.has_bom {
            out.push_str(BYTE_ORDER_MARK);
        }

        for entry in &self.entries {
            out.push_str(&format!("{ENTRY_PREFIX}{}{nl}", entry.id));
            for comment in &entry.comments {
                out.push_str(comment);
                out.push_str(nl);
            }

            out.push_str(&format!("{{{nl}{}{nl}}}{nl}", entry.text));
            out.push_str(&nl.repeat(entry.blank_lines_after));
        }

        out
    }

    pub fn get(&self, id: u32) -> Option<&str> {
        self.entries
            .iter()
            .find(|e| e.id == id)
            .map(|e| e.text.as_str())
    }

    pub fn resolve<'a>(&'a self, value: &'a str) -> &'a str {
        reference_id(value)
            .and_then(|id| self.get(id))
            .unwrap_or(value)
    }

    pub fn set(&mut self, id: u32, text: &str) -> bool {
        let normalized = self.normalize(text);

        match self.entries.iter_mut().find(|e| e.id == id) {
            Some(entry) => {
                entry.text = normalized;
                true
            }
            None => false,
        }
    }

    pub fn remove(&mut self, id: u32) -> bool {
        let before = self.entries.len();

        self.entries.retain(|e| e.id != id);
        self.entries.len() != before
    }

    pub fn add(&mut self, text: &str, comment: Option<&str>) -> String {
        let id = self.entries.iter().map(|e| e.id).max().map_or(1, |m| m + 1);

        self.entries.push(StringEntry {
            id,
            comments: comment.map(|c| vec![format!("// {c}")]).unwrap_or_default(),
            text: self.normalize(text),
            blank_lines_after: 1,
        });

        reference(id)
    }

    fn normalize(&self, text: &str) -> String {
        text.replace(CRLF, LF).replace(LF, self.newline)
    }
}

pub fn reference_id(value: &str) -> Option<u32> {
    value.strip_prefix(REFERENCE_PREFIX)?.parse().ok()
}

pub fn reference(id: u32) -> String {
    format!("{REFERENCE_PREFIX}{id:03}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\u{feff}STRING 1\r\n{\r\nPlayer 1\r\n}\r\n\r\n\
                          STRING 7\r\n// Units: H000 (test_hero), Name (Name)\r\n{\r\ntest_hero\r\n}\r\n\r\n";

    #[test]
    fn reads_and_resolves_references() {
        let strings = MapStrings::parse(SAMPLE).unwrap();

        assert_eq!(strings.entries.len(), 2);
        assert_eq!(strings.resolve("TRIGSTR_007"), "test_hero");
        assert_eq!(strings.resolve("TRIGSTR_7"), "test_hero");
        assert_eq!(strings.resolve("Paladin"), "Paladin");
        assert_eq!(
            strings.entries[1].comments,
            ["// Units: H000 (test_hero), Name (Name)"]
        );
    }

    #[test]
    fn writes_back_byte_for_byte() {
        assert_eq!(MapStrings::parse(SAMPLE).unwrap().to_text(), SAMPLE);
    }

    #[test]
    fn edits_and_adds_multi_line_strings() {
        let mut strings = MapStrings::parse(SAMPLE).unwrap();

        assert!(strings.set(7, "Holy\nwarrior"));
        assert_eq!(strings.add("new", None), "TRIGSTR_008");

        let reparsed = MapStrings::parse(&strings.to_text()).unwrap();

        assert_eq!(reparsed.get(7), Some("Holy\r\nwarrior"));
        assert_eq!(reparsed.get(8), Some("new"));
    }

    #[test]
    fn reports_malformed_files() {
        assert!(MapStrings::parse("STRING x\n{\nhi\n}\n").is_err());
        assert!(MapStrings::parse("STRING 1\n{\nnever closed\n").is_err());
        assert!(MapStrings::parse("STRING 1\nhi\n}\n").is_err());
    }
}
