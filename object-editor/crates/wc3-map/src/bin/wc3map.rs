//! Command-line tool for a map's unit data, mainly for checking reads and
//! writes against the World Editor.

// Errors and usage go to stderr directly (AGE-QUAL-003: documented, file-scoped).
// Regular output goes through `say!`, which handles a closed pipe.
#![allow(clippy::print_stderr)]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use wc3_data::game::{ArtMode, GameFiles, GameOptions};
use wc3_data::units::UnitDatabase;
use wc3_map::lua;
use wc3_map::units::MapUnits;
use wc3_map::{FourCC, Layout, Map, OBJECT_FILES, ObjectData, STRINGS_FILE, save_files};

const USAGE: &str = "\
usage: wc3map [--game DIR] [--balance SET] [--hd] <map> <command>

options:
  --game DIR      Warcraft III install folder (default: $WC3_GAME_DIR)
  --balance SET   balance data set, e.g. custom_v1 (default: base data)
  --hd            use HD art values

commands:
  check                              rewrite every object file and the string table in
                                     memory and confirm they come out byte for byte identical
  units                              custom and changed units
  unit <ID>                          every field of a unit; * marks the map's changes
  set <ID> <FIELD> <VALUE> -o <OUT>  change a field and save the map as OUT
  new <BASE> -o <OUT>                create a custom unit copying BASE and save as OUT
  lua <DIR> [-o OUT] [--script FILE] build a [DEV] copy of the map with the Lua modules in
                                     DIR added to its script (default OUT: <map>_dev.<ext>);
                                     --script also saves the final script to FILE

OUT may be the input map itself; it is replaced only after a complete save.";

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
    game_dir: Option<PathBuf>,
    options: GameOptions,
    mode: ArtMode,
    output: Option<PathBuf>,
    script_out: Option<PathBuf>,
    positional: Vec<String>,
}

fn parse_args() -> Result<Args, String> {
    let mut parsed = Args {
        game_dir: std::env::var_os(GAME_DIR_VAR).map(PathBuf::from),
        options: GameOptions::default(),
        mode: ArtMode::Sd,
        output: None,
        script_out: None,
        positional: Vec::new(),
    };
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--game" => parsed.game_dir = Some(args.next().ok_or("--game needs a folder")?.into()),
            "--balance" => {
                parsed.options.balance = Some(args.next().ok_or("--balance needs a set name")?)
            }
            "--hd" => parsed.mode = ArtMode::Hd,
            "-o" | "--out" => parsed.output = Some(args.next().ok_or("-o needs a file")?.into()),
            "--script" => {
                parsed.script_out = Some(args.next().ok_or("--script needs a file")?.into())
            }
            "-h" | "--help" => return Err(String::new()),
            _ => parsed.positional.push(arg),
        }
    }

    Ok(parsed)
}

fn fourcc(code: &str) -> Result<FourCC, String> {
    FourCC::parse(code).ok_or(format!("{code:?} is not a four-character code"))
}

fn check(map: &Map) -> Result<(), String> {
    let mut checked = 0;

    for name in OBJECT_FILES.iter().copied().chain([STRINGS_FILE]) {
        let Some(original) = map.read_file(name).map_err(|e| e.to_string())? else {
            continue;
        };

        let rewritten = match Layout::for_file(name) {
            Some(layout) => ObjectData::parse(&original, layout)
                .map_err(|e| format!("{name}: {e}"))?
                .to_bytes(),
            None => map
                .strings()
                .map_err(|e| e.to_string())?
                .to_text()
                .into_bytes(),
        };

        let same = rewritten == original;

        say!(
            "  {name:<18} {:>8} bytes  {}",
            original.len(),
            if same { "identical" } else { "DIFFERENT" }
        );
        if !same {
            return Err(format!("{name} does not round-trip"));
        }

        checked += 1;
    }

    say!("{checked} files identical");
    Ok(())
}

fn load_db(args: &Args) -> Result<UnitDatabase, String> {
    let dir = args
        .game_dir
        .as_ref()
        .ok_or(format!("no game folder: pass --game or set {GAME_DIR_VAR}"))?;
    let files = GameFiles::open(dir, args.options.clone()).map_err(|e| e.to_string())?;
    UnitDatabase::load(&files).map_err(|e| e.to_string())
}

