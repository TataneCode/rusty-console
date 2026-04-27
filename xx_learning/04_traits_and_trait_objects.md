# 4. Traits and trait objects

Traits describe shared behavior. If structs are Rust's data building blocks, traits are one of Rust's main abstraction tools.

This project uses traits as the boundary between the application layer and the infrastructure layer.

## Traits as contracts

The container repository is defined as a trait:

**File:** `src/02_application/container/traits.rs`

```rust
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait ContainerRepository: Send + Sync {
    async fn get_all(&self) -> Result<Vec<Container>, AppError>;
    async fn get_by_id(&self, id: &str) -> Result<Option<Container>, AppError>;
    async fn get_logs(&self, id: &str, tail: Option<usize>) -> Result<String, AppError>;
    ...
}
```

That says:

- any type implementing this trait can act as a container repository
- the application layer depends on behavior, not on a concrete Docker adapter
- tests can substitute a mock implementation

## A service depends on `dyn Trait`

The service stores a trait object behind `Arc`:

**File:** `src/02_application/container/service.rs`

```rust
pub struct ContainerService {
    repository: Arc<dyn ContainerRepository>,
}
```

This is a common Rust architecture pattern:

- `Arc` gives shared ownership
- `dyn ContainerRepository` means dynamic dispatch through the trait
- the service does not need to know whether the implementation is real or mocked

## A concrete type implements the trait

The infrastructure adapter provides the real behavior:

**File:** `src/03_infrastructure/docker/container/adapter.rs`

```rust
#[async_trait]
impl ContainerRepository for ContainerAdapter {
    async fn get_all(&self) -> Result<Vec<Container>, AppError> {
        ...
    }
    ...
}
```

That is the key relationship:

- the trait defines the interface
- the adapter implements it
- the service consumes it

## Traits are the architectural seam

This seam keeps dependencies pointing inward:

- `02_application` defines the trait
- `03_infrastructure` implements it
- `04_presentation` calls services, not adapters directly

That is why traits matter here beyond syntax. They support the whole DDD layering strategy.

## Why `Send + Sync` appears

Repository traits are declared with:

```rust
pub trait ContainerRepository: Send + Sync
```

That means implementations are safe to share across async contexts used by the application.

## Traits also help tests

This attribute appears right above the trait:

```rust
#[cfg_attr(test, mockall::automock)]
```

When tests compile, `mockall` can generate a `MockContainerRepository` from the trait. That is a big reason traits are valuable in this repository.

## Where to look next in this repo

- `src/02_application/container/traits.rs` for trait definitions
- `src/02_application/stack/traits.rs` for another repository trait
- `src/02_application/container/service.rs` for `Arc<dyn Trait>`
- `src/03_infrastructure/docker/container/adapter.rs` for a real implementation

Next: [async with Tokio](./05_async_with_tokio.md).
