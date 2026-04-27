# 3. Non-nullable returns with `Option` and `Result`

Rust does not use `null` as the normal way to represent "nothing was found" or "something failed".

Instead:

- use `Option<T>` when a value may be absent
- use `Result<T, E>` when an operation may fail

This codebase uses both heavily.

## `Option<T>` for absence

A repository lookup may or may not find a container:

**File:** `src/02_application/container/traits.rs`

```rust
async fn get_by_id(&self, id: &str) -> Result<Option<Container>, AppError>;
```

Read it from the inside out:

- `Container` is the success value
- `Option<Container>` means "maybe found, maybe not"
- `Result<..., AppError>` means the lookup itself can still fail

So there are three distinct outcomes:

1. `Ok(Some(container))`
2. `Ok(None)`
3. `Err(app_error)`

That is much clearer than returning `null` and guessing why.

## `Option<T>` in UI state

The TUI stores optional state explicitly:

**File:** `src/04_presentation/tui/app.rs`

```rust
pub previous_screen: Option<Screen>,
pub confirm_dialog: Option<(ConfirmAction, bool)>,
pub popup_message: Option<PopupMessage>,
```

If there is no popup, the value is `None`. If there is one, it is `Some(...)`.

The same idea appears in presenters:

**File:** `src/04_presentation/tui/container/presenter.rs`

```rust
pub logs: Option<ContainerLogsDto>,
pub selected_container: Option<ContainerDto>,
pub error: Option<String>,
```

## `Result<T, E>` for failure

Application services return `Result` everywhere failure matters:

**File:** `src/02_application/container/service.rs`

```rust
pub async fn get_all_containers(&self) -> Result<Vec<ContainerDto>, AppError> {
    let containers = self.repository.get_all().await?;
    Ok(ContainerMapper::to_dto_list(&containers))
}
```

The `?` operator means:

- if the repository call succeeds, keep going
- if it fails, return early with the error

## Errors are typed, not stringly by default

This repo keeps separate error enums per layer:

- `src/01_domain/error.rs`
- `src/02_application/error.rs`
- `src/03_infrastructure/error.rs`

Example:

```rust
pub enum AppError {
    Domain(#[from] DomainError),
    Repository(String),
    NotFound(String),
    OperationFailed(String),
    Connection(String),
}
```

That makes error handling explicit and composable.

## Practical reading rule

When you see a return type like this:

```rust
Result<Option<T>, E>
```

read it as:

"The operation can fail, and even if it succeeds, the requested item may still be absent."

## Where to look next in this repo

- `src/02_application/container/traits.rs` for `Result<Option<T>, E>`
- `src/02_application/container/service.rs` for `?`
- `src/04_presentation/tui/app.rs` for optional UI state
- `src/02_application/error.rs` and `src/03_infrastructure/error.rs` for typed errors

Next: [traits and trait objects](./04_traits_and_trait_objects.md).
