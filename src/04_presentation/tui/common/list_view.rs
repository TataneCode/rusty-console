//! Generic rendering for list screens.
//!
//! [`render_list`] factors out the identical skeleton of the five list views:
//! split content/help areas, table with filter-aware title, and a help bar
//! that switches between loading spinner, filter input mode, and normal help.

use super::{
    filter_input_help, filter_prompt_title, render_action_bar, render_loading_bar, render_table,
    split_content_area,
};
use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Row, TableState},
    Frame,
};

pub struct ListHelpBar<'text> {
    pub is_loading: bool,
    pub spinner_char: &'text str,
}

pub fn render_list<'frame, TItem>(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    headers: Vec<&'frame str>,
    widths: Vec<Constraint>,
    items: &[TItem],
    row: impl Fn(&TItem) -> Row<'frame>,
    state: &mut TableState,
    active_filter: Option<&str>,
    help_bar: ListHelpBar<'_>,
    help_text: &str,
    filter_enter_label: &str,
) {
    let [content_area, help_area] = split_content_area(area);

    let rows: Vec<Row> = items.iter().map(row).collect();

    let title = filter_prompt_title(title, active_filter);
    render_table(
        frame,
        content_area,
        &title,
        headers,
        rows,
        widths,
        state,
    );

    if help_bar.is_loading {
        render_loading_bar(frame, help_area, help_bar.spinner_char);
    } else {
        let help = match active_filter {
            Some(_) => filter_input_help(filter_enter_label),
            None => help_text.to_string(),
        };
        render_action_bar(frame, help_area, &help);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};

    fn buffer_text(buffer: &Buffer) -> String {
        buffer
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
    }

    struct Row_ {
        name: String,
    }

    #[test]
    fn test_render_list_shows_title_rows_and_help() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let items = vec![
            Row_ {
                name: "alpha".to_string(),
            },
            Row_ {
                name: "beta".to_string(),
            },
        ];
        let mut state = TableState::default();

        terminal
            .draw(|frame| {
                render_list(
                    frame,
                    frame.area(),
                    "Items",
                    vec!["Name"],
                    vec![Constraint::Percentage(100)],
                    &items,
                    |item| Row::new(vec![item.name.clone()]),
                    &mut state,
                    None,
                    ListHelpBar {
                        is_loading: false,
                        spinner_char: "|",
                    },
                    " q: Quit | /: Filter ",
                    "Details",
                );
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Items"));
        assert!(text.contains("Name"));
        assert!(text.contains("alpha"));
        assert!(text.contains("beta"));
        assert!(text.contains("Quit"));
    }

    #[test]
    fn test_render_list_shows_filter_prompt_and_input_help_when_filtering() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let items = vec![Row_ {
            name: "alpha".to_string(),
        }];
        let mut state = TableState::default();

        terminal
            .draw(|frame| {
                render_list(
                    frame,
                    frame.area(),
                    "Items",
                    vec!["Name"],
                    vec![Constraint::Percentage(100)],
                    &items,
                    |item| Row::new(vec![item.name.clone()]),
                    &mut state,
                    Some("alp"),
                    ListHelpBar {
                        is_loading: false,
                        spinner_char: "|",
                    },
                    " q: Quit | /: Filter ",
                    "Details",
                );
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("[/: alp"));
        assert!(text.contains("Type: Filter"));
        assert!(text.contains("Enter: Details"));
        assert!(!text.contains("Quit"));
    }

    #[test]
    fn test_render_list_shows_loading_bar_when_loading() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let items = vec![Row_ {
            name: "alpha".to_string(),
        }];
        let mut state = TableState::default();

        terminal
            .draw(|frame| {
                render_list(
                    frame,
                    frame.area(),
                    "Items",
                    vec!["Name"],
                    vec![Constraint::Percentage(100)],
                    &items,
                    |item| Row::new(vec![item.name.clone()]),
                    &mut state,
                    None,
                    ListHelpBar {
                        is_loading: true,
                        spinner_char: "/",
                    },
                    " q: Quit | /: Filter ",
                    "Details",
                );
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Loading..."));
        assert!(!text.contains("Quit"));
    }
}
