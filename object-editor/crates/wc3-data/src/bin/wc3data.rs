//! Command-line inspector for the game data, mainly for checking the loader
//! against the World Editor.

// Errors and usage go to stderr directly (AGE-QUAL-003: documented, file-scoped).
// Regular output goes through `say!`, which handles a closed pipe.
#![allow(clippy::print_stderr)]

use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use wc3_data::game::{ArtMode, GameFiles, GameOptions};
use wc3_data::icon::load_texture;
use wc3_data::meta::ObjectKind;
use wc3_data::units::UnitDatabase;

const USAGE: &str = "\
usage: wc3data [--game DIR] [--balance SET] [--hd] <command>

options:
  --game DIR      Warcraft III install folder (default: $WC3_GAME_DIR)
  --balance SET   balance data set, e.g. custom_v1 (default: base data)
  --hd            use HD art values and icons instead of SD

commands:
  heroes                 list every hero
  units                  list every unit, hero and building
  unit <ID>              every Object Editor field of one unit
  icon <ID> <OUT.png>    save a unit's icon as PNG";

const GAME_DIR_VAR: &str = "WC3_GAME_DIR";

/// Marks output that stopped because the reader went away (e.g. `| head`).
const OUTPUT_CLOSED: &str = "\0output closed";

/// Writes a line to stdout. A closed pipe ends the program quietly instead of
/// panicking like `println!` does.
fn emit(line: std::fmt::Arguments<'_>) -> Result<(), String> {
    let mut stdout = std::io::stdout().lock();

    match writeln!(stdout, "{line}") {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => Err(OUTPUT_CLOSED.to_owned()),
        Err(e) => Err(format!("cannot write output: {e}")),
    }
}

macro_rules! say {
    ($($arg:tt)*) => {
        emit(format_args!($($arg)*))?
    };
}

struct Args {
    game_dir: PathBuf,
    options: GameOptions,
    mode: ArtMode,
    command: Vec<String>,
}

fn parse_args() -> Result<Args, String> {
    let mut game_dir = std::env::var_os(GAME_DIR_VAR).map(PathBuf::from);
    let mut options = GameOptions::default();
    let mut mode = ArtMode::Sd;
    let mut command = Vec::new();
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--game" => game_dir = Some(args.next().ok_or("--game needs a folder")?.into()),
            "--balance" => options.balance = Some(args.next().ok_or("--balance needs a set name")?),
            "--hd" => mode = ArtMode::Hd,
            "-h" | "--help" => return Err(String::new()),
            _ => command.push(arg),
        }
    }

    let game_dir = game_dir.ok_or(format!("no game folder: pass --game or set {GAME_DIR_VAR}"))?;

    Ok(Args {
        game_dir,
        options,
        mode,
        command,
    })
}

fn kind_label(kind: ObjectKind) -> &'static str {
    match kind {
        ObjectKind::Hero => "hero",
        ObjectKind::Unit => "unit",
        ObjectKind::Building => "building",
        ObjectKind::Item => "item",
        ObjectKind::Buff => "buff",
        ObjectKind::Destructable => "destructable",
    }
}

fn list(db: &UnitDatabase, heroes_only: bool) -> Result<(), String> {
    let mut count = 0;

    for id in db.ids() {
        let kind = db.kind(id);

        if heroes_only && kind != ObjectKind::Hero {
            continue;
        }

        say!("{id}  {:<8}  {}", kind_label(kind), db.name(id));
        count += 1;
    }

    say!("{count} entries");
    Ok(())
}

fn show_unit(db: &UnitDatabase, id: &str, mode: ArtMode) -> Result<(), String> {
    if !db.contains(id) {
        return Err(format!("unknown unit ID {id}"));
    }

    let mut rows: Vec<(String, String, &str)> = db
        .fields_for(id)
        .map(|f| {
            (
                db.field_label(f),
                db.display_value(id, f, mode),
                f.id.as_str(),
            )
        })
        .collect();

    rows.sort();

    say!(
        "{id}: {} ({}), {} fields",
        db.name(id),
        kind_label(db.kind(id)),
        rows.len()
    );
    for (label, value, field_id) in rows {
        say!("  {field_id}  {label:<55} {value}");
    }

    Ok(())
}

fn save_icon(
    files: &GameFiles,
    db: &UnitDatabase,
    id: &str,
    out: &str,
    mode: ArtMode,
) -> Result<(), String> {
    let path = db.icon_path(id, mode).ok_or(format!("{id} has no icon"))?;
    let image = load_texture(files, &path, mode).map_err(|e| e.to_string())?;
    let png = image.to_png().map_err(|e| e.to_string())?;
    std::fs::write(out, png).map_err(|e| format!("cannot write {out}: {e}"))?;

    say!("{path} -> {out} ({}x{})", image.width, image.height);
    Ok(())
}

fn run(args: Args) -> Result<(), String> {
    let files = GameFiles::open(&args.game_dir, args.options).map_err(|e| e.to_string())?;
    let db = UnitDatabase::load(&files).map_err(|e| e.to_string())?;
    let command: Vec<&str> = args.command.iter().map(String::as_str).collect();

    match command.as_slice() {
        ["heroes"] => list(&db, true)?,
        ["units"] => list(&db, false)?,
        ["unit", id] => show_unit(&db, id, args.mode)?,
        ["icon", id, out] => save_icon(&files, &db, id, out, args.mode)?,
        _ => return Err(String::new()),
    }

    Ok(())
}

fn main() -> ExitCode {
    let result = parse_args().and_then(run);

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) if message == OUTPUT_CLOSED => ExitCode::SUCCESS,
        Err(message) if message.is_empty() => {
            eprintln!("{USAGE}");
            ExitCode::FAILURE
        }
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}
