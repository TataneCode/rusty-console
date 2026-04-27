use crate::application::container::ContainerDto;
use crate::presentation::tui::common::{
    map_key_to_action, render_confirm_dialog, render_error_popup, render_main_menu, resources,
    AppAction, PopupMessage,
};
use crate::presentation::tui::container::{
    filter_containers, render_container_details, render_container_list, render_container_logs,
    ContainerActions, ContainerPresenter,
};
use crate::presentation::tui::event::{AppEvent, EventHandler};
use crate::presentation::tui::image::{
    filter_images, render_image_details, render_image_list, ImageActions, ImagePresenter,
};
use crate::presentation::tui::stack::{
    filter_stacks, render_stack_containers, render_stack_list, StackActions, StackPresenter,
};
use crate::presentation::tui::volume::{
    filter_volumes, render_volume_list, VolumeActions, VolumePresenter,
};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, widgets::ListState, Frame, Terminal};
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
    StackList,
    StackContainers,
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
    RemoveAllStackContainers,
}

pub struct App {
    pub screen: Screen,
    pub previous_screen: Option<Screen>,
    pub should_quit: bool,
    pub menu_state: ListState,
    pub container_presenter: ContainerPresenter,
    pub volume_presenter: VolumePresenter,
    pub image_presenter: ImagePresenter,
    pub stack_presenter: StackPresenter,
    pub container_actions: ContainerActions,
    pub volume_actions: VolumeActions,
    pub image_actions: ImageActions,
    pub stack_actions: StackActions,
    pub confirm_dialog: Option<(ConfirmAction, bool)>,
    pub popup_message: Option<PopupMessage>,
}

