use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;

pub enum AppEvent {
    Key(KeyEvent),
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
            if let Event::Key(key) = event::read()? {
                return Ok(AppEvent::Key(key));
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
