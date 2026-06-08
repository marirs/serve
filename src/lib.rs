pub mod auth;
pub mod banner;
pub mod cli;
pub mod errors;
pub mod fs_guard;
pub mod listing;
pub mod tls;
pub mod upload;

use axum::{
    Router,
    extract::{Request, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::post,
};
use std::path::PathBuf;
use std::sync::Arc;
use tower::ServiceExt;
use tower_http::services::ServeDir;

use crate::cli::Auth;
use crate::upload::UploadState;

/// State for the directory/file handler: the canonical served root and whether
/// the listing should advertise the upload widget.
#[derive(Clone)]
struct AppState {
    root: Arc<PathBuf>,
    upload: bool,
}

/// Build the application router. Exposed so integration tests can mount it over
/// plain HTTP without setting up a TLS listener.
///
/// `root` must already be canonicalized by the caller so that path resolution
/// and `ServeDir` agree on the same base.
pub fn build_router(root: Arc<PathBuf>, upload: bool, auth: Option<Auth>) -> Router {
    let state = AppState {
        root: root.clone(),
        upload,
    };

    let mut app = Router::new().fallback(serve_or_list).with_state(state);

    if upload {
        let up_state = UploadState { root: root.clone() };
        let up = Router::new()
            .route("/upload", post(upload::upload))
            .with_state(up_state);
        app = app.merge(up);
    }

    if let Some(creds) = auth {
        app = app.layer(axum::middleware::from_fn_with_state(
            creds,
            auth::require_auth,
        ));
    }

    app
}

/// For a GET request: if the target path is an existing directory, render the
/// listing; otherwise delegate to `ServeDir`, which streams files with range
/// support and applies its own traversal protection and 404 handling.
async fn serve_or_list(State(state): State<AppState>, req: Request) -> Response {
    let url_path = req.uri().path().to_string();
    let rel = url_path.trim_start_matches('/');

    let dir_target = if rel.is_empty() {
        Some((*state.root).clone())
    } else {
        fs_guard::safe_join(&state.root, rel).filter(|p| p.is_dir())
    };

    if let Some(dir) = dir_target
        && dir.is_dir()
    {
        return match listing::render_listing(&dir, &url_path, state.upload) {
            Ok(html) => Html(html).into_response(),
            Err(_) => errors::error_page(StatusCode::INTERNAL_SERVER_ERROR),
        };
    }

    // Files (and not-found) go through ServeDir.
    let serve = ServeDir::new(state.root.as_path());
    match serve.oneshot(req).await {
        Ok(resp) => {
            // Re-skin any error status (404, 405, ...) from ServeDir with our
            // styled error page; pass successful file responses through as-is.
            let status = resp.status();
            if status.is_client_error() || status.is_server_error() {
                errors::error_page(status)
            } else {
                resp.into_response()
            }
        }
        Err(_) => errors::error_page(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
