use qrcode::{QrCode, render::unicode};

/// Print the startup banner: scheme, served path, local + network URLs, and a
/// terminal QR code of the network URL (unless suppressed).
#[allow(clippy::too_many_arguments)]
pub fn print_banner(
    version: &str,
    path: &str,
    scheme: &str,
    port: u16,
    network_ip: Option<&str>,
    upload: bool,
    show_qr: bool,
    auth_without_tls: bool,
) {
    let mode = if upload {
        "upload enabled"
    } else {
        "read-only"
    };
    println!("serve v{version}  •  serving {path}  ({mode})");
    println!("  Local:    {scheme}://127.0.0.1:{port}");

    let network_url = network_ip.map(|ip| format!("{scheme}://{ip}:{port}"));
    if let Some(ref url) = network_url {
        println!("  Network:  {url}");
    }

    if auth_without_tls {
        println!("  WARNING: --auth without --tls sends credentials in cleartext.");
    }

    if show_qr
        && let Some(url) = network_url
        && let Ok(code) = QrCode::new(url.as_bytes())
    {
        let image = code.render::<unicode::Dense1x2>().quiet_zone(true).build();
        println!("\n{image}");
    }
}
