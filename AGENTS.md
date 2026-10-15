# AGENTS.md

This file provides guidance to AI coding agents when working with the rusty-console codebase. Agent roles and reusable procedures live in [AGENT_ARCHITECTURE.md](.legacy/AGENT_ARCHITECTURE.md) (`agents/`, `.agents/skills/`), project context in [CONTEXT.md](.legacy/CONTEXT.md), per-layer rules in [rules/](rules/).

## Confirmation Gate (mandatory)

Before performing ANY write action on this repository — creating or modifying files, creating a branch, committing, pushing, opening/updating a PR or merge request, merging — the agent MUST:

1. Present a concise plan (target files, commands, intended Git actions).
2. Ask the user for explicit confirmation.
3. Wait for the answer before proceeding.

This applies to every entry point: Vibe Code, Vibe CLI, and the Mistral CLI. Only pure read-only operations (reading, searching, running tests/lint/build checks) are exempt. Silence or an ambiguous answer means "do not proceed". Full procedure: [.agents/skills/confirmation-gate/SKILL.md](.agents/skills/confirmation-gate/SKILL.md).

## Project Overview

A Rust TUI application for managing Docker containers, volumes, images, and stacks. Port of FancyContainerConsole (C#/.NET) to Rust.

**Current State**: Actively maintained with recent spinner/loading feature implementation (June 2026).

## Commands

```bash
# Development
cargo check                # Fast compilation check
cargo build                # Build debug binary
cargo build --release      # Build optimized binary
cargo fmt                  # Format code
cargo clippy -- -D warnings  # Lint (CI treats warnings as errors)

# Testing
cargo test                 # Run all tests
cargo test <name>          # Run a single test by name

# Coverage
cargo llvm-cov             # Generate coverage report (requires cargo-llvm-cov)
cargo llvm-cov --open      # Open HTML coverage report

# Execution
cargo run                  # Run the application (requires a running Docker daemon)
```

## Architecture

Domain-Driven Design with strict inward dependency flow. **No outer layer may import an inner one.**

```
ui  →  application  →  domain  ←  infrastructure
```

### Layers

- **`domain/`** — Entities (`Container`, `Volume`, `Image`, `Stack`), value objects (`ContainerId`, `PortMapping`, `ImageId`, etc.), domain errors. Pure Rust with zero external crates.
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

## Spinner/Loading Feature (June 2026)

### Overview
Provides visual feedback during async operations (start/stop/delete containers, pull images, etc.) by displaying an animated spinner in the help bar.

### Components

1. **Spinner Widget** (`ui/common/spinner.rs`):
   - ASCII animation: `|/-\` sequence
   - Frame duration: 200ms per character
   - Methods: `new()`, `next()`, `current()`, `reset()`

2. **Presenter Loading State**:
   - All presenters (container, volume, image, stack) have:
     - `loading: bool` field
     - `set_loading(bool)` method
     - `is_loading()` method

3. **View Integration**:
   - All list views accept `is_loading: bool` and `spinner_char: &str` parameters
   - Help text is replaced with "Loading... <spinner>" when loading

4. **App Integration** (`ui/app.rs`):
   - `spinner: Spinner` state in `App` struct
   - `spinner_char: char` derived from current spinner frame
   - Async actions call `presenter.set_loading(true)` before operation, `false` after
   - Special handling for exec shell: loading cleared before spawning external process

5. **Behavior Rules**:
   - Spinner replaces entire help text during loading (not appended)
   - 200ms frame duration for smooth animation
   - Exec shell clears loading state before spawning (external process)
   - Image pull streams layer progress from the Docker API and renders a `docker pull`-style progress bar in the help bar (spinner only before the first progress event)
   - Error state: loading cleared and error popup shown

### Usage Pattern in Async Handlers

```rust
async fn start_container(&mut self) {
    let presenter = &mut self.container_presenter;
    presenter.set_loading(true);
    
    match self.container_actions.start_container(...).await {
        Ok(_) => {
            presenter.set_loading(false);
            // Refresh data
        }
        Err(e) => {
            presenter.set_loading(false);
            self.error_message = Some(e.to_string());
        }
    }
}
```

## Testing

### Requirements
- **Minimum Coverage**: 70% (current: ~91%)
- **Test Location**: Unit tests live in `#[cfg(test)]` modules within each file
- **Test Framework**: `mockall` for mocking, `tokio-test` for async tests

### Test Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    
    #[tokio::test]
    async fn test_something() {
        // Arrange
        let mut mock_repo = MockContainerRepository::new();
        mock_repo.expect_get_all()
            .returning(|| Ok(vec![]));
        
        // Act
        let result = service.get_all(&mock_repo).await;
        
        // Assert
        assert!(result.is_ok());
    }
}
```

### Coverage Check

Before declaring a feature complete:
1. Run `cargo test` - all tests must pass (currently 297+ tests)
2. Run `cargo llvm-cov` - check coverage is ≥70%
3. Run `cargo clippy -- -D warnings` - no warnings
4. Run `cargo fmt` - code formatted

## Bollard 0.18 API Specifics

- `df()` takes no arguments (removed `DataUsageOptions`)
- `Port.typ` (not `Port.r#type`)
- `Port.private_port` is `u16` (not `Option<u16>`)

