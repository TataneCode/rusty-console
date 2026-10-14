# Rusty Console - Implementation Plan

## Executive Summary

This plan outlines the implementation of `rusty_console`, a Rust TUI application for Docker management that ports the existing C# FancyContainerConsole. The application follows Domain-Driven Design (DDD), SOLID principles, and a screaming architecture with feature-based organization.

---

## 1. Implementation Phases

### Phase 1: Project Setup and Domain Layer
- Set up Cargo.toml with dependencies
- Create directory structure
- Implement Container domain (establishes patterns)
- Implement Volume domain
- Implement Image domain
- Write domain unit tests

### Phase 2: Application Layer
- Container module (traits, DTOs, mapper, service)
- Volume module
- Image module
- Application-level error types

### Phase 3: Infrastructure Layer
- Docker client setup with bollard
- Container adapter
- Volume adapter (includes size calculation)
- Image adapter
- Infrastructure mappers

### Phase 4: UI Layer
- App state and event loop
- Common widgets (table selection, theme, keys)
- Container presenter and views
- Volume presenter and views
- Image presenter and views
- Main menu navigation

### Phase 5: Integration and Polish
- Wire everything in main.rs
- Error handling
- Cross-platform testing

---

## 2. Dependencies (Cargo.toml)

```toml
[package]
name = "rusty_console"
version = "0.1.0"
edition = "2021"

[dependencies]
# TUI Framework
ratatui = "0.29"
crossterm = "0.28"

# Docker API
bollard = "0.18"

# Async Runtime
tokio = { version = "1.43", features = ["full"] }

# Async traits
async-trait = "0.1"

# Error Handling
anyhow = "1.0"
thiserror = "2.0"

# Date/Time
chrono = { version = "0.4", features = ["serde"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

[dev-dependencies]
mockall = "0.13"
tokio-test = "0.4"
```

---

## 3. File Structure

```
rusty_console/
├── Cargo.toml
├── .clinerules
├── PLAN.md
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── domain/
│   │   ├── mod.rs
│   │   ├── error.rs
│   │   ├── container/
│   │   │   ├── mod.rs
│   │   │   ├── entity.rs
│   │   │   ├── value_objects.rs
│   │   │   └── state.rs
│   │   ├── volume/
│   │   │   ├── mod.rs
│   │   │   ├── entity.rs
│   │   │   └── value_objects.rs
│   │   └── image/
│   │       ├── mod.rs
│   │       ├── entity.rs
│   │       └── value_objects.rs
│   ├── application/
│   │   ├── mod.rs
│   │   ├── error.rs
│   │   ├── container/
│   │   │   ├── mod.rs
│   │   │   ├── dto.rs
│   │   │   ├── mapper.rs
│   │   │   ├── service.rs
│   │   │   └── traits.rs
│   │   ├── volume/
│   │   │   ├── mod.rs
│   │   │   ├── dto.rs
│   │   │   ├── mapper.rs
│   │   │   ├── service.rs
│   │   │   └── traits.rs
│   │   └── image/
│   │       ├── mod.rs
│   │       ├── dto.rs
│   │       ├── mapper.rs
│   │       ├── service.rs
│   │       └── traits.rs
│   ├── infrastructure/
│   │   ├── mod.rs
│   │   ├── error.rs
│   │   ├── docker/
│   │   │   └── mod.rs
│   │   ├── container/
│   │   │   ├── mod.rs
│   │   │   ├── adapter.rs
│   │   │   └── mapper.rs
│   │   ├── volume/
│   │   │   ├── mod.rs
│   │   │   ├── adapter.rs
│   │   │   └── mapper.rs
│   │   └── image/
│   │       ├── mod.rs
│   │       ├── adapter.rs
│   │       └── mapper.rs
│   └── ui/
│       ├── mod.rs
│       ├── app.rs
│       ├── event.rs
│       ├── container/
│       │   ├── mod.rs
│       │   ├── presenter.rs
│       │   ├── view.rs
│       │   └── actions.rs
│       ├── volume/
│       │   ├── mod.rs
│       │   ├── presenter.rs
│       │   ├── view.rs
│       │   └── actions.rs
│       ├── image/
│       │   ├── mod.rs
│       │   ├── presenter.rs
│       │   ├── view.rs
│       │   └── actions.rs
│       └── common/
│           ├── mod.rs
│           ├── widgets.rs
│           ├── theme.rs
│           └── keys.rs
└── tests/
    └── integration/
        └── mod.rs
```

---

## 4. Key Structs and Traits

### Domain Layer

