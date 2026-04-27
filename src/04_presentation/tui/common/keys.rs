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
    PauseUnpause,
    Restart,
    Prune,
    ActivateFilter,
    Exec,
    StopAll,
    StartAll,
    RemoveAll,
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
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Some(AppAction::StartAll)
        }
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
        KeyCode::Char('p') => Some(AppAction::PauseUnpause),
        KeyCode::Char('R') => Some(AppAction::Restart),
        KeyCode::Char('X') => Some(AppAction::Prune),
        KeyCode::Char('/') => Some(AppAction::ActivateFilter),
        KeyCode::Char('e') => Some(AppAction::Exec),
        KeyCode::Char('S') => Some(AppAction::StopAll),
        KeyCode::Char('D') => Some(AppAction::RemoveAll),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::empty())
    }

    fn key_event_with_modifiers(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn test_quit() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::Char('q'))),
            Some(AppAction::Quit)
        );
    }

    #[test]
    fn test_back_esc() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::Esc)),
            Some(AppAction::Back)
        );
    }

    #[test]
    fn test_navigate_up_k() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::Char('k'))),
            Some(AppAction::NavigateUp)
        );
    }

    #[test]
    fn test_navigate_up_arrow() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::Up)),
            Some(AppAction::NavigateUp)
        );
    }

    #[test]
    fn test_navigate_down_j() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::Char('j'))),
            Some(AppAction::NavigateDown)
        );
    }

    #[test]
    fn test_navigate_down_arrow() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::Down)),
            Some(AppAction::NavigateDown)
        );
    }

    #[test]
    fn test_select_enter() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::Enter)),
            Some(AppAction::Select)
        );
    }

    #[test]
    fn test_page_up() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::PageUp)),
            Some(AppAction::PageUp)
        );
    }

    #[test]
    fn test_page_down() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::PageDown)),
            Some(AppAction::PageDown)
        );
    }

    #[test]
    fn test_view_logs() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::Char('l'))),
            Some(AppAction::ViewLogs)
        );
    }

    #[test]
    fn test_start_stop() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::Char('s'))),
            Some(AppAction::StartStop)
        );
    }

    #[test]
    fn test_delete() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::Char('d'))),
            Some(AppAction::Delete)
        );
    }

    #[test]
    fn test_view_details() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::Char('c'))),
            Some(AppAction::ViewDetails)
        );
    }

    #[test]
    fn test_refresh() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::Char('r'))),
            Some(AppAction::Refresh)
        );
    }

    #[test]
    fn test_pause_unpause() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::Char('p'))),
            Some(AppAction::PauseUnpause)
        );
    }

    #[test]
    fn test_scroll_down_ctrl_d() {
        assert_eq!(
            map_key_to_action(key_event_with_modifiers(
                KeyCode::Char('d'),
                KeyModifiers::CONTROL
            )),
            Some(AppAction::ScrollDown)
        );
    }

    #[test]
    fn test_scroll_up_ctrl_u() {
        assert_eq!(
            map_key_to_action(key_event_with_modifiers(
                KeyCode::Char('u'),
                KeyModifiers::CONTROL
            )),
            Some(AppAction::ScrollUp)
        );
    }

    #[test]
    fn test_restart() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::Char('R'))),
            Some(AppAction::Restart)
        );
    }

    #[test]
    fn test_prune() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::Char('X'))),
            Some(AppAction::Prune)
        );
    }

    #[test]
    fn test_activate_filter() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::Char('/'))),
            Some(AppAction::ActivateFilter)
        );
    }

    #[test]
    fn test_exec() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::Char('e'))),
            Some(AppAction::Exec)
        );
    }

    #[test]
    fn test_stop_all() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::Char('S'))),
            Some(AppAction::StopAll)
        );
    }

    #[test]
    fn test_start_all_ctrl_s() {
        assert_eq!(
            map_key_to_action(key_event_with_modifiers(
                KeyCode::Char('s'),
                KeyModifiers::CONTROL
            )),
            Some(AppAction::StartAll)
        );
    }

    #[test]
    fn test_remove_all() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::Char('D'))),
            Some(AppAction::RemoveAll)
        );
    }

    #[test]
    fn test_unknown_key_returns_none() {
        assert_eq!(map_key_to_action(key_event(KeyCode::F(12))), None);
    }
}
