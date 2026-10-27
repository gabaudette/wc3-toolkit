//! Saving into maps, using archives built on the fly (no game files needed).

use std::path::Path;

use wc3_map::{Map, save_files};

const MAX_FILES: u32 = 8;

fn make_map(path: &Path, files: &[(&str, &[u8])]) -> mpq::Result<()> {
    let mut archive = mpq::Archive::create(path, MAX_FILES)?;

    for (name, data) in files {
        archive.write(name, data)?;
    }

    archive.save()
}

#[test]
fn saving_to_a_new_file_leaves_the_source_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.w3m");
    let dest = dir.path().join("dest.w3m");

    make_map(&source, &[("war3map.lua", b"-- script")]).unwrap();
    let before = std::fs::read(&source).unwrap();

    save_files(
        &source,
        &dest,
        &[("war3map.wts", b"STRING 1\r\n{\r\nhi\r\n}\r\n\r\n".to_vec())],
    )
    .unwrap();

    assert_eq!(std::fs::read(&source).unwrap(), before);
    let saved = Map::open(&dest).unwrap();

    assert_eq!(
        saved.read_file("war3map.lua").unwrap().unwrap(),
        b"-- script"
    );
    assert_eq!(saved.strings().unwrap().get(1), Some("hi"));
}

#[test]
fn saving_over_the_source_replaces_it() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("map.w3m");

    make_map(&path, &[("war3map.lua", b"-- old")]).unwrap();

    save_files(&path, &path, &[("war3map.lua", b"-- new".to_vec())]).unwrap();

    let saved = Map::open(&path).unwrap();

    assert_eq!(saved.read_file("war3map.lua").unwrap().unwrap(), b"-- new");
    assert_eq!(
        std::fs::read_dir(dir.path()).unwrap().count(),
        1,
        "temporary file left behind"
    );
}

#[test]
fn a_failed_save_reports_the_error_and_leaves_nothing_behind() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("not-a-map.w3m");
    std::fs::write(&source, b"this is not an MPQ archive").unwrap();

    let result = save_files(
        &source,
        &dir.path().join("out.w3m"),
        &[("a.txt", b"a".to_vec())],
    );

    assert!(result.is_err());
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}
