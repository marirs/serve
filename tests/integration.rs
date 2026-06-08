use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use serve::build_router;

/// Spawn the router on a free ephemeral port in a background thread and return
/// its address. The brief sleep lets the listener come up before tests hit it.
fn spawn_server(root: PathBuf, upload: bool) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let app = build_router(Arc::new(root.canonicalize().unwrap()), upload, None);
    thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            axum_server::bind(addr)
                .serve(app.into_make_service())
                .await
                .unwrap();
        });
    });
    thread::sleep(Duration::from_millis(300));
    addr
}

#[test]
fn serves_a_file_and_lists_dir() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("hello.txt"), b"world").unwrap();
    let addr = spawn_server(dir.path().to_path_buf(), false);

    let body = reqwest::blocking::get(format!("http://{addr}/hello.txt"))
        .unwrap()
        .text()
        .unwrap();
    assert_eq!(body, "world");

    let index = reqwest::blocking::get(format!("http://{addr}/"))
        .unwrap()
        .text()
        .unwrap();
    assert!(index.contains("hello.txt"));
}

#[test]
fn rejects_traversal() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("ok.txt"), b"ok").unwrap();
    let addr = spawn_server(dir.path().to_path_buf(), false);

    let resp = reqwest::blocking::get(format!("http://{addr}/../../../etc/passwd")).unwrap();
    assert!(
        resp.status().is_client_error() || resp.status().is_redirection(),
        "traversal must not return file contents, got {}",
        resp.status()
    );
}

#[test]
fn upload_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let addr = spawn_server(dir.path().to_path_buf(), true);

    let part = reqwest::blocking::multipart::Part::bytes(b"uploaded".to_vec()).file_name("up.txt");
    let form = reqwest::blocking::multipart::Form::new().part("file", part);

    let resp = reqwest::blocking::Client::new()
        .post(format!("http://{addr}/upload"))
        .multipart(form)
        .send()
        .unwrap();
    assert!(resp.status().is_success());

    let got = std::fs::read_to_string(dir.path().join("up.txt")).unwrap();
    assert_eq!(got, "uploaded");
}
