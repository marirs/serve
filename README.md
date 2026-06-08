# serve

[![CI](https://github.com/marirs/serve/actions/workflows/ci.yml/badge.svg)](https://github.com/marirs/serve/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/marirs/serve)](https://github.com/marirs/serve/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Platforms](https://img.shields.io/badge/platforms-linux%20%7C%20macos%20%7C%20windows-lightgrey)

A single static binary that serves a folder over HTTP(S) — like
`python -m http.server`, but one native executable you drop in
`/usr/local/bin` and run from anywhere. Built for quickly sharing files across
devices on the same network.

## Features

- Serve any folder over **HTTP** or **HTTPS** (`--tls`).
- In-memory **self-signed TLS** — regenerated each start, never written to disk.
- Pure-Rust TLS stack (rustls + ring) — **no OpenSSL** dependency.
- Clean **directory listing** UI (embedded, no external assets).
- **Range requests** — resumable / seekable downloads.
- Styled **HTML error pages** (404 / 401 / 500 / …) instead of bare text.
- Optional **file upload** (`--upload`).
- Optional **HTTP Basic auth** (`--auth user:pass`), constant-time compare.
- **QR code** of the network URL printed on startup — scan from a phone.

## Install

Download the archive for your platform from
[Releases](https://github.com/marirs/serve/releases), then:

```bash
tar -xzf serve-x86_64-unknown-linux-gnu.tar.gz
chmod +x serve
sudo mv serve /usr/local/bin/
```

Or build from source (Rust 1.85+ for the 2024 edition):

```bash
cargo install --path .
```

## Usage

```bash
serve                      # serve ./ over HTTP on 0.0.0.0:8080
serve --tls                # serve ./ over HTTPS
serve --tls /root          # serve /root over HTTPS
serve -p 9000 ~/Downloads  # custom port + folder
serve --upload .           # allow uploads into the current folder
serve --auth alice:secret --tls   # password-protected over HTTPS
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `[PATH]` | Folder to serve | `.` |
| `-p, --port` | Port | `8080` |
| `-b, --bind` | Bind address | `0.0.0.0` |
| `--tls` | HTTPS with in-memory self-signed cert | off |
| `-u, --upload` | Enable uploads | off |
| `--auth USER:PASS` | Require HTTP Basic auth | off |
| `--no-qr` | Suppress startup QR code | off |
| `-q, --quiet` | Reduce logging | off |

## Security notes

- Binds to `0.0.0.0` by default — **reachable by anyone on your network**.
  Use `-b 127.0.0.1` to keep it local.
- `--tls` uses a self-signed certificate, so browsers show a warning. This is
  expected; the connection is still encrypted. There is no trusted CA chain.
- `--auth` over plain HTTP sends credentials in cleartext. Combine `--auth`
  with `--tls`. The server warns you on startup if you don't.
- Upload is disabled unless `--upload` is passed. Uploaded filenames are
  reduced to a safe basename; path traversal is rejected on both serving and
  upload.

## Supported platforms

Prebuilt binaries are produced for each release:

| OS | x86_64 | aarch64 |
|----|--------|---------|
| Linux | ✅ | ✅ |
| macOS | ✅ | ✅ |
| Windows | ✅ | ✅* |

\* `aarch64-pc-windows-msvc` is best-effort; see the release notes if a build
is missing.

## License

MIT — see [LICENSE](LICENSE).
