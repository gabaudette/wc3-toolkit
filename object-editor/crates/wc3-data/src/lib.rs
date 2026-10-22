//! Warcraft III game data: SLK tables, profile text files and the Object
//! Editor's field metadata.

pub mod abilities;
pub mod game;
pub mod guide;
pub mod icon;
pub mod meta;
pub mod model;
pub mod objects;
pub mod profile;
pub mod slk;
pub mod strings;
pub mod units;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Casc(#[from] casc::Error),
    #[error("line {line}: {reason}")]
    Parse { line: usize, reason: String },
    #[error("{0}")]
    Image(String),
    #[error("row {row}: missing or invalid {column}")]
    Field { row: usize, column: &'static str },
    #[error("{file}: {source}")]
    InFile {
        file: String,
        #[source]
        source: Box<Error>,
    },
}

impl Error {
    /// Adds the name of the file being read to an error.
    pub fn in_file(self, file: impl Into<String>) -> Self {
        Self::InFile {
            file: file.into(),
            source: Box::new(self),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
