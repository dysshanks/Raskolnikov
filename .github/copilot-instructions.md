# Raskolnikov — Copilot Instructions

## Language
Rust (edition 2021). Tokio async runtime. Ratatui for TUI. Apache-2.0 license.

## Naming
- Structs/enums: UpperCamelCase. Functions/variables: snake_case. Constants: SCREAMING_SNAKE_CASE.
- Unit structs as namespaces: `PromptBuilder`, `Transcript`, `FindingsExport`.

## Structure
- `src/lib.rs` declares top-level `pub mod`. Sub-module `mod.rs` has only `pub mod` — no re-exports.
- Full-path references: `crate::tools::nmap::NmapPort`. No barrel imports.
- Flat modules, max 2 levels deep. Match `spec-mvp.md`.

## Style
- Public fields, no getters/setters. `::new(...)` constructors.
- `#[derive(Debug, Clone)]` on data structs. Config structs also get `Serialize, Deserialize` with `#[serde(default)]`.
- Imports: std → external → crate, blank-line grouped. No wildcard imports except `use super::*` in tests.
- Prefer match over if-let. Prefer iterators over loops.

## Error handling
- `thiserror` enums. `crate::Result<T, E = Error>` alias. `map_err` / `context` over raw `?`.
- No `unwrap()` in production — use `?` or `expect("message")`.

## Testing
- `#[cfg(test)] mod tests { use super::*; }` in every file. `#[tokio::test]` for async.
- `wiremock`, `tempfile`, `temp-env` for mocks. No real tools/providers. Deterministic only.

## Config
- TOML + serde. `#[serde(default)]` + `fn default_*()`. API keys from env only.

## Before submitting
`cargo build && cargo test && cargo clippy && cargo fmt --check`

## Security
- Sanitise shell args. Never log API keys. Session files: 0600. No telemetry.
- Authorised security testing only. No liability for misuse.