## Async

All I/O is async via `tokio`. Application traits use `#[async_trait]`. Infrastructure adapters implement these traits against the bollard async API.

## Recent Changes (June 2026)

### Commit: feat: implement loading spinner for async operations
- Added `ui/common/spinner.rs` with Spinner widget
- Updated all presenters with loading state
- Modified all view functions to accept loading parameters
- Updated app.rs with spinner animation logic
- Added 11 new tests for spinner functionality
- Coverage maintained at 91%+

## Branching Strategy

- Main branch: `main` (stable)
- Feature branches: prefixed with feature name (e.g., `spinner_loading`)
- PRs merge into main via GitHub

## CI/CD

- GitHub Actions workflow runs on push/PR
- Checks: build, test, clippy, fmt
- Coverage threshold: 70% minimum

## File Structure Conventions

```
src/
├── 01_domain/              # Domain layer (pure Rust)
│   └── container/
│       ├── entity.rs       # Container entity
│       ├── state.rs        # State enum
│       ├── value_objects.rs
│       └── error.rs
├── 02_application/         # Application layer
│   └── container/
│       ├── traits.rs       # Repository trait
│       ├── service.rs      # Service implementation
│       ├── dto.rs          # Data transfer objects
│       └── mapper.rs       # Domain ↔ DTO mapping
├── 03_infrastructure/      # Infrastructure layer
│   └── docker/
│       └── container/
│           ├── adapter.rs   # Bollard adapter
│           └── mapper.rs    # Docker API → Domain mapping
├── 04_presentation/        # UI layer (Ratatui)
│   ├── tui/
│   │   ├── app.rs          # Main app state machine
│   │   ├── common/         # Shared UI components
│   │   │   ├── spinner.rs   # Loading spinner
│   │   │   ├── keys.rs      # Key mapping
│   │   │   ├── mod.rs       # Module exports
│   │   │   └── ...
│   │   └── container/       # Container UI
│   │       ├── presenter.rs # State
│   │       ├── view.rs      # Rendering
│   │       └── actions.rs   # Action handlers
│   └── resources/          # Localized strings
└── main.rs                 # Entry point with DI wiring
```

## Best Practices

1. **Read before editing**: Always read a file completely before modifying it
2. **Match existing style**: Indentation, naming, error handling density
3. **Minimal diff**: Only change what's necessary
4. **Test all changes**: Add tests for new functionality
5. **Keep layer boundaries clean**: No outer layer imports inner layers
6. **Use async everywhere for I/O**: All repository methods are async
7. **Document public APIs**: Add doc comments for public items

## Common Pitfalls

1. **Missing async/await**: All I/O operations must be async
2. **Layer violation**: UI cannot import domain directly (goes through application)
3. **Forgetting Arc**: Shared state across async tasks needs `Arc`
4. **Not handling errors**: All async methods return `Result<T, AppError>`
5. **Test coverage**: Always add tests to maintain 70%+ coverage

## Additional Resources

- [PLAN.md](.legacy/PLAN.md) - Project roadmap
- [EVOLUTION.md](.legacy/EVOLUTION.md) - Feature evolution history
- [README.md](README.md) - User documentation
- [AGENT_ARCHITECTURE.md](.legacy/AGENT_ARCHITECTURE.md) - Agent roles (architect, developer, reviewer) and skills orchestration
- [CONTEXT.md](.legacy/CONTEXT.md) - General project context (pitch, stack, current state, resource map)
- [rules/](rules/) - Per-layer DDD rules shared by agents and skills (domain, application, infrastructure, presentation, quality)
- [.agents/skills/](.agents/skills/) - Reusable procedure skills (Agent Skills standard, one folder per skill, `SKILL.md` format; auto-discovered by Vibe Code and Vibe CLI), including the mandatory [confirmation-gate](.agents/skills/confirmation-gate/SKILL.md)
- [agents/](agents/) - Independent agent definitions invoking the skills

## Agent Routing

When the user asks for a review of a pull request (e.g. "lance la review", "review PR #N"), invoke the **Reviewer** agent ([agents/reviewer.md](agents/reviewer.md)) — not an external review service. Follow its mode opératoire exactly:

1. Read [agents/reviewer.md](agents/reviewer.md) and the skills it references: [.agents/skills/rust-code-review/SKILL.md](.agents/skills/rust-code-review/SKILL.md) (main procedure), [.agents/skills/rust-quality-gate/SKILL.md](.agents/skills/rust-quality-gate/SKILL.md), [.agents/skills/rust-feature-ddd/SKILL.md](.agents/skills/rust-feature-ddd/SKILL.md), [.agents/skills/rust-testing/SKILL.md](.agents/skills/rust-testing/SKILL.md) (read-only references).
2. Execute its 6-step procedure without skipping any step (context, scope, layer-by-layer DDD examination, quality/tests, structured verdict, publication).
3. Publish the review on the PR: verdict first (approve / request changes / comment), findings by severity 🔴/🟡/🟢 with `file:line` and a concrete suggestion. Note: GitHub forbids `REQUEST_CHANGES` by the PR author — fall back to `COMMENT` with the verdict stated in the body.
4. Report, do not fix: a review never includes implementation code, and never merge unless explicitly instructed.