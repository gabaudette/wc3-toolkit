"""Build test_dev.w3m: the editor's test.w3m plus every Lua file in src/.

The editor-generated war3map.lua is kept as is; our code is appended and
hooked into main(), so editor saves never conflict with it. The output map is
written next to the input map.

Usage: python3 -I tools/build.py <path/to/test.w3m>

Temporary tool: it loads the Linux builds of StormLib and luac from tools/, so
it only runs on Linux. Map building moves into the Rust object editor later.
"""

import ctypes
import re
import shutil
import subprocess
import sys
import tempfile
from ctypes import byref, c_bool, c_char_p, c_uint32, c_void_p
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LUAC = ROOT / "tools" / "bin" / "luac"
STORMLIB = ROOT / "tools" / "lib" / "libstorm.so"
OUTPUT_NAME = "test_dev.w3m"
DEV_PREFIX = "[DEV] "
LOADER_MODULE = "_dev.lua"

MPQ_FILE_COMPRESS = 0x200
MPQ_FILE_REPLACE_EXISTING = 0x8000_0000
MPQ_COMPRESSION_ZLIB = 0x02

HOOK = """
-- ===== dev hook: run src/ modules after the editor's main() =====
do
    local editor_main = main
    function main()
        editor_main()
        for _, fn in ipairs(Dev.starters) do fn() end
    end
end
"""


def load_stormlib() -> ctypes.CDLL:
    lib = ctypes.CDLL(str(STORMLIB))
    lib.SFileOpenArchive.argtypes = [c_char_p, c_uint32, c_uint32, ctypes.POINTER(c_void_p)]
    lib.SFileOpenArchive.restype = c_bool
    lib.SFileExtractFile.argtypes = [c_void_p, c_char_p, c_char_p, c_uint32]
    lib.SFileExtractFile.restype = c_bool
    lib.SFileAddFileEx.argtypes = [c_void_p, c_char_p, c_char_p, c_uint32, c_uint32, c_uint32]
    lib.SFileAddFileEx.restype = c_bool
    lib.SFileCloseArchive.argtypes = [c_void_p]
    lib.SFileCloseArchive.restype = c_bool
    lib.SErrGetLastError.restype = c_uint32
    return lib


def extract(storm: ctypes.CDLL, archive: c_void_p, name: str, dest: Path) -> str | None:
    if storm.SFileExtractFile(archive, name.encode(), str(dest).encode(), 0):
        return None

    return f"{name} not in map (error {storm.SErrGetLastError()})"


def replace(storm: ctypes.CDLL, archive: c_void_p, source: Path, name: str) -> str | None:
    flags = MPQ_FILE_COMPRESS | MPQ_FILE_REPLACE_EXISTING
    written = storm.SFileAddFileEx(
        archive,
        str(source).encode(),
        name.encode(),
        flags,
        MPQ_COMPRESSION_ZLIB,
        MPQ_COMPRESSION_ZLIB,
    )
    if written:
        return None

    return f"cannot write {name} (error {storm.SErrGetLastError()})"


def rename_map(storm: ctypes.CDLL, archive: c_void_p, tmp: Path, editor_lua: str) -> str | None:
    """Prefix the map's display name with [DEV] so it stands out in the game's map list."""
    match = re.search(r'SetMapName\("TRIGSTR_(\d+)"\)', editor_lua)
    if not match:
        print("warning: map name is not a TRIGSTR, leaving it unchanged")
        return None

    wts = tmp / "war3map.wts"
    error = extract(storm, archive, "war3map.wts", wts)
    if error:
        return error

    text = wts.read_text(encoding="utf-8-sig")
    block = re.compile(r"(STRING %d\s*\r?\n\{\r?\n)" % int(match.group(1)))
    if not block.search(text):
        return f"map name string {match.group(1)} not found in war3map.wts"

    text = block.sub(lambda b: b.group(1) + DEV_PREFIX, text, count=1)
    with wts.open("w", encoding="utf-8-sig", newline="") as out:
        out.write(text)

    return replace(storm, archive, wts, "war3map.wts")


def src_files() -> list[Path]:
    """Lua modules in load order: the loader first, then the rest alphabetically."""
    files = sorted((ROOT / "src").glob("*.lua"))
    return sorted(files, key=lambda f: f.name != LOADER_MODULE)


def build_script(editor_lua: str) -> str:
    parts = [editor_lua]

    for module in src_files():
        source = module.read_text(encoding="utf-8")
        parts.append(f"\n-- ===== src/{module.name} =====\n{source}")

    parts.append(HOOK)
    return "".join(parts)


def patch_archive(storm: ctypes.CDLL, archive: c_void_p, tmp: Path) -> str | None:
    """Swaps in the combined script and the [DEV] name. Leaves the archive open."""
    editor_path = tmp / "editor.lua"
    if extract(storm, archive, "war3map.lua", editor_path):
        return "war3map.lua not in map (is the script language set to Lua?)"

    editor_lua = editor_path.read_text(encoding="utf-8-sig")
    combined = tmp / "war3map.lua"
    with combined.open("w", encoding="utf-8", newline="\n") as out:
        out.write(build_script(editor_lua))

    check = subprocess.run([LUAC, "-p", combined], capture_output=True, text=True)
    if check.returncode != 0:
        return "Lua syntax error\n" + check.stderr

    error = rename_map(storm, archive, tmp, editor_lua)
    if error:
        return error

    error = replace(storm, archive, combined, "war3map.lua")
    if error:
        return error

    # Keep the final script around for reading and debugging.
    (ROOT / "build").mkdir(exist_ok=True)
    shutil.copyfile(combined, ROOT / "build" / "war3map.lua")
    return None


def build(source_map: Path) -> str | None:
    """Builds the dev map next to `source_map`. Returns an error message on failure."""
    if not source_map.is_file():
        return f"{source_map} not found"

    storm = load_stormlib()

    with tempfile.TemporaryDirectory() as tmp_dir:
        tmp = Path(tmp_dir)
        work = tmp / OUTPUT_NAME
        shutil.copyfile(source_map, work)

        archive = c_void_p()
        if not storm.SFileOpenArchive(str(work).encode(), 0, 0, byref(archive)):
            return f"cannot open {source_map} (error {storm.SErrGetLastError()})"

        error = patch_archive(storm, archive, tmp)
        saved = storm.SFileCloseArchive(archive)
        if error:
            return error

        if not saved:
            return f"cannot save archive (error {storm.SErrGetLastError()})"

        shutil.move(work, source_map.with_name(OUTPUT_NAME))

    return None


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__, file=sys.stderr)
        return 2

    source_map = Path(sys.argv[1])
    error = build(source_map)
    if error:
        print(f"build failed: {error}", file=sys.stderr)
        return 1

    print(f"built {source_map.with_name(OUTPUT_NAME)}")
    print("modules:", ", ".join(f.name for f in src_files()))
    return 0


if __name__ == "__main__":
    sys.exit(main())
