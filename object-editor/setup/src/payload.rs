//! What the setup carries: the Tinkerbench binary (see `build.rs`), its license
//! files and its icons.

use std::io::Read;

use flate2::read::DeflateDecoder;

const COMPRESSED_APP: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/payload.deflate"));

pub const LICENSE: &str = include_str!("../../../LICENSE");
pub const APP_NOTICES: &str = include_str!("../../app/src-tauri/THIRD-PARTY-NOTICES.txt");
pub const SETUP_NOTICES: &str = include_str!("../THIRD-PARTY-NOTICES.txt");
pub const LICENSE_FILE: &str = "LICENSE";
pub const NOTICES_FILE: &str = "THIRD-PARTY-NOTICES.txt";

/// The app icon in each size the Linux icon theme uses.
pub const ICONS: &[(u32, &[u8])] = &[
    (32, include_bytes!("../../app/src-tauri/icons/32x32.png")),
    (64, include_bytes!("../../app/src-tauri/icons/64x64.png")),
    (128, include_bytes!("../../app/src-tauri/icons/128x128.png")),
    (
        256,
        include_bytes!("../../app/src-tauri/icons/128x128@2x.png"),
    ),
    (512, include_bytes!("../../app/src-tauri/icons/icon.png")),
];
pub const WINDOW_ICON: &[u8] = include_bytes!("../../app/src-tauri/icons/64x64.png");

/// Whether this build carries Tinkerbench; development builds do not.
pub fn has_app() -> bool {
    !COMPRESSED_APP.is_empty()
}

pub fn app_binary() -> std::io::Result<Vec<u8>> {
    let mut binary = Vec::new();

    DeflateDecoder::new(COMPRESSED_APP).read_to_end(&mut binary)?;
    Ok(binary)
}
