//! TGA textures, as maps import them: true-color (24 or 32 bits) or grayscale
//! (8 bits), uncompressed or run-length encoded. Color-mapped TGA files are
//! not used for Warcraft III art and are rejected.

use crate::icon::Image;
use crate::{Error, Result};

const HEADER_SIZE: usize = 18;
const ID_LENGTH: usize = 0;
const COLOR_MAP_TYPE: usize = 1;
const IMAGE_TYPE: usize = 2;
const WIDTH: usize = 12;
const HEIGHT: usize = 14;
const BITS_PER_PIXEL: usize = 16;
const DESCRIPTOR: usize = 17;

const TRUE_COLOR: u8 = 2;
const GRAYSCALE: u8 = 3;
const RLE_TRUE_COLOR: u8 = 10;
const RLE_GRAYSCALE: u8 = 11;
/// Descriptor bit set when rows run top to bottom; otherwise the first row is the bottom one.
const TOP_DOWN: u8 = 0x20;
/// A packet header with this bit repeats one pixel; without it, raw pixels follow.
const RUN_PACKET: u8 = 0x80;
const PACKET_COUNT: u8 = 0x7f;
const RGBA_CHANNELS: usize = 4;
const OPAQUE: u8 = u8::MAX;

/// Whether the data looks like a TGA image this decoder reads. TGA files have
/// no magic number, so the header is checked field by field.
pub fn is_tga(data: &[u8]) -> bool {
    header(data).is_ok()
}

struct Header {
    width: usize,
    height: usize,
    /// Bytes per stored pixel: 1 (gray), 3 (BGR) or 4 (BGRA).
    pixel_bytes: usize,
    rle: bool,
    top_down: bool,
    /// Where the pixel data starts, after the image ID.
    start: usize,
}

fn header(data: &[u8]) -> Result<Header> {
    let invalid =
        |reason: &str| Error::Image(format!("not a TGA file this editor reads: {reason}"));

    if data.len() < HEADER_SIZE {
        return Err(invalid("too short"));
    }

    if data[COLOR_MAP_TYPE] != 0 {
        return Err(invalid("color-mapped images are not supported"));
    }

    let image_type = data[IMAGE_TYPE];
    let gray = matches!(image_type, GRAYSCALE | RLE_GRAYSCALE);

    if !gray && !matches!(image_type, TRUE_COLOR | RLE_TRUE_COLOR) {
        return Err(invalid("unknown image type"));
    }

    let pixel_bytes = match (gray, data[BITS_PER_PIXEL]) {
        (true, 8) => 1,
        (false, 24) => 3,
        (false, 32) => 4,
        _ => return Err(invalid("unsupported bits per pixel")),
    };

    let width = usize::from(u16::from_le_bytes([data[WIDTH], data[WIDTH + 1]]));
    let height = usize::from(u16::from_le_bytes([data[HEIGHT], data[HEIGHT + 1]]));

    if width == 0 || height == 0 {
        return Err(invalid("empty image"));
    }

    Ok(Header {
        width,
        height,
        pixel_bytes,
        rle: matches!(image_type, RLE_TRUE_COLOR | RLE_GRAYSCALE),
        top_down: data[DESCRIPTOR] & TOP_DOWN != 0,
        start: HEADER_SIZE + usize::from(data[ID_LENGTH]),
    })
}

pub fn decode_tga(data: &[u8]) -> Result<Image> {
    let header = header(data)?;
    let pixels = header.width * header.height;
    let stored = if header.rle {
        unpack_rle(data, &header, pixels)?
    } else {
        let end = header.start + pixels * header.pixel_bytes;
        data.get(header.start..end)
            .ok_or_else(|| Error::Image("TGA data truncated".to_owned()))?
            .to_vec()
    };

    let row_bytes = header.width * RGBA_CHANNELS;
    let mut rgba = vec![0u8; pixels * RGBA_CHANNELS];

    for (i, pixel) in stored.chunks_exact(header.pixel_bytes).enumerate() {
        let (row, column) = (i / header.width, i % header.width);
        let out_row = if header.top_down {
            row
        } else {
            header.height - 1 - row
        };
        let at = out_row * row_bytes + column * RGBA_CHANNELS;

        rgba[at..at + RGBA_CHANNELS].copy_from_slice(&to_rgba(pixel));
    }

    Ok(Image {
        width: u32::try_from(header.width).unwrap_or(u32::MAX),
        height: u32::try_from(header.height).unwrap_or(u32::MAX),
        rgba,
    })
}

/// TGA stores blue, green, red (and alpha); gray images store one value.
fn to_rgba(pixel: &[u8]) -> [u8; RGBA_CHANNELS] {
    match *pixel {
        [gray] => [gray, gray, gray, OPAQUE],
        [b, g, r] => [r, g, b, OPAQUE],
        [b, g, r, a] => [r, g, b, a],
        _ => [0, 0, 0, OPAQUE],
    }
}

