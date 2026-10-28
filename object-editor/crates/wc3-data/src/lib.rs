pub mod abilities;
pub mod blp;
pub mod constants;
pub mod doodads;
pub mod game;
pub mod guide;
pub mod icon;
pub mod leveled;
pub mod meta;
pub mod model;
pub mod objects;
pub mod profile;
pub mod slk;
pub mod spells;
pub mod strings;
pub mod tga;
pub mod units;
pub mod upgrades;
pub mod weather;

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
    pub fn in_file(self, file: impl Into<String>) -> Self {
        Self::InFile {
            file: file.into(),
            source: Box::new(self),
        }
    }

    pub fn is_not_found(&self) -> bool {
        match self {
            Self::Casc(casc::Error::NotFound(_)) => true,
            Self::InFile { source, .. } => source.is_not_found(),
            _ => false,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
