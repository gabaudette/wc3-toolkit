//! 3D models (`.mdx`) for previews.
//!
//! The current game writes some chunks in newer layouts than open-source model
//! readers understand. [`load_model`] rewrites a model into the layout they
//! expect, and every unit model then parses and draws:
//!
//! - Camera and light chunks are removed; a preview needs neither.
//! - Skin weights (bone indices and weights per vertex) are stored as 16-bit
//!   values instead of bytes; they are narrowed back to bytes. Read as bytes,
//!   the wide layout scrambles the skeleton and the model draws as a dark,
//!   torn silhouette.

use crate::game::{ArtMode, GameFiles};
use crate::{Error, Result};

const MAGIC: &[u8; 4] = b"MDLX";
const MODEL_EXTENSION: &str = ".mdx";
const TAG_SIZE: usize = 4;
const U32_SIZE: usize = 4;
const CHUNK_HEADER: usize = TAG_SIZE + U32_SIZE;
/// Portrait cameras and lights: not needed to show a model.
const PREVIEW_SKIPPED_CHUNKS: &[&[u8; 4]] = &[b"CAMS", b"LITE"];
const VERSION_CHUNK: &[u8; 4] = b"VERS";
const GEOSETS_CHUNK: &[u8; 4] = b"GEOS";

/// The arrays every geoset starts with, in order, and the bytes per element.
const GEOSET_ARRAYS: &[(&[u8; 4], usize)] = &[
    (b"VRTX", 12),
    (b"NRMS", 12),
    (b"PTYP", 4),
    (b"PCNT", 4),
    (b"PVTX", 2),
    (b"GNDX", 1),
    (b"MTGC", 4),
    (b"MATS", 4),
];
/// Material ID, selection group and selection flags.
const GEOSET_FIELDS_SIZE: usize = 3 * U32_SIZE;
/// Geosets have a level of detail and a name from this model version on.
const GEOSET_NAME_VERSION: u32 = 900;
const GEOSET_NAME_SIZE: usize = 80;
/// Bounds radius, minimum and maximum corners.
const EXTENT_SIZE: usize = 28;
const TANGENTS: &[u8; 4] = b"TANG";
const TANGENT_SIZE: usize = 16;
const SKIN: &[u8; 4] = b"SKIN";
/// Texture coordinates: the last part of a geoset.
const UV_SETS: &[u8; 4] = b"UVAS";
/// Tags that can follow a skin chunk inside a geoset.
const AFTER_SKIN: &[&[u8; 4]] = &[TANGENTS, UV_SETS];
const WIDE_SKIN_VALUE_SIZE: usize = 2;

/// The model file for a model path as unit data stores it, e.g.
/// `units\human\HeroPaladin\HeroPaladin` → `units\human\HeroPaladin\HeroPaladin.mdx`.
/// Any other extension is replaced, as the game does: `.mdl`, and the `.ma` a few
/// cinematic doodads name.
pub fn model_file(path: &str) -> String {
    if path.to_ascii_lowercase().ends_with(MODEL_EXTENSION) {
        return path.to_owned();
    }

    let name_start = path.rfind(['\\', '/']).map_or(0, |i| i + 1);
    let stem = match path[name_start..].rfind('.') {
        Some(dot) => &path[..name_start + dot],
        None => path,
    };

    format!("{stem}{MODEL_EXTENSION}")
}

/// Reads a model for previewing, in the layout model readers understand.
pub fn load_model(files: &GameFiles, path: &str, mode: ArtMode) -> Result<Vec<u8>> {
    let file = model_file(path);
    let data = files.read_art(&file, mode)?;

    preview_copy(&data).map_err(|e| e.in_file(file))
}

fn invalid(reason: String) -> Error {
    Error::Parse { line: 0, reason }
}

fn read_u32(data: &[u8], pos: usize) -> Result<u32> {
    let mut bytes = [0u8; U32_SIZE];

    bytes.copy_from_slice(
        data.get(pos..pos + U32_SIZE)
            .ok_or_else(|| invalid(format!("cut off at byte {pos}")))?,
    );
    Ok(u32::from_le_bytes(bytes))
}

fn read_len(data: &[u8], pos: usize) -> Result<usize> {
    read_u32(data, pos).map(|n| n as usize)
}