```rust
// Container state enum
pub enum ContainerState {
    Running, Paused, Stopped, Exited, Dead, Created, Removing, Restarting,
}

// Value objects
pub struct ContainerId(String);
pub struct VolumeId(String);
pub struct ImageId(String);
pub struct PortMapping { pub private_port: u16, pub public_port: u16, pub protocol: String }
pub struct NetworkInfo { pub name: String, pub ip_address: String }

// Entities with business logic
pub struct Container { /* ... */ }
impl Container {
    pub fn is_running(&self) -> bool;
    pub fn can_be_started(&self) -> bool;
    pub fn can_be_stopped(&self) -> bool;
}

pub struct Volume { /* ... */ }
impl Volume {
    pub fn can_be_deleted(&self) -> bool; // !self.in_use
}

pub struct Image { /* ... */ }
impl Image {
    pub fn can_be_deleted(&self) -> bool;
    pub fn full_name(&self) -> String;
}
```

### Application Layer

```rust
// Traits (interfaces)
#[async_trait]
pub trait ContainerRepository: Send + Sync {
    async fn get_containers(&self) -> Result<Vec<Container>, AppError>;
    async fn get_logs(&self, id: &str) -> Result<String, AppError>;
    async fn start_container(&self, id: &str) -> Result<(), AppError>;
    async fn stop_container(&self, id: &str) -> Result<(), AppError>;
    async fn delete_container(&self, id: &str) -> Result<(), AppError>;
}

#[async_trait]
pub trait VolumeRepository: Send + Sync {
    async fn get_volumes(&self) -> Result<Vec<Volume>, AppError>;
    async fn get_volume_by_name(&self, name: &str) -> Result<Option<Volume>, AppError>;
    async fn delete_volume(&self, name: &str) -> Result<(), AppError>;
}

#[async_trait]
pub trait ImageRepository: Send + Sync {
    async fn get_images(&self) -> Result<Vec<Image>, AppError>;
    async fn get_image_by_id(&self, id: &str) -> Result<Option<Image>, AppError>;
    async fn delete_image(&self, id: &str) -> Result<(), AppError>;
}

// DTOs
pub struct ContainerDto { pub id: String, pub name: String, /* ... */ }
pub struct VolumeDto { pub id: String, pub name: String, pub size: i64, pub in_use: bool, /* ... */ }
pub struct ImageDto { pub id: String, pub repository: String, pub tag: String, /* ... */ }
```

### UI Layer

```rust
// Application state
pub enum Screen {
    MainMenu,
    ContainerList,
    ContainerLogs { container_id: String },
    ContainerDetails { container: ContainerDto },
    VolumeList,
    ImageList,
    ImageDetails { image: ImageDto },
}

pub struct App {
    pub screen: Screen,
    pub should_quit: bool,
    pub selected_index: Option<usize>,
    pub containers: Vec<ContainerDto>,
    pub volumes: Vec<VolumeDto>,
    pub images: Vec<ImageDto>,
    // services...
}

// Key bindings
pub enum AppAction {
    Quit, Back, NavigateUp, NavigateDown, Select,
    ViewLogs, StartStop, Delete, ViewDetails,
}
```

---

## 5. Error Handling Strategy

```rust
// Domain errors
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Invalid container ID")]
    InvalidContainerId,
    #[error("Invalid volume ID")]
    InvalidVolumeId,
    #[error("Invalid image ID")]
    InvalidImageId,
}

// Application errors
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Domain error: {0}")]
    Domain(#[from] DomainError),
    #[error("Repository error: {0}")]
    Repository(String),
    #[error("Not found: {0}")]
    NotFound(String),
}

// Infrastructure errors convert to AppError
```

---

## 6. Key Architectural Decisions

1. **Layer Separation**: Domain has zero external deps, infrastructure owns bollard
2. **Async Throughout**: All I/O via tokio, traits use `async_trait`
3. **Docker Connection**: `bollard::Docker::connect_with_socket_defaults()` handles platform
4. **Volume Sizes**: Use `bollard.df()` API for volume size data
5. **In-Use Detection**: Check container mounts for volumes, ImageID for images

---

## 7. Key Bindings (Matching C# Version)

| Key | Action |
|-----|--------|
| `↑`/`k` | Navigate up |
| `↓`/`j` | Navigate down |
| `Enter` | Select/Confirm |
| `Esc`/`q` | Back/Quit |
| `l` | View logs (containers) |
| `s` | Start/Stop (containers) |
| `d` | Delete (with confirmation) |
| `c` | View details |

---

## 8. Verification

1. `cargo build` - Project compiles
2. `cargo test` - Unit tests pass
3. `cargo run` - TUI launches and connects to Docker
4. Manual testing:
   - Navigate main menu
   - List containers/volumes/images
   - Start/stop a container
   - View container logs
   - Delete a stopped container (with confirmation)
