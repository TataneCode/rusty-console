# Generic Parameter Naming — 2026-10-22

Pure readability refactoring: every named generic parameter in the codebase now
carries a meaningful name instead of a single-letter placeholder (`T`, `'a`).

## Convention adopted

| Name | Meaning | Where |
|------|---------|-------|
| `TItem` | a list item type | `apply_name_sort` (`common/sort.rs`); future `ListCore<TItem>` |
| `'frame` | lifetime of ratatui widgets borrowed for one frame render | `render_table` (`common/widgets.rs`) |
| `'text` | lifetime of borrowed strings/slices used for display | `InputField` (`common/input.rs`), `ImageHelpBarState`, `StackHelpBarState`, `LogSearchView` (view structs) |
| `'items` | lifetime of references into the filtered input slice | `filter_containers` / `filter_images` / `filter_networks` / `filter_volumes` / `filter_stacks` (presenters) |

The `T` prefix on type parameters mirrors the project's C#/.NET heritage
(FancyContainerConsole port).

## Scope decisions

- **Renamed**: all named type parameters and lifetimes listed above (11 files).
- **Left as `'_`**: use sites where the lifetime is elided and obvious
  (`ImageHelpBarState<'_>`, `impl LogSearchView<'_>`, `&[InputField<'_>]`).
- **Untouched**: anonymous generics (`impl Into<String>`, `impl Fn(...)`) —
  idiomatic, nothing to name; local inference (`collect::<Vec<_>>()`);
  `std::fmt::Formatter<'_>` in `Display` impls (standard pattern);
  concrete generics (`Arc<dyn XxxRepository>`, `Result<Vec<Container>, AppError>`).

## Verification

- `cargo fmt --check` — clean
- `cargo clippy --all-targets -- -D warnings` — clean
- `cargo test` — 480 passed, 0 failed (469 unit + 11 integration)

No test was modified; no behavior changed.
