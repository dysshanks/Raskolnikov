# Phase 1 — Project Scaffold & Configuration

**Objective:** Establish the Rust project skeleton, toolchain configuration, repository settings, and all config file plumbing. Nothing runs yet — this is the foundation every other phase builds on.

---

## Files to Create

### Project Configuration

| File | Purpose |
|------|---------|
| `Cargo.toml` | Package manifest: name = "raskolnikov", edition 2021, 3 `[[bin]]` entries |
| `rust-toolchain.toml` | Pin Rust stable 1.75+ |
| `rustfmt.toml` | Formatting rules (`edition = 2021`, `tab_spaces = 4`) |
| `clippy.toml` | Linting config, deny warnings in CI |
| `deny.toml` | `cargo-deny` — license allowlist, advisory DB |

### Repository Settings

| File | Purpose |
|------|---------|
| `.gitignore` | `target/`, editor dirs, OS junk, test artifacts |
| `.editorconfig` | indent_style = space, indent_size = 4, eol = lf |
| `.gitattributes` | `* text=auto eol=lf`, Rust-specific diff settings |
| `LICENSE` | Apache-2.0 full text |
| `.env.example` | Document all 7 API key env vars with placeholders |

### CI/CD

| File | Purpose |
|------|---------|
| `.github/workflows/ci.yml` | Build + test (offline mocked) + clippy + fmt-check on push/PR |
| `.github/workflows/release.yml` | Build release binaries, GitHub release, attach .deb + PKGBUILD |

### Rust Source (Skeleton)

| File | Purpose |
|------|---------|
| `src/main.rs` | Entrypoint — clap arg parsing, config load, TUI launch (stub) |
| `src/config.rs` | TOML config loader, env var merge, CLI override merge, first-launch detection |
| `src/lib.rs` | Library root — all modules re-exported here for testing |

### Directory Stubs (Empty mod.rs files)

| Directory | mod.rs exports |
|-----------|----------------|
| `src/agent/` | `mod.rs` — placeholder |
| `src/tools/` | `mod.rs` — placeholder |
| `src/ai/` | `mod.rs` — placeholder |
| `src/session/` | `mod.rs` — placeholder |
| `src/tui/` | `mod.rs` — placeholder |

---

## Dependencies (Cargo.toml)

```
clap          — CLI argument parsing (derive feature)
ratatui       — TUI framework
crossterm     — Terminal backend for ratatui
tokio         — Async runtime (features: full, process)
reqwest       — HTTP client (features: json, stream)
serde         — Serialization
serde_json    — JSON for session logging + API payloads
toml          — Config file parsing
quick-xml     — nmap XML output parsing (features: serde)
chrono        — Timestamps for session IDs and logs
tracing       — Structured logging (file + stderr, optional)
tracing-subscriber — Log formatting
```

Dev dependencies:
```
tempfile      — Temp dirs in tests
wiremock      — HTTP mock server for AI provider tests
```

---

## User Stories

| ID | Title | Description |
|----|-------|-------------|
| US-1.1 | Developer can build the project | As a developer, I can run `cargo build` and get a binary called `raskolnikov` (plus `rsk` and `rk` symlinks). |
| US-1.2 | Developer can run tests | As a developer, I can run `cargo test` and see placeholder tests pass. |
| US-1.3 | CLI accepts basic flags | As a user, I can run `rsk --version` and see the version, or `rsk --help` and see all flags. |
| US-1.4 | Config loads on startup | As a user, when I launch `rsk`, it reads `~/.config/raskolnikov/config.toml` and merges with env vars. |
| US-1.5 | First-launch detection | As a user on first run, I see a message that no config exists and defaults will be used. |
| US-1.6 | CI passes | As a maintainer, every PR triggers CI that builds, tests, lints, and formats. |

---

## Acceptance Criteria

| ID | Criterion | How to Verify |
|----|-----------|---------------|
| AC-1.1 | `cargo build --release` produces `target/release/raskolnikov` | Run the command, check file exists |
| AC-1.2 | `cargo test --all-targets` passes with exit code 0 | Run the command |
| AC-1.3 | `rsk --version` prints `raskolnikov 0.1.0` | Run binary with flag |
| AC-1.4 | `rsk --help` prints usage with all flags/subcommands | Run binary with flag |
| AC-1.5 | Setting `RASKOLNIKOV_CONFIG=/tmp/test.toml` overrides config path | Set env, run, verify config loaded from that path |
| AC-1.6 | Missing config file does not crash — defaults are used | Remove config, run binary, no panic |
| AC-1.7 | `cargo clippy -- -D warnings` passes | Run the command |
| AC-1.8 | `cargo fmt --check` passes with no diffs | Run the command |
| AC-1.9 | `.env.example` documents all 7 provider env vars | Manual review |
| AC-1.10 | CI workflow runs on every push and PR to main | Push a branch, check Actions tab |

---

## Key Design Decisions

1. **Three binary names, one source**: `Cargo.toml` defines 3 `[[bin]]` entries all pointing at `src/main.rs`. No symlinks needed during development. The binary checks `std::env::current_exe()` to determine its invoked name.

2. **Config merge order (last wins)**: Built-in defaults → `config.toml` → env vars → CLI flags. This lets operators set base config in TOML, override API keys via env (security), and override model/provider per session via CLI.

3. **Config path**: `RASKOLNIKOV_CONFIG` env var to override, fallback to `~/.config/raskolnikov/config.toml`, fallback to `/etc/raskolnikov/config.toml`.

4. **Session directory**: `RASKOLNIKOV_DATA` env var to override, fallback to `~/.local/share/raskolnikov/`.

5. **Library root**: `src/lib.rs` re-exports all modules so integration tests can import the crate as a library.

---

## Wiring Diagram (Phase 1)

```
main.rs
  ├── clap::Parser  → Args { model, provider, version }
  ├── config::load  → Config struct
  ├── session::init → create data directory
  └── tui::run      → (stub — just prints "TUI starting..." then exits)
```

## Order of Implementation

1. `Cargo.toml` + `rust-toolchain.toml`
2. `src/lib.rs` + all stub `mod.rs` files
3. `src/main.rs` — clap args, config load call
4. `src/config.rs` — Config struct, TOML deserialize, env merge, CLI merge
5. Repo config files: `.gitignore`, `.editorconfig`, `.gitattributes`, `LICENSE`, `.env.example`
6. Lint config: `rustfmt.toml`, `clippy.toml`, `deny.toml`
7. CI/CD: `.github/workflows/ci.yml`, `.github/workflows/release.yml`
8. Verify: `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt --check` all pass
