# Copilot Instructions

Rust TUI application for managing Docker containers, volumes, and images. Port of FancyContainerConsole (C#/.NET) to Rust.

## Commands

```bash
cargo check                # Fast compilation check
cargo build                # Build debug binary
cargo build --release      # Build optimized binary
cargo fmt                  # Format code
cargo clippy -- -D warnings  # Lint (CI treats warnings as errors)
cargo test                 # Run all tests
cargo test <name>          # Run a single test by name
cargo run                  # Run the application (requires a running Docker daemon)
```

## Architecture

Domain-Driven Design with strict inward dependency flow. No outer layer may import an inner one.

```
ui  →  application  →  domain  ←  infrastructure
```

### Layers

- **`domain/`** — Entities (`Container`, `Volume`, `Image`), value objects (`ContainerId`, `PortMapping`, etc.), domain errors. Pure Rust with zero external crates.
- **`application/`** — Repository traits (the inversion-of-control boundary), stateless services, DTOs, and domain→DTO mappers.
- **`infrastructure/`** — Bollard adapters implementing application traits. Infrastructure mappers convert Docker API types to domain entities.
- **`ui/`** — Ratatui rendering. Each domain area follows a **presenter/view/actions triad**:
  - `presenter.rs` — mutable display state (selection, scroll offset, loaded data)
  - `view.rs` — pure rendering functions that take references and produce widgets
  - `actions.rs` — thin async wrappers around application services

### Data Flow

```
Docker API → Infrastructure Mapper → Domain Entity → Application Mapper → DTO → Presenter → View
```

### Dependency Injection

`main.rs` manually wires all layers bottom-up with no DI framework:

```
DockerClient → Arc<*Adapter> → *Service → *Actions → App::new(...)
```

Adapters are wrapped in `Arc` for shared ownership across async tasks. Services receive `Arc<dyn Trait>`.

### UI State Machine

`ui/app.rs` owns a `Screen` enum that drives rendering and key handling. Overlay states (`confirm_dialog`, `error_message`) are evaluated before screen-specific handlers. `ui/common/keys.rs` maps raw `KeyEvent`s to `AppAction` variants.

## Key Conventions

### Error Handling — Three-Layer Chain

Each layer defines its own error enum with `thiserror`:
- `DomainError` — pure domain invariant violations
- `AppError` — wraps `DomainError` via `#[from]`, adds `Repository`, `NotFound`, `Connection` variants with builder methods
- `InfraError` — converts `bollard::errors::Error`, then converts into `AppError` via `From` impl

The UI catches `AppError` and sets `App::error_message` for display.

### Repository Traits

Defined in `application/*/traits.rs`. All use `#[async_trait]` with `Send + Sync` bounds. Every method returns `Result<T, AppError>`.

### Mappers Are Stateless Structs

Both infrastructure and application mappers are unit structs with associated functions (no `self`). Infrastructure mappers return `Option<Entity>` using `filter_map` to skip malformed Docker API responses.

### Entity Builder Pattern

Domain entities use `with_*` methods for optional fields:
```rust
Container::new(id, name, image, state, status, created)
    .with_ports(ports)
    .with_networks(networks)
    .with_mounts(mounts)
```

### Testing

- Unit tests live in `#[cfg(test)]` modules within each file
- Tests focus on the domain layer using helper factory functions (e.g., `create_test_container(state)`)
- `mockall` and `tokio-test` are available as dev-dependencies

### Bollard 0.18 API Specifics

- `df()` takes no arguments (removed `DataUsageOptions`)
- `Port.typ` (not `Port.r#type`)
- `Port.private_port` is `u16` (not `Option<u16>`)

### Async

All I/O is async via `tokio`. Application traits use `#[async_trait]`. Infrastructure adapters implement these traits against the bollard async API.