fn list_units(db: &UnitDatabase, units: &MapUnits) -> Result<(), String> {
    for unit in units.units() {
        let kind = if unit.is_custom { "custom " } else { "changed" };

        say!(
            "{}  {kind}  based on {}  {} field(s) changed  {}",
            unit.id,
            unit.base,
            unit.modified_fields,
            units.name(db, unit.id)
        );
    }

    Ok(())
}

fn show_unit(db: &UnitDatabase, units: &MapUnits, id: FourCC, mode: ArtMode) -> Result<(), String> {
    let mut rows: Vec<(String, String, bool, String)> = units
        .fields(db, id)
        .into_iter()
        .map(|f| {
            let label = db.field_label(f);
            (
                label,
                units.display_value(db, id, f, mode),
                units.is_modified(id, f),
                f.id.clone(),
            )
        })
        .collect();

    if rows.is_empty() {
        return Err(format!("unknown unit {id}"));
    }

    rows.sort();
    let changed = rows.iter().filter(|r| r.2).count();

    say!(
        "{id}: {}, {} fields, {changed} changed",
        units.name(db, id),
        rows.len()
    );
    for (label, value, modified, field_id) in rows {
        let mark = if modified { '*' } else { ' ' };

        say!("{mark} {field_id}  {label:<55} {value}");
    }

    Ok(())
}

fn save(map_path: &Path, output: Option<&PathBuf>, units: &MapUnits) -> Result<(), String> {
    let output = output.ok_or("this command needs -o <OUT>")?;
    let files: Vec<(&str, Vec<u8>)> = units.files();

    save_files(map_path, output, &files).map_err(|e| e.to_string())?;
    say!("saved {}", output.display());
    Ok(())
}

fn build_lua(map_path: &Path, modules_dir: &Path, args: &Args) -> Result<(), String> {
    let output = args
        .output
        .clone()
        .unwrap_or_else(|| lua::default_output(map_path));
    let report = lua::build_dev_map(map_path, modules_dir, &output).map_err(|e| e.to_string())?;

    if let Some(script_out) = &args.script_out {
        std::fs::write(script_out, &report.script)
            .map_err(|e| format!("cannot write {}: {e}", script_out.display()))?;
    }

    say!("built {}", report.output.display());
    say!("modules: {}", report.modules.join(", "));
    if !report.renamed {
        say!("note: the map name is not a string reference, so it has no [DEV] prefix");
    }

    Ok(())
}

fn run(args: Args) -> Result<(), String> {
    let positional: Vec<&str> = args.positional.iter().map(String::as_str).collect();
    let Some((map_arg, command)) = positional.split_first() else {
        return Err(String::new());
    };

    let map_path = Path::new(map_arg);

    if let ["lua", modules_dir] = command {
        return build_lua(map_path, Path::new(modules_dir), &args);
    }

    let map = Map::open(map_path).map_err(|e| e.to_string())?;

    if command == ["check"] {
        return check(&map);
    }

    let db = load_db(&args)?;
    let mut units = MapUnits::load(&map).map_err(|e| e.to_string())?;

    drop(map);

    match command {
        ["units"] => list_units(&db, &units)?,
        ["unit", id] => show_unit(&db, &units, fourcc(id)?, args.mode)?,
        ["set", id, field_id, value] => {
            let id = fourcc(id)?;
            let field = db
                .field(field_id)
                .ok_or(format!("unknown field {field_id}"))?;

            units
                .set(&db, id, field, value)
                .map_err(|e| e.to_string())?;
            say!(
                "{id} {}: {}",
                db.field_label(field),
                units.display_value(&db, id, field, args.mode)
            );
            save(map_path, args.output.as_ref(), &units)?;
        }
        ["new", base] => {
            let id = units
                .create(&db, fourcc(base)?)
                .map_err(|e| e.to_string())?;

            say!("created {id} based on {base}");
            save(map_path, args.output.as_ref(), &units)?;
        }
        _ => return Err(String::new()),
    }

    Ok(())
}

fn main() -> ExitCode {
    match parse_args().and_then(run) {
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
