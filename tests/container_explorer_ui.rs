mod common;

use common::build_app;
use ratatui::style::{Color, Modifier};
use rusty_console::application::container::{FileEntryDto, FileListingDto};
use rusty_console::domain::container::FileKind;
use rusty_console::presentation::tui::app::{App, Screen};
use rusty_console::presentation::tui::common::keys::AppAction;

fn make_entry(name: &str, kind: FileKind) -> FileEntryDto {
    FileEntryDto {
        name: name.to_string(),
        path: format!("/{name}"),
        kind,
        size: 1536,
        modified: None,
        permissions: if kind == FileKind::Directory {
            "drwxr-xr-x".to_string()
        } else {
            "-rw-r--r--".to_string()
        },
        link_target: None,
    }
}

fn explorer_app() -> Option<App> {
    let mut app = build_app()?;
    app.container_explorer.open("abc123", "rc-explorer-web");
    app.container_explorer.set_listing(FileListingDto {
        container_id: "abc123".to_string(),
        container_name: "rc-explorer-web".to_string(),
        path: "/".to_string(),
        entries: vec![
            make_entry("etc", FileKind::Directory),
            make_entry("app.conf", FileKind::File),
        ],
    });
    app.screen = Screen::ContainerExplorer;
    Some(app)
}

fn render_app_text_and_styles(app: &mut App) -> (String, Vec<(Color, Color, Modifier)>) {
    let backend = ratatui::backend::TestBackend::new(100, 20);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();

    let buffer = terminal.backend().buffer();
    let text: String = buffer.content.iter().map(|cell| cell.symbol()).collect();
    let styles = buffer
        .content
        .iter()
        .map(|cell| (cell.bg, cell.fg, cell.modifier))
        .collect();
    (text, styles)
}

fn find_text_cell_index(buffer: &ratatui::buffer::Buffer, target: &str) -> Option<usize> {
    let symbols: Vec<&str> = buffer.content.iter().map(|cell| cell.symbol()).collect();
    (0..symbols.len()).find(|&i| {
        target
            .chars()
            .enumerate()
            .all(|(k, ch)| symbols.get(i + k).and_then(|s| s.chars().next()) == Some(ch))
    })
}

#[test]
fn explorer_screen_shows_columns_entries_and_help() {
    let Some(mut app) = explorer_app() else {
        return;
    };
    let (text, _) = render_app_text_and_styles(&mut app);
    assert!(
        text.contains("Files: rc-explorer-web /"),
        "missing title: {text}"
    );
    assert!(text.contains("Name"));
    assert!(text.contains("Size"));
    assert!(text.contains("Modified"));
    assert!(text.contains("Permissions"));
    assert!(text.contains("etc/"));
    assert!(text.contains("app.conf"));
    assert!(text.contains("1.50 KB"));
    assert!(text.contains("drwxr-xr-x"));
    assert!(text.contains("Space: All commands"));
}

#[test]
fn explorer_screen_breadcrumb_shows_container_and_files_path() {
    let Some(mut app) = explorer_app() else {
        return;
    };
    let (text, _) = render_app_text_and_styles(&mut app);
    assert!(text.contains("Rusty - docker manager"));
    assert!(text.contains("Containers"));
    assert!(text.contains("rc-explorer-web files /"));
}

#[test]
fn explorer_screen_directories_have_their_own_color() {
    let Some(mut app) = explorer_app() else {
        return;
    };
    let backend = ratatui::backend::TestBackend::new(100, 20);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    let buffer = terminal.backend().buffer();

    let dir_index = find_text_cell_index(buffer, "etc/").expect("directory shown");
    let file_index = find_text_cell_index(buffer, "app.conf").expect("file shown");
    assert_eq!(
        buffer.content[dir_index].fg,
        Color::Cyan,
        "directories are cyan"
    );
    assert_eq!(
        buffer.content[file_index].fg,
        Color::White,
        "files are white"
    );
}

#[test]
fn explorer_screen_selected_entry_is_highlighted_like_logs() {
    let Some(mut app) = explorer_app() else {
        return;
    };
    app.container_explorer.navigate_down();
    let backend = ratatui::backend::TestBackend::new(100, 20);
    let mut terminal = ratatui::Terminal::new(backend).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    let buffer = terminal.backend().buffer();

    let selected_index = find_text_cell_index(buffer, "app.conf").expect("file shown");
    let selected = &buffer.content[selected_index];
    assert_eq!(selected.bg, Color::DarkGray, "selected row background");
    assert!(
        selected.modifier.contains(Modifier::BOLD),
        "selected row is bold"
    );

    let other_index = find_text_cell_index(buffer, "etc/").expect("directory shown");
    assert_ne!(buffer.content[other_index].bg, Color::DarkGray);
}

#[tokio::test]
async fn explorer_space_opens_help_popup_with_explorer_commands() {
    let Some(mut app) = explorer_app() else {
        return;
    };
    app.handle_action(AppAction::ToggleHelp).await;
    assert!(app.help_popup_open);

    let (text, _) = render_app_text_and_styles(&mut app);
    assert!(text.contains("Keybindings"));
    assert!(text.contains("c: Copy"));
    assert!(text.contains("v: View"));
    assert!(text.contains("h: Up"));
    assert!(text.contains("o: Sort"));
    assert!(text.contains("/: Search"));

    app.handle_action(AppAction::ToggleHelp).await;
    assert!(!app.help_popup_open);
}

#[tokio::test]
async fn explorer_esc_returns_to_origin_screen() {
    let Some(mut app) = explorer_app() else {
        return;
    };
    app.handle_action(AppAction::Back).await;
    assert_eq!(app.screen, Screen::ContainerList);
    assert!(app.container_explorer.listing.is_none());
}

#[tokio::test]
async fn explorer_q_returns_to_origin_screen_instead_of_quitting() {
    let Some(mut app) = explorer_app() else {
        return;
    };
    app.handle_action(AppAction::Quit).await;
    assert!(
        !app.should_quit,
        "q must not quit the app from the explorer"
    );
    assert_eq!(app.screen, Screen::ContainerList);
    assert!(app.container_explorer.listing.is_none());
}

#[tokio::test]
async fn explorer_sort_cycles_three_states() {
    let Some(mut app) = explorer_app() else {
        return;
    };
    let names = |app: &App| -> Vec<String> {
        app.container_explorer
            .filtered_items()
            .iter()
            .map(|e| e.name.clone())
            .collect()
    };
    assert_eq!(names(&app), vec!["etc", "app.conf"]);
    app.handle_action(AppAction::ToggleSort).await;
    assert_eq!(names(&app), vec!["app.conf", "etc"]);
    app.handle_action(AppAction::ToggleSort).await;
    assert_eq!(names(&app), vec!["etc", "app.conf"]);
    app.handle_action(AppAction::ToggleSort).await;
    assert_eq!(names(&app), vec!["etc", "app.conf"]);
}

#[test]
fn explorer_filter_input_shows_contextual_help_bar() {
    let Some(mut app) = explorer_app() else {
        return;
    };
    app.container_explorer.activate_filter();
    app.container_explorer.push_filter_char('a');
    app.container_explorer.push_filter_char('p');
    let (text, _) = render_app_text_and_styles(&mut app);
    assert!(text.contains("[/: ap"), "filter prompt in title: {text}");
    assert!(text.contains("Type: Filter"));
    assert!(text.contains("Enter: Open"));
    assert!(text.contains("Esc: Clear"));
    assert!(!text.contains("Quit"));
}
