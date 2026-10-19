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
    PullImages,
    Add,
    ToggleSort,
    ActivateLogSearch,
    NextLogMatch,
    PreviousLogMatch,
    ExportLogs,
    DisconnectContainer,
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
        KeyCode::Char('n') => Some(AppAction::NextLogMatch),
        KeyCode::Char('N') => Some(AppAction::PreviousLogMatch),
        KeyCode::Char('e') => Some(AppAction::Exec),
        KeyCode::Char('E') => Some(AppAction::ExportLogs),
        KeyCode::Char('S') => Some(AppAction::StopAll),
        KeyCode::Char('D') => Some(AppAction::RemoveAll),
        KeyCode::Char('P') => Some(AppAction::PullImages),
        KeyCode::Char('A') => Some(AppAction::Add),
        KeyCode::Char('o') => Some(AppAction::ToggleSort),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyContext {
    Global,
    ContainerLogs,
    NetworkDetails,
}

pub fn map_key_to_action_for_context(key: KeyEvent, context: KeyContext) -> Option<AppAction> {
    let action = map_key_to_action(key)?;
    match context {
        KeyContext::ContainerLogs => match action {
            AppAction::ActivateFilter => Some(AppAction::ActivateLogSearch),
            other => Some(other),
        },
        KeyContext::NetworkDetails => match action {
            AppAction::Delete => Some(AppAction::DisconnectContainer),
            other => Some(other),
        },
        KeyContext::Global => match action {
            AppAction::NextLogMatch | AppAction::PreviousLogMatch => None,
            AppAction::ActivateLogSearch => None,
            AppAction::ExportLogs => None,
            AppAction::DisconnectContainer => None,
            other => Some(other),
        },
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
    fn test_pull_images() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::Char('P'))),
            Some(AppAction::PullImages)
        );
    }

    #[test]
    fn test_add_key() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::Char('A'))),
            Some(AppAction::Add)
        );
    }

    #[test]
    fn test_toggle_sort_key() {
        assert_eq!(
            map_key_to_action(key_event(KeyCode::Char('o'))),
            Some(AppAction::ToggleSort)
        );
    }

    #[test]
    fn test_toggle_sort_allowed_in_global_context() {
        assert_eq!(
            map_key_to_action_for_context(key_event(KeyCode::Char('o')), KeyContext::Global),
            Some(AppAction::ToggleSort)
        );
    }

    #[test]
    fn test_unknown_key_returns_none() {
        assert_eq!(map_key_to_action(key_event(KeyCode::F(12))), None);
    }

    #[test]
    fn test_export_logs_key_e() {
        assert_eq!(
            map_key_to_action_for_context(key_event(KeyCode::Char('E')), KeyContext::ContainerLogs),
            Some(AppAction::ExportLogs)
        );
        assert_eq!(
            map_key_to_action_for_context(key_event(KeyCode::Char('E')), KeyContext::Global),
            None
        );
    }

    #[test]
    fn test_n_keys_inert_off_logs_screen() {
        assert_eq!(
            map_key_to_action_for_context(key_event(KeyCode::Char('n')), KeyContext::Global),
            None
        );
        assert_eq!(
            map_key_to_action_for_context(key_event(KeyCode::Char('N')), KeyContext::Global),
            None
        );
        assert_eq!(
            map_key_to_action_for_context(key_event(KeyCode::Char('n')), KeyContext::ContainerLogs),
            Some(AppAction::NextLogMatch)
        );
    }

    #[test]
    fn test_d_maps_to_disconnect_only_on_network_details() {
        assert_eq!(
            map_key_to_action_for_context(
                key_event(KeyCode::Char('d')),
                KeyContext::NetworkDetails
            ),
            Some(AppAction::DisconnectContainer)
        );
        assert_eq!(
            map_key_to_action_for_context(key_event(KeyCode::Char('d')), KeyContext::Global),
            Some(AppAction::Delete)
        );
    }
}
