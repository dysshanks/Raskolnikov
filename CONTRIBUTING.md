# Contributing

## Getting started

```bash
git clone <repo-url>
cd raskolnikov
cargo build
cargo test
```

Requires Rust stable with `rustfmt` and `clippy` components (see
`rust-toolchain.toml`).

## Before submitting

Run the full check suite and fix all warnings:

```bash
cargo build
cargo test
cargo clippy
cargo fmt --check
```

No warnings, no dead code, no `unwrap()` in production code.

## Code conventions

See [CLAUDE.md](CLAUDE.md) for detailed conventions. Key points:

- Structs: UpperCamelCase. Functions/variables: snake_case.
  Constants: SCREAMING_SNAKE_CASE.
- Public fields, no getters/setters. `::new(...)` constructors.
- `#[derive(Debug, Clone)]` on data structs. Config structs also get
  `Serialize, Deserialize` with `#[serde(default)]`.
- Imports: std → external crates → crate-internal, grouped with blank lines.
  No wildcard imports except `use super::*` in tests.
- Prefer match over if-let chains. Prefer iterators over explicit loops.
- Use `thiserror` for error enums. Use `crate::Result<T, E = Error>` type
  aliases. Contextualise errors with `map_err` / `context`.
- No `unwrap()` in production code — use `?` or `expect("message")`.

## Module structure

- `src/lib.rs` declares top-level modules as `pub mod`.
- Each sub-module's `mod.rs` contains only `pub mod` declarations — no
  re-exports. Reference via full path: `crate::tools::nmap::NmapPort`.
- Flat within each module — max 2 levels deep.
- Module structure must match the Project Structure section in `spec-mvp.md`.

## Testing

- Inline tests: `#[cfg(test)] mod tests { use super::*; }` in every source
  file. No separate `tests/` directory.
- Async tests use `#[tokio::test]`.
- Mock HTTP with `wiremock`. Mock filesystem with `tempfile`.
  Mock env vars with `temp-env`.
- Tests must be deterministic — no real tools or AI providers.
- Cover: happy path, error cases, edge cases (empty output, duplicates,
  timeouts).

## Security

- Sanitise shell arguments — never pass user input raw to shell commands.
- Never log API keys, session data, or findings.
- Session files get 0600 permissions.
- No telemetry. No phone-home.
- The tool is for authorised security testing only. Contributors assume no
  liability for misuse.

## Spec

`spec-mvp.md` is the authoritative specification. Keep it in sync with the
implementation. If you change behaviour, update the spec.

## Pull requests

1. One feature or fix per PR — keep them small.
2. Include tests for new functionality.
3. Update `spec-mvp.md` if the change affects documented behaviour.
4. Update `CHANGELOG.md` with the change.
5. Ensure the full check suite passes before requesting review.
