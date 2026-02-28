# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

A Rust TUI application for managing Docker containers, volumes, and images. Port of FancyContainerConsole (C#/.NET) to Rust.

## Commands

```bash
cargo run          # Run the application
cargo check        # Fast compilation check
cargo build        # Build debug binary
cargo build --release  # Build optimized binary
cargo fmt          # Format code
cargo clippy       # Lint
cargo test         # Run all tests
cargo test <name>  # Run a single test by name
```

## Architecture

Domain-Driven Design with strict inward dependency flow: `ui` → `application` → `domain` ← `infrastructure`. The `domain` layer has zero external dependencies.

### Layer Responsibilities

- **`domain/`** — Entities (`Container`, `Volume`, `Image`), value objects (`ContainerId`, `PortMapping`), domain errors. Pure Rust, no crates.
- **`application/`** — Repository traits (`ContainerRepository`, etc.), service implementations, DTOs, domain↔DTO mappers.
- **`infrastructure/`** — Bollard adapters implementing application traits; maps Docker API types to domain entities.
- **`ui/`** — Ratatui rendering. `app.rs` owns the `Screen` state machine and async event loop. Each domain area has a presenter (state), view (widgets), and actions (service calls).

### Data Flow

```
Docker API → Infrastructure Mapper → Domain Entity → Application Mapper → DTO → Presenter → View
```

### Dependency Injection

`main.rs` wires all layers: creates `DockerClient`, wraps it in adapters (`Arc<ContainerAdapter>`), injects into services, then into `*Actions` structs, then into `App`.

### UI State Machine

`ui/app.rs::Screen` enum drives rendering and key handling. Overlay states (`confirm_dialog`, `error_message`) are handled before screen-specific actions. `ui/common/keys.rs` maps raw key events to `AppAction` variants.

### Testing

- Unit tests live in `#[cfg(test)]` modules within each file.
- Use `mockall` to mock repository traits when testing services.
- `tokio-test` for async test helpers.

## Key Implementation Notes

### Bollard 0.18 API
- `df()` takes no arguments (removed `DataUsageOptions`)
- `Port.typ` (not `Port.r#type`)
- `Port.private_port` is `u16` (not `Option<u16>`)

### Async
All I/O is async via `tokio`. Application traits use `#[async_trait]`. Infrastructure adapters implement these traits against the bollard async API.

### Error Handling
`domain/error.rs` → `application/error.rs` (wraps domain) → `infrastructure/error.rs` (converts bollard errors). UI catches `AppError` and sets `App::error_message` for display.
