//! Icons and other textures: decoding the game's DDS images to RGBA.
//!
//! Data files and models name textures `*.blp` or `*.tif`, but Reforged ships
//! them as `.dds` (DXT1, DXT5, or ATI2 for normal maps) in its SD and HD layers.

use crate::game::{ArtMode, GameFiles};
use crate::{Error, Result};

const DDS_MAGIC: &[u8; 4] = b"DDS ";
const DDS_HEADER_SIZE: usize = 128;
const HEIGHT_OFFSET: usize = 12;
const WIDTH_OFFSET: usize = 16;
const FOURCC_OFFSET: usize = 84;
const RGBA_CHANNELS: usize = 4;

/// A decoded image, 8-bit RGBA, rows top to bottom.
#[derive(Debug, Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Bytes before the pixels in [`Image::to_raw`]: width and height, little-endian.
pub const RAW_HEADER_SIZE: usize = 8;

impl Image {
    /// Width and height (u32 little-endian each), then the RGBA pixels. Faster
    /// than PNG for large textures that are only passed to a renderer.
    pub fn to_raw(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(RAW_HEADER_SIZE + self.rgba.len());
        out.extend_from_slice(&self.width.to_le_bytes());
        out.extend_from_slice(&self.height.to_le_bytes());
        out.extend_from_slice(&self.rgba);
        out
    }

    pub fn to_png(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        let mut encoder = png::Encoder::new(&mut out, self.width, self.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header().map_err(png_error)?;
        writer.write_image_data(&self.rgba).map_err(png_error)?;
        writer.finish().map_err(png_error)?;
        Ok(out)
    }
}

fn png_error(e: png::EncodingError) -> Error {
    Error::Image(format!("PNG encoding failed: {e}"))
}

/// Loads a texture by the path the game data uses, e.g.
/// `ReplaceableTextures\CommandButtons\BTNHeroPaladin.blp`.
pub fn load_texture(files: &GameFiles, path: &str, mode: ArtMode) -> Result<Image> {
    let dds_path = dds_file(path);

    let data = files.read_art(&dds_path, mode)?;
    decode_dds(&data).map_err(|e| e.in_file(dds_path))
}

/// The `.dds` file Reforged ships for a texture path: `x.blp`, `x.tif` and `x` → `x.dds`.
pub fn dds_file(path: &str) -> String {
    let normalized = path.replace('/', "\\");
    let file_start = normalized.rfind('\\').map_or(0, |i| i + 1);
    let stem = match normalized[file_start..].rfind('.') {
        Some(dot) => &normalized[..file_start + dot],
        None => &normalized,
    };

    format!("{stem}.dds")
}

/// Decodes the top mip level of a DXT1, DXT3, DXT5, ATI1 or ATI2 DDS file.
pub fn decode_dds(data: &[u8]) -> Result<Image> {
    if data.len() < DDS_HEADER_SIZE || &data[..DDS_MAGIC.len()] != DDS_MAGIC {
        return Err(Error::Image("not a DDS file".to_owned()));
    }

    let height = read_u32(data, HEIGHT_OFFSET);
    let width = read_u32(data, WIDTH_OFFSET);
    let fourcc = &data[FOURCC_OFFSET..FOURCC_OFFSET + 4];
    let format = match fourcc {
        b"DXT1" => texpresso::Format::Bc1,
        b"DXT3" => texpresso::Format::Bc2,
        b"DXT5" => texpresso::Format::Bc3,
        b"ATI1" | b"BC4U" => texpresso::Format::Bc4,
        b"ATI2" | b"BC5U" => texpresso::Format::Bc5,
        other => {
            return Err(Error::Image(format!(
                "unsupported DDS format {:?}",
                String::from_utf8_lossy(other)
            )));
        }
    };

    let (w, h) = (width as usize, height as usize);
    let compressed = &data[DDS_HEADER_SIZE..];
    let needed = format.compressed_size(w, h);
    if compressed.len() < needed {
        return Err(Error::Image(format!(
            "DDS data truncated: {} bytes, {needed} needed for {width}x{height}",
            compressed.len()
        )));
    }

    let mut rgba = vec![0u8; w * h * RGBA_CHANNELS];
    format.decompress(&compressed[..needed], w, h, &mut rgba);
    Ok(Image {
        width,
        height,
        rgba,
    })
}

fn read_u32(data: &[u8], offset: usize) -> u32 {
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(&data[offset..offset + 4]);
    u32::from_le_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dds_header(width: u32, height: u32, fourcc: &[u8; 4]) -> Vec<u8> {
        let mut data = vec![0u8; DDS_HEADER_SIZE];
        data[..4].copy_from_slice(DDS_MAGIC);
        data[HEIGHT_OFFSET..HEIGHT_OFFSET + 4].copy_from_slice(&height.to_le_bytes());
        data[WIDTH_OFFSET..WIDTH_OFFSET + 4].copy_from_slice(&width.to_le_bytes());
        data[FOURCC_OFFSET..FOURCC_OFFSET + 4].copy_from_slice(fourcc);
        data
    }

    #[test]
    fn decodes_a_solid_dxt1_block() {
        // One 4x4 BC1 block: both endpoints pure red (RGB565 0xF800), all indices 0.
        let mut data = dds_header(4, 4, b"DXT1");
        data.extend_from_slice(&[0x00, 0xF8, 0x00, 0xF8, 0, 0, 0, 0]);

        let image = decode_dds(&data).unwrap();

        assert_eq!((image.width, image.height), (4, 4));
        assert_eq!(&image.rgba[..4], &[255, 0, 0, 255]);
        assert!(image.to_png().unwrap().starts_with(b"\x89PNG"));
    }

    #[test]
    fn decodes_ati2_normal_maps() {
        // One 4x4 BC5 block: red and green both at 255 everywhere.
        let mut data = dds_header(4, 4, b"ATI2");
        data.extend_from_slice(&[255, 255, 0, 0, 0, 0, 0, 0, 255, 255, 0, 0, 0, 0, 0, 0]);

        let image = decode_dds(&data).unwrap();

        assert_eq!(&image.rgba[..2], &[255, 255]);
        assert_eq!(image.to_raw().len(), RAW_HEADER_SIZE + image.rgba.len());
    }

    #[test]
    fn texture_paths_map_to_dds_files() {
        assert_eq!(
            dds_file("Units/Human/HeroPaladin/Main_Diffuse.tif"),
            "Units\\Human\\HeroPaladin\\Main_Diffuse.dds"
        );
        assert_eq!(dds_file("Textures\\Black32.blp"), "Textures\\Black32.dds");
        assert_eq!(
            dds_file("Textures\\NoExtension"),
            "Textures\\NoExtension.dds"
        );
        assert_eq!(dds_file("Dir.v2\\File"), "Dir.v2\\File.dds");
    }

    #[test]
    fn rejects_unknown_formats_and_truncated_data() {
        assert!(decode_dds(b"nope").is_err());
        assert!(decode_dds(&dds_header(4, 4, b"BC7X")).is_err());
        assert!(decode_dds(&dds_header(64, 64, b"DXT5")).is_err());
    }
}