impl App {
    pub fn new(
        container_actions: ContainerActions,
        volume_actions: VolumeActions,
        image_actions: ImageActions,
        stack_actions: StackActions,
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
            stack_presenter: StackPresenter::new(),
            container_actions,
            volume_actions,
            image_actions,
            stack_actions,
            confirm_dialog: None,
            popup_message: None,
        }
    }

    pub async fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_inner(&mut terminal).await;

        let _ = disable_raw_mode();
        let _ = execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = terminal.show_cursor();

        result
    }

    async fn run_inner(
        &mut self,
        terminal: &mut ratatui::Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
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

        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        match &self.screen {
            Screen::MainMenu => render_main_menu(frame, area, &mut self.menu_state),
            Screen::ContainerList => {
                let presenter = &mut self.container_presenter;
                let active_filter = presenter.active_filter().map(str::to_string);
                let filter = presenter.filter.value().to_string();
                let containers = &presenter.containers;
                let selection_state = &mut presenter.selection.state;
                let filtered = filter_containers(containers, &filter);
                render_container_list(
                    frame,
                    area,
                    &filtered,
                    selection_state,
                    active_filter.as_deref(),
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
                let presenter = &mut self.volume_presenter;
                let active_filter = presenter.active_filter().map(str::to_string);
                let filter = presenter.filter.value().to_string();
                let volumes = &presenter.volumes;
                let selection_state = &mut presenter.selection.state;
                let filtered = filter_volumes(volumes, &filter);
                render_volume_list(
                    frame,
                    area,
                    &filtered,
                    selection_state,
                    active_filter.as_deref(),
                );
            }
            Screen::ImageList => {
                let presenter = &mut self.image_presenter;
                let active_filter = presenter.active_filter().map(str::to_string);
                let filter = presenter.filter.value().to_string();
                let images = &presenter.images;
                let selection_state = &mut presenter.selection.state;
                let filtered = filter_images(images, &filter);
                render_image_list(
                    frame,
                    area,
                    &filtered,
                    selection_state,
                    active_filter.as_deref(),
                );
            }
            Screen::ImageDetails => {
                if let Some(image) = &self.image_presenter.selected_image {
                    render_image_details(frame, area, image);
                }
            }
            Screen::StackList => {
                let presenter = &mut self.stack_presenter;
                let active_filter = presenter.active_filter().map(str::to_string);
                let filter = presenter.filter.value().to_string();
                let stacks = &presenter.stacks;
                let selection_state = &mut presenter.selection.state;
                let filtered = filter_stacks(stacks, &filter);
                render_stack_list(
                    frame,
                    area,
                    &filtered,
                    selection_state,
                    active_filter.as_deref(),
                );
            }
            Screen::StackContainers => {
                let stack_name = self
                    .stack_presenter
                    .selected_stack()
                    .map(|s| s.name.as_str())
                    .unwrap_or("Stack")
                    .to_string();
                render_stack_containers(
                    frame,
                    area,
                    &stack_name,
                    &self.stack_presenter.stack_containers,
                    &mut self.stack_presenter.container_selection.state,
                );
            }
        }

        if let Some((action, selected_yes)) = &self.confirm_dialog {
            render_confirm_dialog(frame, confirm_message(*action), *selected_yes);
        }

        if let Some(message) = &self.popup_message {
            render_error_popup(frame, message);
        }
    }

    async fn handle_action(&mut self, action: AppAction) {
        if self.popup_message.is_some() {
            self.popup_message = None;
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
            Screen::StackList => self.handle_stack_list_action(action).await,
            Screen::StackContainers => self.handle_stack_containers_action(action).await,
        }
    }

    async fn handle_main_menu_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => self.should_quit = true,
            AppAction::NavigateUp => {
                if let Some(selected) = self.menu_state.selected() {
                    let last_index = resources::MAIN_MENU_ITEMS.len() - 1;
                    let new_selected = if selected == 0 {
                        last_index
                    } else {
                        selected - 1
                    };
                    self.menu_state.select(Some(new_selected));
                }
            }
            AppAction::NavigateDown => {
                if let Some(selected) = self.menu_state.selected() {
                    let last_index = resources::MAIN_MENU_ITEMS.len() - 1;
                    let new_selected = if selected >= last_index {
                        0
                    } else {
                        selected + 1
                    };
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
                        3 => {
                            self.screen = Screen::StackList;
                            self.load_stacks().await;
                        }
                        4 => self.should_quit = true,
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
                        self.popup_message =
                            Some(PopupMessage::Error(resources::VOLUME_IN_USE_DELETE_MESSAGE.to_string()));
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
                let container_id = match self.screen {
                    Screen::StackContainers => self
                        .stack_presenter
                        .selected_stack_container()
                        .map(|container| container.id.clone()),
                    _ => self
                        .container_presenter
                        .selected_container()
                        .map(|container| container.id.clone()),
                };

                if let Some(container_id) = container_id {
                    if let Err(e) = self
                        .container_actions
                        .delete_container(&container_id, force)
                        .await
                    {
                        self.popup_message = Some(PopupMessage::Error(e.to_string()));
                    } else if matches!(self.screen, Screen::StackContainers) {
                        self.refresh_stack_containers().await;
                    } else {
                        self.load_containers().await;
                    }
                }
            }
            ConfirmAction::DeleteVolume => {
                if let Some(volume) = self.volume_presenter.selected_volume().cloned() {
                    if let Err(e) = self.volume_actions.delete_volume(&volume.name).await {
                        self.popup_message = Some(PopupMessage::Error(e.to_string()));
                    } else {
                        self.load_volumes().await;
                    }
                }
            }
            ConfirmAction::DeleteImage(force) => {
                if let Some(image) = self.image_presenter.selected_image().cloned() {
                    if let Err(e) = self.image_actions.delete_image(&image.id, force).await {
                        self.popup_message = Some(PopupMessage::Error(e.to_string()));
                    } else {
                        self.load_images().await;
                    }
                }
            }
            ConfirmAction::PruneContainers => {
                match self.container_actions.prune_containers().await {
                    Ok(result) => {
                        self.popup_message = Some(PopupMessage::Info(resources::prune_result_message(
                            "container",
                            result.deleted_count,
                            &format_bytes(result.space_freed),
                        )));
                        self.load_containers().await;
                    }
                    Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
                }
            }
            ConfirmAction::PruneVolumes => match self.volume_actions.prune_volumes().await {
                Ok(result) => {
                    self.popup_message = Some(PopupMessage::Info(resources::prune_result_message(
                        "volume",
                        result.deleted_count,
                        &format_bytes(result.space_freed),
                    )));
                    self.load_volumes().await;
                }
                Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
            },
            ConfirmAction::PruneImages => match self.image_actions.prune_images().await {
                Ok(result) => {
                    self.popup_message = Some(PopupMessage::Info(resources::prune_result_message(
                        "image",
                        result.deleted_count,
                        &format_bytes(result.space_freed),
                    )));
                    self.load_images().await;
                }
                Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
            },
            ConfirmAction::RemoveAllStackContainers => {
                let ids: Vec<String> = self
                    .stack_presenter
                    .stack_containers
                    .iter()
                    .map(|c| c.id.clone())
                    .collect();
                match self.stack_actions.remove_all(&ids).await {
                    Ok(_) => {
                        self.screen = Screen::StackList;
                        self.load_stacks().await;
                    }
                    Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
                }
            }
        }
    }

    fn is_filter_active(&self) -> bool {
        match self.screen {
            Screen::ContainerList => self.container_presenter.is_filter_active(),
            Screen::VolumeList => self.volume_presenter.is_filter_active(),
            Screen::ImageList => self.image_presenter.is_filter_active(),
            Screen::StackList => self.stack_presenter.is_filter_active(),
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
                    Screen::StackList => self.stack_presenter.deactivate_filter(),
                    _ => {}
                }
                true
            }
            KeyCode::Backspace => {
                match self.screen {
                    Screen::ContainerList => self.container_presenter.pop_filter_char(),
                    Screen::VolumeList => self.volume_presenter.pop_filter_char(),
                    Screen::ImageList => self.image_presenter.pop_filter_char(),
                    Screen::StackList => self.stack_presenter.pop_filter_char(),
                    _ => {}
                }
                true
            }
            KeyCode::Char(c) => {
                match self.screen {
                    Screen::ContainerList => self.container_presenter.push_filter_char(c),
                    Screen::VolumeList => self.volume_presenter.push_filter_char(c),
                    Screen::ImageList => self.image_presenter.push_filter_char(c),
                    Screen::StackList => self.stack_presenter.push_filter_char(c),
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
            Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
        }
    }

    async fn load_container_details(&mut self, id: &str) {
        match self.container_actions.load_container_details(id).await {
            Ok(Some(container)) => self.container_presenter.set_details(container),
            Ok(None) => {
                self.popup_message = Some(PopupMessage::Error(resources::CONTAINER_NOT_FOUND_MESSAGE.to_string()))
            }
            Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
        }
    }

    async fn load_container_logs(&mut self, container: &ContainerDto) {
        match self.container_actions.load_logs(container, Some(500)).await {
            Ok(logs) => self.container_presenter.set_logs(logs),
            Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
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
            Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
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
            Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
        }
    }

    async fn restart_container(&mut self, container: &ContainerDto) {
        if !container.can_restart {
            return;
        }

        match self
            .container_actions
            .restart_container(&container.id)
            .await
        {
            Ok(_) => self.load_containers().await,
            Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
        }
    }

    async fn load_volumes(&mut self) {
        match self.volume_actions.load_volumes().await {
            Ok(volumes) => self.volume_presenter.set_volumes(volumes),
            Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
        }
    }

    async fn load_images(&mut self) {
        match self.image_actions.load_images().await {
            Ok(images) => self.image_presenter.set_images(images),
            Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
        }
    }

    async fn load_stacks(&mut self) {
        match self.stack_actions.load_stacks().await {
            Ok(stacks) => self.stack_presenter.set_stacks(stacks),
            Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
        }
    }

    async fn refresh_stack_containers(&mut self) {
        let selected_stack_name = self
            .stack_presenter
            .selected_stack()
            .map(|stack| stack.name.clone());
        self.load_stacks().await;
        if let Some(stack_name) = selected_stack_name {
            self.stack_presenter.select_stack_by_name(&stack_name);
        }

        let containers = self
            .stack_presenter
            .selected_stack()
            .map(|stack| stack.containers.clone())
            .unwrap_or_default();
        self.stack_presenter.set_stack_containers(containers);
    }

    async fn handle_stack_list_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => self.should_quit = true,
            AppAction::Back => self.screen = Screen::MainMenu,
            AppAction::NavigateUp => self.stack_presenter.navigate_up(),
            AppAction::NavigateDown => self.stack_presenter.navigate_down(),
            AppAction::Select => {
                if let Some(stack) = self.stack_presenter.selected_stack() {
                    let containers = stack.containers.clone();
                    self.stack_presenter.set_stack_containers(containers);
                    self.screen = Screen::StackContainers;
                }
            }
            AppAction::StartStop => {
                if let Some(stack) = self.stack_presenter.selected_stack() {
                    let ids = stack
                        .containers
                        .iter()
                        .filter(|c| c.can_start)
                        .map(|c| c.id.clone())
                        .collect::<Vec<_>>();
                    if !ids.is_empty() {
                        if let Err(e) = self.stack_actions.start_all(&ids).await {
                            self.popup_message = Some(PopupMessage::Error(e.to_string()));
                        } else {
                            self.load_stacks().await;
                        }
                    }
                }
            }
            AppAction::StopAll => {
                if let Some(stack) = self.stack_presenter.selected_stack() {
                    let ids = stack
                        .containers
                        .iter()
                        .filter(|c| c.can_stop)
                        .map(|c| c.id.clone())
                        .collect::<Vec<_>>();
                    if !ids.is_empty() {
                        if let Err(e) = self.stack_actions.stop_all(&ids).await {
                            self.popup_message = Some(PopupMessage::Error(e.to_string()));
                        } else {
                            self.load_stacks().await;
                        }
                    }
                }
            }
            AppAction::Refresh => self.load_stacks().await,
            AppAction::ActivateFilter => {
                self.stack_presenter.activate_filter();
            }
            _ => {}
        }
    }

    async fn handle_stack_containers_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => self.should_quit = true,
            AppAction::Back => {
                self.screen = Screen::StackList;
                self.stack_presenter.set_stack_containers(vec![]);
                self.load_stacks().await;
            }
            AppAction::NavigateUp => self.stack_presenter.navigate_container_up(),
            AppAction::NavigateDown => self.stack_presenter.navigate_container_down(),
            AppAction::StartStop => {
                if let Some(container) = self.stack_presenter.selected_stack_container().cloned() {
                    let result = if container.can_stop {
                        self.container_actions.stop_container(&container.id).await
                    } else if container.can_start {
                        self.container_actions.start_container(&container.id).await
                    } else {
                        return;
                    };
                    match result {
                        Ok(_) => self.refresh_stack_containers().await,
                        Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
                    }
                }
            }
            AppAction::Delete => {
                if let Some(container) = self.stack_presenter.selected_stack_container() {
                    let force = container.can_stop;
                    self.confirm_dialog = Some((ConfirmAction::DeleteContainer(force), true));
                }
            }
            AppAction::StopAll => {
                if let Some(stack) = self.stack_presenter.selected_stack() {
                    let ids: Vec<String> = stack
                        .containers
                        .iter()
                        .filter(|c| c.can_stop)
                        .map(|c| c.id.clone())
                        .collect();
                    if !ids.is_empty() {
                        match self.stack_actions.stop_all(&ids).await {
                            Ok(_) => self.refresh_stack_containers().await,
                            Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
                        }
                    }
                }
            }
            AppAction::StartAll => {
                if let Some(stack) = self.stack_presenter.selected_stack() {
                    let ids: Vec<String> = stack
                        .containers
                        .iter()
                        .filter(|c| c.can_start)
                        .map(|c| c.id.clone())
                        .collect();
                    if !ids.is_empty() {
                        match self.stack_actions.start_all(&ids).await {
                            Ok(_) => self.refresh_stack_containers().await,
                            Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
                        }
                    }
                }
            }
            AppAction::RemoveAll if !self.stack_presenter.stack_containers.is_empty() => {
                self.confirm_dialog = Some((ConfirmAction::RemoveAllStackContainers, true));
            }
            AppAction::Refresh => self.refresh_stack_containers().await,
            _ => {}
        }
    }
}

