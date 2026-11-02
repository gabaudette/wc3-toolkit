use std::collections::HashMap;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::Serialize;
use tauri::State;
use wc3_data::game::{ArtMode, GameFiles, decode_text};
use wc3_data::icon::{Image, load_texture};
use wc3_data::slk::Slk;
use wc3_map::terrain::{Corner, Terrain};

use crate::state::{AppState, NO_MAP};

const TILE_TABLE: &str = "terrainart\\terrain.slk";
const TILE_ID: &str = "tileID";
const TILE_DIR: &str = "dir";
const TILE_FILE: &str = "file";
const TEXTURE_EXTENSION: &str = ".blp";
const FALLBACK_COLOR: [f32; 3] = [96.0, 112.0, 72.0];
const WATER_COLOR: [f32; 3] = [40.0, 80.0, 130.0];
const DEEP_WATER: f32 = 128.0;
const SLOPE_LIGHT: f32 = 0.004;
const MIN_LIGHT: f32 = 0.55;
const MAX_LIGHT: f32 = 1.35;
const CLIFF_SHADE: f32 = 0.7;
const BOUNDARY_SHADE: f32 = 0.35;
const OPAQUE: u8 = 255;
const PNG_URL: &str = "data:image/png;base64,";
const NO_TERRAIN: &str = "This map has no terrain file (war3map.w3e), so it has no map view.";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapView {
    pub bounds: [f32; 4],
    pub width: usize,
    pub height: usize,
    pub image: String,
}

fn tile_paths(files: &GameFiles) -> HashMap<String, String> {
    let Ok(text) = files
        .read_base(TILE_TABLE)
        .and_then(|bytes| decode_text(bytes, TILE_TABLE))
    else {
        return HashMap::new();
    };
    let Ok(table) = Slk::parse(&text) else {
        return HashMap::new();
    };

    table
        .records()
        .filter_map(|record| {
            let id = record.get(TILE_ID)?;
            let dir = record.get(TILE_DIR)?;
            let file = record.get(TILE_FILE)?;

            Some((
                (*id).to_owned(),
                format!("{dir}\\{file}{TEXTURE_EXTENSION}"),
            ))
        })
        .collect()
}

fn average(image: &Image) -> [f32; 3] {
    let mut sum = [0.0f64; 3];
    let mut count = 0.0f64;

    for pixel in image.rgba.as_chunks::<4>().0 {
        for (total, channel) in sum.iter_mut().zip(pixel) {
            *total += f64::from(*channel);
        }

        count += 1.0;
    }

    if count == 0.0 {
        return FALLBACK_COLOR;
    }

    sum.map(|s| (s / count) as f32)
}

fn tile_colors(
    files: &GameFiles,
    terrain: &Terrain,
    cache: &mut HashMap<String, [f32; 3]>,
) -> Vec<[f32; 3]> {
    let mut paths: Option<HashMap<String, String>> = None;

    terrain
        .ground_tiles
        .iter()
        .map(|tile| {
            let id = tile.to_string();

            if let Some(color) = cache.get(&id) {
                return *color;
            }

            let paths = paths.get_or_insert_with(|| tile_paths(files));
            let color = paths
                .get(&id)
                .and_then(|path| load_texture(files, path, ArtMode::Sd).ok())
                .map_or(FALLBACK_COLOR, |image| average(&image));

            cache.insert(id, color);
            color
        })
        .collect()
}

fn shade(color: [f32; 3], light: f32) -> [f32; 3] {
    color.map(|c| c * light)
}

fn mix(a: [f32; 3], b: [f32; 3], amount: f32) -> [f32; 3] {
    [0, 1, 2].map(|i| a[i] * (1.0 - amount) + b[i] * amount)
}

fn render(terrain: &Terrain, colors: &[[f32; 3]]) -> Image {
    let (width, height) = (terrain.width, terrain.height);
    let mut rgba = Vec::with_capacity(width * height * 4);
    let at = |x: usize, y: usize| terrain.corner(x, y).map_or(0.0, |c| c.height);

    for row in (0..height).rev() {
        for column in 0..width {
            let Some(corner) = terrain.corner(column, row) else {
                continue;
            };
            let left = at(column.saturating_sub(1), row);
            let right = at((column + 1).min(width - 1), row);
            let below = at(column, row.saturating_sub(1));
            let above = at(column, (row + 1).min(height - 1));
            let light = (1.0 + ((left - right) + (above - below)) * SLOPE_LIGHT)
                .clamp(MIN_LIGHT, MAX_LIGHT);
            let base = colors
                .get(usize::from(corner.texture))
                .copied()
                .unwrap_or(FALLBACK_COLOR);
            let mut color = shade(base, light);
            let neighbours: [Option<&Corner>; 2] = [
                terrain.corner((column + 1).min(width - 1), row),
                terrain.corner(column, (row + 1).min(height - 1)),
            ];

            if neighbours
                .iter()
                .flatten()
                .any(|n| n.layer != corner.layer && !n.ramp && !corner.ramp)
            {
                color = shade(color, CLIFF_SHADE);
            }

            if corner.water && corner.water_level > corner.height {
                let depth = ((corner.water_level - corner.height) / DEEP_WATER).clamp(0.4, 0.9);

                color = mix(color, WATER_COLOR, depth);
            }

            if corner.boundary || corner.edge {
                color = shade(color, BOUNDARY_SHADE);
            }

            for channel in color {
                rgba.push(channel.clamp(0.0, 255.0) as u8);
            }

            rgba.push(OPAQUE);
        }
    }

    Image {
        width: u32::try_from(width).unwrap_or_default(),
        height: u32::try_from(height).unwrap_or_default(),
        rgba,
    }
}

#[tauri::command]
pub fn map_view(state: State<'_, AppState>) -> Result<MapView, String> {
    let (terrain, files) = state.with(|session| {
        let map = session.map.as_ref().ok_or(NO_MAP)?;
        let terrain = map.terrain.clone().ok_or(NO_TERRAIN)?;

        Ok((terrain, session.files.clone()))
    })?;
    let mut cache = state.with(|session| Ok(std::mem::take(&mut session.tile_colors)))?;
    let colors = tile_colors(&files, &terrain, &mut cache);
    let png = render(&terrain, &colors)
        .to_png()
        .map_err(|e| e.to_string())?;

    state.with(|session| {
        session.tile_colors = cache;
        Ok(())
    })?;

    Ok(MapView {
        bounds: terrain.bounds(),
        width: terrain.width,
        height: terrain.height,
        image: format!("{PNG_URL}{}", BASE64.encode(png)),
    })
}
