//! Serves game icons to the window as PNG through the `wc3icon:` URL scheme.
//! The path after the host is the icon's game path, URL-encoded.

use tauri::http::{Request, Response, StatusCode, header};
use tauri::{Manager, Runtime, UriSchemeContext, UriSchemeResponder};
use wc3_data::icon::load_texture;

use crate::state::AppState;

pub const SCHEME: &str = "wc3icon";
const PNG: &str = "image/png";
const TEXT: &str = "text/plain";
const HEX_RADIX: u32 = 16;

/// Decodes on a worker thread so a full list of icons never blocks the window.
pub fn handle<R: Runtime>(
    ctx: UriSchemeContext<'_, R>,
    request: Request<Vec<u8>>,
    responder: UriSchemeResponder,
) {
    let app = ctx.app_handle().clone();
    std::thread::spawn(move || {
        let path = percent_decode(request.uri().path().trim_start_matches('/'));
        let response = match icon_png(&app, &path) {
            Ok(png) => response(StatusCode::OK, PNG, png),
            Err(message) => response(StatusCode::NOT_FOUND, TEXT, message.into_bytes()),
        };
        responder.respond(response);
    });
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

fn response(status: StatusCode, content_type: &str, body: Vec<u8>) -> Response<Vec<u8>> {
    let mut response = Response::new(body);
    *response.status_mut() = status;
    if let Ok(value) = content_type.parse() {
        response.headers_mut().insert(header::CONTENT_TYPE, value);
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
