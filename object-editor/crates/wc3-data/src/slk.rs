//! SYLK (`.slk`) tables, the spreadsheet format Warcraft III uses for most game data.
//!
//! Only cell records (`C;X..;Y..;K..`) matter. Row 1 holds column names and the
//! first column of every other row is that row's key (e.g. a unit ID).

use std::collections::HashMap;

use crate::{Error, Result};

const HEADER_ROW: usize = 1;

#[derive(Debug, Default, Clone)]
pub struct Slk {
    columns: Vec<String>,
    column_index: HashMap<String, usize>,
    rows: Vec<Vec<Option<String>>>,
    row_index: HashMap<String, usize>,
}

impl Slk {
    pub fn parse(text: &str) -> Result<Self> {
        let mut cells: HashMap<(usize, usize), String> = HashMap::new();
        let (mut x, mut y) = (1usize, 1usize);
        let (mut max_x, mut max_y) = (0usize, 0usize);

        for (line_index, line) in text.lines().enumerate() {
            let Some(rest) = line.strip_prefix("C;") else {
                continue;
            };

            // K is always the last field and may itself contain ';' inside quotes.
            let (coords, value) = match rest.find(";K") {
                Some(i) => (&rest[..i], Some(&rest[i + 2..])),
                None => match rest.strip_prefix('K') {
                    Some(v) => ("", Some(v)),
                    None => (rest, None),
                },
            };

            for part in coords.split(';') {
                let target = match part.as_bytes().first() {
                    Some(b'X') => &mut x,
                    Some(b'Y') => &mut y,
                    _ => continue,
                };

                *target = part[1..].parse().map_err(|_| Error::Parse {
                    line: line_index + 1,
                    reason: format!("invalid cell coordinate {part:?}"),
                })?;
            }

            if let Some(v) = value {
                let v = v
                    .strip_prefix('"')
                    .and_then(|v| v.strip_suffix('"'))
                    .unwrap_or(v);

                cells.insert((x, y), v.to_owned());
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }

        let columns: Vec<String> = (1..=max_x)
            .map(|cx| cells.remove(&(cx, HEADER_ROW)).unwrap_or_default())
            .collect();
        let column_index = columns
            .iter()
            .enumerate()
            .filter(|(_, c)| !c.is_empty())
            .map(|(i, c)| (c.to_ascii_lowercase(), i))
            .collect();

        let mut rows = Vec::new();
        let mut row_index = HashMap::new();

        for cy in HEADER_ROW + 1..=max_y {
            let row: Vec<Option<String>> = (1..=max_x).map(|cx| cells.remove(&(cx, cy))).collect();
            let Some(Some(key)) = row.first() else {
                continue;
            };

            row_index.insert(key.clone(), rows.len());
            rows.push(row);
        }

        Ok(Self {
            columns,
            column_index,
            rows,
            row_index,
        })
    }

    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    /// Row keys (first column) in file order.
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.rows.iter().filter_map(|r| r.first()?.as_deref())
    }

    pub fn has_row(&self, key: &str) -> bool {
        self.row_index.contains_key(key)
    }

    /// Cell value by row key and column name (column names are case-insensitive).
    pub fn get(&self, key: &str, column: &str) -> Option<&str> {
        let row = &self.rows[*self.row_index.get(key)?];
        let col = *self.column_index.get(&column.to_ascii_lowercase())?;

        row.get(col)?.as_deref()
    }

    /// All rows as column-name → value maps, for tables read whole (like metadata).
    pub fn records(&self) -> impl Iterator<Item = HashMap<&str, &str>> {
        self.rows.iter().map(|row| {
            self.columns
                .iter()
                .zip(row)
                .filter_map(|(c, v)| Some((c.as_str(), v.as_deref()?)))
                .collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "ID;PWXL;N;E\nB;X3;Y3;D0\n\
                          C;X1;Y1;K\"unitID\"\nC;X2;K\"HP\"\nC;X3;K\"name\"\n\
                          C;X1;Y2;K\"Hpal\"\nC;X2;K100\nC;X3;K\"a;b\"\n\
                          C;X1;Y3;K\"hfoo\"\nC;X3;K\"Footman\"\nE\n";

    #[test]
    fn parses_cells_and_keys() {
        let slk = Slk::parse(SAMPLE).unwrap();

        assert_eq!(slk.columns(), ["unitID", "HP", "name"]);
        assert_eq!(slk.get("Hpal", "hp"), Some("100"));
        assert_eq!(slk.get("Hpal", "name"), Some("a;b"));
        assert_eq!(slk.get("hfoo", "HP"), None);
        assert_eq!(slk.keys().collect::<Vec<_>>(), ["Hpal", "hfoo"]);
    }

    #[test]
    fn reports_bad_coordinates_with_line_number() {
        let err = Slk::parse("ID;P\nC;X1;Yoops;K1\n").unwrap_err();

        assert!(matches!(err, Error::Parse { line: 2, .. }), "{err}");
    }
}
