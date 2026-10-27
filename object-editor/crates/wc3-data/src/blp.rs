//! BLP1 textures, the format maps import icons and model textures in.
//!
//! A BLP1 file holds a 156-byte header (size, alpha depth, mip offsets and
//! sizes), then either a shared JPEG header and one JPEG per mip level, or a
//! 256-color palette and one block of palette indices plus alpha per mip level.
//! Only the top mip level is decoded.

use crate::icon::Image;
use crate::{Error, Result};

pub const MAGIC: &[u8; 4] = b"BLP1";
const COMPRESSION_OFFSET: usize = 4;
const ALPHA_BITS_OFFSET: usize = 8;
const WIDTH_OFFSET: usize = 12;
const HEIGHT_OFFSET: usize = 16;
const MIP_OFFSETS_OFFSET: usize = 28;
const MIP_SIZES_OFFSET: usize = 92;
const HEADER_SIZE: usize = 156;
const JPEG_COMPRESSION: u32 = 0;
const PALETTE_COMPRESSION: u32 = 1;
const PALETTE_SIZE: usize = 256;
const U32_SIZE: usize = 4;
const CHANNELS: usize = 4;
const OPAQUE: u8 = 255;
const BITS_PER_BYTE: usize = 8;
const NIBBLE_MASK: u8 = 0x0F;
const NIBBLE_SCALE: u8 = 17;
const MAX_SIDE: u32 = 8192;

pub fn decode_blp(data: &[u8]) -> Result<Image> {
    if data.len() < HEADER_SIZE || &data[..MAGIC.len()] != MAGIC {
        return Err(invalid("not a BLP1 file"));
    }

    let width = read_u32(data, WIDTH_OFFSET)?;
    let height = read_u32(data, HEIGHT_OFFSET)?;

    if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE {
        return Err(invalid(&format!("bad size {width}x{height}")));
    }

    let alpha_bits = read_u32(data, ALPHA_BITS_OFFSET)?;
    let offset = read_u32(data, MIP_OFFSETS_OFFSET)? as usize;
    let size = read_u32(data, MIP_SIZES_OFFSET)? as usize;
    let mip = data
        .get(offset..offset.saturating_add(size))
        .ok_or_else(|| invalid("top mip level runs past the end"))?;
    let (w, h) = (width as usize, height as usize);

    let rgba = match read_u32(data, COMPRESSION_OFFSET)? {
        JPEG_COMPRESSION => decode_jpeg(data, mip, w, h, alpha_bits != 0)?,
        PALETTE_COMPRESSION => decode_paletted(data, mip, w, h, alpha_bits)?,
        other => return Err(invalid(&format!("unknown compression {other}"))),
    };

    Ok(Image {
        width,
        height,
        rgba,
    })
}

fn invalid(reason: &str) -> Error {
    Error::Image(format!("BLP: {reason}"))
}

fn read_u32(data: &[u8], offset: usize) -> Result<u32> {
    let bytes = data
        .get(offset..offset + U32_SIZE)
        .ok_or_else(|| invalid(&format!("cut off at byte {offset}")))?;
    let mut word = [0u8; U32_SIZE];

    word.copy_from_slice(bytes);
    Ok(u32::from_le_bytes(word))
}

/// A mip level is the shared JPEG header followed by that level's data. The
/// JPEG holds blue, green, red and alpha as four plain channels.
fn decode_jpeg(data: &[u8], mip: &[u8], w: usize, h: usize, alpha: bool) -> Result<Vec<u8>> {
    let header_size = read_u32(data, HEADER_SIZE)? as usize;
    let header_start = HEADER_SIZE + U32_SIZE;
    let header = data
        .get(header_start..header_start + header_size)
        .ok_or_else(|| invalid("JPEG header runs past the end"))?;
    let jpeg = [header, mip].concat();

    // Decoded as CMYK, which inverts every channel (the Adobe convention);
    // inverting again below gives the stored values. jpeg-decoder's "no
    // transform" mode would not invert, but it returns each row's channels
    // one after another instead of interleaved.
    let mut decoder = jpeg_decoder::Decoder::new(jpeg.as_slice());

    decoder.set_color_transform(jpeg_decoder::ColorTransform::CMYK);
    let pixels = decoder
        .decode()
        .map_err(|e| invalid(&format!("JPEG: {e}")))?;
    let info = decoder.info().ok_or_else(|| invalid("JPEG has no image"))?;

    if info.pixel_format != jpeg_decoder::PixelFormat::CMYK32 {
        return Err(invalid(&format!(
            "JPEG has {:?} pixels, not four channels",
            info.pixel_format
        )));
    }

    // The JPEG can be larger than the texture; the texture is its top-left part.
    let jpeg_width = usize::from(info.width);

    if jpeg_width < w || usize::from(info.height) < h {
        return Err(invalid(&format!(
            "JPEG is {}x{}, smaller than the texture",
            info.width, info.height
        )));
    }

    let mut rgba = Vec::with_capacity(w * h * CHANNELS);

    for row in pixels.chunks_exact(jpeg_width * CHANNELS).take(h) {
        let (row_pixels, _) = row.as_chunks::<CHANNELS>();

        for inverted in row_pixels.iter().take(w) {
            let [b, g, r, a] = inverted.map(|c| OPAQUE - c);

            rgba.extend_from_slice(&[r, g, b, if alpha { a } else { OPAQUE }]);
        }
    }

    Ok(rgba)
}

