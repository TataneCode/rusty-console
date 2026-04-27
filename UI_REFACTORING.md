# UI refactoring

## User Instructions

This concern the 04_presentation layer.

### Texts

All texts should be store in a separate file, as resource file in angular and i8n for example. Only one file is enough at that time, I don't want to plan translation for now.

### Components factorization

Review all components and functions in the presentation layer to see what can be extracted or simplified.

### Styles

Components styles (color and size if possible), should be defined in a separated file (or at the top) from the original view.

### Optimization

As you are deeply reviewing the presentation layer, don't hesitate to analyze optimizations.

### Cross platform analysis

Using your knowledge, please list any code in the presentation layer that can present differents behaviour once packages are done for windows and linux.

## Copilot plan

### Current state

- The presentation layer lives in `src/04_presentation/tui`, not `src/ui`.
- The four feature areas (`container`, `volume`, `image`, `stack`) already use an `actions` / `presenter` / `view` split.
- `common/` already centralizes part of the UI surface (`keys.rs`, `theme.rs`, `widgets.rs`), but `app.rs` still owns most orchestration.
- User-facing text is still hardcoded across `app.rs` and the feature view files.
- Filter state, filter editing, selection syncing, table layout setup, and string truncation are repeated across several presenters and views.

### Planned refactor

1. Add a dedicated presentation resource module for all user-facing texts: menu labels, table titles, help text, dialog prompts, and success/error messages.
2. Extract the repeated presenter filter behavior into shared helpers so list screens stop reimplementing the same filter and selection logic.
3. Factor repeated view concerns into reusable presentation helpers: truncation, title formatting, common layout blocks, and other repeated table/detail setup.
4. Reduce the size of `src/04_presentation/tui/app.rs` by moving screen-specific rendering and action orchestration behind smaller presentation-focused modules while keeping the current screen flow intact.
5. Keep `common/theme.rs` as the styling anchor, but extend the refactor so view-local style and layout constants are no longer scattered inline when they can be shared safely.
6. Finish with a cross-platform review and existing Rust validation commands.

### Cross-platform analysis focus

- `crossterm` raw mode, alternate screen, mouse capture, and `KeyEventKind::Press` handling may not behave identically on Windows and Linux terminals.
- Shortcut combinations such as `Ctrl+S`, `Ctrl+U`, `Ctrl+D`, plus uppercase bindings like `R`, `S`, `D`, can vary depending on terminal emulator and platform keyboard handling.
- Unicode rendering for `▏`, `…`, and selection markers can differ by terminal font and platform packaging target.
- Fixed-width truncation based on string length may misalign when wide Unicode characters are present.
- Color choices like `DarkGray`, plus general ANSI/color support, can render differently across terminal environments.

## Copilot review

The plan is accurate in its description of the current state and planned refactors. Below are concrete code-level observations that should sharpen implementation priorities.

### Texts

All user-facing strings are currently hardcoded and scattered:

- Help bar strings are inline in every `view.rs` file (e.g. `" q: Quit | /: Filter | j/k: Navigate | ..."`).
- Confirm dialog prompts are inline `match` arms in `app.rs` (lines 262–283).
- Column headers (`vec!["Name", "Image", "State", ...]`) are inline in each list view function.
- Menu labels (`"  Containers"`, `"  Volumes"`, …) are inline in `render_main_menu` inside `app.rs`.
- Table title strings (`" Containers "`, `" Container Details "`, …) are inline in each view function.
- Dialog and popup titles (`" Confirm "`, `" Error "`) are inline in `widgets.rs`.
- The filter cursor character `▏` is repeated across all four list views.

A `resources.rs` module in `common/` with `pub const` string slices would centralize these cleanly.

### Components factorization

- **`truncate_string` is duplicated four times**: `container/view.rs`, `volume/view.rs`, `image/view.rs`, and `stack/view.rs`. The stack view uses a correct char-based truncation (`c.image.chars().take(29)`) while the other three use byte indexing (`&s[..max_len - 3]`), which **will panic** if the cut point falls inside a multi-byte UTF-8 character. Extracting a single `truncate_str` to `common/` using the char-based approach fixes both the duplication and this latent bug.
- **Filter behaviour is duplicated across all four presenters**: every presenter (`ContainerPresenter`, `VolumePresenter`, `ImagePresenter`, `StackPresenter`) has identical `filter: String`, `filter_active: bool` fields and identical `activate_filter`, `deactivate_filter`, `push_filter_char`, `pop_filter_char`, and `update_filtered_selection` methods — roughly 30 lines each. A shared `FilterState` struct in `common/` would eliminate ~120 lines of repetition.
- **Active filter title pattern is duplicated** in all four list views: the `match active_filter { Some(f) => format!(" {} [/: {}▏] ", ...), None => ... }` block is copy-pasted verbatim. A helper in `common/` (`fn filter_title(base: &str, filter: Option<&str>) -> String`) would fix this.
- **`render_main_menu` in `app.rs`** (lines 292–339) is a rendering function that mutably borrows `App` and uses raw inline styles instead of `Theme`. Moving it to a dedicated `menu/view.rs` (or `common/`) function accepting `&mut ListState` would reduce `app.rs` and align with the rest of the codebase.

### Styles

- `common/theme.rs` is well-structured and already covers almost all styles.
- **`render_main_menu` bypasses `Theme` entirely**: it uses four raw `Style::default()` calls that duplicate existing `Theme` methods — `Theme::title_style()`, `Theme::help_style()`, and `Theme::highlight_style()` — instead of calling them.
- Layout constants (help bar `Constraint::Length(2)`, title bar `Constraint::Length(3)`, dialog size percentages `(50, 20)` and `(60, 20)`) are inline across multiple files. Exposing them as `const` in `common/` would make them adjustable from one place.

### Optimization

- **Redundant clone in the `app.rs` render loop**: every list screen converts `Vec<&DTO>` to `Vec<DTO>` via `.into_iter().cloned().collect()` before passing to the view function. Changing view functions to accept `&[&DTO]` instead would eliminate these per-frame allocations.
- **Double style computation in `image/view.rs`**: for each row, `style` is computed for the repository/tag cells (lines 26–33), then a second `in_use_style` is recomputed inline for the In Use cell (lines 41–45). The second call is redundant and can be replaced by a simple conditional on `img.in_use`.
- **Encapsulation gap in `StackPresenter`**: `select_stack_by_name` directly writes `self.selection.state.select(selected)`, bypassing `TableSelection::set_items` and its bounds-checking logic. This should go through the existing `TableSelection` API.

### Cross-platform analysis

The plan's cross-platform section is accurate. Two additional concrete points from code inspection:

1. **UTF-8 byte-indexing bug** (see Components above): `container`, `volume`, and `image` views use `&s[..n]` byte slicing in `truncate_string`, which panics on multi-byte characters at the cut point. This is a correctness issue that appears before packaging and should be the highest priority fix in this refactor.
2. `stack/view.rs` uses the `…` Unicode ellipsis while the other three views use `...` (three ASCII dots). Inconsistent truncation markers compound cross-platform font rendering differences. Standardizing on one form and documenting the choice is recommended.
