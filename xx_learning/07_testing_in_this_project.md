# 7. Testing in this project

This repository is a good place to learn Rust testing because it mixes several styles:

- plain unit tests
- async tests
- trait-based mocking
- terminal UI rendering tests

## The common pattern: `#[cfg(test)]`

Most files keep tests beside the production code:

**File:** `src/shared.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byte_size_human_readable() {
        assert_eq!(ByteSize::new(500).human_readable(), "500 B");
    }
}
```

This is idiomatic Rust:

- `#[cfg(test)]` compiles the module only for tests
- `use super::*;` imports the code from the same file
- `#[test]` marks a synchronous unit test

## Async tests with Tokio

Service tests often await async code:

**File:** `src/02_application/container/service.rs`

```rust
#[tokio::test]
async fn test_get_all_containers_returns_mapped_dtos() {
    let mut mock = MockContainerRepository::new();
    ...
    let result = service.get_all_containers().await.unwrap();
    assert_eq!(result.len(), 2);
}
```

`#[tokio::test]` creates a runtime for the test so async code can be awaited naturally.

## Mocking through traits

Because repository behavior is defined by traits, tests can use generated mocks:

```rust
let mut mock = MockContainerRepository::new();
mock.expect_get_all().returning(|| {
    Ok(vec![...])
});
```

That lets the test focus on service logic without talking to Docker.

The mock type exists because the trait was declared with:

```rust
#[cfg_attr(test, mockall::automock)]
```

## Testing Ratatui rendering

The UI tests use Ratatui's test backend:

**File:** `src/04_presentation/tui/container/view.rs`

```rust
let backend = TestBackend::new(100, 20);
let mut terminal = Terminal::new(backend).unwrap();

terminal
    .draw(|frame| {
        render_container_list(frame, frame.area(), &items, &mut state, Some("ng"))
    })
    .unwrap();
```

Then the test reads the buffer and checks for visible text:

```rust
let text = buffer_text(terminal.backend().buffer());
assert!(text.contains("Containers"));
assert!(text.contains("web"));
```

That is a practical way to test UI output without opening a real terminal.

## What these tests tell you about the architecture

The testing style matches the architecture:

- domain code is easy to unit test directly
- application services are tested with mocked repositories
- view code is tested with a fake terminal backend

That separation is one sign the layering is working well.

## Where to look next in this repo

- `src/shared.rs` for simple unit tests
- `src/01_domain/*` for domain-focused tests
- `src/02_application/*/service.rs` for async + mock-based tests
- `src/04_presentation/tui/*/view.rs` for Ratatui rendering tests

Next: [how the full project fits together](./08_from_rust_basics_to_project_architecture.md).
