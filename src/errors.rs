use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};

const TEMPLATE: &str = include_str!("templates/error.html");

/// Render a consistent, styled HTML error page for any status code. The page
/// shows the numeric code and its canonical reason phrase (e.g. "404 Not
/// Found", "401 Unauthorized", "423 Locked", "500 Internal Server Error") plus
/// a link back to the index. Used for every error path so the UI is uniform
/// instead of a bare plain-text body.
pub fn error_page(status: StatusCode) -> Response {
    let code = status.as_u16();
    let message = status.canonical_reason().unwrap_or("Error");
    let html = TEMPLATE
        .replace("{CODE}", &code.to_string())
        .replace("{MESSAGE}", message);
    (status, Html(html)).into_response()
}
