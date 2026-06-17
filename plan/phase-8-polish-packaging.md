# Phase 8 — Polish & Packaging

**Objective:** Complete the CLI reference, add documentation files, build packaging for Arch (AUR) and Debian-based distros, add shell completions, and run a full verification pass against the spec.

---

## Files to Create / Modify

| File | Status | Purpose |
|------|--------|---------|
| `README.md` | Rewrite | Project overview, quick start, badges |
| `CONTRIBUTING.md` | Create | Contribution guidelines per spec |
| `packaging/PKGBUILD` | Create | Arch Linux AUR package |
| `packaging/raskolnikov.spec` | Create | RPM spec (optional stretch) |
| `packaging/raskolnikov.deb.spec` | Create | .deb packaging config |
| `packaging/raskolnikov.1` | Create | Man page |
| `completions/raskolnikov.bash` | Create | Bash completions |
| `completions/raskolnikov.zsh` | Create | Zsh completions |
| `completions/raskolnikov.fish` | Create | Fish completions |
| `src/main.rs` | Modify | Verify all CLI flags/subcommands match spec exactly |

---

## CLI Reference Verification

Cross-check the CLI against the spec:

```
USAGE
  raskolnikov [options]
  rsk [options]
  rk [options]

OPTIONS
  --version             Print version and exit
  --model <name>        Override default model for this session
  --provider <name>     Override default provider for this session

SUBCOMMANDS
  rsk sessions                  list past sessions
  rsk sessions show <id>        print conversation transcript
  rsk sessions findings <id>    print findings summary
  rsk sessions log <id>         dump raw JSON log
  rsk sessions prune <flags>    remove old sessions
  rsk config                    show current config
  rsk config provider <p>       set AI provider
  rsk config model <m>          set default model
  rsk config set <key> <val>    set an arbitrary config value
  rsk tools                     check tool availability and versions
  rsk help                      show help
```

Every flag and subcommand must work. Non-TUI subcommands (sessions, config, tools) exit immediately after printing output.

---

## README.md

```markdown
# Raskolnikov

A terminal-native, markdown-driven AI security operating environment.

[!badge CI] [!badge License] [!badge Version]

## Quick Start

```bash
# Install
yay -S raskolnikov           # Arch
sudo dpkg -i raskolnikov.deb  # Debian/Ubuntu/Kali
cargo install raskolnikov     # from source

# Launch
rsk
```

## Documentation
- [SPEC.md](spec-mvp.md) — Full specification
- [CONTRIBUTING.md](CONTRIBUTING.md) — How to contribute

## License
Apache-2.0
```

---

## CONTRIBUTING.md

Based on spec section:

```markdown
# Contributing to Raskolnikov

## Most Valuable Contributions (Alpha)

1. **Bug reports** — especially on non-Arch distros
2. **Tool output parsing** — nmap XML edge cases, sqlmap output variants
3. **System prompt tuning** — better agent behaviour on smaller local models
4. **Wordlist path detection** — common paths across distros

## Development Setup

```bash
git clone https://github.com/raskolnikov-security/raskolnikov
cd raskolnikov
cargo build
cargo test
```

## Pull Request Process

1. All tests pass: `cargo test --all-targets`
2. No clippy warnings: `cargo clippy -- -D warnings`
3. Code formatted: `cargo fmt --check`
4. CI must be green
```

---

## Packaging

### AUR (PKGBUILD)

```bash
# Maintainer: Raskolnikov Security Team
# Contributor: ...

pkgname=raskolnikov
pkgver=0.1.0
pkgrel=1
pkgdesc="Terminal-native AI security operating environment"
arch=('x86_64')
url="https://github.com/raskolnikov-security/raskolnikov"
license=('Apache-2.0')
depends=('nmap' 'gobuster' 'nikto' 'sqlmap' 'ollama')
source=("$url/releases/download/v$pkgver/raskolnikov-v$pkgver-x86_64-linux.tar.gz")
sha256sums=('...')

package() {
  install -Dm755 raskolnikov "$pkgdir/usr/bin/raskolnikov"
  ln -s raskolnikov "$pkgdir/usr/bin/rsk"
  ln -s raskolnikov "$pkgdir/usr/bin/rk"
  install -Dm644 raskolnikov.1 "$pkgdir/usr/share/man/man1/raskolnikov.1"
  install -Dm644 completions/* "$pkgdir/usr/share/bash-completion/completions/"
}
```

### .deb Packaging

Build process:

```bash
# packaging/build-deb.sh
cargo build --release
mkdir -p deb/usr/bin deb/usr/share/man/man1 deb/usr/share/bash-completion/completions
cp target/release/raskolnikov deb/usr/bin/
ln -s raskolnikov deb/usr/bin/rsk
ln -s raskolnikov deb/usr/bin/rk
gzip -c packaging/raskolnikov.1 > deb/usr/share/man/man1/raskolnikov.1.gz
cp completions/* deb/usr/share/bash-completion/completions/
dpkg-deb --build deb raskolnikov_0.1.0_amd64.deb
```

