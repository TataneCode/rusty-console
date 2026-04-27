# 6. Terminal UI with Crossterm and Ratatui

The project renders a full-screen terminal UI. The backend is **Crossterm**, and the widget/layout library is **Ratatui**.

If you were expecting "x-term", the practical equivalent in this codebase is: a terminal app running in an xterm-compatible terminal, driven by Crossterm and rendered with Ratatui.

## Entering terminal mode

**File:** `src/04_presentation/tui/app.rs`

```rust
enable_raw_mode()?;
let mut stdout = io::stdout();
execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
let backend = CrosstermBackend::new(stdout);
let mut terminal = Terminal::new(backend)?;
```

This does a few important things:

- raw mode stops normal line-buffered terminal behavior
- the alternate screen gives the app its own full-screen canvas
- `CrosstermBackend` connects Ratatui to the terminal

## Event handling comes from Crossterm

**File:** `src/04_presentation/tui/event.rs`

```rust
if event::poll(self.tick_rate)? {
    if let Event::Key(key) = event::read()? {
        if key.kind == KeyEventKind::Press {
            return Ok(AppEvent::Key(key));
        }
    }
}
Ok(AppEvent::Tick)
```

The event loop polls for input and turns raw terminal events into application events.

## Ratatui renders borrowed state

View functions stay close to pure rendering:

**File:** `src/04_presentation/tui/container/view.rs`

```rust
pub fn render_container_list(
    frame: &mut Frame,
    area: Rect,
    containers: &[&ContainerDto],
    state: &mut TableState,
    active_filter: Option<&str>,
) {
```

That signature is very Rusty:

- borrow the frame
- borrow the area description
- borrow the container data
- mutate only the table state that needs selection updates

## Widgets are built from composition

Inside the view, rows and widgets are assembled from smaller pieces:

```rust
let rows: Vec<Row> = containers
    .iter()
    .copied()
    .map(|c| {
        Row::new(vec![
            Cell::from(c.name.clone()),
            Cell::from(truncate_text(&c.image, 30)),
            Cell::from(c.state_display()).style(state_style),
            ...
        ])
    })
    .collect();
```

Ratatui encourages composition:

- data becomes rows and cells
- rows go into tables
- blocks and paragraphs wrap content
- theme helpers centralize styling

## UI text is centralized

Most visible strings live here:

**File:** `src/04_presentation/tui/common/resources.rs`

That makes the rendering code easier to read and keeps labels/help text in one place.

## Where to look next in this repo

- `src/04_presentation/tui/app.rs` for setup and the main render loop
- `src/04_presentation/tui/event.rs` for terminal events
- `src/04_presentation/tui/common/resources.rs` for UI text
- `src/04_presentation/tui/*/view.rs` for Ratatui widget composition

Next: [testing in this project](./07_testing_in_this_project.md).
