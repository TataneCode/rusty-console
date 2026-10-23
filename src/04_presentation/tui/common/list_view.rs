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

/// Static description of a list screen: what the table shows and what the
/// help bar says. The variable parts (items, selection, filter, loading)
/// are passed separately to [`render_list`].
pub struct ListViewSpec<'frame> {
    pub title: &'frame str,
    pub headers: Vec<&'frame str>,
    pub widths: Vec<Constraint>,
    pub help_text: &'frame str,
    pub filter_enter_label: &'frame str,
}

/// Runtime inputs of a list screen render: the items to display, their
/// selection state, the active filter text and the loading/spinner state.
pub struct ListRenderInput<'items, 'text, TItem> {
    pub items: &'items [TItem],
    pub state: &'items mut TableState,
    pub active_filter: Option<&'text str>,
    pub help_bar: ListHelpBar<'text>,
}

pub fn render_list<'frame, TItem>(
    frame: &mut Frame,
    area: Rect,
    spec: &ListViewSpec<'frame>,
    input: &mut ListRenderInput<'_, '_, TItem>,
    row: impl Fn(&TItem) -> Row<'frame>,
) {
    let ListRenderInput {
        items,
        state,
        active_filter,
        help_bar,
    } = input;
    let [content_area, help_area] = split_content_area(area);

    let rows: Vec<Row> = items.iter().map(row).collect();

    let title = filter_prompt_title(spec.title, *active_filter);
    render_table(
        frame,
        content_area,
        &title,
        spec.headers.clone(),
        rows,
        spec.widths.clone(),
        state,
    );

    if help_bar.is_loading {
        render_loading_bar(frame, help_area, help_bar.spinner_char);
    } else {
        let help = match active_filter {
            Some(_) => filter_input_help(spec.filter_enter_label),
            None => spec.help_text.to_string(),
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

    fn make_spec() -> ListViewSpec<'static> {
        ListViewSpec {
            title: "Items",
            headers: vec!["Name"],
            widths: vec![Constraint::Percentage(100)],
            help_text: " q: Quit | /: Filter ",
            filter_enter_label: "Details",
        }
    }

    fn draw_list(
        terminal: &mut Terminal<TestBackend>,
        items: &[Row_],
        active_filter: Option<&str>,
        is_loading: bool,
        spinner_char: &str,
    ) {
        let mut state = TableState::default();
        terminal
            .draw(|frame| {
                let mut input = ListRenderInput {
                    items,
                    state: &mut state,
                    active_filter,
                    help_bar: ListHelpBar {
                        is_loading,
                        spinner_char,
                    },
                };
                render_list(frame, frame.area(), &make_spec(), &mut input, |item| {
                    Row::new(vec![item.name.clone()])
                });
            })
            .unwrap();
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

        draw_list(&mut terminal, &items, None, false, "|");

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

        draw_list(&mut terminal, &items, Some("alp"), false, "|");

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

        draw_list(&mut terminal, &items, None, true, "/");

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Loading..."));
        assert!(!text.contains("Quit"));
    }
}
