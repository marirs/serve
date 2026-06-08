use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

use crate::fs_guard::sanitize_filename;

/// Shared state for the upload handler: the absolute root the server serves.
#[derive(Clone)]
pub struct UploadState {
    pub root: Arc<PathBuf>,
}

/// Handle a multipart POST: write each file part into the served root using a
/// sanitized basename. Rejects unsafe names. Redirects back to "/" on success.
pub async fn upload(State(state): State<UploadState>, mut multipart: Multipart) -> Response {
    while let Ok(Some(field)) = multipart.next_field().await {
        let raw_name = field.file_name().unwrap_or("").to_string();
        let safe = sanitize_filename(&raw_name);
        if safe.is_empty() {
            return (StatusCode::BAD_REQUEST, "invalid filename").into_response();
        }
        let Ok(bytes) = field.bytes().await else {
            return (StatusCode::BAD_REQUEST, "read failed").into_response();
        };
        let dest = state.root.join(&safe);
        // Defense in depth: dest must remain directly under root.
        if dest.parent() != Some(state.root.as_path()) {
            return (StatusCode::BAD_REQUEST, "invalid path").into_response();
        }
        let Ok(mut file) = tokio::fs::File::create(&dest).await else {
            return (StatusCode::INTERNAL_SERVER_ERROR, "write failed").into_response();
        };
        if file.write_all(&bytes).await.is_err() {
            return (StatusCode::INTERNAL_SERVER_ERROR, "write failed").into_response();
        }
    }
    Redirect::to("/").into_response()
}
