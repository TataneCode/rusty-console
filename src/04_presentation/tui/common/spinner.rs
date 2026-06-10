use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::time::{Duration, Instant};

/// ASCII spinner frames for loading animation
const SPINNER_FRAMES: [&str; 4] = ["|", "/", "-", "\\"];

/// Duration for each spinner frame (200ms as per user requirement)
const SPINNER_FRAME_DURATION: Duration = Duration::from_millis(200);

/// Spinner widget for displaying loading state
#[derive(Debug, Clone)]
pub struct Spinner {
    /// Current frame index (0-3 for ASCII spinner)
    current_frame: usize,
    /// Last time the frame was updated
    last_update: Instant,
}

impl Spinner {
    /// Create a new spinner
    pub fn new() -> Self {
        Spinner {
            current_frame: 0,
            last_update: Instant::now(),
        }
    }

    /// Update the spinner frame based on elapsed time
    /// Call this once per render frame
    pub fn update(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update);

        if elapsed >= SPINNER_FRAME_DURATION {
            self.current_frame = (self.current_frame + 1) % SPINNER_FRAMES.len();
            self.last_update = now;
        }
    }

    /// Get the current spinner character
    pub fn current_char(&self) -> &'static str {
        SPINNER_FRAMES[self.current_frame]
    }

    /// Reset the spinner to initial state
    pub fn reset(&mut self) {
        self.current_frame = 0;
        self.last_update = Instant::now();
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

/// Render a loading spinner in the help bar area
/// Replaces the entire help text as per user requirement
/// Uses ASCII spinner character | / - \
pub fn render_loading_spinner(frame: &mut Frame, area: Rect, spinner_char: &str) {
    let loading_text = format!(" {} Loading...", spinner_char);
    let spinner = Paragraph::new(loading_text).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default()),
    );

    frame.render_widget(spinner, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_new_starts_at_frame_0() {
        let spinner = Spinner::new();
        assert_eq!(spinner.current_frame, 0);
        assert_eq!(spinner.current_char(), "|");
    }

    #[test]
    fn test_spinner_cycles_through_frames() {
        let mut spinner = Spinner::new();
        assert_eq!(spinner.current_char(), "|");

        // Manually advance frames
        spinner.current_frame = 1;
        assert_eq!(spinner.current_char(), "/");

        spinner.current_frame = 2;
        assert_eq!(spinner.current_char(), "-");

        spinner.current_frame = 3;
        assert_eq!(spinner.current_char(), "\\");

        // Wrap around using modulo
        spinner.current_frame = 4 % SPINNER_FRAMES.len();
        assert_eq!(spinner.current_char(), "|");
    }

    #[test]
    fn test_spinner_reset() {
        let mut spinner = Spinner::new();
        spinner.current_frame = 3;
        spinner.reset();
        assert_eq!(spinner.current_frame, 0);
        assert_eq!(spinner.current_char(), "|");
    }

    #[test]
    fn test_spinner_update_advances_frame_after_duration() {
        use std::thread;
        let mut spinner = Spinner::new();
        assert_eq!(spinner.current_frame, 0);

        // Sleep for 250ms (longer than SPINNER_FRAME_DURATION of 200ms)
        thread::sleep(Duration::from_millis(250));
        spinner.update();
        assert_eq!(spinner.current_frame, 1);
        assert_eq!(spinner.current_char(), "/");
    }

    #[test]
    fn test_spinner_update_does_not_advance_before_duration() {
        use std::thread;
        let mut spinner = Spinner::new();
        assert_eq!(spinner.current_frame, 0);

        // Sleep for 100ms (shorter than SPINNER_FRAME_DURATION of 200ms)
        thread::sleep(Duration::from_millis(100));
        spinner.update();
        assert_eq!(spinner.current_frame, 0);
        assert_eq!(spinner.current_char(), "|");
    }
}
