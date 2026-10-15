# Architecture

rusty-console follows **Domain-Driven Design (DDD)** with a clean, strictly inward dependency graph:

```
04_presentation  →  02_application  →  01_domain  ←  03_infrastructure
```

No outer layer may be imported by an inner one. The `domain` layer remains the business core, but it currently uses a small number of external crates for timestamps and typed errors (`chrono` and `thiserror`).

## Layer-first structure

The codebase uses a **numbered, layer-first** layout to make the architecture easier to read:

```text
src/
  01_domain/
  02_application/
  03_infrastructure/
  04_presentation/
```

Rust module names remain valid identifiers (`domain`, `application`, `infrastructure`, `presentation`) and are mapped onto these numbered directories with `#[path = "..."]`.

The numbered layer modules are the primary navigation and import surface. The dependency rules for the target structure are:

1. `01_domain` contains business concepts and invariants only.
2. `02_application` orchestrates use cases and defines ports/DTOs.
3. `03_infrastructure` contains Docker/Bollard and adapter code.
4. `04_presentation` owns TUI rendering, presenters, actions, and app flow.
5. Dependencies always point inward; outer layers may depend on inner ones, never the reverse.

The shared Docker client wrapper lives under `03_infrastructure/docker/client.rs`, and the layer error types live under:

- `01_domain/error.rs`
- `02_application/error.rs`
- `03_infrastructure/error.rs`

## Folder layout

The primary structure is layer-first, with business modules inside each layer:

```text
src/
  01_domain/
    container/
    image/
    volume/
    stack/
  02_application/
    container/
    image/
    volume/
    stack/
  03_infrastructure/
    docker/
      client.rs
      container/
      image/
      volume/
      stack/
  04_presentation/
    tui/
      container/
      image/
      volume/
      stack/
```

At the root, `shared.rs` remains as the small shared value module used across layers.

## Layers

### `01_domain/*`

The core of the application. Contains the business logic and domain concepts. It is isolated from application, infrastructure, and presentation code, but it currently depends on a small number of external crates for timestamps and typed errors.

- **Entities** — `Container`, `Volume`, `Image`, `Network`, `Stack`. Each entity encapsulates its own business rules (e.g. `Container::can_be_started()`, `Network::can_be_deleted()`).
- **Value Objects** — `ContainerId`, `PortMapping`, `NetworkInfo`, `MountInfo`, `VolumeId`, `ImageId`, `NetworkId`, `NetworkScope`. Immutable, identity-free wrappers.
- **Domain state** — `ContainerState` enum (`Running`, `Paused`, `Exited`, `Dead`, …) with derived predicates used to drive UI affordances.
- **Errors** — `01_domain/error.rs`, re-wrapped at each outer layer.

### `02_application/*`

Orchestration layer. Defines the contracts the rest of the system depends on and provides use-case implementations.

