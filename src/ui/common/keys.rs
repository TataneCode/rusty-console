use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppAction {
    Quit,
    Back,
    NavigateUp,
    NavigateDown,
    PageUp,
    PageDown,
    Select,
    ViewLogs,
    StartStop,
    Delete,
    ViewDetails,
    Refresh,
    ScrollUp,
    ScrollDown,
}

pub fn map_key_to_action(key: KeyEvent) -> Option<AppAction> {
    match key.code {
        KeyCode::Char('q') => Some(AppAction::Quit),
        KeyCode::Esc => Some(AppAction::Back),
        KeyCode::Up | KeyCode::Char('k') => Some(AppAction::NavigateUp),
        KeyCode::Down | KeyCode::Char('j') => Some(AppAction::NavigateDown),
        KeyCode::PageUp => Some(AppAction::PageUp),
        KeyCode::PageDown => Some(AppAction::PageDown),
        KeyCode::Enter => Some(AppAction::Select),
        KeyCode::Char('l') => Some(AppAction::ViewLogs),
        KeyCode::Char('s') => Some(AppAction::StartStop),
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(AppAction::ScrollDown)
        }
        KeyCode::Char('d') => Some(AppAction::Delete),
        KeyCode::Char('c') => Some(AppAction::ViewDetails),
        KeyCode::Char('r') => Some(AppAction::Refresh),
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(AppAction::ScrollUp)
        }
        _ => None,
    }
}
