# Rust learning path from this project

This folder turns the codebase into a progressive Rust learning path.

The order matters:

1. [Ownership and borrowing](./01_ownership_and_borrowing.md)
2. [Control flow with `if` and `match`](./02_control_flow_if_and_match.md)
3. [Non-nullable returns with `Option` and `Result`](./03_option_result_and_non_nullable_returns.md)
4. [Traits and trait objects](./04_traits_and_trait_objects.md)
5. [Async with Tokio](./05_async_with_tokio.md)
6. [Terminal UI with Crossterm and Ratatui](./06_terminal_ui_with_crossterm_and_ratatui.md)
7. [Testing in this project](./07_testing_in_this_project.md)
8. [How the full project fits together](./08_from_rust_basics_to_project_architecture.md)

## How to use these notes

- Read the explanation first.
- Open the referenced Rust files beside the lesson.
- Compare the short snippets here with the fuller production code in `src/`.

## Why this project is a good learning base

It contains:

- plain structs and enums in the domain layer
- borrowed references and slices in getters and view functions
- `if` / `else if` and `match` in real UI and infrastructure code
- `Option` and `Result` everywhere absence or failure must be explicit
- repository traits, trait objects, and real implementations
- async I/O with Tokio, Bollard, and `async_trait`
- async streams and `tokio::sync::mpsc` for live container stats
- a real terminal UI built with Crossterm and Ratatui
- unit tests, async tests, mocks, and Ratatui rendering tests

The lessons start with core language ideas, then move outward into async and architecture.
