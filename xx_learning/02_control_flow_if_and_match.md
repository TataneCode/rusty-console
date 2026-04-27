# 2. Control flow with `if` and `match`

Rust has `if` and `match`. It does **not** have a `switch` keyword.

If you come from C#, Java, or JavaScript, think of `match` as Rust's more powerful and more explicit relative of `switch`.

## `if` / `else if` for range-like logic

`ByteSize::human_readable` is a clean example:

**File:** `src/shared.rs`

```rust
if self.0 < 0 {
    return "N/A".to_string();
}

if self.0 >= GB {
    format!("{:.2} GB", self.0 as f64 / GB as f64)
} else if self.0 >= MB {
    format!("{:.2} MB", self.0 as f64 / MB as f64)
} else if self.0 >= KB {
    format!("{:.2} KB", self.0 as f64 / KB as f64)
} else {
    format!("{} B", self.0)
}
```

This is a good fit for `if` because the branches depend on ordered numeric conditions.

## `match` for enum-driven branching

The UI chooses what to render by matching on `Screen`:

**File:** `src/04_presentation/tui/app.rs`

```rust
match &self.screen {
    Screen::MainMenu => render_main_menu(frame, area, &mut self.menu_state),
    Screen::ContainerList => { ... }
    Screen::ContainerLogs => { ... }
    Screen::ContainerDetails => { ... }
    ...
}
```

This is the idiomatic Rust pattern:

- model states with an enum
- use `match` to handle each state explicitly

## `match` for input mapping

Keyboard handling is another strong example:

**File:** `src/04_presentation/tui/common/keys.rs`

```rust
match key.code {
    KeyCode::Char('q') => Some(AppAction::Quit),
    KeyCode::Esc => Some(AppAction::Back),
    KeyCode::Up | KeyCode::Char('k') => Some(AppAction::NavigateUp),
    KeyCode::Down | KeyCode::Char('j') => Some(AppAction::NavigateDown),
    ...
    _ => None,
}
```

Compared with a classic `switch`, Rust `match` gives you:

- pattern alternatives with `|`
- enum and pattern matching
- exhaustiveness checking

## `match` for error translation

The infrastructure layer converts one error type into another:

**File:** `src/03_infrastructure/error.rs`

```rust
match err {
    InfraError::Docker(msg) => AppError::repository(msg),
    InfraError::Connection(msg) => AppError::connection(msg),
    InfraError::Serialization(msg) => AppError::repository(msg),
}
```

That is more than "choose a case." It destructures values and names the inner data (`msg`) at the same time.

## When to prefer which

- Prefer `if` when conditions are boolean checks, ranges, or ordered comparisons.
- Prefer `match` when branching on enums, tagged states, or several known patterns.

## Where to look next in this repo

- `src/shared.rs` for `if` / `else if`
- `src/04_presentation/tui/app.rs` for `match` on app state
- `src/04_presentation/tui/common/keys.rs` for `match` on key input
- `src/03_infrastructure/error.rs` for `match` with data extraction

Next: [non-nullable returns with `Option` and `Result`](./03_option_result_and_non_nullable_returns.md).
