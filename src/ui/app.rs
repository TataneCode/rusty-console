use crate::application::{ContainerDto, ImageDto, VolumeDto};
use crate::ui::common::{map_key_to_action, render_confirm_dialog, render_error_popup, AppAction};
use crate::ui::container::{
    render_container_details, render_container_list, render_container_logs, ContainerActions,
    ContainerPresenter,
};
use crate::ui::event::{AppEvent, EventHandler};
use crate::ui::image::{render_image_details, render_image_list, ImageActions, ImagePresenter};
use crate::ui::volume::{render_volume_list, VolumeActions, VolumePresenter};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    MainMenu,
    ContainerList,
    ContainerLogs,
    ContainerDetails,
    VolumeList,
    ImageList,
    ImageDetails,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmAction {
    DeleteContainer(bool),
    DeleteVolume,
    DeleteImage(bool),
    PruneContainers,
    PruneVolumes,
    PruneImages,
}

pub struct App {
    pub screen: Screen,
    pub previous_screen: Option<Screen>,
    pub should_quit: bool,
    pub menu_state: ListState,
    pub container_presenter: ContainerPresenter,
    pub volume_presenter: VolumePresenter,
    pub image_presenter: ImagePresenter,
    pub container_actions: ContainerActions,
    pub volume_actions: VolumeActions,
    pub image_actions: ImageActions,
    pub confirm_dialog: Option<(ConfirmAction, bool)>,
    pub error_message: Option<String>,
}

impl App {
    pub fn new(
        container_actions: ContainerActions,
        volume_actions: VolumeActions,
        image_actions: ImageActions,
    ) -> Self {
        let mut menu_state = ListState::default();
        menu_state.select(Some(0));

        App {
            screen: Screen::MainMenu,
            previous_screen: None,
            should_quit: false,
            menu_state,
            container_presenter: ContainerPresenter::new(),
            volume_presenter: VolumePresenter::new(),
            image_presenter: ImagePresenter::new(),
            container_actions,
            volume_actions,
            image_actions,
            confirm_dialog: None,
            error_message: None,
        }
    }

    pub async fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let event_handler = EventHandler::default();

        loop {
            terminal.draw(|frame| self.render(frame))?;

            match event_handler.next()? {
                AppEvent::Key(key) => {
                    if self.is_filter_active() && self.handle_filter_key(key.code) {
                        // Key consumed by filter input
                    } else if let Some(action) = map_key_to_action(key) {
                        self.handle_action(action).await;
                    }
                }
                AppEvent::Tick => {}
            }

            if self.should_quit {
                break;
            }
        }

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        match &self.screen {
            Screen::MainMenu => self.render_main_menu(frame, area),
            Screen::ContainerList => {
                let filtered: Vec<ContainerDto> = self
                    .container_presenter
                    .filtered_containers()
                    .into_iter()
                    .cloned()
                    .collect();
                let active_filter = if self.container_presenter.filter_active {
                    Some(self.container_presenter.filter.as_str())
                } else {
                    None
                };
                render_container_list(
                    frame,
                    area,
                    &filtered,
                    &mut self.container_presenter.selection.state,
                    active_filter,
                );
            }
            Screen::ContainerLogs => {
                if let Some(logs) = &self.container_presenter.logs {
                    render_container_logs(frame, area, logs, self.container_presenter.logs_scroll);
                }
            }
            Screen::ContainerDetails => {
                if let Some(container) = &self.container_presenter.selected_container {
                    render_container_details(frame, area, container);
                }
            }
            Screen::VolumeList => {
                let filtered: Vec<VolumeDto> = self
                    .volume_presenter
                    .filtered_volumes()
                    .into_iter()
                    .cloned()
                    .collect();
                let active_filter = if self.volume_presenter.filter_active {
                    Some(self.volume_presenter.filter.as_str())
                } else {
                    None
                };
                render_volume_list(
                    frame,
                    area,
                    &filtered,
                    &mut self.volume_presenter.selection.state,
                    active_filter,
                );
            }
            Screen::ImageList => {
                let filtered: Vec<ImageDto> = self
                    .image_presenter
                    .filtered_images()
                    .into_iter()
                    .cloned()
                    .collect();
                let active_filter = if self.image_presenter.filter_active {
                    Some(self.image_presenter.filter.as_str())
                } else {
                    None
                };
                render_image_list(
                    frame,
                    area,
                    &filtered,
                    &mut self.image_presenter.selection.state,
                    active_filter,
                );
            }
            Screen::ImageDetails => {
                if let Some(image) = &self.image_presenter.selected_image {
                    render_image_details(frame, area, image);
                }
            }
        }

