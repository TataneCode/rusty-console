use ratatui::{
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, BorderType},
};

pub struct Theme;

impl Theme {
    pub fn default_style() -> Style {
        Style::default().fg(Color::White)
    }

    pub fn highlight_style() -> Style {
        Style::default()
            .bg(Color::DarkGray)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD)
    }

    pub fn header_style() -> Style {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    }

    pub fn title_style() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }

    pub fn running_style() -> Style {
        Style::default().fg(Color::Green)
    }

    pub fn stopped_style() -> Style {
        Style::default().fg(Color::Red)
    }

    pub fn paused_style() -> Style {
        Style::default().fg(Color::Yellow)
    }

    pub fn in_use_style() -> Style {
        Style::default().fg(Color::Yellow)
    }

    pub fn dangling_style() -> Style {
        Style::default().fg(Color::Red)
    }

    pub fn info_style() -> Style {
        Style::default().fg(Color::Cyan)
    }

    pub fn error_style() -> Style {
        Style::default().fg(Color::Red)
    }

    pub fn success_style() -> Style {
        Style::default().fg(Color::Green)
    }

    pub fn help_style() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    pub fn border_style() -> Style {
        Style::default().fg(Color::White)
    }

    pub fn selected_border_style() -> Style {
        Style::default().fg(Color::Cyan)
    }

    pub fn action_key_style() -> Style {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    }

    pub fn action_label_style() -> Style {
        Style::default().fg(Color::White)
    }

    /// Shared rounded box for the three layout sections (breadcrumb, main
    /// content, action bar) so borders stay homogeneous across screens.
    pub fn section_block(title: Option<&str>) -> Block<'static> {
        let mut block = Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Self::border_style());
        if let Some(title) = title {
            block = block.title(Line::from(title.to_string()).style(Self::title_style()));
        }
        block
    }
}

#[cfg(test)]
mod tests {
    use super::Theme;
    use ratatui::style::{Color, Modifier};

    #[test]
    fn test_default_related_styles() {
        assert_eq!(Theme::default_style().fg, Some(Color::White));
        assert_eq!(Theme::border_style().fg, Some(Color::White));
        let selected_border = Theme::selected_border_style();
        assert_eq!(selected_border.fg, Some(Color::Cyan));
        assert_eq!(Theme::help_style().fg, Some(Color::DarkGray));
    }

    #[test]
    fn test_section_block_renders_rounded_border_and_optional_title() {
        use ratatui::{backend::TestBackend, widgets::Paragraph, Terminal};

        fn render_to_string(block: ratatui::widgets::Block<'static>) -> String {
            let backend = TestBackend::new(20, 4);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|frame| frame.render_widget(Paragraph::new("").block(block), frame.area()))
                .unwrap();
            terminal
                .backend()
                .buffer()
                .content
                .iter()
                .map(|cell| cell.symbol())
                .collect()
        }

        let untitled = render_to_string(Theme::section_block(None));
        assert!(untitled.contains('╭'));
        assert!(untitled.contains('╰'));
        assert!(!untitled.contains("Containers"));

        let titled = render_to_string(Theme::section_block(Some("Containers")));
        assert!(titled.contains('╭'));
        assert!(titled.contains("Containers"));
    }

    #[test]
    fn test_semantic_color_styles() {
        assert_eq!(Theme::running_style().fg, Some(Color::Green));
        assert_eq!(Theme::stopped_style().fg, Some(Color::Red));
        assert_eq!(Theme::paused_style().fg, Some(Color::Yellow));
        assert_eq!(Theme::in_use_style().fg, Some(Color::Yellow));
        assert_eq!(Theme::dangling_style().fg, Some(Color::Red));
        assert_eq!(Theme::error_style().fg, Some(Color::Red));
        assert_eq!(Theme::info_style().fg, Some(Color::Cyan));
        assert_eq!(Theme::success_style().fg, Some(Color::Green));
    }

    #[test]
    fn test_action_bar_styles() {
        let key = Theme::action_key_style();
        assert_eq!(key.fg, Some(Color::Yellow));
        assert!(key.add_modifier.contains(Modifier::BOLD));

        let label = Theme::action_label_style();
        assert_eq!(label.fg, Some(Color::White));
    }

    #[test]
    fn test_highlight_header_and_title_styles() {
        let highlight = Theme::highlight_style();
        assert_eq!(highlight.fg, Some(Color::White));
        assert_eq!(highlight.bg, Some(Color::DarkGray));
        assert!(highlight.add_modifier.contains(Modifier::BOLD));

        let header = Theme::header_style();
        assert_eq!(header.fg, Some(Color::Cyan));
        assert!(header.add_modifier.contains(Modifier::BOLD));

        let title = Theme::title_style();
        assert_eq!(title.fg, Some(Color::Yellow));
        assert!(title.add_modifier.contains(Modifier::BOLD));
    }
}
