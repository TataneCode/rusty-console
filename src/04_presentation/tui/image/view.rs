use crate::application::image::ImageDto;
use crate::presentation::tui::common::{
    filter_prompt_title, render_help, render_table, resources, split_content_area, truncate_text,
    Theme,
};
use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Block, Borders, Cell, Paragraph, Row, TableState, Wrap},
    Frame,
};

pub fn render_image_list(
    frame: &mut Frame,
    area: Rect,
    images: &[&ImageDto],
    state: &mut TableState,
    active_filter: Option<&str>,
) {
    let [content_area, help_area] = split_content_area(area);

    let rows: Vec<Row> = images
        .iter()
        .copied()
        .map(|img| {
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
        })
        .collect();

    let widths = vec![
        Constraint::Percentage(25),
        Constraint::Percentage(15),
        Constraint::Percentage(15),
        Constraint::Percentage(12),
        Constraint::Percentage(8),
        Constraint::Percentage(25),
    ];

    let title = filter_prompt_title(resources::IMAGE_TITLE, active_filter);
    render_table(
        frame,
        content_area,
        &title,
        resources::IMAGE_HEADERS.to_vec(),
        rows,
        widths,
        state,
    );

    render_help(frame, help_area, resources::IMAGE_LIST_HELP);
}

pub fn render_image_details(frame: &mut Frame, area: Rect, image: &ImageDto) {
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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(resources::IMAGE_DETAILS_TITLE)
                .title_style(Theme::title_style())
                .border_style(Theme::border_style()),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(details_widget, content_area);

    render_help(frame, help_area, resources::IMAGE_DETAILS_HELP);
}

#[cfg(test)]
mod tests {
    use super::{render_image_details, render_image_list};
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
        let items = vec![&image];
        let mut state = TableState::default();

        terminal
            .draw(|frame| render_image_list(frame, frame.area(), &items, &mut state, Some("ng")))
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Images"));
        assert!(text.contains("nginx"));
        assert!(text.contains("latest"));
        assert!(text.contains("Delete"));
    }

    #[test]
    fn test_render_image_details_shows_expected_fields() {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        let image = make_image();

        terminal
            .draw(|frame| render_image_details(frame, frame.area(), &image))
            .unwrap();

        let text = buffer_text(terminal.backend().buffer());
        assert!(text.contains("Image Details"));
        assert!(text.contains("nginx:latest"));
        assert!(text.contains("In Use"));
        assert!(text.contains("Yes"));
    }
}
