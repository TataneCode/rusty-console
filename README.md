# Rusty Console

A terminal user interface (TUI) for managing Docker containers, volumes, and images — written in Rust. This is a port of [FancyContainerConsole](https://github.com) (originally in C#/.NET) to Rust.

## Features

- **Containers** — list all containers regardless of state, start/stop, pause/unpause, restart, view real-time logs, inspect details (incl. env vars), delete (with force option for running containers), prune stopped containers, filter by name
- **Volumes** — list all volumes, detect which ones are in use, delete unused volumes, prune unused volumes, filter by name
- **Images** — list all images with usage status, inspect details, delete (with force option for in-use images), prune dangling images, filter by name
- **Stacks** — detect Docker Compose stacks from container labels, list stacks with running/total counts, drill down into a stack's containers, start/stop/remove all containers in a stack at once
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

### Global

| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Enter` | Select / confirm |
| `Esc` / `q` | Go back / quit |
| `r` | Refresh list |
| `/` | Activate filter (type to search, `Esc` to clear) |

### Containers

| Key | Action |
|-----|--------|
| `s` | Start or stop the selected container |
| `p` | Pause or unpause the selected container |
| `R` | Restart the selected container |
| `l` | View container logs |
| `c` | View container details |
| `d` | Delete (opens confirmation dialog) |
| `X` | Prune all stopped containers |
| `Ctrl+U` | Scroll up (in log view) |
| `Ctrl+D` | Scroll down (in log view) |

### Stacks

| Key | Action |
|-----|--------|
| `Enter` | Drill down into stack's containers |
| `s` | Start All containers in the selected stack |
| `S` | Stop All containers in the selected stack |

### Stack Containers (drill-down)

| Key | Action |
|-----|--------|
| `s` | Start or stop the selected container |
| `d` | Delete the selected container |
| `Ctrl+S` | Start All containers in the stack |
| `S` | Stop All containers in the stack |
| `D` | Remove All containers in the stack (force, with confirmation) |

### Volumes & Images

| Key | Action |
|-----|--------|
| `d` | Delete (opens confirmation dialog) |
| `c` | View image details |
| `X` | Prune unused volumes / dangling images |

---

## Architecture

The project follows **Domain-Driven Design (DDD)** with a clean, strictly inward dependency graph:

```
ui  →  application  →  domain  ←  infrastructure
```

No outer layer may be imported by an inner one. The `domain` layer has zero third-party dependencies.

### Folder layout

Source code is organized **feature-first**: each domain concept gets its own top-level folder containing all its layers as sub-folders.

```
src/
  container/
    domain/          Entity, ContainerState, value objects
    application/     DTO, mapper, service, repository trait
    infrastructure/  Bollard adapter and infra mapper
    ui/              Actions, presenter, view
  image/             (same structure)
  volume/            (same structure)
  stack/             (same structure — groups containers by compose label)
  errors/            DomainError, AppError, InfraError
  docker/            DockerClient (shared Bollard wrapper)
  shared.rs          PruneResultDto
  ui/
    app.rs           Screen state-machine and event loop
    event.rs         Terminal event handler
    common/          Shared widgets, colour theme, key bindings
```

### Layers

#### `*/domain/`
The core of the application. Contains pure Rust business logic with no external crates.

- **Entities** — `Container`, `Volume`, `Image`. Each entity encapsulates its own business rules (e.g. `Container::can_be_started()`, `Container::uses_volume()`).
- **Value Objects** — `ContainerId`, `PortMapping`, `NetworkInfo`, `MountInfo`, `VolumeId`, `ImageId`. Immutable, identity-free wrappers.
- **Domain state** — `ContainerState` enum (`Running`, `Paused`, `Exited`, `Dead`, …) with derived predicates used to drive UI affordances.
- **Errors** — `errors/domain.rs`, re-wrapped at each outer layer.

#### `*/application/`
Orchestration layer. Defines the contracts the rest of the system depends on and provides use-case implementations.

- **Repository traits** — `ContainerRepository`, `VolumeRepository`, `ImageRepository` (in `traits.rs` of each feature). These are the inversion-of-control boundaries: the application layer calls them; the infrastructure layer implements them.
- **Services** — `ContainerService`, `VolumeService`, `ImageService`. Thin use-case coordinators that call a repository and return DTOs.
- **DTOs** — `ContainerDto`, `VolumeDto`, `ImageDto`, `ContainerLogsDto`. Plain data structs crossing the application→UI boundary.
- **Mappers** — `ContainerMapper`, etc. Convert domain entities into DTOs.

#### `*/infrastructure/`
Adapts the Docker daemon API to the application's repository traits using [bollard](https://crates.io/crates/bollard).

- **Adapters** — `ContainerAdapter`, `VolumeAdapter`, `ImageAdapter`. Each implements the corresponding repository trait. Log streaming uses `futures_util::StreamExt` to consume bollard's async stream.
- **Mappers** — `ContainerInfraMapper`, etc. Convert raw bollard API types into domain entities.
- **`docker/`** — `DockerClient`, a thin newtype wrapper around `bollard::Docker`, shared via `Arc<T>` across all adapters.

#### `*/ui/`
Presentation layer built on [ratatui](https://crates.io/crates/ratatui).

- **`ui/app.rs`** — `App` struct owns a `Screen` state-machine enum and runs the main event loop. Overlay states (`confirm_dialog`, `error_message`) are evaluated before any screen-specific handler.
- **Screens** — `MainMenu`, `ContainerList`, `ContainerLogs`, `ContainerDetails`, `VolumeList`, `ImageList`, `ImageDetails`, `StackList`, `StackContainers`.
- **Per-feature triad** (e.g. `container/ui/`):
  - `presenter.rs` — holds display state (selected item, scroll offset, loaded data)
  - `view.rs` — pure ratatui widget composition functions
  - `actions.rs` — async wrappers around application services; called from `app.rs` event handlers
- **`ui/common/`** — shared widgets, colour theme, and `keys.rs` which maps raw `KeyEvent`s to `AppAction` variants.

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
  └─► Arc<StackAdapter>      →  StackService      →  StackActions
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
