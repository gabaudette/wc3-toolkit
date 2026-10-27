use std::path::Path;

use wc3_map::lua::{SCRIPT_FILE, build_dev_map};
use wc3_map::{Error, Map, STRINGS_FILE};

const MAX_FILES: u32 = 8;
const EDITOR_SCRIPT: &[u8] =
    b"function main()\nend\n\nfunction config()\nSetMapName(\"TRIGSTR_003\")\nend\n";
const STRINGS: &[u8] = b"\xef\xbb\xbfSTRING 3\r\n{\r\nTest map\r\n}\r\n\r\n";

fn make_map(path: &Path) -> mpq::Result<()> {
    let mut archive = mpq::Archive::create(path, MAX_FILES)?;

    archive.write(SCRIPT_FILE, EDITOR_SCRIPT)?;
    archive.write(STRINGS_FILE, STRINGS)?;
    archive.save()
}

#[test]
fn builds_a_dev_copy_with_the_modules() {
    let dir = tempfile::tempdir().unwrap();
    let map = dir.path().join("test.w3m");
    let modules = dir.path().join("src");
    let output = dir.path().join("test_dev.w3m");

    make_map(&map).unwrap();
    std::fs::create_dir(&modules).unwrap();
    std::fs::write(
        modules.join("b.lua"),
        "Dev.OnStart(function() B = true end)\n",
    )
    .unwrap();
    std::fs::write(
        modules.join("a.lua"),
        "Dev.OnStart(function() A = true end)\n",
    )
    .unwrap();
    std::fs::write(modules.join("notes.txt"), "ignored").unwrap();
    let before = std::fs::read(&map).unwrap();

    let report = build_dev_map(&map, &modules, &output).unwrap();

    assert_eq!(report.modules, ["a.lua", "b.lua"]);
    assert!(report.renamed);
    assert_eq!(std::fs::read(&map).unwrap(), before, "source map changed");

    let dev = Map::open(&output).unwrap();
    let script = String::from_utf8(dev.read_file(SCRIPT_FILE).unwrap().unwrap()).unwrap();

    assert!(script.starts_with("function main()"));
    assert!(script.find("A = true").unwrap() < script.find("B = true").unwrap());
    assert_eq!(dev.strings().unwrap().get(3), Some("[DEV] Test map"));
}

#[test]
fn a_syntax_error_stops_the_build_before_writing() {
    let dir = tempfile::tempdir().unwrap();
    let map = dir.path().join("test.w3m");
    let modules = dir.path().join("src");
    let output = dir.path().join("test_dev.w3m");

    make_map(&map).unwrap();
    std::fs::create_dir(&modules).unwrap();
    std::fs::write(modules.join("broken.lua"), "function oops(\n").unwrap();

    let err = build_dev_map(&map, &modules, &output).unwrap_err();

    assert!(
        matches!(&err, Error::Lua { file, .. } if file == "broken.lua"),
        "{err}"
    );
    assert!(!output.exists());
}

#[test]
fn a_map_without_a_lua_script_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let map = dir.path().join("jass.w3m");
    let mut archive = mpq::Archive::create(&map, MAX_FILES).unwrap();

    archive
        .write(
            "war3map.j",
            b"function main takes nothing returns nothing\nendfunction\n",
        )
        .unwrap();
    archive.save().unwrap();

    let err = build_dev_map(&map, dir.path(), &dir.path().join("out.w3m")).unwrap_err();

    assert!(err.to_string().contains("script language to Lua"), "{err}");
}
