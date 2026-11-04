//! Embeds the Tinkerbench binary the setup installs. Release builds name it in
//! `TINKERBENCH_PAYLOAD`; without it the payload is empty, so the setup's pages
//! can still be built and tried, and installing says there is nothing to install.
//! On Windows it also gives the setup the app's icon.

use std::error::Error;
use std::io::Write;
use std::path::PathBuf;

use flate2::Compression;
use flate2::write::DeflateEncoder;

const PAYLOAD_VAR: &str = "TINKERBENCH_PAYLOAD";
const PAYLOAD_FILE: &str = "payload.deflate";
const WINDOWS_ICON: &str = "../app/src-tauri/icons/icon.ico";

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-env-changed={PAYLOAD_VAR}");

    let out = PathBuf::from(std::env::var_os("OUT_DIR").ok_or("OUT_DIR is not set")?);
    let mut compressed = Vec::new();

    if let Some(path) = std::env::var_os(PAYLOAD_VAR) {
        let path = PathBuf::from(path);

        println!("cargo:rerun-if-changed={}", path.display());

        let binary = std::fs::read(&path)
            .map_err(|e| format!("cannot read {PAYLOAD_VAR} ({}): {e}", path.display()))?;
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::best());

        encoder.write_all(&binary)?;
        compressed = encoder.finish()?;
    }

    std::fs::write(out.join(PAYLOAD_FILE), compressed)?;

    if std::env::var("CARGO_CFG_TARGET_OS").is_ok_and(|os| os == "windows") {
        println!("cargo:rerun-if-changed={WINDOWS_ICON}");
        winresource::WindowsResource::new()
            .set_icon(WINDOWS_ICON)
            .compile()?;
    }

    Ok(())
}
