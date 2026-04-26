use ratatui::style::{Color, Modifier, Style};

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
}