/// Expands run-length packets: a header byte, then one pixel repeated or
/// several raw pixels (up to 128 either way).
fn unpack_rle(data: &[u8], header: &Header, pixels: usize) -> Result<Vec<u8>> {
    let truncated = || Error::Image("TGA data truncated".to_owned());
    let size = header.pixel_bytes;
    let mut out = Vec::with_capacity(pixels * size);
    let mut pos = header.start;

    while out.len() < pixels * size {
        let packet = *data.get(pos).ok_or_else(truncated)?;

        pos += 1;
        let count = usize::from(packet & PACKET_COUNT) + 1;

        if packet & RUN_PACKET != 0 {
            let pixel = data.get(pos..pos + size).ok_or_else(truncated)?;

            for _ in 0..count {
                out.extend_from_slice(pixel);
            }

            pos += size;
        } else {
            let raw = data.get(pos..pos + count * size).ok_or_else(truncated)?;

            out.extend_from_slice(raw);
            pos += count * size;
        }
    }

    // A last packet may run past the image; the extra pixels are dropped.
    out.truncate(pixels * size);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tga(image_type: u8, bits: u8, width: u16, height: u16, top_down: bool) -> Vec<u8> {
        let mut data = vec![0u8; HEADER_SIZE];

        data[IMAGE_TYPE] = image_type;
        data[WIDTH..WIDTH + 2].copy_from_slice(&width.to_le_bytes());
        data[HEIGHT..HEIGHT + 2].copy_from_slice(&height.to_le_bytes());
        data[BITS_PER_PIXEL] = bits;
        if top_down {
            data[DESCRIPTOR] = TOP_DOWN;
        }

        data
    }

    fn pixel(image: &Image, x: usize, y: usize) -> &[u8] {
        let at = (y * image.width as usize + x) * RGBA_CHANNELS;

        &image.rgba[at..at + RGBA_CHANNELS]
    }

    #[test]
    fn reads_bottom_up_24_bit_images() {
        // Two rows of one pixel, stored bottom first: blue at the bottom, red on top.
        let mut data = tga(TRUE_COLOR, 24, 1, 2, false);

        data.extend_from_slice(&[255, 0, 0, 0, 0, 255]);
        let image = decode_tga(&data).unwrap();

        assert_eq!((image.width, image.height), (1, 2));
        assert_eq!(pixel(&image, 0, 0), [255, 0, 0, 255]);
        assert_eq!(pixel(&image, 0, 1), [0, 0, 255, 255]);
    }

    #[test]
    fn reads_top_down_32_bit_images_with_alpha_and_an_id() {
        let mut data = tga(TRUE_COLOR, 32, 2, 1, true);

        data[ID_LENGTH] = 3;
        data.extend_from_slice(b"abc");
        data.extend_from_slice(&[10, 20, 30, 40, 50, 60, 70, 80]);
        let image = decode_tga(&data).unwrap();

        assert_eq!(pixel(&image, 0, 0), [30, 20, 10, 40]);
        assert_eq!(pixel(&image, 1, 0), [70, 60, 50, 80]);
    }

    #[test]
    fn expands_run_length_packets() {
        // Three pixels: a run of two greens, then one raw white.
        let mut data = tga(RLE_TRUE_COLOR, 24, 3, 1, true);

        data.extend_from_slice(&[0x81, 0, 255, 0, 0x00, 255, 255, 255]);
        let image = decode_tga(&data).unwrap();

        assert_eq!(pixel(&image, 0, 0), [0, 255, 0, 255]);
        assert_eq!(pixel(&image, 1, 0), [0, 255, 0, 255]);
        assert_eq!(pixel(&image, 2, 0), [255, 255, 255, 255]);
    }

    #[test]
    fn reads_grayscale_images() {
        let mut data = tga(RLE_GRAYSCALE, 8, 2, 1, true);

        data.extend_from_slice(&[0x81, 128]);
        let image = decode_tga(&data).unwrap();

        assert_eq!(pixel(&image, 1, 0), [128, 128, 128, 255]);
    }

    #[test]
    fn rejects_other_files_and_truncated_data() {
        assert!(!is_tga(b"DDS \x7c"));
        assert!(!is_tga(&tga(1, 8, 2, 2, true)), "color-mapped");
        assert!(!is_tga(&tga(TRUE_COLOR, 16, 2, 2, true)), "16 bits");
        assert!(decode_tga(&tga(TRUE_COLOR, 24, 4, 4, true)).is_err());
        let mut rle = tga(RLE_TRUE_COLOR, 24, 4, 1, true);

        rle.push(0x83);
        assert!(decode_tga(&rle).is_err());
    }
}
