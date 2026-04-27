# 8. From Rust basics to project architecture

At this point the language ideas connect directly to the architecture.

## The flow of one feature

A simplified path for listing containers is:

1. `main.rs` builds the dependencies and starts the app
2. the UI triggers a container action
3. the application service calls a repository trait
4. the infrastructure adapter talks to Docker asynchronously
5. raw Docker data is mapped into domain entities
6. domain entities are mapped into DTOs
7. the presenter stores display state
8. the view renders that state with Ratatui

That is the code version of:

```text
Docker API -> Infrastructure -> Domain -> Application DTO -> Presenter -> View
```

## Ownership appears differently in each layer

- **Domain** owns business data in structs like `Container`
- **Application** borrows or maps data while orchestrating use cases
- **Presentation** often borrows DTOs during rendering instead of cloning everything

This is why ownership is not just a low-level detail; it shapes architecture.

## `Option` and `Result` keep state transitions honest

Examples from the project:

- `Option<PopupMessage>` means a popup may or may not exist
- `Option<ContainerDto>` means details may or may not be loaded
- `Result<_, AppError>` means failure is always explicit at the boundary

That makes state visible in types instead of hidden in conventions.

## Async stays near the boundaries

The domain layer is mostly plain data and rules.

Async is concentrated in places that actually wait on the outside world:

- Tokio-powered startup in `src/main.rs`
- repository traits in `src/02_application/*/traits.rs`
- Docker adapters in `src/03_infrastructure/docker/*/adapter.rs`

That separation keeps the core easier to test and reason about.

## Enums drive the UI state machine

The app uses `Screen` and action enums to make UI flow explicit.

That is a recurring Rust pattern:

- define the possible states as an enum
- use `match` to handle them exhaustively
- let the compiler help when states evolve

## A good way to keep reading the project

If you want to continue learning from the code itself, follow this order:

1. `src/shared.rs`
2. `src/01_domain/container/entity.rs`
3. `src/02_application/container/traits.rs`
4. `src/02_application/container/service.rs`
5. `src/03_infrastructure/docker/container/adapter.rs`
6. `src/04_presentation/tui/container/presenter.rs`
7. `src/04_presentation/tui/container/view.rs`
8. `src/04_presentation/tui/app.rs`

That path mirrors the progression of this folder: basics first, then deeper Rust in a real application.