fn confirm_message(action: ConfirmAction) -> &'static str {
    match action {
        ConfirmAction::DeleteContainer(force) => {
            if force {
                resources::FORCE_DELETE_CONTAINER_MESSAGE
            } else {
                resources::DELETE_CONTAINER_MESSAGE
            }
        }
        ConfirmAction::DeleteVolume => resources::DELETE_VOLUME_MESSAGE,
        ConfirmAction::DeleteImage(force) => {
            if force {
                resources::FORCE_DELETE_IMAGE_MESSAGE
            } else {
                resources::DELETE_IMAGE_MESSAGE
            }
        }
        ConfirmAction::PruneContainers => resources::PRUNE_CONTAINERS_MESSAGE,
        ConfirmAction::PruneVolumes => resources::PRUNE_VOLUMES_MESSAGE,
        ConfirmAction::PruneImages => resources::PRUNE_IMAGES_MESSAGE,
        ConfirmAction::RemoveAllStackContainers => resources::REMOVE_ALL_STACK_CONTAINERS_MESSAGE,
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

#[cfg(test)]
mod tests {
    use super::{confirm_message, format_bytes, App, ConfirmAction, Screen};
    use crate::presentation::tui::common::PopupMessage;
    use crate::application::container::traits::MockContainerRepository;
    use crate::application::container::{ContainerDto, ContainerService};
    use crate::application::image::traits::MockImageRepository;
    use crate::application::image::{ImageDto, ImageService};
    use crate::application::stack::traits::MockStackRepository;
    use crate::application::stack::{StackContainerDto, StackDto, StackService};
    use crate::application::volume::traits::MockVolumeRepository;
    use crate::application::volume::{VolumeDto, VolumeService};
    use crate::domain::container::ContainerState;
    use crate::domain::image::{Image, ImageId, ImageSize};
    use crate::domain::stack::{Stack, StackContainerState, StackName};
    use crate::presentation::tui::common::{resources, AppAction};
    use crate::presentation::tui::container::ContainerActions;
    use crate::presentation::tui::image::ImageActions;
    use crate::presentation::tui::stack::StackActions;
    use crate::presentation::tui::volume::VolumeActions;
    use crate::shared::PruneResultDto;
    use chrono::Utc;
    use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};
    use std::sync::Arc;

    fn buffer_text(buffer: &Buffer) -> String {
        buffer
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>()
    }

    fn make_container_dto() -> ContainerDto {
        ContainerDto {
            id: "abc123".to_string(),
            short_id: "abc123".to_string(),
            name: "web".to_string(),
            image: "nginx:latest".to_string(),
            state: ContainerState::Running,
            status: "Up 5 minutes".to_string(),
            created: "2024-01-01".to_string(),
            ports: "80:80".to_string(),
            networks: "bridge".to_string(),
            can_start: false,
            can_stop: true,
            can_delete: true,
            can_restart: true,
            can_pause: true,
            can_unpause: false,
            env_vars: vec!["RUST_LOG=info".to_string()],
        }
    }

    fn make_stopped_container_dto() -> ContainerDto {
        ContainerDto {
            can_start: true,
            can_stop: false,
            can_pause: false,
            can_unpause: false,
            can_restart: false,
            state: ContainerState::Stopped,
            status: "Exited".to_string(),
            ..make_container_dto()
        }
    }

    fn make_volume_dto(in_use: bool, can_delete: bool) -> VolumeDto {
        VolumeDto {
            id: "vol-1".to_string(),
            name: "db-data".to_string(),
            driver: "local".to_string(),
            mountpoint: "/var/lib/docker/volumes/db-data/_data".to_string(),
            size: "10 MB".to_string(),
            created: "2024-01-01".to_string(),
            in_use,
            can_delete,
        }
    }

    fn make_image_dto(in_use: bool) -> ImageDto {
        ImageDto {
            id: "sha256:abc".to_string(),
            short_id: "abc".to_string(),
            repository: "nginx".to_string(),
            tag: "latest".to_string(),
            full_name: "nginx:latest".to_string(),
            size: "12 MB".to_string(),
            created: "2024-01-01".to_string(),
            in_use,
            is_dangling: false,
            can_delete: true,
        }
    }

    fn make_stack_container(can_start: bool, can_stop: bool) -> StackContainerDto {
        StackContainerDto {
            id: "stack-1".to_string(),
            name: "web".to_string(),
            image: "nginx:latest".to_string(),
            state: if can_stop {
                StackContainerState::Running
            } else {
                StackContainerState::Stopped
            },
            status: if can_stop {
                "Up".to_string()
            } else {
                "Exited".to_string()
            },
            ports: "80/tcp".to_string(),
            can_start,
            can_stop,
        }
    }

    fn make_stack_dto(containers: Vec<StackContainerDto>) -> StackDto {
        StackDto {
            name: "compose-app".to_string(),
            container_count: containers.len(),
            running_count: containers
                .iter()
                .filter(|container| container.can_stop)
                .count(),
            containers,
        }
    }

    fn make_image() -> Image {
        Image::new(
            ImageId::new("sha256:abc").unwrap(),
            "nginx",
            "latest",
            ImageSize::new(1_000_000),
            Utc::now(),
        )
    }

    fn make_stack(containers: usize) -> Stack {
        let domain_containers = (0..containers)
            .map(|index| {
                crate::domain::stack::StackContainer::new(
                    format!("id-{index}"),
                    format!("svc-{index}"),
                    "nginx:latest",
                    crate::domain::stack::StackContainerState::Running,
                    "Up",
                    "80/tcp",
                )
            })
            .collect();
        Stack::new(StackName::new("compose-app").unwrap(), domain_containers)
    }

    fn build_app(
        container_repo: MockContainerRepository,
        volume_repo: MockVolumeRepository,
        image_repo: MockImageRepository,
        stack_repo: MockStackRepository,
    ) -> App {
        let container_actions =
            ContainerActions::new(ContainerService::new(Arc::new(container_repo)));
        let volume_actions = VolumeActions::new(VolumeService::new(Arc::new(volume_repo)));
        let image_actions = ImageActions::new(ImageService::new(Arc::new(image_repo)));
        let stack_actions = StackActions::new(StackService::new(Arc::new(stack_repo)));
        App::new(
            container_actions,
            volume_actions,
            image_actions,
            stack_actions,
        )
    }

    fn empty_app() -> App {
        build_app(
            MockContainerRepository::new(),
            MockVolumeRepository::new(),
            MockImageRepository::new(),
            MockStackRepository::new(),
        )
    }

    fn render_app(app: &mut App) -> String {
        let backend = TestBackend::new(100, 20);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| app.render(frame)).unwrap();
        buffer_text(terminal.backend().buffer())
    }

    #[test]
    fn test_new_render_confirm_message_and_format_bytes() {
        let mut app = empty_app();
        let rendered = render_app(&mut app);
        assert_eq!(app.screen, Screen::MainMenu);
        assert_eq!(app.menu_state.selected(), Some(0));
        assert!(rendered.contains(resources::MAIN_MENU_TITLE));

        assert_eq!(
            confirm_message(ConfirmAction::DeleteContainer(false)),
            resources::DELETE_CONTAINER_MESSAGE
        );
        assert_eq!(
            confirm_message(ConfirmAction::DeleteContainer(true)),
            resources::FORCE_DELETE_CONTAINER_MESSAGE
        );
        assert_eq!(
            confirm_message(ConfirmAction::RemoveAllStackContainers),
            resources::REMOVE_ALL_STACK_CONTAINERS_MESSAGE
        );

        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(2 * 1024), "2.0 KB");
        assert_eq!(format_bytes(3 * 1024 * 1024), "3.0 MB");
        assert_eq!(format_bytes(4 * 1024 * 1024 * 1024), "4.0 GB");
    }

    #[test]
    fn test_render_for_all_screens_and_overlays() {
        let mut app = empty_app();
        let container = make_container_dto();
        let volume = make_volume_dto(false, true);
        let image = make_image_dto(true);
        let stack = make_stack_dto(vec![make_stack_container(false, true)]);

        app.container_presenter
            .set_containers(vec![container.clone()]);
        app.container_presenter
            .set_logs(crate::application::container::ContainerLogsDto {
                container_id: container.id.clone(),
                container_name: container.name.clone(),
                logs: "line1".to_string(),
            });
        app.container_presenter.set_details(container.clone());
        app.volume_presenter.set_volumes(vec![volume]);
        app.image_presenter.set_images(vec![image.clone()]);
        app.image_presenter.set_details(image);
        app.stack_presenter.set_stacks(vec![stack.clone()]);
        app.stack_presenter
            .set_stack_containers(stack.containers.clone());

        app.screen = Screen::ContainerList;
        assert!(render_app(&mut app).contains("Containers"));
        app.screen = Screen::ContainerLogs;
        assert!(render_app(&mut app).contains("line1"));
        app.screen = Screen::ContainerDetails;
        assert!(render_app(&mut app).contains("Container Details"));
        app.screen = Screen::VolumeList;
        assert!(render_app(&mut app).contains("Volumes"));
        app.screen = Screen::ImageList;
        assert!(render_app(&mut app).contains("Images"));
        app.screen = Screen::ImageDetails;
        assert!(render_app(&mut app).contains("Image Details"));
        app.screen = Screen::StackList;
        assert!(render_app(&mut app).contains("Stacks"));
        app.screen = Screen::StackContainers;
        assert!(render_app(&mut app).contains("compose-app"));

        app.confirm_dialog = Some((ConfirmAction::DeleteVolume, true));
        let rendered = render_app(&mut app);
        assert!(rendered.contains(resources::DELETE_VOLUME_MESSAGE));

        app.popup_message = Some(PopupMessage::Error("boom".to_string()));
        let rendered = render_app(&mut app);
        assert!(rendered.contains("boom"));
        assert!(rendered.contains(resources::ERROR_TITLE.trim()));

        app.popup_message = Some(PopupMessage::Info("all good".to_string()));
        let rendered = render_app(&mut app);
        assert!(rendered.contains("all good"));
        assert!(rendered.contains(resources::INFO_TITLE.trim()));
    }

    #[tokio::test]
    async fn test_handle_action_clears_popup_and_handles_confirm_dialog() {
        let mut app = empty_app();
        app.popup_message = Some(PopupMessage::Error("boom".to_string()));
        app.handle_action(AppAction::Quit).await;
        assert!(app.popup_message.is_none());
        assert!(!app.should_quit);

        app.confirm_dialog = Some((ConfirmAction::DeleteVolume, true));
        app.handle_action(AppAction::NavigateDown).await;
        assert_eq!(
            app.confirm_dialog,
            Some((ConfirmAction::DeleteVolume, false))
        );

        app.handle_action(AppAction::Back).await;
        assert!(app.confirm_dialog.is_none());

        app.confirm_dialog = Some((ConfirmAction::DeleteVolume, false));
        app.handle_action(AppAction::Select).await;
        assert!(app.confirm_dialog.is_none());
    }

    #[tokio::test]
    async fn test_handle_main_menu_navigation_and_quit() {
        let mut app = empty_app();

        app.handle_main_menu_action(AppAction::NavigateUp).await;
        assert_eq!(
            app.menu_state.selected(),
            Some(resources::MAIN_MENU_ITEMS.len() - 1)
        );

        app.handle_main_menu_action(AppAction::NavigateDown).await;
        assert_eq!(app.menu_state.selected(), Some(0));

        app.handle_main_menu_action(AppAction::Quit).await;
        assert!(app.should_quit);
    }

    #[tokio::test]
    async fn test_handle_main_menu_select_loads_each_screen() {
        let mut container_repo = MockContainerRepository::new();
        container_repo.expect_get_all().returning(|| {
            Ok(vec![crate::domain::container::Container::new(
                crate::domain::container::ContainerId::new("abc123").unwrap(),
                "web",
                "nginx:latest",
                ContainerState::Running,
                "Up",
                Utc::now(),
            )])
        });
        let mut volume_repo = MockVolumeRepository::new();
        volume_repo.expect_get_all().returning(|| {
            Ok(vec![crate::domain::volume::Volume::new(
                crate::domain::volume::VolumeId::new("vol1").unwrap(),
                "db-data",
                "local",
                "/tmp".to_string(),
            )])
        });
        let mut image_repo = MockImageRepository::new();
        image_repo
            .expect_get_all()
            .returning(|| Ok(vec![make_image()]));
        let mut stack_repo = MockStackRepository::new();
        stack_repo
            .expect_get_all()
            .returning(|| Ok(vec![make_stack(1)]));

        let mut app = build_app(container_repo, volume_repo, image_repo, stack_repo);

        app.menu_state.select(Some(0));
        app.handle_main_menu_action(AppAction::Select).await;
        assert_eq!(app.screen, Screen::ContainerList);
        assert_eq!(app.container_presenter.containers.len(), 1);

        app.screen = Screen::MainMenu;
        app.menu_state.select(Some(1));
        app.handle_main_menu_action(AppAction::Select).await;
        assert_eq!(app.screen, Screen::VolumeList);
        assert_eq!(app.volume_presenter.volumes.len(), 1);

        app.screen = Screen::MainMenu;
        app.menu_state.select(Some(2));
        app.handle_main_menu_action(AppAction::Select).await;
        assert_eq!(app.screen, Screen::ImageList);
        assert_eq!(app.image_presenter.images.len(), 1);

        app.screen = Screen::MainMenu;
        app.menu_state.select(Some(3));
        app.handle_main_menu_action(AppAction::Select).await;
        assert_eq!(app.screen, Screen::StackList);
        assert_eq!(app.stack_presenter.stacks.len(), 1);
    }

    #[tokio::test]
    async fn test_handle_container_list_actions_cover_navigation_and_mutations() {
        let mut container_repo = MockContainerRepository::new();
        container_repo
            .expect_get_logs()
            .returning(|_, _| Ok("line1\nline2".to_string()));
        container_repo.expect_get_by_id().returning(|_| {
            Ok(Some(crate::domain::container::Container::new(
                crate::domain::container::ContainerId::new("abc123").unwrap(),
                "web",
                "nginx:latest",
                ContainerState::Running,
                "Up",
                Utc::now(),
            )))
        });
        container_repo.expect_stop().returning(|_| Ok(()));
        container_repo.expect_pause().returning(|_| Ok(()));
        container_repo.expect_restart().returning(|_| Ok(()));
        container_repo.expect_get_all().times(4).returning(|| {
            Ok(vec![crate::domain::container::Container::new(
                crate::domain::container::ContainerId::new("abc123").unwrap(),
                "web",
                "nginx:latest",
                ContainerState::Running,
                "Up",
                Utc::now(),
            )])
        });

        let mut app = build_app(
            container_repo,
            MockVolumeRepository::new(),
            MockImageRepository::new(),
            MockStackRepository::new(),
        );
        app.screen = Screen::ContainerList;
        app.container_presenter
            .set_containers(vec![make_container_dto()]);

        app.handle_container_list_action(AppAction::NavigateDown)
            .await;
        app.handle_container_list_action(AppAction::NavigateUp)
            .await;
        app.handle_container_list_action(AppAction::ActivateFilter)
            .await;
        assert!(app.container_presenter.is_filter_active());

        app.handle_container_list_action(AppAction::ViewLogs).await;
        assert_eq!(app.screen, Screen::ContainerLogs);

        app.screen = Screen::ContainerList;
        app.handle_container_list_action(AppAction::ViewDetails)
            .await;
        assert_eq!(app.screen, Screen::ContainerDetails);

        app.screen = Screen::ContainerList;
        app.handle_container_list_action(AppAction::StartStop).await;
        app.handle_container_list_action(AppAction::PauseUnpause)
            .await;
        app.handle_container_list_action(AppAction::Restart).await;
        app.handle_container_list_action(AppAction::Refresh).await;

        app.handle_container_list_action(AppAction::Delete).await;
        assert_eq!(
            app.confirm_dialog,
            Some((ConfirmAction::DeleteContainer(true), true))
        );

        app.confirm_dialog = None;
        app.handle_container_list_action(AppAction::Prune).await;
        assert_eq!(
            app.confirm_dialog,
            Some((ConfirmAction::PruneContainers, true))
        );

        app.handle_container_list_action(AppAction::Back).await;
        assert_eq!(app.screen, Screen::MainMenu);
    }

    #[tokio::test]
    async fn test_handle_container_log_and_details_actions() {
        let mut app = empty_app();
        app.screen = Screen::ContainerLogs;
        app.container_presenter
            .set_logs(crate::application::container::ContainerLogsDto {
                container_id: "abc123".to_string(),
                container_name: "web".to_string(),
                logs: "line1".to_string(),
            });
        app.handle_container_logs_action(AppAction::ScrollDown);
        app.handle_container_logs_action(AppAction::ScrollUp);
        app.handle_container_logs_action(AppAction::Back);
        assert_eq!(app.screen, Screen::ContainerList);
        assert!(app.container_presenter.logs.is_none());

        app.screen = Screen::ContainerDetails;
        app.container_presenter.set_details(make_container_dto());
        app.handle_details_action(AppAction::Quit);
        assert_eq!(app.screen, Screen::ContainerList);

        app.screen = Screen::ImageDetails;
        app.image_presenter.set_details(make_image_dto(true));
        app.handle_details_action(AppAction::Back);
        assert_eq!(app.screen, Screen::ImageList);
    }

    #[tokio::test]
    async fn test_volume_and_image_list_actions() {
        let mut volume_repo = MockVolumeRepository::new();
        volume_repo.expect_get_all().returning(|| Ok(vec![]));
        let mut image_repo = MockImageRepository::new();
        image_repo
            .expect_get_all()
            .returning(|| Ok(vec![make_image()]));

        let mut app = build_app(
            MockContainerRepository::new(),
            volume_repo,
            image_repo,
            MockStackRepository::new(),
        );

        app.screen = Screen::VolumeList;
        app.volume_presenter
            .set_volumes(vec![make_volume_dto(true, false)]);
        app.handle_volume_list_action(AppAction::Delete).await;
        assert!(matches!(
            app.popup_message,
            Some(PopupMessage::Error(_))
        ));
        assert!(app.popup_message.as_ref().unwrap().as_str().contains(resources::VOLUME_IN_USE_DELETE_MESSAGE));
        app.popup_message = None;
        app.volume_presenter
            .set_volumes(vec![make_volume_dto(false, true)]);
        app.handle_volume_list_action(AppAction::Delete).await;
        assert_eq!(
            app.confirm_dialog,
            Some((ConfirmAction::DeleteVolume, true))
        );
        app.confirm_dialog = None;
        app.handle_volume_list_action(AppAction::Prune).await;
        assert_eq!(
            app.confirm_dialog,
            Some((ConfirmAction::PruneVolumes, true))
        );
        app.confirm_dialog = None;
        app.handle_volume_list_action(AppAction::ActivateFilter)
            .await;
        assert!(app.volume_presenter.is_filter_active());
        app.handle_volume_list_action(AppAction::Refresh).await;

        app.screen = Screen::ImageList;
        app.image_presenter.set_images(vec![make_image_dto(true)]);
        app.handle_image_list_action(AppAction::ViewDetails).await;
        assert_eq!(app.screen, Screen::ImageDetails);
        app.screen = Screen::ImageList;
        app.handle_image_list_action(AppAction::Delete).await;
        assert_eq!(
            app.confirm_dialog,
            Some((ConfirmAction::DeleteImage(true), true))
        );
        app.confirm_dialog = None;
        app.handle_image_list_action(AppAction::Prune).await;
        assert_eq!(app.confirm_dialog, Some((ConfirmAction::PruneImages, true)));
        app.confirm_dialog = None;
        app.handle_image_list_action(AppAction::ActivateFilter)
            .await;
        assert!(app.image_presenter.is_filter_active());
        app.handle_image_list_action(AppAction::Refresh).await;
    }

    #[tokio::test]
    async fn test_execute_confirm_action_delete_and_prune_paths() {
        let mut container_repo = MockContainerRepository::new();
        container_repo
            .expect_delete()
            .times(2)
            .returning(|_, _| Ok(()));
        container_repo.expect_prune().returning(|| {
            Ok(PruneResultDto {
                deleted_count: 2,
                space_freed: 1024,
            })
        });
        container_repo
            .expect_get_all()
            .times(2)
            .returning(|| Ok(vec![]));
        let mut volume_repo = MockVolumeRepository::new();
        volume_repo.expect_delete().returning(|_| Ok(()));
        volume_repo.expect_prune().returning(|| {
            Ok(PruneResultDto {
                deleted_count: 1,
                space_freed: 256,
            })
        });
        volume_repo
            .expect_get_all()
            .times(2)
            .returning(|| Ok(vec![]));
        let mut image_repo = MockImageRepository::new();
        image_repo.expect_delete().returning(|_, _| Ok(()));
        image_repo.expect_prune().returning(|| {
            Ok(PruneResultDto {
                deleted_count: 3,
                space_freed: 2048,
            })
        });
        image_repo
            .expect_get_all()
            .times(2)
            .returning(|| Ok(vec![make_image()]));
        let mut stack_repo = MockStackRepository::new();
        stack_repo
            .expect_get_all()
            .returning(|| Ok(vec![make_stack(1)]));
        stack_repo.expect_remove_all().returning(|_| Ok(()));

        let mut app = build_app(container_repo, volume_repo, image_repo, stack_repo);
        app.container_presenter
            .set_containers(vec![make_container_dto()]);
        app.volume_presenter
            .set_volumes(vec![make_volume_dto(false, true)]);
        app.image_presenter.set_images(vec![make_image_dto(true)]);
        app.stack_presenter
            .set_stacks(vec![make_stack_dto(vec![make_stack_container(
                false, true,
            )])]);
        app.stack_presenter
            .set_stack_containers(vec![make_stack_container(false, true)]);

        app.execute_confirm_action(ConfirmAction::DeleteContainer(true))
            .await;
        app.execute_confirm_action(ConfirmAction::PruneContainers)
            .await;
        assert!(matches!(app.popup_message, Some(PopupMessage::Info(_))));
        assert!(app.popup_message.as_ref().unwrap().as_str().contains("Pruned 2 container(s)"));

        app.execute_confirm_action(ConfirmAction::DeleteVolume)
            .await;
        app.execute_confirm_action(ConfirmAction::PruneVolumes)
            .await;
        assert!(matches!(app.popup_message, Some(PopupMessage::Info(_))));
        assert!(app.popup_message.as_ref().unwrap().as_str().contains("Pruned 1 volume(s)"));

        app.execute_confirm_action(ConfirmAction::DeleteImage(true))
            .await;
        app.execute_confirm_action(ConfirmAction::PruneImages).await;
        assert!(matches!(app.popup_message, Some(PopupMessage::Info(_))));
        assert!(app.popup_message.as_ref().unwrap().as_str().contains("Pruned 3 image(s)"));

        app.screen = Screen::StackContainers;
        app.execute_confirm_action(ConfirmAction::DeleteContainer(true))
            .await;
        app.execute_confirm_action(ConfirmAction::RemoveAllStackContainers)
            .await;
        assert_eq!(app.screen, Screen::StackList);
    }

    #[tokio::test]
    async fn test_filter_key_and_load_helpers() {
        let mut container_repo = MockContainerRepository::new();
        container_repo.expect_get_all().returning(|| Ok(vec![]));
        container_repo.expect_get_by_id().returning(|_| Ok(None));
        let mut volume_repo = MockVolumeRepository::new();
        volume_repo.expect_get_all().returning(|| Ok(vec![]));
        let mut image_repo = MockImageRepository::new();
        image_repo
            .expect_get_all()
            .returning(|| Ok(vec![make_image()]));
        let mut stack_repo = MockStackRepository::new();
        stack_repo
            .expect_get_all()
            .returning(|| Ok(vec![make_stack(1)]));

        let mut app = build_app(container_repo, volume_repo, image_repo, stack_repo);

        app.screen = Screen::ContainerList;
        app.container_presenter.activate_filter();
        assert!(app.handle_filter_key(crossterm::event::KeyCode::Char('a')));
        assert_eq!(app.container_presenter.filter.value(), "a");
        assert!(app.handle_filter_key(crossterm::event::KeyCode::Backspace));
        assert!(app.container_presenter.filter.value().is_empty());
        assert!(app.handle_filter_key(crossterm::event::KeyCode::Esc));
        assert!(!app.container_presenter.is_filter_active());
        assert!(!app.handle_filter_key(crossterm::event::KeyCode::Enter));

        app.load_containers().await;
        app.load_volumes().await;
        app.load_images().await;
        app.load_stacks().await;
        app.load_container_details("missing").await;
        assert!(matches!(app.popup_message, Some(PopupMessage::Error(_))));
        assert!(app.popup_message.as_ref().unwrap().as_str().contains(resources::CONTAINER_NOT_FOUND_MESSAGE));
    }

    #[tokio::test]
    async fn test_toggle_pause_restart_and_stack_actions() {
        let mut container_repo = MockContainerRepository::new();
        container_repo.expect_start().returning(|_| Ok(()));
        container_repo.expect_pause().returning(|_| Ok(()));
        container_repo.expect_unpause().returning(|_| Ok(()));
        container_repo.expect_restart().returning(|_| Ok(()));
        container_repo.expect_get_all().returning(|| Ok(vec![]));
        let mut stack_repo = MockStackRepository::new();
        stack_repo
            .expect_get_all()
            .times(4)
            .returning(|| Ok(vec![make_stack(1)]));
        stack_repo.expect_start_all().times(1).returning(|_| Ok(()));
        stack_repo.expect_stop_all().times(2).returning(|_| Ok(()));

        let mut app = build_app(
            container_repo,
            MockVolumeRepository::new(),
            MockImageRepository::new(),
            stack_repo,
        );

        app.toggle_container(&make_stopped_container_dto()).await;
        let mut unpause_container = make_stopped_container_dto();
        unpause_container.can_unpause = true;
        app.pause_unpause_container(&make_container_dto()).await;
        app.pause_unpause_container(&unpause_container).await;
        app.restart_container(&make_container_dto()).await;
        app.restart_container(&make_stopped_container_dto()).await;

        let stack = make_stack_dto(vec![
            make_stack_container(true, false),
            make_stack_container(false, true),
        ]);
        app.stack_presenter.set_stacks(vec![stack.clone()]);
        app.screen = Screen::StackList;
        app.handle_stack_list_action(AppAction::Select).await;
        assert_eq!(app.screen, Screen::StackContainers);

        app.screen = Screen::StackList;
        app.handle_stack_list_action(AppAction::StartStop).await;
        app.handle_stack_list_action(AppAction::StopAll).await;
        app.handle_stack_list_action(AppAction::ActivateFilter)
            .await;
        assert!(app.stack_presenter.is_filter_active());

        app.stack_presenter.set_stack_containers(stack.containers);
        app.screen = Screen::StackContainers;
        app.handle_stack_containers_action(AppAction::StartStop)
            .await;
        app.handle_stack_containers_action(AppAction::Delete).await;
        assert_eq!(
            app.confirm_dialog,
            Some((ConfirmAction::DeleteContainer(true), true))
        );
        app.confirm_dialog = None;
        app.handle_stack_containers_action(AppAction::StopAll).await;
        app.handle_stack_containers_action(AppAction::StartAll)
            .await;
        app.handle_stack_containers_action(AppAction::RemoveAll)
            .await;
        assert_eq!(
            app.confirm_dialog,
            Some((ConfirmAction::RemoveAllStackContainers, true))
        );
    }
}
