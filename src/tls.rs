use rcgen::generate_simple_self_signed;

/// Generate a fresh in-memory self-signed certificate covering `sans` (IP and
/// DNS subject-alt-names) and return its DER-encoded cert and private key.
///
/// Nothing is written to disk — the key material exists only for this
/// process's lifetime, and a new pair is created on every start. That is
/// acceptable here because the binary's sole job is short-lived file serving.
pub fn generate_cert_der(sans: &[String]) -> (Vec<u8>, Vec<u8>) {
    let certified = generate_simple_self_signed(sans.to_vec())
        .expect("self-signed certificate generation failed");
    let cert_der = certified.cert.der().to_vec();
    let key_der = certified.signing_key.serialize_der();
    (cert_der, key_der)
}

/// Collect SAN entries for the cert: localhost, loopback, and the detected
/// LAN IP if available.
pub fn default_sans() -> Vec<String> {
    let mut sans = vec!["localhost".to_string(), "127.0.0.1".to_string()];
    if let Ok(ip) = local_ip_address::local_ip() {
        sans.push(ip.to_string());
    }
    sans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_non_empty_der_blobs() {
        let (cert, key) = generate_cert_der(&["127.0.0.1".into(), "localhost".into()]);
        assert!(!cert.is_empty());
        assert!(!key.is_empty());
    }

    #[test]
    fn default_sans_include_loopback() {
        let sans = default_sans();
        assert!(sans.iter().any(|s| s == "127.0.0.1"));
        assert!(sans.iter().any(|s| s == "localhost"));
    }
}
