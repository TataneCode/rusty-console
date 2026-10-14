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

For live container stats, the app loop also drains a Tokio `mpsc` receiver and applies any pending stats updates before/after drawing. That keeps the UI responsive while still refreshing CPU, memory, and network values on screen.

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
    is_loading: bool,
    spinner_char: &str,
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

The container list is also a good example of incremental UI growth:

- the table started as static container metadata
- live **CPU** and **Memory** columns were added without changing the rendering pattern
- container details gained a **Network I/O** section fed by the same presenter state

That is a practical Ratatui lesson: the view stays simple when the presenter already owns display-ready state.

## UI text is centralized

Most visible strings live here:

**File:** `src/04_presentation/tui/common/resources.rs`

That makes the rendering code easier to read and keeps labels/help text in one place.

## Breadcrumb navigation

Every non-menu screen renders a breadcrumb bar at the top showing the current location, e.g. `Rusty - docker manager > Containers > web logs`.

**File:** `src/04_presentation/tui/common/breadcrumb.rs`

The interesting part is how it adapts to narrow terminals (`fit_breadcrumb_text`):

- if the full path fits, it is rendered as-is
- otherwise, **leading segments are dropped first** (`Rusty - docker manager > Containers` disappears before the current segment)
- only when the current segment alone overflows is it cut **from the left**, prefixed with `...`, so the right end (usually the container name's distinctive suffix) stays readable

That is a small but real lesson in graceful degradation: the UI degrades the least important information first.

## Homogeneous section design

The breadcrumb, the main content, and the action bar all share one visual style: a rounded bordered box with horizontal padding.

**File:** `src/04_presentation/tui/common/theme.rs`

```rust
pub fn section_block(title: Option<&str>) -> Block<'static> {
    let mut block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Self::border_style())
        .padding(Padding::horizontal(1));
    if let Some(title) = title {
        block = block.title(Line::from(title.to_string()).style(Self::title_style()));
    }
    block
}
```

Before this helper existed, each view built its own `Block` with slightly different borders (the action bar had only a top border, the breadcrumb used a different colour). Now:

- **one factory** (`Theme::section_block`) defines the look of every section: `Borders::ALL`, `BorderType::Rounded` (rounded corners), and `Padding::horizontal(1)` to keep content off the borders
- **colour hierarchy is uniform**: white borders for the three chrome sections; cyan (`selected_border_style`) is reserved for focused/overlay widgets (dialogs, selection lists)
- **layout heights are symmetric**: the action bar box (`HELP_BAR_HEIGHT = 3`) matches the breadcrumb bar height, so the top and bottom chrome mirror each other

The lesson: centralizing widget styling in the theme turns "design consistency" from a convention into a type-checked guarantee. A view cannot accidentally drift from the design system, because it does not build its own `Block` anymore.

## Loading Spinner Widget

The project includes a custom spinner widget for async operations:

**File:** `src/04_presentation/tui/common/spinner.rs`

```rust
pub struct Spinner {
    frames: Vec<char>,
    current_index: usize,
    last_update: Instant,
    frame_duration: Duration,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            frames: vec!['|', '/', '-', '\\'],
            current_index: 0,
            last_update: Instant::now(),
            frame_duration: Duration::from_millis(200),
        }
    }
    
    pub fn next(&mut self) -> char {
        if self.last_update.elapsed() >= self.frame_duration {
            self.current_index = (self.current_index + 1) % self.frames.len();
            self.last_update = Instant::now();
        }
        self.current()
    }
    
    pub fn current(&self) -> char {
        self.frames[self.current_index]
    }
    
    pub fn reset(&mut self) {
        self.current_index = 0;
        self.last_update = Instant::now();
    }
}
```

Key lessons from the spinner implementation:

- Uses ASCII characters for a simple, terminal-compatible animation
- 200ms frame duration provides smooth but not overwhelming animation
- State management: tracks current frame index and last update time
- Integration: all list views accept `is_loading` and `spinner_char` parameters
- Help text is replaced with "Loading... <spinner>" when loading

## Where to look next in this repo

- `src/04_presentation/tui/app.rs` for setup and the main render loop
- `src/04_presentation/tui/event.rs` for terminal events
- `src/04_presentation/tui/common/resources.rs` for UI text
- `src/04_presentation/tui/common/breadcrumb.rs` for the breadcrumb bar and its truncation strategy
- `src/04_presentation/tui/common/theme.rs` for the shared section design (`section_block`)
- `src/04_presentation/tui/common/spinner.rs` for the loading spinner widget
- `src/04_presentation/tui/container/presenter.rs` for list/detail state enriched with live stats
- `src/04_presentation/tui/*/view.rs` for Ratatui widget composition

Next: [testing in this project](./07_testing_in_this_project.md).
