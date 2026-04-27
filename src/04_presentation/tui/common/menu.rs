use super::{render_help, resources, split_menu_area, Theme};
use ratatui::{
    layout::Rect,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn render_main_menu(frame: &mut Frame, area: Rect, state: &mut ListState) {
    let [title_area, menu_area, help_area] = split_menu_area(area);

    let title = Paragraph::new(resources::MAIN_MENU_TITLE)
        .style(Theme::title_style())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Theme::border_style()),
        );
    frame.render_widget(title, title_area);

    let items: Vec<ListItem> = resources::MAIN_MENU_ITEMS
        .iter()
        .copied()
        .map(ListItem::new)
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(resources::MAIN_MENU_BLOCK_TITLE)
                .title_style(Theme::title_style())
                .border_style(Theme::border_style()),
        )
        .highlight_style(Theme::highlight_style())
        .highlight_symbol(resources::MAIN_MENU_HIGHLIGHT_SYMBOL);

    frame.render_stateful_widget(list, menu_area, state);
    render_help(frame, help_area, resources::MAIN_MENU_HELP);
}

#[cfg(test)]
mod tests {
    use super::render_main_menu;
    use crate::presentation::tui::common::resources;
    use ratatui::{backend::TestBackend, buffer::Buffer, widgets::ListState, Terminal};

    fn buffer_text(buffer: &Buffer) -> String {
        buffer
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
    }

    #[test]
    fn test_render_main_menu_displays_core_text() {
        let backend = TestBackend::new(80, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let mut state = ListState::default();
        state.select(Some(1));

        terminal
            .draw(|frame| render_main_menu(frame, frame.area(), &mut state))
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains(resources::MAIN_MENU_TITLE));
        assert!(text.contains(resources::MAIN_MENU_BLOCK_TITLE.trim()));
        assert!(text.contains(resources::MAIN_MENU_HELP.trim()));
        assert!(text.contains("Containers"));
        assert!(text.contains("Volumes"));
        assert!(text.contains("Images"));
        assert!(text.contains("Stacks"));
        assert!(text.contains("Quit"));
    }
}
