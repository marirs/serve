use clap::Parser;
use std::path::PathBuf;

/// Single-binary HTTP(S) file server for the current folder.
#[derive(Parser, Debug)]
#[command(name = "serve", version, about)]
pub struct Args {
    /// Folder to serve (defaults to current directory).
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Port to listen on.
    #[arg(short, long, default_value_t = 8080)]
    pub port: u16,

    /// Address to bind (0.0.0.0 = visible on the local network).
    #[arg(short, long, default_value = "0.0.0.0")]
    pub bind: String,

    /// Serve over HTTPS with an in-memory self-signed certificate.
    #[arg(long)]
    pub tls: bool,

    /// Allow uploading files into the served folder (POST).
    #[arg(short, long)]
    pub upload: bool,

    /// Require HTTP Basic auth, e.g. --auth user:pass.
    #[arg(long, value_parser = Auth::parse)]
    pub auth: Option<Auth>,

    /// Do not print the startup QR code.
    #[arg(long)]
    pub no_qr: bool,

    /// Reduce request logging.
    #[arg(short, long)]
    pub quiet: bool,
}

#[derive(Clone, Debug)]
pub struct Auth {
    pub user: String,
    pub pass: String,
}

impl Auth {
    /// Parse `user:pass`. The password may itself contain colons (we split on
    /// the first colon only), so passwords like `p:a:ss` are preserved.
    pub fn parse(s: &str) -> Result<Auth, String> {
        match s.split_once(':') {
            Some((user, pass)) if !user.is_empty() => Ok(Auth {
                user: user.to_string(),
                pass: pass.to_string(),
            }),
            _ => Err("expected USER:PASS".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_user_pass() {
        let a = Auth::parse("alice:secret").unwrap();
        assert_eq!(a.user, "alice");
        assert_eq!(a.pass, "secret");
    }

    #[test]
    fn pass_may_contain_colon() {
        let a = Auth::parse("bob:p:a:ss").unwrap();
        assert_eq!(a.user, "bob");
        assert_eq!(a.pass, "p:a:ss");
    }

    #[test]
    fn rejects_missing_colon() {
        assert!(Auth::parse("nopassword").is_err());
    }
}
