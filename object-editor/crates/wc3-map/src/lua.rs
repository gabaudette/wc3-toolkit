//! Lua script harness: builds a development copy of a map with Lua modules
//! from a folder added to the World Editor's script.
//!
//! The editor regenerates `war3map.lua` on every save, so code written into the
//! map directly would be lost. Instead, modules live in their own files and are
//! appended to the editor's script in a copy of the map, where a small hook runs
//! them after the editor's `main()`. The copy's name gets a `[DEV]` prefix so it
//! stands out in the game's map list.
//!
//! Modules register start functions with `Dev.OnStart(fn)`.

use std::path::{Path, PathBuf};

use wc3_data::strings::{MapStrings, reference_id};

use crate::{Error, Map, Result, STRINGS_FILE, save_files};

pub const SCRIPT_FILE: &str = "war3map.lua";
pub const DEV_NAME_PREFIX: &str = "[DEV] ";
const MODULE_EXTENSION: &str = "lua";
const BYTE_ORDER_MARK: char = '\u{feff}';

const PRELUDE: &str = "
-- ===== wc3-toolkit: module loader =====
Dev = { starters = {} }

function Dev.OnStart(fn)
    table.insert(Dev.starters, fn)
end
";

const HOOK: &str = "
-- ===== wc3-toolkit: run modules after the editor's main() =====
do
    local editor_main = main
    function main()
        editor_main()
        for _, fn in ipairs(Dev.starters) do fn() end
    end
end
";

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub name: String,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct BuildReport {
    pub output: PathBuf,
    pub modules: Vec<String>,
    pub script: String,
    pub renamed: bool,
}

pub fn default_output(map: &Path) -> PathBuf {
    let stem = map.file_stem().unwrap_or_default().to_string_lossy();
    let name = match map.extension() {
        Some(ext) => format!("{stem}_dev.{}", ext.to_string_lossy()),
        None => format!("{stem}_dev"),
    };

    map.with_file_name(name)
}

pub fn load_modules(dir: &Path) -> Result<Vec<Module>> {
    let io_error = |path: &Path| {
        let path = path.to_owned();
        move |source| Error::Io { path, source }
    };

    let mut paths: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(io_error(dir))?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .is_some_and(|e| e.eq_ignore_ascii_case(MODULE_EXTENSION))
        })
        .collect();

    paths.sort();

    paths
        .into_iter()
        .map(|path| {
            let source = std::fs::read_to_string(&path).map_err(io_error(&path))?;
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();

            Ok(Module {
                name,
                source: strip_bom(&source).to_owned(),
            })
        })
        .collect()
}

/// Checks that `source` is valid Lua 5.3, the version Warcraft III runs.
pub fn check_syntax(file: &str, source: &str) -> Result<()> {
    let result = full_moon::parse_fallible(source, full_moon::LuaVersion::lua53()).into_result();
    let Err(errors) = result else {
        return Ok(());
    };

    let Some(first) = errors.first() else {
        return Ok(());
    };

    let (start, _) = first.range();

    Err(Error::Lua {
        file: file.to_owned(),
        line: start.line(),
        column: start.character(),
        message: first.error_message().into_owned(),
    })
}

pub fn build_script(editor_script: &str, modules: &[Module]) -> String {
    let mut script = String::from(editor_script);

    script.push_str(PRELUDE);

    for module in modules {
        script.push_str(&format!("\n-- ===== module: {} =====\n", module.name));
        script.push_str(&module.source);
        if !module.source.ends_with('\n') {
            script.push('\n');
        }
    }

    script.push_str(HOOK);
    script
}

pub fn mark_dev_name(editor_script: &str, strings: &mut MapStrings) -> bool {
    let Some(id) = map_name_reference(editor_script) else {
        return false;
    };

    let Some(name) = strings.get(id).map(str::to_owned) else {
        return false;
    };

    if name.starts_with(DEV_NAME_PREFIX) {
        return true;
    }

    strings.set(id, &format!("{DEV_NAME_PREFIX}{name}"))
}

/// The string ID in the editor's `SetMapName("TRIGSTR_003")` call.
fn map_name_reference(editor_script: &str) -> Option<u32> {
    const CALL: &str = "SetMapName(\"";

    let start = editor_script.find(CALL)? + CALL.len();
    let end = start + editor_script[start..].find('"')?;

    reference_id(&editor_script[start..end])
}

