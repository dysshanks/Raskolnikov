# Raskolnikov — Agent Guide for Claude Code

## Project
Raskolnikov is a terminal-native, markdown-driven AI security operating environment for authorised penetration testing and CTF competitions. Written in Rust.

## Naming conventions
- Structs: UpperCamelCase (`Config`, `AgentShell`, `OllamaProvider`)
- Enums: UpperCamelCase (`AppState`, `Role`, `ProviderKind`)
- Functions/methods: snake_case (`load`, `add_finding`, `parse_nmap_xml`)
- Variables: snake_case (`tool_names`, `current_port`)
- Constants: SCREAMING_SNAKE_CASE (`BUILTIN_WORDLIST`)
- Unit structs as namespaces: `PromptBuilder`, `Transcript`, `FindingsExport`

## Module organisation
- Top-level modules declared in `src/lib.rs` as `pub mod`
- Each sub-module's `mod.rs` contains only `pub mod` declarations — no re-exports
- Reference via full path: `crate::tools::nmap::NmapPort`
- Flat within each module — no nesting beyond 2 levels
- Module structure must match `spec-mvp.md` Project Structure section

## Code style
- All struct fields are public — direct field access, no getters/setters
- Constructors via `::new(...)` for stateful objects
- `#[derive(Debug, Clone)]` on data structs
- `#[derive(Debug, Clone, Serialize, Deserialize)]` on config structs with `#[serde(default)]`
- Imports: std → external crates → crate-internal, grouped with blank lines
- No wildcard imports except `use super::*` in tests
- Prefer match over if-let chains
- Prefer iterators over explicit loops where clearer

## Error handling
- Use `thiserror` for error enums
- Use `crate::Result<T, E = Error>` type aliases
- Contextualise errors with `map_err` / `context` rather than raw `?`
- `eprintln!` for startup warnings only; `tracing` for runtime diagnostics
- No `unwrap()` in production code — use `?` or `expect("message")`

## Async patterns
- Tokio with `features = ["full"]`
- Async I/O via `tokio::process`, `tokio::io`, `tokio::time`, `tokio::select!`
- Interrupt signalling via `tokio::sync::watch::channel`
- HTTP via `reqwest` — no SDKs
- Provider trait uses `#[async_trait]`

## Testing
- Inline tests: `#[cfg(test)] mod tests { use super::*; }` in every source file
- No separate `tests/` directory
- Async tests use `#[tokio::test]`
- Mock HTTP with `wiremock`
- Mock filesystem with `tempfile`
- Mock env vars with `temp-env`
- Tests must be deterministic — no real tools or AI providers
- Cover: happy path, error cases, edge cases (empty output, duplicates, timeouts)

## Config
- TOML-based with serde. All fields use `#[serde(default)]` with standalone `fn default_*()`
- API keys from environment variables only — never in config files or logs
- Config merge: built-in defaults → config.toml → env vars → CLI flags

## Documentation
- `spec-mvp.md` is the authoritative spec — keep in sync with implementation
- Rustdoc `///` on public API items
- `//` comments on non-trivial internal logic (why, not what)

## Before submitting
Run: `cargo build && cargo test && cargo clippy && cargo fmt --check`
Fix all warnings. No dead code.

## Security
- Sanitise shell arguments — never pass user input raw to shell commands
- Never log API keys, session data, or findings
- Session files get 0600 permissions. No telemetry. No phone-home.
- The tool is for **authorised** security testing only. Developers assume no liability for misuse.