/// Copies an MDX file without the chunks previews skip, with skin weights narrowed.
pub fn preview_copy(data: &[u8]) -> Result<Vec<u8>> {
    if data.get(..MAGIC.len()) != Some(MAGIC.as_slice()) {
        return Err(invalid("not an MDX model".to_owned()));
    }

    let mut out = Vec::with_capacity(data.len());

    out.extend_from_slice(MAGIC);
    let mut pos = MAGIC.len();
    let mut version = None;

    while pos < data.len() {
        let tag = data
            .get(pos..pos + TAG_SIZE)
            .ok_or_else(|| invalid(format!("chunk header cut off at byte {pos}")))?;
        let size = read_len(data, pos + TAG_SIZE)?;
        let body_start = pos + CHUNK_HEADER;
        let end = body_start + size;
        let chunk = data
            .get(pos..end)
            .ok_or_else(|| invalid(format!("chunk at byte {pos} runs past the end")))?;
        let body = &chunk[CHUNK_HEADER..];

        if tag == VERSION_CHUNK {
            version = Some(read_u32(body, 0)?);
        }

        if PREVIEW_SKIPPED_CHUNKS
            .iter()
            .any(|skipped| tag == skipped.as_slice())
        {
            // Left out.
        } else if tag == GEOSETS_CHUNK {
            let version =
                version.ok_or_else(|| invalid("geosets come before the version".to_owned()))?;
            let geosets = narrow_skins(body, version)?;

            out.extend_from_slice(tag);
            out.extend_from_slice(&chunk_size(geosets.len())?.to_le_bytes());
            out.extend_from_slice(&geosets);
        } else {
            out.extend_from_slice(chunk);
        }

        pos = end;
    }

    Ok(out)
}

fn chunk_size(len: usize) -> Result<u32> {
    u32::try_from(len).map_err(|_| invalid(format!("chunk of {len} bytes is too large")))
}

/// Rewrites every geoset of a `GEOS` chunk body. Each geoset starts with its
/// own size, that size included.
fn narrow_skins(body: &[u8], version: u32) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(body.len());
    let mut pos = 0;

    while pos < body.len() {
        let size = read_len(body, pos)?;
        let geoset = body
            .get(pos..pos + size)
            .filter(|_| size >= U32_SIZE)
            .ok_or_else(|| invalid(format!("geoset at byte {pos} has a bad size")))?;

        narrow_geoset_skin(geoset, version, &mut out)?;
        pos += size;
    }

    Ok(out)
}

/// Copies one geoset, narrowing its skin chunk from 16-bit values to bytes
/// when it is stored wide. The value count stays; only its size changes.
fn narrow_geoset_skin(geoset: &[u8], version: u32, out: &mut Vec<u8>) -> Result<()> {
    let mut pos = U32_SIZE;

    for (tag, element_size) in GEOSET_ARRAYS {
        if geoset.get(pos..pos + TAG_SIZE) != Some(tag.as_slice()) {
            return Err(invalid(format!(
                "geoset is missing its {} array",
                tag.escape_ascii()
            )));
        }

        pos += TAG_SIZE + read_len(geoset, pos + TAG_SIZE)? * element_size + U32_SIZE;
    }

    pos += GEOSET_FIELDS_SIZE;
    if version >= GEOSET_NAME_VERSION {
        pos += U32_SIZE + GEOSET_NAME_SIZE;
    }

    pos += EXTENT_SIZE;
    pos += U32_SIZE + read_len(geoset, pos)? * EXTENT_SIZE;

    loop {
        let tag = geoset
            .get(pos..pos + TAG_SIZE)
            .ok_or_else(|| invalid("geoset ends before its texture coordinates".to_owned()))?;
        let count = read_len(geoset, pos + TAG_SIZE)?;
        let data_start = pos + CHUNK_HEADER;

        if tag == UV_SETS {
            out.extend_from_slice(geoset);
            return Ok(());
        } else if tag == TANGENTS {
            pos = data_start + count * TANGENT_SIZE;
        } else if tag == SKIN {
            if is_tag_at(geoset, data_start + count, AFTER_SKIN) {
                pos = data_start + count;
            } else if is_tag_at(
                geoset,
                data_start + count * WIDE_SKIN_VALUE_SIZE,
                AFTER_SKIN,
            ) {
                let wide_end = data_start + count * WIDE_SKIN_VALUE_SIZE;
                let narrow = narrow_values(&geoset[data_start..wide_end])?;
                let size = chunk_size(geoset.len() - (wide_end - data_start) + narrow.len())?;

                out.extend_from_slice(&size.to_le_bytes());
                out.extend_from_slice(&geoset[U32_SIZE..data_start]);
                out.extend_from_slice(&narrow);
                out.extend_from_slice(&geoset[wide_end..]);
                return Ok(());
            } else {
                return Err(invalid("skin weights have an unknown layout".to_owned()));
            }
        } else {
            return Err(invalid(format!(
                "unknown geoset chunk {}",
                tag.escape_ascii()
            )));
        }
    }
}