/// Palette indices, one byte per pixel, then the alpha values packed at
/// `alpha_bits` per pixel, lowest bits first.
fn decode_paletted(
    data: &[u8],
    mip: &[u8],
    w: usize,
    h: usize,
    alpha_bits: u32,
) -> Result<Vec<u8>> {
    let palette = data
        .get(HEADER_SIZE..HEADER_SIZE + PALETTE_SIZE * CHANNELS)
        .ok_or_else(|| invalid("palette runs past the end"))?;
    let count = w * h;
    let alpha_size = match alpha_bits {
        0 => 0,
        1 | 4 | 8 => (count * alpha_bits as usize).div_ceil(BITS_PER_BYTE),
        other => return Err(invalid(&format!("unsupported alpha depth {other}"))),
    };

    if mip.len() < count + alpha_size {
        return Err(invalid(&format!(
            "top mip level has {} bytes, {} needed",
            mip.len(),
            count + alpha_size
        )));
    }

    let (indices, alphas) = mip.split_at(count);
    let mut rgba = Vec::with_capacity(count * CHANNELS);

    for (i, &index) in indices.iter().enumerate() {
        let entry = usize::from(index) * CHANNELS;
        let bgr = &palette[entry..entry + 3];

        rgba.extend_from_slice(&[bgr[2], bgr[1], bgr[0], alpha_at(alphas, i, alpha_bits)]);
    }

    Ok(rgba)
}

