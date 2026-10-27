use std::sync::Arc;
use tauri::http::{Request, Response, StatusCode, header};
use tauri::{Manager, Runtime, UriSchemeContext, UriSchemeResponder};

use wc3_data::game::{ArtMode, GameFiles};
use wc3_data::icon::{dds_file, decode_texture, decode_texture_fitting, load_texture};
use wc3_data::model::{load_model, preview_copy};

use crate::state::{AppState, Session};

pub const ICON_SCHEME: &str = "wc3icon";
pub const TEXTURE_SCHEME: &str = "wc3tex";
pub const MODEL_SCHEME: &str = "wc3model";
const PNG: &str = "image/png";
const BINARY: &str = "application/octet-stream";
const TEXT: &str = "text/plain";
const HEX_RADIX: u32 = 16;

type Loader<R> = fn(&tauri::AppHandle<R>, &str) -> Result<Vec<u8>, String>;

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

enum Art {
    Map(String, Vec<u8>),
    Game(Arc<GameFiles>, ArtMode),
}

fn art_source(session: &Session, path: &str) -> Art {
    match session.map.as_ref().and_then(|map| map.art(path)) {
        Some((file, data)) => Art::Map(file, data),
        None => Art::Game(Arc::clone(&session.files), session.mode),
    }
}

fn icon_png<R: Runtime>(app: &tauri::AppHandle<R>, path: &str) -> Result<Vec<u8>, String> {
    let state = app.state::<AppState>();
    let source = state.with(|session| {
        let cached = match &session.map {
            Some(map) => map.icons.get(path),
            None => None,
        }
        .or_else(|| session.icons.get(&(session.mode, path.to_owned())));
        Ok(cached.cloned().ok_or_else(|| art_source(session, path)))
    })?;

    match source {
        Ok(png) => Ok(png),
        Err(Art::Map(file, data)) => {
            let image = decode_texture(&data).map_err(|e| format!("{file}: {e}"))?;
            let png = image.to_png().map_err(|e| e.to_string())?;

            state.with(|session| {
                if let Some(map) = &mut session.map {
                    map.icons.insert(path.to_owned(), png.clone());
                }

                Ok(())
            })?;

            Ok(png)
        }
        Err(Art::Game(files, mode)) => {
            let image = load_texture(&files, path, mode).map_err(|e| e.to_string())?;
            let png = image.to_png().map_err(|e| e.to_string())?;

            state.with(|session| {
                session.icons.insert((mode, path.to_owned()), png.clone());
                Ok(())
            })?;

            Ok(png)
        }
    }
}

/// Textures for the 3D preview, at most this many pixels on a side: the
/// preview is a few hundred pixels wide, and HD textures are 2048.
const PREVIEW_TEXTURE_SIZE: u32 = 512;

fn texture_raw<R: Runtime>(app: &tauri::AppHandle<R>, path: &str) -> Result<Vec<u8>, String> {
    let state = app.state::<AppState>();
    let (file, data) = match state.with(|session| Ok(art_source(session, path)))? {
        Art::Map(file, data) => (file, data),
        Art::Game(files, mode) => {
            let file = dds_file(path);
            let data = files.read_art(&file, mode).map_err(|e| e.to_string())?;
            (file, data)
        }
    };

    let image =
        decode_texture_fitting(&data, PREVIEW_TEXTURE_SIZE).map_err(|e| format!("{file}: {e}"))?;

    Ok(image.to_raw())
}

fn model_bytes<R: Runtime>(app: &tauri::AppHandle<R>, path: &str) -> Result<Vec<u8>, String> {
    let state = app.state::<AppState>();

    match state.with(|session| Ok(art_source(session, path)))? {
        Art::Map(file, data) => preview_copy(&data).map_err(|e| format!("{file}: {e}")),
        Art::Game(files, mode) => load_model(&files, path, mode).map_err(|e| {
            // Some game objects name a model that was never shipped, such as placeholders.
            if e.is_not_found() {
                "it is not in the game files or the map.".to_owned()
            } else {
                e.to_string()
            }
        }),
    }
}

fn response(status: StatusCode, content_type: &str, body: Vec<u8>) -> Response<Vec<u8>> {
    let mut response = Response::new(body);

    *response.status_mut() = status;
    if let Ok(value) = content_type.parse() {
        response.headers_mut().insert(header::CONTENT_TYPE, value);
    }

    // The page fetches models and textures from these schemes; in development it
    // runs from the dev server's origin, so the response must allow that origin.
    // These schemes only exist inside this app and only serve game and map art.
    if let Ok(value) = "*".parse() {
        response
            .headers_mut()
            .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, value);
    }

    response
}

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
