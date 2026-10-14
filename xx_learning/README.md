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
- a full feature slice (networks: domain rules, repository trait, Bollard adapter, service, TUI screens) added end-to-end through every layer
- breadcrumb navigation and a centralized design system for the three layout sections (see lesson 6)
- unit tests, async tests, mocks, and Ratatui rendering tests
- custom widgets like the loading spinner for async operations

The lessons start with core language ideas, then move outward into async and architecture.

## Recent Additions

Recent features that make good study material beyond the numbered lessons:

- **Networks management** (Phase 4) — a complete vertical slice through the four layers: domain entity + `NetworkScope` value object + `can_be_deleted()` invariant, `NetworkRepository` trait, Bollard adapter (with per-network `inspect_network` enrichment), `NetworkService`, and the `NetworkList`/`NetworkDetails` screens including the disconnect-container flow
- **Breadcrumb navigation** — a top bar rendered on every non-menu screen with a graceful truncation strategy (leading segments dropped first; current segment cut from the left)
- **Homogeneous section design** — `Theme::section_block` centralizes the rounded bordered box + horizontal padding shared by the breadcrumb, main content, and action bar; cyan borders are reserved for focused/overlay widgets. Covered in [lesson 6](./06_terminal_ui_with_crossterm_and_ratatui.md)
- **Loading spinner** (June 2026) — custom widget creation in Ratatui, state management for animations, integration across multiple screens, and testing of timing-based behavior