fn alpha_at(alphas: &[u8], i: usize, alpha_bits: u32) -> u8 {
    match alpha_bits {
        1 => {
            let set = alphas[i / BITS_PER_BYTE] >> (i % BITS_PER_BYTE) & 1;

            set * OPAQUE
        }
        4 => {
            let byte = alphas[i / 2];
            let nibble = if i.is_multiple_of(2) { byte } else { byte >> 4 };

            (nibble & NIBBLE_MASK) * NIBBLE_SCALE
        }
        8 => alphas[i],
        _ => OPAQUE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paletted(
        width: u32,
        height: u32,
        alpha_bits: u32,
        palette: &[[u8; 4]],
        mip: &[u8],
    ) -> Vec<u8> {
        let mut data = vec![0u8; HEADER_SIZE];

        data[..4].copy_from_slice(MAGIC);
        data[COMPRESSION_OFFSET..][..4].copy_from_slice(&PALETTE_COMPRESSION.to_le_bytes());
        data[ALPHA_BITS_OFFSET..][..4].copy_from_slice(&alpha_bits.to_le_bytes());
        data[WIDTH_OFFSET..][..4].copy_from_slice(&width.to_le_bytes());
        data[HEIGHT_OFFSET..][..4].copy_from_slice(&height.to_le_bytes());
        let offset = u32::try_from(HEADER_SIZE + PALETTE_SIZE * CHANNELS).unwrap();

        data[MIP_OFFSETS_OFFSET..][..4].copy_from_slice(&offset.to_le_bytes());
        let size = u32::try_from(mip.len()).unwrap();

        data[MIP_SIZES_OFFSET..][..4].copy_from_slice(&size.to_le_bytes());

        let mut colors = vec![0u8; PALETTE_SIZE * CHANNELS];

        for (i, color) in palette.iter().enumerate() {
            colors[i * CHANNELS..][..CHANNELS].copy_from_slice(color);
        }

        data.extend(colors);
        data.extend_from_slice(mip);
        data
    }

    const BLUE_BGRA: [u8; 4] = [255, 0, 0, 0];
    const RED_BGRA: [u8; 4] = [0, 0, 255, 0];

    #[test]
    fn decodes_paletted_textures_with_byte_alpha() {
        let data = paletted(2, 1, 8, &[BLUE_BGRA, RED_BGRA], &[1, 0, 200, 7]);

        let image = decode_blp(&data).unwrap();

        assert_eq!((image.width, image.height), (2, 1));
        assert_eq!(image.rgba, [255, 0, 0, 200, 0, 0, 255, 7]);
    }

    #[test]
    fn unpacks_one_and_four_bit_alpha() {
        let one_bit = paletted(3, 1, 1, &[RED_BGRA], &[0, 0, 0, 0b101]);
        let alphas: Vec<u8> = decode_blp(&one_bit)
            .unwrap()
            .rgba
            .iter()
            .skip(3)
            .step_by(4)
            .copied()
            .collect();

        assert_eq!(alphas, [255, 0, 255]);

        let four_bit = paletted(3, 1, 4, &[RED_BGRA], &[0, 0, 0, 0xF1, 0x08]);
        let alphas: Vec<u8> = decode_blp(&four_bit)
            .unwrap()
            .rgba
            .iter()
            .skip(3)
            .step_by(4)
            .copied()
            .collect();

        assert_eq!(alphas, [17, 255, 136]);
    }

    #[test]
    fn textures_without_alpha_are_opaque() {
        let data = paletted(1, 1, 0, &[RED_BGRA], &[0]);

        assert_eq!(decode_blp(&data).unwrap().rgba, [255, 0, 0, 255]);
    }

    /// A 16x8 four-channel baseline JPEG, written for this test: the left half
    /// stores (B, G, R, A) = (40, 80, 200, 255), the right half (200, 40, 80, 0).
    /// No color transform, as in BLP files.
    const TEST_JPEG: &[u8] = include_bytes!("../testdata/bgra.jpg");
    const START_OF_SCAN: [u8; 2] = [0xFF, 0xDA];
    const JPEG_TOLERANCE: u8 = 3;

    /// A JPEG BLP1 as the game's tools write one: the JPEG is split into the
    /// shared header (up to the scan) and the top mip level's data.
    fn jpeg_blp(width: u32, height: u32, alpha_bits: u32) -> Vec<u8> {
        let split = TEST_JPEG
            .windows(2)
            .position(|w| w == START_OF_SCAN)
            .unwrap();
        let (header, mip) = TEST_JPEG.split_at(split);

        let mut data = vec![0u8; HEADER_SIZE];

        data[..4].copy_from_slice(MAGIC);
        data[ALPHA_BITS_OFFSET..][..4].copy_from_slice(&alpha_bits.to_le_bytes());
        data[WIDTH_OFFSET..][..4].copy_from_slice(&width.to_le_bytes());
        data[HEIGHT_OFFSET..][..4].copy_from_slice(&height.to_le_bytes());
        let offset = u32::try_from(HEADER_SIZE + U32_SIZE + header.len()).unwrap();

        data[MIP_OFFSETS_OFFSET..][..4].copy_from_slice(&offset.to_le_bytes());
        let size = u32::try_from(mip.len()).unwrap();

        data[MIP_SIZES_OFFSET..][..4].copy_from_slice(&size.to_le_bytes());
        data.extend_from_slice(&u32::try_from(header.len()).unwrap().to_le_bytes());
        data.extend_from_slice(header);
        data.extend_from_slice(mip);
        data
    }

    fn assert_near(actual: &[u8], expected: &[u8]) {
        assert_eq!(actual.len(), expected.len());
        for (a, e) in actual.iter().zip(expected) {
            assert!(
                a.abs_diff(*e) <= JPEG_TOLERANCE,
                "{actual:?} is not near {expected:?}"
            );
        }
    }

    #[test]
    fn decodes_jpeg_textures_as_bgra() {
        let image = decode_blp(&jpeg_blp(16, 8, 8)).unwrap();

        assert_eq!((image.width, image.height), (16, 8));
        assert_near(&image.rgba[..4], &[200, 80, 40, 255]);
        let right = 15 * CHANNELS;

        assert_near(&image.rgba[right..right + 4], &[80, 40, 200, 0]);
    }

    #[test]
    fn jpeg_textures_take_the_top_left_part_and_drop_unused_alpha() {
        let image = decode_blp(&jpeg_blp(4, 2, 0)).unwrap();

        assert_eq!(image.rgba.len(), 4 * 2 * CHANNELS);
        assert_near(&image.rgba[..4], &[200, 80, 40, 255]);
        assert!(image.rgba.iter().skip(3).step_by(4).all(|&a| a == OPAQUE));
    }

    #[test]
    fn rejects_broken_textures() {
        assert!(decode_blp(b"BLP2").is_err());
        assert!(decode_blp(&paletted(0, 1, 8, &[], &[])).is_err());
        assert!(decode_blp(&paletted(4, 4, 8, &[], &[0; 20])).is_err());
        assert!(decode_blp(&paletted(1, 1, 3, &[], &[0, 0])).is_err());
        assert!(decode_blp(&jpeg_blp(32, 8, 8)).is_err());

        let mut cut = jpeg_blp(16, 8, 8);

        cut.truncate(cut.len() - 10);
        assert!(decode_blp(&cut).is_err());
    }
}