        if let Some((action, selected_yes)) = &self.confirm_dialog {
            let message = match action {
                ConfirmAction::DeleteContainer(force) => {
                    if *force {
                        "Force delete this container?"
                    } else {
                        "Delete this container?"
                    }
                }
                ConfirmAction::DeleteVolume => "Delete this volume?",
                ConfirmAction::DeleteImage(force) => {
                    if *force {
                        "Force delete this image?"
                    } else {
                        "Delete this image?"
                    }
                }
                ConfirmAction::PruneContainers => "Prune all stopped containers?",
                ConfirmAction::PruneVolumes => "Prune all unused volumes?",
                ConfirmAction::PruneImages => "Prune all dangling images?",
            };
            render_confirm_dialog(frame, message, *selected_yes);
        }

        if let Some(error) = &self.error_message {
            render_error_popup(frame, error);
        }
    }

    fn render_main_menu(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(2),
            ])
            .split(area);

        let title = Paragraph::new("Rusty Console - Docker Manager")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(title, chunks[0]);

        let items: Vec<ListItem> = vec![
            ListItem::new("  Containers"),
            ListItem::new("  Volumes"),
            ListItem::new("  Images"),
            ListItem::new("  Quit"),
        ];

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Menu ")
                    .title_style(Style::default().fg(Color::Yellow)),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, chunks[1], &mut self.menu_state);

        let help = Paragraph::new(" j/k: Navigate | Enter: Select | q: Quit ")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::TOP));
        frame.render_widget(help, chunks[2]);
    }

    async fn handle_action(&mut self, action: AppAction) {
        if self.error_message.is_some() {
            self.error_message = None;
            return;
        }

        if let Some((confirm_action, selected_yes)) = &mut self.confirm_dialog {
            match action {
                AppAction::NavigateUp | AppAction::NavigateDown => {
                    *selected_yes = !*selected_yes;
                }
                AppAction::Select => {
                    if *selected_yes {
                        let confirm_action = *confirm_action;
                        self.confirm_dialog = None;
                        self.execute_confirm_action(confirm_action).await;
                    } else {
                        self.confirm_dialog = None;
                    }
                }
                AppAction::Back | AppAction::Quit => {
                    self.confirm_dialog = None;
                }
                _ => {}
            }
            return;
        }

        match &self.screen {
            Screen::MainMenu => self.handle_main_menu_action(action).await,
            Screen::ContainerList => self.handle_container_list_action(action).await,
            Screen::ContainerLogs => self.handle_container_logs_action(action),
            Screen::ContainerDetails => self.handle_details_action(action),
            Screen::VolumeList => self.handle_volume_list_action(action).await,
            Screen::ImageList => self.handle_image_list_action(action).await,
            Screen::ImageDetails => self.handle_details_action(action),
        }
    }

    async fn handle_main_menu_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => self.should_quit = true,
            AppAction::NavigateUp => {
                if let Some(selected) = self.menu_state.selected() {
                    let new_selected = if selected == 0 { 3 } else { selected - 1 };
                    self.menu_state.select(Some(new_selected));
                }
            }
            AppAction::NavigateDown => {
                if let Some(selected) = self.menu_state.selected() {
                    let new_selected = if selected >= 3 { 0 } else { selected + 1 };
                    self.menu_state.select(Some(new_selected));
                }
            }
            AppAction::Select => {
                if let Some(selected) = self.menu_state.selected() {
                    match selected {
                        0 => {
                            self.screen = Screen::ContainerList;
                            self.load_containers().await;
                        }
                        1 => {
                            self.screen = Screen::VolumeList;
                            self.load_volumes().await;
                        }
                        2 => {
                            self.screen = Screen::ImageList;
                            self.load_images().await;
                        }
                        3 => self.should_quit = true,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    async fn handle_container_list_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => self.should_quit = true,
            AppAction::Back => self.screen = Screen::MainMenu,
            AppAction::NavigateUp => self.container_presenter.navigate_up(),
            AppAction::NavigateDown => self.container_presenter.navigate_down(),
            AppAction::ViewLogs => {
                if let Some(container) = self.container_presenter.selected_container().cloned() {
                    self.load_container_logs(&container).await;
                    if self.container_presenter.logs.is_some() {
                        self.screen = Screen::ContainerLogs;
                    }
                }
            }
            AppAction::ViewDetails => {
                if let Some(container) = self.container_presenter.selected_container().cloned() {
                    self.load_container_details(&container.id).await;
                    if self.container_presenter.selected_container.is_some() {
                        self.screen = Screen::ContainerDetails;
                    }
                }
            }
            AppAction::StartStop => {
                if let Some(container) = self.container_presenter.selected_container().cloned() {
                    self.toggle_container(&container).await;
                }
            }
            AppAction::Delete => {
                if let Some(container) = self.container_presenter.selected_container() {
                    let force = container.can_stop;
                    self.confirm_dialog = Some((ConfirmAction::DeleteContainer(force), true));
                }
            }
            AppAction::PauseUnpause => {
                if let Some(container) = self.container_presenter.selected_container().cloned() {
                    self.pause_unpause_container(&container).await;
                }
            }
            AppAction::Restart => {
                if let Some(container) = self.container_presenter.selected_container().cloned() {
                    self.restart_container(&container).await;
                }
            }
            AppAction::Refresh => self.load_containers().await,
            AppAction::Prune => {
                self.confirm_dialog = Some((ConfirmAction::PruneContainers, true));
            }
            AppAction::ActivateFilter => {
                self.container_presenter.activate_filter();
            }
            _ => {}
        }
    }

    fn handle_container_logs_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit | AppAction::Back => {
                self.container_presenter.clear_logs();
                self.screen = Screen::ContainerList;
            }
            AppAction::ScrollUp => self.container_presenter.scroll_logs_up(10),
            AppAction::ScrollDown => self.container_presenter.scroll_logs_down(10),
            _ => {}
        }
    }

    fn handle_details_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit | AppAction::Back => match &self.screen {
                Screen::ContainerDetails => {
                    self.container_presenter.clear_details();
                    self.screen = Screen::ContainerList;
                }
                Screen::ImageDetails => {
                    self.image_presenter.clear_details();
                    self.screen = Screen::ImageList;
                }
                _ => {}
            },
            _ => {}
        }
    }

    async fn handle_volume_list_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => self.should_quit = true,
            AppAction::Back => self.screen = Screen::MainMenu,
            AppAction::NavigateUp => self.volume_presenter.navigate_up(),
            AppAction::NavigateDown => self.volume_presenter.navigate_down(),
            AppAction::Delete => {
                if let Some(volume) = self.volume_presenter.selected_volume() {
                    if volume.can_delete {
                        self.confirm_dialog = Some((ConfirmAction::DeleteVolume, true));
                    } else {
                        self.error_message = Some("Cannot delete volume: it is in use".to_string());
                    }
                }
            }
            AppAction::Refresh => self.load_volumes().await,
            AppAction::Prune => {
                self.confirm_dialog = Some((ConfirmAction::PruneVolumes, true));
            }
            AppAction::ActivateFilter => {
                self.volume_presenter.activate_filter();
            }
            _ => {}
        }
    }

    async fn handle_image_list_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => self.should_quit = true,
            AppAction::Back => self.screen = Screen::MainMenu,
            AppAction::NavigateUp => self.image_presenter.navigate_up(),
            AppAction::NavigateDown => self.image_presenter.navigate_down(),
            AppAction::ViewDetails => {
                if let Some(image) = self.image_presenter.selected_image().cloned() {
                    self.image_presenter.set_details(image);
                    self.screen = Screen::ImageDetails;
                }
            }
            AppAction::Delete => {
                if let Some(image) = self.image_presenter.selected_image() {
                    let force = image.in_use;
                    self.confirm_dialog = Some((ConfirmAction::DeleteImage(force), true));
                }
            }
            AppAction::Refresh => self.load_images().await,
            AppAction::Prune => {
                self.confirm_dialog = Some((ConfirmAction::PruneImages, true));
            }
            AppAction::ActivateFilter => {
                self.image_presenter.activate_filter();
            }
            _ => {}
        }
    }

    async fn execute_confirm_action(&mut self, action: ConfirmAction) {
        match action {
            ConfirmAction::DeleteContainer(force) => {
                if let Some(container) = self.container_presenter.selected_container().cloned() {
                    if let Err(e) = self
                        .container_actions
                        .delete_container(&container.id, force)
                        .await
                    {
                        self.error_message = Some(e.to_string());
                    } else {
                        self.load_containers().await;
                    }
                }
            }
            ConfirmAction::DeleteVolume => {
                if let Some(volume) = self.volume_presenter.selected_volume().cloned() {
                    if let Err(e) = self.volume_actions.delete_volume(&volume.name).await {
                        self.error_message = Some(e.to_string());
                    } else {
                        self.load_volumes().await;
                    }
                }
            }
            ConfirmAction::DeleteImage(force) => {
                if let Some(image) = self.image_presenter.selected_image().cloned() {
                    if let Err(e) = self.image_actions.delete_image(&image.id, force).await {
                        self.error_message = Some(e.to_string());
                    } else {
                        self.load_images().await;
                    }
                }
            }
            ConfirmAction::PruneContainers => {
                match self.container_actions.prune_containers().await {
                    Ok(result) => {
                        self.error_message = Some(format!(
                            "Pruned {} container(s), freed {}",
                            result.deleted_count,
                            format_bytes(result.space_freed)
                        ));
                        self.load_containers().await;
                    }
                    Err(e) => self.error_message = Some(e.to_string()),
                }
            }
            ConfirmAction::PruneVolumes => {
                match self.volume_actions.prune_volumes().await {
                    Ok(result) => {
                        self.error_message = Some(format!(
                            "Pruned {} volume(s), freed {}",
                            result.deleted_count,
                            format_bytes(result.space_freed)
                        ));
                        self.load_volumes().await;
                    }
                    Err(e) => self.error_message = Some(e.to_string()),
                }
            }
            ConfirmAction::PruneImages => {
                match self.image_actions.prune_images().await {
                    Ok(result) => {
                        self.error_message = Some(format!(
                            "Pruned {} image(s), freed {}",
                            result.deleted_count,
                            format_bytes(result.space_freed)
                        ));
                        self.load_images().await;
                    }
                    Err(e) => self.error_message = Some(e.to_string()),
                }
            }
        }
    }

    fn is_filter_active(&self) -> bool {
        match self.screen {
            Screen::ContainerList => self.container_presenter.filter_active,
            Screen::VolumeList => self.volume_presenter.filter_active,
            Screen::ImageList => self.image_presenter.filter_active,
            _ => false,
        }
    }

    /// Returns true if the key was consumed by the filter input.
    fn handle_filter_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Esc => {
                match self.screen {
                    Screen::ContainerList => self.container_presenter.deactivate_filter(),
                    Screen::VolumeList => self.volume_presenter.deactivate_filter(),
                    Screen::ImageList => self.image_presenter.deactivate_filter(),
                    _ => {}
                }
                true
            }
            KeyCode::Backspace => {
                match self.screen {
                    Screen::ContainerList => self.container_presenter.pop_filter_char(),
                    Screen::VolumeList => self.volume_presenter.pop_filter_char(),
                    Screen::ImageList => self.image_presenter.pop_filter_char(),
                    _ => {}
                }
                true
            }
            KeyCode::Char(c) => {
                match self.screen {
                    Screen::ContainerList => self.container_presenter.push_filter_char(c),
                    Screen::VolumeList => self.volume_presenter.push_filter_char(c),
                    Screen::ImageList => self.image_presenter.push_filter_char(c),
                    _ => {}
                }
                true
            }
            _ => false,
        }
    }

    async fn load_containers(&mut self) {
        match self.container_actions.load_containers().await {
            Ok(containers) => self.container_presenter.set_containers(containers),
            Err(e) => self.error_message = Some(e.to_string()),
        }
    }

    async fn load_container_details(&mut self, id: &str) {
        match self.container_actions.load_container_details(id).await {
            Ok(Some(container)) => self.container_presenter.set_details(container),
            Ok(None) => self.error_message = Some("Container not found".to_string()),
            Err(e) => self.error_message = Some(e.to_string()),
        }
    }

    async fn load_container_logs(&mut self, container: &ContainerDto) {
        match self.container_actions.load_logs(container, Some(500)).await {
            Ok(logs) => self.container_presenter.set_logs(logs),
            Err(e) => self.error_message = Some(e.to_string()),
        }
    }

    async fn toggle_container(&mut self, container: &ContainerDto) {
        let result = if container.can_stop {
            self.container_actions.stop_container(&container.id).await
        } else if container.can_start {
            self.container_actions.start_container(&container.id).await
        } else {
            return;
        };

        match result {
            Ok(_) => self.load_containers().await,
            Err(e) => self.error_message = Some(e.to_string()),
        }
    }

    async fn pause_unpause_container(&mut self, container: &ContainerDto) {
        let result = if container.can_pause {
            self.container_actions.pause_container(&container.id).await
        } else if container.can_unpause {
            self.container_actions
                .unpause_container(&container.id)
                .await
        } else {
            return;
        };

        match result {
            Ok(_) => self.load_containers().await,
            Err(e) => self.error_message = Some(e.to_string()),
        }
    }

    async fn restart_container(&mut self, container: &ContainerDto) {
        if !container.can_restart {
            return;
        }

        match self.container_actions.restart_container(&container.id).await {
            Ok(_) => self.load_containers().await,
            Err(e) => self.error_message = Some(e.to_string()),
        }
    }

    async fn load_volumes(&mut self) {
        match self.volume_actions.load_volumes().await {
            Ok(volumes) => self.volume_presenter.set_volumes(volumes),
            Err(e) => self.error_message = Some(e.to_string()),
        }
    }

    async fn load_images(&mut self) {
        match self.image_actions.load_images().await {
            Ok(images) => self.image_presenter.set_images(images),
            Err(e) => self.error_message = Some(e.to_string()),
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
