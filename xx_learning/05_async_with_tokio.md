# 5. Async with Tokio

This project talks to Docker and updates a terminal UI, so blocking the thread would hurt responsiveness. Rust solves that here with `async` functions and the Tokio runtime.

## The app starts inside Tokio

**File:** `src/main.rs`

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    ...
    app.run().await?;
    Ok(())
}
```

`#[tokio::main]` creates the runtime and lets `main` be async.

## Async traits in the application boundary

Rust traits do not support async methods directly in stable Rust without help, so this project uses `async-trait`.

**File:** `src/02_application/container/traits.rs`

```rust
#[async_trait]
pub trait ContainerRepository: Send + Sync {
    async fn get_all(&self) -> Result<Vec<Container>, AppError>;
    async fn get_by_id(&self, id: &str) -> Result<Option<Container>, AppError>;
    ...
}
```

This is the contract the infrastructure layer implements.

## Services await repositories

Application services are thin async coordinators:

**File:** `src/02_application/container/service.rs`

```rust
pub async fn get_all_containers(&self) -> Result<Vec<ContainerDto>, AppError> {
    let containers = self.repository.get_all().await?;
    Ok(ContainerMapper::to_dto_list(&containers))
}
```

The pattern is simple:

1. await a repository call
2. map domain data into DTOs
3. return a typed result

## Infrastructure adapters await Docker calls

The adapter layer is where real I/O happens:

**File:** `src/03_infrastructure/docker/container/adapter.rs`

```rust
let containers = self
    .docker
    .inner()
    .list_containers(Some(options))
    .await
    .map_err(InfraError::from)
    .map_err(AppError::from)?;
```

This is a realistic async stack:

- async trait method
- Docker API request
- `.await`
- error conversion
- domain mapping

## Async streams for logs

Container logs arrive as a stream, not a single value:

```rust
let mut stream = self.docker.inner().logs(id, Some(options));

while let Some(result) = stream.next().await {
    match result {
        Ok(output) => {
            logs.push_str(&output.to_string());
        }
        Err(e) => {
            return Err(AppError::from(InfraError::from(e)));
        }
    }
}
```

This combines:

- `while let` pattern matching
- `await` on each incoming item
- explicit error propagation

## Shared async dependencies with `Arc`

`main.rs` wires adapters and services with `Arc`:

```rust
let container_adapter = Arc::new(ContainerAdapter::new(docker.clone()));
let container_service = ContainerService::new(container_adapter);
```

`Arc` gives shared ownership across async parts of the application without copying the underlying client.

## Where to look next in this repo

- `src/main.rs` for Tokio startup and dependency wiring
- `src/02_application/*/traits.rs` for async trait contracts
- `src/02_application/*/service.rs` for awaited use cases
- `src/03_infrastructure/docker/*/adapter.rs` for real async Docker work

Next: [terminal UI with Crossterm and Ratatui](./06_terminal_ui_with_crossterm_and_ratatui.md).
