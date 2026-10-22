//! Serves game files to the window through custom URL schemes. The path after
//! the host is the file's game path, URL-encoded.
//!
//! - `wc3icon:` icons as PNG (cached; small and shown in lists)
//! - `wc3tex:` textures as raw RGBA with a size header (large, for the 3D view)
//! - `wc3model:` models as MDX bytes, without the chunks previews do not use

use tauri::http::{Request, Response, StatusCode, header};
use tauri::{Manager, Runtime, UriSchemeContext, UriSchemeResponder};
use wc3_data::icon::{dds_file, decode_dds, load_texture};
use wc3_data::model::load_model;

use crate::state::AppState;

pub const ICON_SCHEME: &str = "wc3icon";
pub const TEXTURE_SCHEME: &str = "wc3tex";
pub const MODEL_SCHEME: &str = "wc3model";
const PNG: &str = "image/png";
const BINARY: &str = "application/octet-stream";
const TEXT: &str = "text/plain";
const HEX_RADIX: u32 = 16;

type Loader<R> = fn(&tauri::AppHandle<R>, &str) -> Result<Vec<u8>, String>;

/// Loads on a worker thread so decoding never blocks the window.
fn serve<R: Runtime>(
    ctx: &UriSchemeContext<'_, R>,
    request: &Request<Vec<u8>>,
    responder: UriSchemeResponder,
    content_type: &'static str,
    load: Loader<R>,
) {
    let app = ctx.app_handle().clone();
    let path = percent_decode(request.uri().path().trim_start_matches('/'));
    std::thread::spawn(move || {
        let response = match load(&app, &path) {
            Ok(bytes) => response(StatusCode::OK, content_type, bytes),
            Err(message) => response(StatusCode::NOT_FOUND, TEXT, message.into_bytes()),
        };
        responder.respond(response);
    });
}

pub fn handle_icon<R: Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    serve(&ctx, &request, responder, PNG, icon_png);
}

pub fn handle_texture<R: Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    serve(&ctx, &request, responder, BINARY, texture_raw);
}

pub fn handle_model<R: Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    serve(&ctx, &request, responder, BINARY, model_bytes);
}

fn icon_png<R: Runtime>(app: &tauri::AppHandle<R>, path: &str) -> Result<Vec<u8>, String> {
    let state = app.state::<AppState>();
    state.with(|session| {
        let key = (session.mode, path.to_owned());
        if let Some(png) = session.icons.get(&key) {
            return Ok(png.clone());
        }

        let image = load_texture(&session.files, path, session.mode).map_err(|e| e.to_string())?;
        let png = image.to_png().map_err(|e| e.to_string())?;
        session.icons.insert(key, png.clone());
        Ok(png)
    })
}

/// Reads the file while holding the session lock, then decodes after
/// releasing it, so a large texture never stalls other commands.
fn texture_raw<R: Runtime>(app: &tauri::AppHandle<R>, path: &str) -> Result<Vec<u8>, String> {
    let file = dds_file(path);
    let state = app.state::<AppState>();
    let data = state.with(|session| {
        session
            .files
            .read_art(&file, session.mode)
            .map_err(|e| e.to_string())
    })?;

    let image = decode_dds(&data).map_err(|e| format!("{file}: {e}"))?;
    Ok(image.to_raw())
}

fn model_bytes<R: Runtime>(app: &tauri::AppHandle<R>, path: &str) -> Result<Vec<u8>, String> {
    let state = app.state::<AppState>();
    state.with(|session| load_model(&session.files, path, session.mode).map_err(|e| e.to_string()))
}

fn response(status: StatusCode, content_type: &str, body: Vec<u8>) -> Response<Vec<u8>> {
    let mut response = Response::new(body);
    *response.status_mut() = status;
    if let Ok(value) = content_type.parse() {
        response.headers_mut().insert(header::CONTENT_TYPE, value);
    }

    // The page fetches models and textures from these schemes; in development it
    // runs from the dev server's origin, so the response must allow that origin.
    // These schemes only exist inside this app and only serve game files.
    if let Ok(value) = "*".parse() {
        response
            .headers_mut()
            .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, value);
    }

    response
}

/// Decodes `%XX` escapes; invalid escapes are kept as they are.
fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        let escaped = (bytes[i] == b'%')
            .then(|| text.get(i + 1..i + 3))
            .flatten()
            .and_then(|hex| u8::from_str_radix(hex, HEX_RADIX).ok());

        match escaped {
            Some(byte) => {
                out.push(byte);
                i += 3;
            }
            None => {
                out.push(bytes[i]);
                i += 1;
            }
        }
    }

    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::percent_decode;

    #[test]
    fn decodes_icon_paths() {
        assert_eq!(
            percent_decode("ReplaceableTextures%5CCommandButtons%5CBTNHeroPaladin.blp"),
            "ReplaceableTextures\\CommandButtons\\BTNHeroPaladin.blp"
        );
        assert_eq!(percent_decode("100%"), "100%");
        assert_eq!(percent_decode("%zz"), "%zz");
    }
}