fn strip_bom(text: &str) -> &str {
    text.strip_prefix(BYTE_ORDER_MARK).unwrap_or(text)
}

pub fn build_dev_map(map: &Path, modules_dir: &Path, output: &Path) -> Result<BuildReport> {
    let source_map = Map::open(map)?;
    let editor_bytes = source_map.read_file(SCRIPT_FILE)?.ok_or_else(|| {
        Error::Invalid(format!(
            "{} has no {SCRIPT_FILE}; set the map's script language to Lua in the World Editor",
            map.display()
        ))
    })?;
    let editor_script = String::from_utf8(editor_bytes)
        .map_err(|_| Error::Invalid(format!("{SCRIPT_FILE} is not UTF-8")))?;
    let editor_script = strip_bom(&editor_script);
    let mut strings = source_map.strings()?;

    drop(source_map);

    check_syntax(SCRIPT_FILE, editor_script)?;
    let modules = load_modules(modules_dir)?;

    for module in &modules {
        check_syntax(&module.name, &module.source)?;
    }

    let script = build_script(editor_script, &modules);

    check_syntax("combined script", &script)?;
    let renamed = mark_dev_name(editor_script, &mut strings);

    let files = [
        (SCRIPT_FILE, script.clone().into_bytes()),
        (STRINGS_FILE, strings.to_text().into_bytes()),
    ];

    save_files(map, output, &files)?;

    Ok(BuildReport {
        output: output.to_owned(),
        modules: modules.into_iter().map(|m| m.name).collect(),
        script,
        renamed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const EDITOR_SCRIPT: &str =
        "function main()\nend\n\nfunction config()\nSetMapName(\"TRIGSTR_003\")\nend\n";
    const STRINGS: &str = "STRING 3\r\n{\r\nTest map\r\n}\r\n\r\n";

    fn module(name: &str, source: &str) -> Module {
        Module {
            name: name.to_owned(),
            source: source.to_owned(),
        }
    }

    #[test]
    fn script_keeps_the_editor_code_then_loader_modules_and_hook() {
        let script = build_script(
            EDITOR_SCRIPT,
            &[module("a.lua", "A = 1"), module("b.lua", "B = 2\n")],
        );

        let order: Vec<usize> = [
            "function main()",
            "Dev = { starters",
            "A = 1",
            "B = 2",
            "local editor_main",
        ]
        .iter()
        .map(|needle| script.find(needle).unwrap())
        .collect();

        assert!(order.windows(2).all(|w| w[0] < w[1]), "{order:?}");
        assert!(check_syntax("script", &script).is_ok());
    }

    #[test]
    fn syntax_errors_name_the_file_and_line() {
        let err = check_syntax("broken.lua", "local x = 1\nif x then\n").unwrap_err();

        let Error::Lua { file, line, .. } = err else {
            panic!("unexpected error {err}");
        };

        assert_eq!(file, "broken.lua");
        assert!(line >= 2, "line {line}");
    }

    #[test]
    fn accepts_lua_53_syntax() {
        assert!(
            check_syntax(
                "ops.lua",
                "local a = 7 // 2\nlocal b = a & 1\ngoto done\n::done::\n"
            )
            .is_ok()
        );
    }

    #[test]
    fn marks_the_map_name_once() {
        let mut strings = MapStrings::parse(STRINGS).unwrap();

        assert!(mark_dev_name(EDITOR_SCRIPT, &mut strings));
        assert!(mark_dev_name(EDITOR_SCRIPT, &mut strings));
        assert_eq!(strings.get(3), Some("[DEV] Test map"));
        assert!(!mark_dev_name("SetMapName(\"Literal\")", &mut strings));
    }

    #[test]
    fn every_spell_template_is_valid_lua_in_the_harness() {
        let templates = wc3_data::spells::spell_templates().unwrap();
        let none = std::collections::HashMap::new();

        for template in &templates {
            let source =
                wc3_data::spells::render_template(template, "A001", "Test spell", &none).unwrap();

            check_syntax(&template.file, &source).unwrap();

            let script = build_script(EDITOR_SCRIPT, &[module(&template.file, &source)]);

            check_syntax("combined script", &script).unwrap();
        }
    }

    #[test]
    fn output_sits_next_to_the_map() {
        let out = default_output(Path::new("maps/test.w3m"));

        assert_eq!(out, Path::new("maps").join("test_dev.w3m"));
    }
}
