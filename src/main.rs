use clap::Parser;
use std::net::SocketAddr;
use std::sync::Arc;

use serve::cli::Args;
use serve::{banner, tls};

#[tokio::main]
async fn main() {
    let args = Args::parse();
    if !args.quiet {
        tracing_subscriber::fmt().with_target(false).init();
    }

    let root = Arc::new(
        args.path
            .canonicalize()
            .unwrap_or_else(|_| panic!("path not found: {:?}", args.path)),
    );

    let app = serve::build_router(root.clone(), args.upload, args.auth.clone());

    let addr: SocketAddr = format!("{}:{}", args.bind, args.port)
        .parse()
        .expect("invalid bind address");

    let scheme = if args.tls { "https" } else { "http" };
    let network_ip = local_ip_address::local_ip().ok().map(|i| i.to_string());
    banner::print_banner(
        env!("CARGO_PKG_VERSION"),
        &root.display().to_string(),
        scheme,
        args.port,
        network_ip.as_deref(),
        args.upload,
        !args.no_qr,
        args.auth.is_some() && !args.tls,
    );

    if args.tls {
        // Install the ring crypto provider (pure-Rust TLS stack, no OpenSSL).
        rustls::crypto::ring::default_provider()
            .install_default()
            .ok();
        let (cert_der, key_der) = tls::generate_cert_der(&tls::default_sans());
        let config = axum_server::tls_rustls::RustlsConfig::from_der(vec![cert_der], key_der)
            .await
            .expect("failed to build TLS config");
        axum_server::bind_rustls(addr, config)
            .serve(app.into_make_service())
            .await
            .expect("server error");
    } else {
        axum_server::bind(addr)
            .serve(app.into_make_service())
            .await
            .expect("server error");
    }
}