Dependencies in `.deb`: `nmap`, `gobuster`, `nikto`, `sqlmap` (recommends: `ollama`).

### Signing

```bash
gpg --detach-sign raskolnikov_0.1.0_amd64.deb
```

---

## Shell Completions

Generated using clap's completion generation:

```rust
// In build.rs or as a separate tool
use clap_complete::{generate_to, Shell};
// Generate for bash, zsh, fish
```

---

## Man Page (raskolnikov.1)

```roff
.TH RASKOLNIKOV 1 "2025-06-16" "0.1.0" "Raskolnikov Manual"
.SH NAME
raskolnikov \- terminal-native AI security operating environment
.SH SYNOPSIS
.B raskolnikov
[--model \fImodel\fR] [--provider \fIprovider\fR] [--version]
.SH DESCRIPTION
Raskolnikov is a terminal-native, markdown-driven AI security operating
environment for penetration testing and CTF challenges.
.SH OPTIONS
.TP
.B --version
Print version and exit.
.TP
.B --model \fImodel\fR
Override the default AI model for this session.
.TP
.B --provider \fIprovider\fR
Override the default AI provider for this session.
...
```

---

## Spec Compliance Checklist

Run through every section of `spec-mvp.md` and verify:

| Spec Section | Verification |
|-------------|-------------|
| Agent Shell — `rsk`/`rk` aliases | Confirm all 3 binary names work |
| Interaction Model — free-form input | Test vague, specific, short, long inputs |
| Interaction Model — mid-session redirect | Verify agent pivots |
| Interaction Model — always-on confirm | Verify 100% gate rate |
| Interaction Model — `/quit` | Verify prompt + save |
| Tool Integrations — all 4 tools | Each runs with correct flags |
| Tool Integrations — missing tool warning | Uninstall, verify graceful handling |
| AI Provider — resolution order | Verify each provider resolves correctly |
| AI Provider — context management | Long session triggers summarisation |
| TUI — all panels and keybindings | Full keybinding walkthrough |
| Session — log, transcript, findings | All files written correctly |
| Session — session commands | list, show, findings, log, prune work |
| Config — all TOML sections | Every field parsed and applied |
| Config — API keys from env only | No keys in config.toml or logs |
| Error Handling — all scenarios | Each scenario tested |
| Security — target sanitisation | Metacharacters blocked |
| Security — proxy support | Proxy applied to reqwest |
| Installation — AUR, .deb, source | Each method installs correctly |

---

## User Stories

| ID | Title | Description |
|----|-------|-------------|
| US-8.1 | All three binary names work | As a user, `raskolnikov`, `rsk`, and `rk` all launch the application |
| US-8.2 | Help output is correct | As a user, `rsk --help` shows all flags and subcommands matching the spec |
| US-8.3 | Man page available | As a user, `man raskolnikov` displays the manual |
| US-8.4 | Tab completion works | As a user, typing `rsk se` and pressing Tab completes to `rsk sessions` |
| US-8.5 | Install via AUR | As an Arch user, `yay -S raskolnikov` installs and works |
| US-8.6 | Install via .deb | As a Debian user, `dpkg -i` installs with dependencies |

---

## Acceptance Criteria

| ID | Criterion | How to Verify |
|----|-----------|---------------|
| AC-8.1 | `raskolnikov --version` prints `raskolnikov 0.1.0` | Run command |
| AC-8.2 | `rsk --version` prints same | Run command |
| AC-8.3 | `rk --version` prints same | Run command |
| AC-8.4 | `rsk --help` matches spec exactly | Compare output to spec |
| AC-8.5 | `rsk tools` lists available tools with versions | Run command |
| AC-8.6 | `rsk config` prints current config | Run command |
| AC-8.7 | `rsk config provider anthropic` updates config file | Run, check config.toml |
| AC-8.8 | Man page renders with correct sections | `man ./packaging/raskolnikov.1` |
| AC-8.9 | Bash completion generates for `rsk` | Source completions, test Tab |
| AC-8.10 | `PKGBUILD` builds successfully | `makepkg -i` in packaging/ |
| AC-8.11 | `.deb` builds and installs | `dpkg-deb --build` then `dpkg -i` |
| AC-8.12 | Full spec compliance checklist passes | Walk through every spec requirement |
| AC-8.13 | `cargo test --all-targets` passes | Run tests |
| AC-8.14 | `cargo clippy -- -D warnings` passes | Run clippy |
| AC-8.15 | `cargo fmt --check` passes | Run fmt check |

---

## Order of Implementation

1. README.md rewrite
2. CONTRIBUTING.md
3. Man page
4. Shell completions (via clap_complete)
5. AUR PKGBUILD
6. .deb packaging script + signing
7. Full spec compliance walkthrough and fixes
8. Final `cargo test`, `cargo clippy`, `cargo fmt --check`
