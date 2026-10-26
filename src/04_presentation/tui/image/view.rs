use crate::application::image::ImageDto;
use crate::application::image::ImagePullProgress;
use crate::presentation::tui::common::{
    render_action_bar, render_list, render_loading_bar, render_pull_progress_bar, resources,
    split_content_area, truncate_text, ListHelpBar, ListRenderInput, ListViewSpec, Theme,
};
use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Cell, Paragraph, Row, TableState, Wrap},
    Frame,
};

pub struct ImageHelpBarState<'text> {
    pub is_loading: bool,
    pub spinner_char: &'text str,
    pub pull_progress: Option<&'text ImagePullProgress>,
}

pub fn render_image_list(
    frame: &mut Frame,
    area: Rect,
    images: &[ImageDto],
    state: &mut TableState,
    active_filter: Option<&str>,
    help_bar: ImageHelpBarState<'_>,
) {
    let widths = vec![
        Constraint::Percentage(25),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
        Constraint::Percentage(12),
        Constraint::Percentage(8),
        Constraint::Percentage(25),
    ];

    let spec = ListViewSpec {
        title: resources::IMAGE_TITLE,
        headers: resources::IMAGE_HEADERS.to_vec(),
        widths,
        filter_enter_label: "Details",
    };

    let mut input = ListRenderInput {
        items: images,
        state,
        active_filter,
        help_bar: ListHelpBar {
            is_loading: help_bar.is_loading,
            spinner_char: help_bar.spinner_char,
        },
    };
    render_list(frame, area, &spec, &mut input, |img| {
        let style = if img.is_dangling {
            Theme::dangling_style()
        } else if img.in_use {
            Theme::in_use_style()
        } else {
            Theme::default_style()
        };

        let in_use_text = if img.in_use {
            resources::VALUE_YES
        } else {
            resources::VALUE_NO
        };
        let in_use_style = if img.in_use {
            Theme::in_use_style()
        } else {
            Theme::default_style()
        };

        Row::new(vec![
            Cell::from(truncate_text(&img.repository, 30)).style(style),
            Cell::from(img.tag.clone()).style(style),
            Cell::from(img.short_id.clone()),
            Cell::from(img.size.clone()),
            Cell::from(in_use_text).style(in_use_style),
            Cell::from(img.created.clone()),
        ])
    });

    // The pull progress bar redraws over the help bar while a pull runs.
    if let Some(progress) = help_bar.pull_progress {
        let help_area = split_content_area(area)[1];
        render_pull_progress_bar(frame, help_area, progress);
    }
}

pub fn render_image_details(
    frame: &mut Frame,
    area: Rect,
    image: &ImageDto,
    help_bar: ImageHelpBarState<'_>,
) {
    let [content_area, help_area] = split_content_area(area);

    let details = format!(
        "{:<12}{}\n\
         {:<12}{}\n\
         {:<12}{}\n\
         {:<12}{}\n\
         {:<12}{}\n\
         {:<12}{}\n\
         {:<12}{}\n\
         {:<12}{}",
        resources::LABEL_ID,
        image.id,
        resources::LABEL_REPOSITORY,
        image.repository,
        resources::LABEL_TAG,
        image.tag,
        resources::LABEL_FULL_NAME,
        image.full_name,
        resources::LABEL_SIZE,
        image.size,
        resources::LABEL_CREATED,
        image.created,
        resources::LABEL_IN_USE,
        if image.in_use {
            resources::VALUE_YES
        } else {
            resources::VALUE_NO
        },
        resources::LABEL_DANGLING,
        if image.is_dangling {
            resources::VALUE_YES
        } else {
            resources::VALUE_NO
        },
    );

    let details_widget = Paragraph::new(details)
        .block(Theme::section_block(Some(resources::IMAGE_DETAILS_TITLE)))
        .wrap(Wrap { trim: false });

    frame.render_widget(details_widget, content_area);

    if let Some(progress) = help_bar.pull_progress {
        render_pull_progress_bar(frame, help_area, progress);
    } else if help_bar.is_loading {
        render_loading_bar(frame, help_area, help_bar.spinner_char);
    } else {
        render_action_bar(frame, help_area, resources::IMAGE_DETAILS_HELP);
    }
}

#[cfg(test)]
mod tests {
    use super::{render_image_details, render_image_list, ImageHelpBarState};
    use crate::application::image::ImageDto;
    use ratatui::{backend::TestBackend, buffer::Buffer, widgets::TableState, Terminal};

    fn buffer_text(buffer: &Buffer) -> String {
        buffer
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
    }

    fn make_image() -> ImageDto {
        ImageDto {
            id: "sha256:abc".to_string(),
            short_id: "abc".to_string(),
            repository: "nginx".to_string(),
            tag: "latest".to_string(),
            full_name: "nginx:latest".to_string(),
            size: "12 MB".to_string(),
            created: "2024-01-01".to_string(),
            in_use: true,
            is_dangling: false,
            can_delete: true,
        }
    }

    #[test]
    fn test_render_image_list_shows_title_row_and_help() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let image = make_image();
        let items = vec![image];
        let mut state = TableState::default();

        terminal
            .draw(|frame| {
                render_image_list(
                    frame,
                    frame.area(),
                    &items,
                    &mut state,
                    None,
                    ImageHelpBarState {
                        is_loading: false,
                        spinner_char: "|",
                        pull_progress: None,
                    },
                )
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Images"));
        assert!(text.contains("nginx"));
        assert!(text.contains("latest"));
        assert!(text.contains("Space: All commands"));
        assert!(!text.contains("Delete"));
    }

    #[test]
    fn test_render_image_list_shows_contextual_help_while_filter_active() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let items = vec![make_image()];
        let mut state = TableState::default();

        terminal
            .draw(|frame| {
                render_image_list(
                    frame,
                    frame.area(),
                    &items,
                    &mut state,
                    Some("ng"),
                    ImageHelpBarState {
                        is_loading: false,
                        spinner_char: "|",
                        pull_progress: None,
                    },
                )
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Type: Filter"));
        assert!(text.contains("j/k: Navigate"));
        assert!(text.contains("Enter: Details"));
        assert!(text.contains("Esc: Clear"));
        assert!(!text.contains("Quit"));
        assert!(!text.contains("Delete"));
        assert!(!text.contains("Prune"));
    }

    #[test]
    fn test_render_image_details_shows_expected_fields() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let image = make_image();

        terminal
            .draw(|frame| {
                render_image_details(
                    frame,
                    frame.area(),
                    &image,
                    ImageHelpBarState {
                        is_loading: false,
                        spinner_char: "|",
                        pull_progress: None,
                    },
                )
            })
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Image Details"));
        assert!(text.contains("nginx:latest"));
        assert!(text.contains("In Use"));
        assert!(text.contains("Yes"));
    }
}
