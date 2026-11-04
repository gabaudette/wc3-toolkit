//! What Tinkerbench Setup and the app share: finding Warcraft III installs,
//! checking their data, and the choices the setup hands to the app.

mod check;
mod choices;
mod games;

pub use check::{GameCheck, GameProblem, check_game, open_game_files};
pub use choices::{APP_IDENTIFIER, ChoicesError, SetupChoices};
pub use games::{FoundGame, GameSource, find_games, game_version, is_game_dir, maps_dir};
