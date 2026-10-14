# Technologies

This document lists the crates used by rusty-console and the role each one plays in the project.

- Edition 2021, MSRV 1.88 (declared via `rust-version` in `Cargo.toml`)
- The binary requires a running Docker daemon; tests do not (pure domain, mocked application)

## Crates

| Crate | Version | Role |
|-------|---------|------|
| [ratatui](https://crates.io/crates/ratatui) | 0.30 | Terminal UI framework (widgets, layout, rendering) |
| [crossterm](https://crates.io/crates/crossterm) | 0.29 | Cross-platform terminal backend for ratatui |
| [bollard](https://crates.io/crates/bollard) | 0.21 | Async Docker Engine API client |
| [tokio](https://crates.io/crates/tokio) | 1.43 | Async runtime (`rt-multi-thread`, `macros`, `sync`, `time`, `process`, `fs`) |
| [async-trait](https://crates.io/crates/async-trait) | 0.1 | Enables `async fn` in trait definitions |
| [futures-util](https://crates.io/crates/futures-util) | 0.3 | `StreamExt` for consuming bollard log streams |
| [anyhow](https://crates.io/crates/anyhow) | 1.0 | Ergonomic error propagation in `main` |
| [thiserror](https://crates.io/crates/thiserror) | 2.0 | Derive-macro for typed error enums across layers |
| [chrono](https://crates.io/crates/chrono) | 0.4 | `DateTime<Utc>` timestamps on domain entities |
| [serde](https://crates.io/crates/serde) / [serde_json](https://crates.io/crates/serde_json) | 1.0 | Serialization support |

### Dev dependencies

| Crate | Version | Role |
|-------|---------|------|
| [mockall](https://crates.io/crates/mockall) | 0.14 | Auto-mock repository traits in unit tests |
| [tokio-test](https://crates.io/crates/tokio-test) | 0.4 | Async test helpers |

## Why these choices

- **ratatui + crossterm** — the de facto standard pairing for immediate-mode terminal UIs in Rust: ratatui owns widget composition and layout, crossterm abstracts terminal input/output and raw mode across Windows and Unix.
- **bollard** — the mature async Docker Engine API client; it drives every infrastructure adapter and its stream-based log API pairs naturally with `futures_util::StreamExt`.
- **tokio** — required by bollard's async API and by the `docker exec -it` child-process spawning; the `process` feature is used for exec shell access.
- **thiserror** — each layer defines its own typed error enum (see [ARCHITECTURE.md](./ARCHITECTURE.md)), re-wrapped at each outer layer. `anyhow` is confined to `main` for ergonomic top-level propagation.
- **chrono** — the domain layer uses a small number of external crates by design, limited to timestamps (`DateTime<Utc>`) and typed errors.

## Packaging

RPM packaging uses `cargo-generate-rpm`, configured through `[package.metadata.generate-rpm]` in `Cargo.toml`: the binary is installed system-wide to `/usr/bin/rusty_console` with executable bits set via `mode = "755"`.
