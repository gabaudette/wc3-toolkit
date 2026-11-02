use crate::binary::Reader;
use crate::{FourCC, Map, Result};

pub const TERRAIN_FILE: &str = "war3map.w3e";

const MAGIC: &[u8; 4] = b"W3E!";
// Version 12 (Reforged) widens the texture and flags field to 16 bits.
const WIDE_TEXTURE_VERSION: u32 = 12;
const HEIGHT_ZERO: f32 = 8192.0;
const HEIGHT_SCALE: f32 = 4.0;
const WATER_MASK: u16 = 0x3FFF;
const EDGE_FLAG: u16 = 0x4000;
pub const TILE_SIZE: f32 = 128.0;
const LAYER_ZERO: i32 = 2;
const LAYER_HEIGHT: f32 = 128.0;
const MIN_CORNER_BYTES: usize = 7;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Corner {
    /// Ground height in game units, cliff layers included.
    pub height: f32,
    pub water_level: f32,
    pub texture: u8,
    pub ramp: bool,
    pub blight: bool,
    pub water: bool,
    pub boundary: bool,
    pub edge: bool,
    pub layer: u8,
}

/// `war3map.w3e`: the terrain grid, read for the map view.
#[derive(Debug, Clone, PartialEq)]
pub struct Terrain {
    pub tileset: u8,
    pub ground_tiles: Vec<FourCC>,
    pub width: usize,
    pub height: usize,
    pub offset_x: f32,
    pub offset_y: f32,
    pub corners: Vec<Corner>,
}

impl Terrain {
    pub fn load(map: &Map) -> Result<Option<Self>> {
        map.read_file(TERRAIN_FILE)?
            .map(|bytes| Self::parse(&bytes))
            .transpose()
    }

    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut r = Reader::new(TERRAIN_FILE, data);

        if r.take(4)? != MAGIC {
            return Err(r.damaged("not a terrain file"));
        }

        let version = r.u32()?;
        let tileset = r.u8()?;

        r.u32()?;
        let n = r.count(4)?;
        let mut ground_tiles = Vec::with_capacity(n);

        for _ in 0..n {
            ground_tiles.push(r.fourcc()?);
        }

        let cliffs = r.count(4)?;

        r.take(cliffs * 4)?;
        let width = r.u32()? as usize;
        let height = r.u32()? as usize;
        let offset_x = r.f32()?;
        let offset_y = r.f32()?;
        let total = width.saturating_mul(height);

        if total.saturating_mul(MIN_CORNER_BYTES) > r.remaining() {
            return Err(r.damaged("the terrain is larger than the file"));
        }

        let mut corners = Vec::with_capacity(total);

        for _ in 0..total {
            let ground = r.take(2)?;
            let ground = u16::from_le_bytes([ground[0], ground[1]]);
            let water = r.take(2)?;
            let water = u16::from_le_bytes([water[0], water[1]]);
            let (texture, ramp, blight, under_water, boundary) = if version >= WIDE_TEXTURE_VERSION
            {
                let bytes = r.take(2)?;
                let value = u16::from_le_bytes([bytes[0], bytes[1]]);

                (
                    (value & 0x3F) as u8,
                    value & 0x40 != 0,
                    value & 0x80 != 0,
                    value & 0x100 != 0,
                    value & 0x200 != 0,
                )
            } else {
                let value = r.u8()?;

                (
                    value & 0x0F,
                    value & 0x10 != 0,
                    value & 0x20 != 0,
                    value & 0x40 != 0,
                    value & 0x80 != 0,
                )
            };

            r.u8()?;
            let misc = r.u8()?;
            let layer = misc & 0x0F;

            corners.push(Corner {
                height: (f32::from(ground) - HEIGHT_ZERO) / HEIGHT_SCALE
                    + (i32::from(layer) - LAYER_ZERO) as f32 * LAYER_HEIGHT,
                water_level: (f32::from(water & WATER_MASK) - HEIGHT_ZERO) / HEIGHT_SCALE,
                texture,
                ramp,
                blight,
                water: under_water,
                boundary,
                edge: water & EDGE_FLAG != 0,
                layer,
            });
        }

        Ok(Self {
            tileset,
            ground_tiles,
            width,
            height,
            offset_x,
            offset_y,
            corners,
        })
    }

    /// The map's extent in game units: left, bottom, right, top.
    pub fn bounds(&self) -> [f32; 4] {
        let span = |n: usize| n.saturating_sub(1) as f32 * TILE_SIZE;

        [
            self.offset_x,
            self.offset_y,
            self.offset_x + span(self.width),
            self.offset_y + span(self.height),
        ]
    }

    pub fn corner(&self, x: usize, y: usize) -> Option<&Corner> {
        self.corners.get(y * self.width + x)
    }

    /// Ground height under a point, from the nearest corner.
    pub fn height_at(&self, x: f32, y: f32) -> f32 {
        let column = ((x - self.offset_x) / TILE_SIZE).round().max(0.0) as usize;
        let row = ((y - self.offset_y) / TILE_SIZE).round().max(0.0) as usize;

        self.corner(
            column.min(self.width.saturating_sub(1)),
            row.min(self.height.saturating_sub(1)),
        )
        .map_or(0.0, |c| c.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

    const CLASSIC_VERSION: u32 = 11;
    const GROUND_ZERO: u16 = 0x2000;
    const WATER_FLAG: u8 = 0x40;

    fn corner(ground: u16, flags: u8, layer: u8) -> Vec<u8> {
        let mut out = ground.to_le_bytes().to_vec();

        out.extend_from_slice(&GROUND_ZERO.to_le_bytes());
        out.extend_from_slice(&[flags | 1, 0, layer]);
        out
    }

    #[test]
    fn reads_heights_layers_and_bounds() -> TestResult {
        let mut data = MAGIC.to_vec();

        data.extend_from_slice(&CLASSIC_VERSION.to_le_bytes());
        data.push(b'L');
        data.extend_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(&2u32.to_le_bytes());
        data.extend_from_slice(b"LdrtLgrs");
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&2u32.to_le_bytes());
        data.extend_from_slice(&2u32.to_le_bytes());
        data.extend_from_slice(&(-256.0f32).to_le_bytes());
        data.extend_from_slice(&(-128.0f32).to_le_bytes());
        data.extend(corner(GROUND_ZERO, 0, 2));
        data.extend(corner(GROUND_ZERO + 400, 0, 2));
        data.extend(corner(GROUND_ZERO, WATER_FLAG, 1));
        data.extend(corner(GROUND_ZERO, 0, 3));

        let terrain = Terrain::parse(&data)?;

        assert_eq!(
            terrain.ground_tiles,
            vec![FourCC(*b"Ldrt"), FourCC(*b"Lgrs")]
        );
        assert_eq!(terrain.bounds(), [-256.0, -128.0, -128.0, 0.0]);
        assert_eq!(terrain.height_at(-128.0, -128.0), 100.0);
        assert_eq!(terrain.height_at(-256.0, 0.0), -128.0);
        assert_eq!(terrain.height_at(-128.0, 0.0), 128.0);
        assert!(
            terrain
                .corner(0, 1)
                .is_some_and(|c| c.water && c.texture == 1)
        );
        Ok(())
    }
}
