use axum::{
    extract::{Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::Next,
    response::Response,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use subtle::ConstantTimeEq;

use crate::cli::Auth;
use crate::errors::error_page;

/// Verify an `Authorization: Basic ...` header against expected creds in
/// constant time. Returns false for any malformed/missing/incorrect input.
pub fn check_credentials(header_value: &str, user: &str, pass: &str) -> bool {
    let Some(b64) = header_value.strip_prefix("Basic ") else {
        return false;
    };
    let Ok(decoded) = STANDARD.decode(b64.trim()) else {
        return false;
    };
    let Ok(text) = String::from_utf8(decoded) else {
        return false;
    };
    let Some((u, p)) = text.split_once(':') else {
        return false;
    };
    let expected = format!("{user}:{pass}");
    let got = format!("{u}:{p}");
    // Constant-time compare to avoid leaking length/content via timing.
    expected.as_bytes().ct_eq(got.as_bytes()).into()
}

/// Tower middleware that gates every request behind Basic auth.
pub async fn require_auth(State(creds): State<Auth>, req: Request, next: Next) -> Response {
    let ok = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|h| check_credentials(h, &creds.user, &creds.pass))
        .unwrap_or(false);

    if ok {
        next.run(req).await
    } else {
        // Styled 401 page, but keep the WWW-Authenticate header so browsers
        // still present the Basic auth prompt.
        let mut resp = error_page(StatusCode::UNAUTHORIZED);
        resp.headers_mut().insert(
            header::WWW_AUTHENTICATE,
            HeaderValue::from_static("Basic realm=\"serve\""),
        );
        resp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_correct_basic_header() {
        // base64("alice:secret") = YWxpY2U6c2VjcmV0
        let header = "Basic YWxpY2U6c2VjcmV0";
        assert!(check_credentials(header, "alice", "secret"));
    }

    #[test]
    fn rejects_wrong_password() {
        let header = "Basic YWxpY2U6c2VjcmV0";
        assert!(!check_credentials(header, "alice", "nope"));
    }

    #[test]
    fn rejects_malformed_header() {
        assert!(!check_credentials("Bearer xyz", "alice", "secret"));
        assert!(!check_credentials("Basic !!!notbase64", "alice", "secret"));
    }
}
