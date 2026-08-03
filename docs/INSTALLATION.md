# Installation

## Binary packages

### Arch Linux (AUR)

```bash
yay -S raskolnikov
```

The AUR package installs the binary, man page, shell completions, and the
`rsk`/`rk` symlinks.

### Kali / Debian / Ubuntu

Signed `.deb` is the recommended install method:

```bash
wget https://github.com/dysshanks/Raskolnikov/releases/download/v0.1.0/raskolnikov_0.1.0_amd64.deb
wget https://github.com/dysshanks/Raskolnikov/releases/download/v0.1.0/raskolnikov_0.1.0_amd64.deb.sig
gpg --verify raskolnikov_0.1.0_amd64.deb.sig raskolnikov_0.1.0_amd64.deb
sudo dpkg -i raskolnikov_0.1.0_amd64.deb
```

### Docker

```bash
docker pull ghcr.io/dysshanks/raskolnikov:latest
docker run -it --rm ghcr.io/dysshanks/raskolnikov:latest
```

Images are published to GitHub Container Registry. Tagged releases publish
`latest` and semver tags; every push to main publishes a `nightly` tag. The
image bundles nmap, gobuster, sqlmap, hydra, whatweb, john, and hashcat.

### Nix

```bash
nix run github:dysshanks/Raskolnikov
```

Or add to your `flake.nix`:

```nix
inputs.raskolnikov.url = "github:dysshanks/Raskolnikov";
```

A dev shell with all tools is available via `nix develop`.

## Build from source

Requires Rust stable 1.80+ (see `rust-toolchain.toml`).

```bash
git clone https://github.com/dysshanks/Raskolnikov.git
cd Raskolnikov
cargo build --release
sudo install -m755 target/release/raskolnikov /usr/local/bin/raskolnikov
sudo ln -s /usr/local/bin/raskolnikov /usr/local/bin/rsk
sudo ln -s /usr/local/bin/raskolnikov /usr/local/bin/rk
```

## Dependencies

All runtime tools are optional — missing tools are reported at startup but do
not prevent Raskolnikov from running. See [TOOLS.md](TOOLS.md) for a full list
and per-distro installation instructions.

## Environment

| Variable | Purpose |
|----------|---------|
| `RASKOLNIKOV_CONFIG` | Override the config file path |
| `RASKOLNIKOV_DATA` | Override the session data directory |
| `RASKOLNIKOV_LOG` | Enable tracing to stderr with a RUST_LOG-style filter |

API keys (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, etc.) are read from the
environment — never stored in config files. See [CONFIGURATION.md](CONFIGURATION.md).
