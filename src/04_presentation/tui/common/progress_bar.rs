use super::theme::Theme;
use crate::application::image::ImagePullProgress;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

const BAR_WIDTH: usize = 20;
const BAR_FILLED: char = '=';
const BAR_HEAD: char = '>';
const BAR_EMPTY: char = ' ';

/// Formats a byte count the same way `docker pull` does (binary units, e.g. 12.4MB).
pub fn format_pull_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let value = bytes as f64;
    if value >= GB {
        format!("{:.1}GB", value / GB)
    } else if value >= MB {
        format!("{:.1}MB", value / MB)
    } else if value >= KB {
        format!("{:.1}kB", value / KB)
    } else {
        format!("{bytes}B")
    }
}

pub fn pull_progress_bar_text(progress: &ImagePullProgress) -> String {
    let percent = progress.percent().min(100) as usize;
    let filled = ((BAR_WIDTH * percent) / 100).min(BAR_WIDTH);
    let head = if percent < 100 && filled < BAR_WIDTH {
        1
    } else {
        0
    };
    let empty = BAR_WIDTH - filled - head;
    format!(
        "[{}{}{}] {}/{} ({percent}%) {}",
        BAR_FILLED.to_string().repeat(filled),
        if head == 1 {
            BAR_HEAD.to_string()
        } else {
            String::new()
        },
        BAR_EMPTY.to_string().repeat(empty),
        format_pull_bytes(progress.downloaded_bytes),
        format_pull_bytes(progress.total_bytes),
        truncate_image(&progress.image)
    )
}

fn truncate_image(image: &str) -> &str {
    const MAX_IMAGE_LEN: usize = 40;
    if image.len() <= MAX_IMAGE_LEN {
        image
    } else {
        &image[image.len() - MAX_IMAGE_LEN..]
    }
}

/// Renders a `docker pull`-style progress bar in the help bar area during an image pull.
pub fn render_pull_progress_bar(frame: &mut Frame, area: Rect, progress: &ImagePullProgress) {
    let line = Line::from(vec![
        Span::styled(" ".to_string(), Theme::default_style()),
        Span::styled(
            pull_progress_bar_text(progress),
            Theme::action_label_style(),
        ),
    ]);
    let bar = Paragraph::new(line).block(Theme::section_block(None));
    frame.render_widget(bar, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_progress(downloaded: u64, total: u64) -> ImagePullProgress {
        ImagePullProgress {
            image: "nginx:latest".to_string(),
            downloaded_bytes: downloaded,
            total_bytes: total,
        }
    }

    #[test]
    fn test_format_pull_bytes_uses_docker_units() {
        assert_eq!(format_pull_bytes(0), "0B");
        assert_eq!(format_pull_bytes(512), "512B");
        assert_eq!(format_pull_bytes(12_884_901), "12.3MB");
        assert_eq!(format_pull_bytes(1_610_612_736), "1.5GB");
    }

    #[test]
    fn test_pull_progress_bar_text_renders_partial_bar() {
        let progress = make_progress(2_500_000, 10_000_000);
        let text = pull_progress_bar_text(&progress);
        assert!(text.contains('>'), "bar should have a head marker: {text}");
        assert!(
            text.contains("2.4MB/9.5MB (25%)"),
            "unexpected text: {text}"
        );
        assert!(text.ends_with("nginx:latest"));
    }

    #[test]
    fn test_pull_progress_bar_text_completed_bar() {
        let progress = make_progress(10_000_000, 10_000_000);
        let text = pull_progress_bar_text(&progress);
        assert!(text.contains("(100%)"));
        assert!(!text.contains('>'));
        assert!(text.contains("9.5MB/9.5MB"));
    }

    #[test]
    fn test_pull_progress_bar_text_zero_total_shows_zero_percent() {
        let progress = make_progress(0, 0);
        let text = pull_progress_bar_text(&progress);
        assert!(text.contains("(0%)"));
        assert!(text.contains("0B/0B"));
    }

    #[test]
    fn test_truncate_image_keeps_tail_of_long_references() {
        let long = "a".repeat(60);
        let truncated = truncate_image(&long);
        assert_eq!(truncated.len(), 40);
        assert!(truncated.ends_with('a'));
    }

    #[test]
    fn test_render_pull_progress_bar_displays_progress() {
        use ratatui::{backend::TestBackend, Terminal};
        let backend = TestBackend::new(100, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        let progress = make_progress(2_500_000, 10_000_000);
        terminal
            .draw(|frame| {
                render_pull_progress_bar(frame, frame.area(), &progress);
            })
            .unwrap();
        let text: String = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol().to_string())
            .collect();
        assert!(text.contains("2.4MB/9.5MB (25%)"));
        assert!(text.contains("nginx:latest"));
    }
}