- **Repository traits** — `ContainerRepository`, `VolumeRepository`, `ImageRepository`, `StackRepository`, `NetworkRepository` (in each feature's `traits.rs`). These are the inversion-of-control boundaries: the application layer calls them; the infrastructure layer implements them.
- **Services** — `ContainerService`, `VolumeService`, `ImageService`, `StackService`, `NetworkService`. Thin use-case coordinators that call a repository and return DTOs.
- **DTOs** — `ContainerDto`, `VolumeDto`, `ImageDto`, `ContainerLogsDto`, `StackDto`, `StackContainerDto`, `NetworkDto`, `NetworkContainerDto`. Plain data structs crossing the application→UI boundary.
- **Mappers** — `ContainerMapper`, etc. Convert domain entities into DTOs.

### `03_infrastructure/*`

Adapts the Docker daemon API to the application's repository traits using [bollard](https://crates.io/crates/bollard).

- **Adapters** — `ContainerAdapter`, `VolumeAdapter`, `ImageAdapter`, `NetworkAdapter`. Each implements the corresponding repository trait. Log streaming uses `futures_util::StreamExt` to consume bollard's async stream. The network adapter enriches each listed network with an `inspect_network` call to fill IPAM and connected-container data.
- **Mappers** — `ContainerInfraMapper`, etc. Convert raw bollard API types into domain entities.
- **`docker/client.rs`** — `DockerClient`, a thin newtype wrapper around `bollard::Docker`, shared via `Arc<T>` across all adapters.

### `04_presentation/tui/*`

Presentation layer built on [ratatui](https://crates.io/crates/ratatui).

- **`tui/app`** — `App` struct owns a `Screen` state-machine enum and runs the main event loop. Overlay states (`confirm_dialog`, `popup_message`, exec shell picker) are evaluated before any screen-specific handler.
- **Screens** — `MainMenu`, `ContainerList`, `ContainerLogs`, `ContainerDetails`, `VolumeList`, `ImageList`, `ImageDetails`, `NetworkList`, `NetworkDetails`, `StackList`, `StackContainers`.
- **Per-feature triad** (e.g. `04_presentation/tui/container/`):
  - `presenter.rs` — holds display state (selected item, scroll offset, loaded data)
  - `view.rs` — pure ratatui widget composition functions
  - `actions.rs` — async wrappers around application services; called from `app.rs` event handlers
- **`tui/common/`** — shared widgets, colour theme, and `keys.rs` which maps raw `KeyEvent`s to `AppAction` variants. The `Theme::section_block` helper centralizes the homogeneous section design (rounded full borders, white borders, horizontal padding) used by the breadcrumb bar, every content block, and the action bar.

## Data flow

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

## Dependency injection

`main.rs` manually wires every layer at startup:

```
DockerClient
  ├──► Arc<ContainerAdapter>  →  ContainerService  →  ContainerActions
  ├──► Arc<VolumeAdapter>     →  VolumeService     →  VolumeActions
  ├──► Arc<ImageAdapter>      →  ImageService      →  ImageActions
  ├──► Arc<StackAdapter>      →  StackService      →  StackActions
  └──► Arc<NetworkAdapter>    →  NetworkService    →  NetworkActions
                                                         │
                                                         ▼
                                                       App::new(…)
```

## Loading spinner for async operations

The application provides visual feedback during long-running async operations (container start/stop/delete, image pull, volume prune, etc.) through a loading spinner:

- **Spinner widget**: ASCII animation (`|/-\`) with 200ms frame duration
- **Display**: replaces help text in the bottom bar with `Loading... <spinner>` during operations
- **Behaviour**:
  - Automatically shown when async operations start
  - Cleared when operations complete or fail
  - Special handling for exec shell: loading cleared before spawning the external process
  - Image pull streams layer progress (`progressDetail.current/total` from the Docker image creation API via Bollard) and renders a `docker pull`-style progress bar with downloaded/total bytes and percentage; the spinner only shows before the first progress event
- **Implementation**: `ui/common/spinner.rs`, integrated in all presenters and views

---

## Architecture decision log

This log records significant structural decisions and refactorings. Each entry is dated and append-only; new entries are added at the bottom.

### ADR-001 — Layer-first numbered layout (2026-04-26)

**Status**: accepted

**Context**: the original codebase used a feature-first layout (`src/container/`, `src/image/`, …), which made the DDD layer boundaries hard to see and navigate.

**Decision**: move to a numbered, layer-first structure (`src/01_domain/` … `src/04_presentation/`) as the primary import surface, keeping valid Rust module names via `#[path = "..."]` remapping.

**Consequences**:

1. Added the numbered layer directories and made them the primary import surface.
2. Moved the shared Docker client to `src/03_infrastructure/docker/client.rs`.
3. Moved layered error definitions to:
   - `src/01_domain/error.rs`
   - `src/02_application/error.rs`
   - `src/03_infrastructure/error.rs`
4. Decoupled the `stack` feature from `container` internals by introducing stack-local models and DTOs.
5. Moved the business slice implementations into the numbered layers:
   - domain code under `01_domain/*`
   - application code under `02_application/*`
   - Docker/infrastructure code under `03_infrastructure/docker/*`
   - TUI code under `04_presentation/tui/*`
6. Deleted the old source trees:
   - `src/container/`
   - `src/image/`
   - `src/volume/`
   - `src/stack/`
   - `src/tui/`

**Rationale**: dependencies point strictly inward and the four DDD layers are visible at a glance in the directory listing; feature modules now live inside their layer.