fn is_tag_at(data: &[u8], pos: usize, tags: &[&[u8; 4]]) -> bool {
    data.get(pos..pos + TAG_SIZE)
        .is_some_and(|found| tags.iter().any(|tag| found == tag.as_slice()))
}

/// Little-endian 16-bit values to bytes. Bone indices and weights fit a byte
/// in every game model; a larger value means the layout guess was wrong.
fn narrow_values(wide: &[u8]) -> Result<Vec<u8>> {
    let (pairs, _) = wide.as_chunks::<WIDE_SKIN_VALUE_SIZE>();

    pairs
        .iter()
        .map(|&pair| {
            let value = u16::from_le_bytes(pair);
            u8::try_from(value)
                .map_err(|_| invalid(format!("skin value {value} does not fit a byte")))
        })
        .collect()
}

/// Texture entries: replaceable ID, a 260-byte path, flags.
const TEXTURES_CHUNK: &[u8; 4] = b"TEXS";
const TEXTURE_ENTRY_SIZE: usize = 268;
const TEXTURE_PATH_SIZE: usize = 260;

/// The texture files a model uses, as written in it. Replaceable textures
/// (team color, glow) have no path and are left out. A file that is not a
/// readable model has none.
pub fn texture_paths(data: &[u8]) -> Vec<String> {
    if data.get(..MAGIC.len()) != Some(MAGIC.as_slice()) {
        return Vec::new();
    }

    let mut paths = Vec::new();
    let mut pos = MAGIC.len();

    while let (Some(tag), Ok(size)) = (
        data.get(pos..pos + TAG_SIZE),
        read_len(data, pos + TAG_SIZE),
    ) {
        let body_start = pos + CHUNK_HEADER;
        let Some(body) = data.get(body_start..body_start + size) else {
            break;
        };

        if tag == TEXTURES_CHUNK {
            let (entries, _) = body.as_chunks::<TEXTURE_ENTRY_SIZE>();

            for entry in entries {
                let path = &entry[U32_SIZE..U32_SIZE + TEXTURE_PATH_SIZE];
                let len = path.iter().position(|b| *b == 0).unwrap_or(path.len());
                let text = String::from_utf8_lossy(&path[..len]).into_owned();

                if !text.is_empty() && !paths.contains(&text) {
                    paths.push(text);
                }
            }
        }

        pos = body_start + size;
    }

    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    const REFORGED_VERSION: u32 = 1000;

    fn chunk(tag: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut out = tag.to_vec();

        out.extend_from_slice(&u32::try_from(body.len()).unwrap().to_le_bytes());
        out.extend_from_slice(body);
        out
    }

    fn model(chunks: &[Vec<u8>]) -> Vec<u8> {
        let mut out = MAGIC.to_vec();

        out.extend(chunk(VERSION_CHUNK, &REFORGED_VERSION.to_le_bytes()));
        for c in chunks {
            out.extend_from_slice(c);
        }

        out
    }

    fn counted(tag: &[u8; 4], count: u32, data: &[u8]) -> Vec<u8> {
        let mut out = tag.to_vec();

        out.extend_from_slice(&count.to_le_bytes());
        out.extend_from_slice(data);
        out
    }

    /// A one-vertex geoset whose skin chunk holds `skin` (`count` values).
    fn geoset(skin: &[u8], count: u32) -> Vec<u8> {
        let mut body = Vec::new();

        body.extend(counted(b"VRTX", 1, &[0; 12]));
        body.extend(counted(b"NRMS", 1, &[0; 12]));
        body.extend(counted(b"PTYP", 1, &4u32.to_le_bytes()));
        body.extend(counted(b"PCNT", 1, &[3, 0, 0, 0]));
        body.extend(counted(b"PVTX", 0, &[]));
        body.extend(counted(b"GNDX", 1, &[0]));
        body.extend(counted(b"MTGC", 1, &[1, 0, 0, 0]));
        body.extend(counted(b"MATS", 1, &[0; 4]));
        body.extend_from_slice(
            &[0; GEOSET_FIELDS_SIZE + U32_SIZE + GEOSET_NAME_SIZE + EXTENT_SIZE],
        );
        body.extend_from_slice(&0u32.to_le_bytes());
        body.extend(counted(TANGENTS, 1, &[7; TANGENT_SIZE]));
        body.extend(counted(SKIN, count, skin));
        body.extend(counted(UV_SETS, 1, &[]));
        body.extend(counted(b"UVBS", 1, &[9; 8]));

        let mut out = u32::try_from(body.len() + U32_SIZE)
            .unwrap()
            .to_le_bytes()
            .to_vec();

        out.extend(body);
        out
    }

    #[test]
    fn removes_only_the_skipped_chunks() {
        let original = model(&[
            chunk(b"SEQS", &[1, 2]),
            chunk(b"CAMS", &[9; 10]),
            chunk(b"TEXS", &[5, 6]),
        ]);

        let copy = preview_copy(&original).unwrap();

        assert_eq!(
            copy,
            model(&[chunk(b"SEQS", &[1, 2]), chunk(b"TEXS", &[5, 6])])
        );
    }

    #[test]
    fn narrows_wide_skin_weights() {
        let narrow = [5, 73, 72, 0, 197, 45, 13, 0];
        let wide: Vec<u8> = narrow.iter().flat_map(|&v| [v, 0]).collect();
        let wide_model = model(&[chunk(
            GEOSETS_CHUNK,
            &[geoset(&wide, 8), geoset(&wide, 8)].concat(),
        )]);
        let narrow_model = model(&[chunk(
            GEOSETS_CHUNK,
            &[geoset(&narrow, 8), geoset(&narrow, 8)].concat(),
        )]);

        assert_eq!(preview_copy(&wide_model).unwrap(), narrow_model);
        assert_eq!(preview_copy(&narrow_model).unwrap(), narrow_model);
    }

    #[test]
    fn rejects_other_files_and_broken_models() {
        assert!(preview_copy(b"DDS stuff").is_err());

        let mut cut = model(&[chunk(b"TEXS", &[5, 6, 7, 8])]);

        cut.truncate(cut.len() - 2);
        assert!(preview_copy(&cut).is_err());

        let too_big: Vec<u8> = [1u8, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].to_vec();

        assert!(preview_copy(&model(&[chunk(GEOSETS_CHUNK, &geoset(&too_big, 8))])).is_err());
    }

    fn texture(path: &str) -> Vec<u8> {
        let mut entry = vec![0; TEXTURE_ENTRY_SIZE];

        entry[U32_SIZE..U32_SIZE + path.len()].copy_from_slice(path.as_bytes());
        entry
    }

    #[test]
    fn lists_texture_paths_once() {
        let mut model = MAGIC.to_vec();

        model.extend(chunk(b"VERS", &800u32.to_le_bytes()));
        let textures = [
            texture("Hero.blp"),
            texture(""),
            texture("Textures\\Glow.blp"),
            texture("Hero.blp"),
        ]
        .concat();

        model.extend(chunk(TEXTURES_CHUNK, &textures));

        assert_eq!(texture_paths(&model), ["Hero.blp", "Textures\\Glow.blp"]);
        assert!(texture_paths(b"not a model").is_empty());
    }

    #[test]
    fn model_paths_get_the_binary_extension() {
        assert_eq!(
            model_file("units\\human\\Footman\\Footman"),
            "units\\human\\Footman\\Footman.mdx"
        );
        assert_eq!(model_file("Abilities\\Spell.mdl"), "Abilities\\Spell.mdx");
        assert_eq!(model_file("a.MDX"), "a.MDX");
        assert_eq!(
            model_file("Doodads/Empty/Empty.ma"),
            "Doodads/Empty/Empty.mdx"
        );
        assert_eq!(
            model_file("Units\\Mr.Rivers\\Rivers"),
            "Units\\Mr.Rivers\\Rivers.mdx"
        );
    }
}
