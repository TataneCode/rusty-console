# Rusty Console

A terminal user interface (TUI) for managing Docker containers, volumes, and images — written in Rust. This is a port of [FancyContainerConsole](https://github.com) (originally in C#/.NET) to Rust.

## Features

- **Containers** — list all containers regardless of state, start/stop, view real-time logs, inspect details, delete (with force option for running containers)
- **Volumes** — list all volumes, detect which ones are in use, delete unused volumes
- **Images** — list all images with usage status, inspect details, delete (with force option for in-use images)
- Confirmation dialogs for all destructive actions
- Error popups with dismiss-on-keypress behaviour

## Prerequisites

- Rust toolchain (`rustup` recommended)
- A running Docker daemon (accessible via the default socket)

## Build & Run

```bash
# Run in development mode
cargo run

# Build an optimized binary
cargo build --release
./target/release/rusty_console
```

## Key Bindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Enter` | Select / confirm |
| `Esc` / `q` | Go back / quit |
| `l` | View container logs |
| `s` | Start or stop a container |
| `c` | View details |
| `d` | Delete (opens confirmation dialog) |
| `r` | Refresh list |
| `Ctrl+U` | Scroll up (in log view) |
| `Ctrl+D` | Scroll down (in log view) |

---

## Architecture

The project follows **Domain-Driven Design (DDD)** with a clean, strictly inward dependency graph:

```
ui  →  application  →  domain  ←  infrastructure
```

No outer layer may be imported by an inner one. The `domain` layer has zero third-party dependencies.

### Layers

#### `domain/`
The core of the application. Contains pure Rust business logic with no external crates.

- **Entities** — `Container`, `Volume`, `Image`. Each entity encapsulates its own business rules (e.g. `Container::can_be_started()`, `Container::uses_volume()`).
- **Value Objects** — `ContainerId`, `PortMapping`, `NetworkInfo`, `MountInfo`, `VolumeId`, `ImageId`. Immutable, identity-free wrappers.
- **Domain state** — `ContainerState` enum (`Running`, `Paused`, `Exited`, `Dead`, …) with derived predicates used to drive UI affordances.
- **Errors** — `domain/error.rs`, re-wrapped at each outer layer.

#### `application/`
Orchestration layer. Defines the contracts the rest of the system depends on and provides use-case implementations.

- **Repository traits** — `ContainerRepository`, `VolumeRepository`, `ImageRepository` (in `traits.rs` of each sub-module). These are the inversion-of-control boundaries: the application layer calls them; the infrastructure layer implements them.
- **Services** — `ContainerService`, `VolumeService`, `ImageService`. Thin use-case coordinators that call a repository and return DTOs.
- **DTOs** — `ContainerDto`, `VolumeDto`, `ImageDto`, `ContainerLogsDto`. Plain data structs crossing the application→UI boundary.
- **Mappers** — `ContainerMapper`, etc. Convert domain entities into DTOs.

#### `infrastructure/`
Adapts the Docker daemon API to the application's repository traits using [bollard](https://crates.io/crates/bollard).

- **Adapters** — `ContainerAdapter`, `VolumeAdapter`, `ImageAdapter`. Each implements the corresponding repository trait. Log streaming uses `futures_util::StreamExt` to consume bollard's async stream.
- **Mappers** — `ContainerInfraMapper`, etc. Convert raw bollard API types into domain entities.
- **`DockerClient`** — a thin newtype wrapper around `bollard::Docker`, shared via `Arc<T>` across all adapters.

#### `ui/`
Presentation layer built on [ratatui](https://crates.io/crates/ratatui).

- **`app.rs`** — `App` struct owns a `Screen` state-machine enum and runs the main event loop. Overlay states (`confirm_dialog`, `error_message`) are evaluated before any screen-specific handler.
- **Screens** — `MainMenu`, `ContainerList`, `ContainerLogs`, `ContainerDetails`, `VolumeList`, `ImageList`, `ImageDetails`.
- **Per-domain triad** (e.g. `ui/container/`):
  - `presenter.rs` — holds display state (selected item, scroll offset, loaded data)
  - `view.rs` — pure ratatui widget composition functions
  - `actions.rs` — async wrappers around application services; called from `app.rs` event handlers
- **`common/`** — shared widgets, colour theme, and `keys.rs` which maps raw `KeyEvent`s to `AppAction` variants.

### Data Flow

```
Docker daemon
    │
    ▼  bollard async API
Infrastructure Mapper  →  Domain Entity
                               │
                               ▼  Application Mapper
                             DTO
                               │
                               ▼  Presenter
                          Display state  →  View (ratatui widgets)
```

### Dependency Injection

`main.rs` manually wires every layer at startup:

```
DockerClient
  └─► Arc<ContainerAdapter>  →  ContainerService  →  ContainerActions
  └─► Arc<VolumeAdapter>     →  VolumeService     →  VolumeActions
  └─► Arc<ImageAdapter>      →  ImageService      →  ImageActions
                                                         │
                                                         ▼
                                                       App::new(…)
```

---

## Crates

| Crate | Version | Role |
|-------|---------|------|
| [ratatui](https://crates.io/crates/ratatui) | 0.29 | Terminal UI framework (widgets, layout, rendering) |
| [crossterm](https://crates.io/crates/crossterm) | 0.28 | Cross-platform terminal backend for ratatui |
| [bollard](https://crates.io/crates/bollard) | 0.18 | Async Docker Engine API client |
| [tokio](https://crates.io/crates/tokio) | 1.43 | Async runtime (`full` features) |
| [async-trait](https://crates.io/crates/async-trait) | 0.1 | Enables `async fn` in trait definitions |
| [futures-util](https://crates.io/crates/futures-util) | 0.3 | `StreamExt` for consuming bollard log streams |
| [anyhow](https://crates.io/crates/anyhow) | 1.0 | Ergonomic error propagation in `main` |
| [thiserror](https://crates.io/crates/thiserror) | 2.0 | Derive-macro for typed error enums across layers |
| [chrono](https://crates.io/crates/chrono) | 0.4 | `DateTime<Utc>` timestamps on domain entities |
| [serde](https://crates.io/crates/serde) / [serde_json](https://crates.io/crates/serde_json) | 1.0 | Serialization support |
| [mockall](https://crates.io/crates/mockall) *(dev)* | 0.13 | Auto-mock repository traits in unit tests |
| [tokio-test](https://crates.io/crates/tokio-test) *(dev)* | 0.4 | Async test helpers |

---

## Testing

Unit tests live alongside production code in `#[cfg(test)]` modules. Repository traits are mocked with `mockall` so services can be tested in isolation without a Docker daemon.

```bash
cargo test              # run all tests
cargo test <name>       # run a specific test by name
```
