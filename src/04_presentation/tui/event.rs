use crossterm::event::{self, Event, KeyEvent, KeyEventKind, MouseEvent};
use std::time::Duration;

pub enum AppEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Tick,
}

pub struct EventHandler {
    tick_rate: Duration,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        EventHandler { tick_rate }
    }

    pub fn next(&self) -> Result<AppEvent, std::io::Error> {
        if event::poll(self.tick_rate)? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    return Ok(AppEvent::Key(key));
                }
                Event::Mouse(mouse) => return Ok(AppEvent::Mouse(mouse)),
                _ => {}
            }
        }
        Ok(AppEvent::Tick)
    }
}

impl Default for EventHandler {
    fn default() -> Self {
        EventHandler::new(Duration::from_millis(250))
    }
}
