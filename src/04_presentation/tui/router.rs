//! Screen routing: maps user actions to screen-specific handlers.

use crate::application::container::ContainerStatsEvent;
use crate::application::container::{
    parse_port_mappings, CreateContainerRequest, FILE_VIEW_MAX_BYTES,
};
use crate::presentation::tui::app::{
    format_bytes, AddDialog, AddDialogKind, App, PendingImagePull, Screen,
};
use crate::presentation::tui::common::{
    copy_text, resources, AppAction, InputDialogState, PopupMessage,
};
use crate::presentation::tui::confirm::ConfirmAction;
use crossterm::event::KeyCode;

use tokio::sync::mpsc::error::TryRecvError;

impl App {
    pub async fn handle_action(&mut self, action: AppAction) {
        if matches!(action, AppAction::Select)
            && self.screen == Screen::ContainerLogs
            && self.container_presenter.is_log_search_active()
        {
            self.container_presenter.confirm_log_search();
            return;
        }
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

        if self.expanded_log_line.is_some() {
            match action {
                AppAction::Back | AppAction::Quit | AppAction::Select => {
                    self.expanded_log_line = None;
                }
                _ => {}
            }
            return;
        }
        if self.exec_shell_dialog.is_some() {
            self.handle_exec_shell_action(action);
            return;
        }
        if self.screen == Screen::ContainerExplorer && self.container_explorer.file_view.is_some() {
            match action {
                AppAction::Back | AppAction::Quit | AppAction::ViewFile => {
                    self.container_explorer.close_file_view();
                }
                AppAction::NavigateDown | AppAction::ScrollDown => {
                    self.container_explorer.scroll_file_view_down(10);
                }
                AppAction::NavigateUp | AppAction::ScrollUp => {
                    self.container_explorer.scroll_file_view_up();
                }
                _ => {}
            }
            return;
        }
        if self.add_dialog.is_some() {
            self.handle_add_dialog_action(action).await;
            return;
        }

        if self.help_popup_open {
            match action {
                AppAction::ToggleHelp | AppAction::Back | AppAction::Quit => {
                    self.help_popup_open = false;
                }
                AppAction::NavigateUp | AppAction::ScrollUp => self.help_popup_state.scroll_up(),
                AppAction::NavigateDown | AppAction::ScrollDown => {
                    self.help_popup_state.scroll_down();
                }
                AppAction::PageUp => {
                    for _ in 0..10 {
                        self.help_popup_state.scroll_up();
                    }
                }
                AppAction::PageDown => {
                    for _ in 0..10 {
                        self.help_popup_state.scroll_down();
                    }
                }
                _ => {}
            }
            return;
        }

        match action {
            AppAction::ToggleHelp => {
                if self.current_help_text().is_some() {
                    self.help_popup_open = true;
                    self.help_popup_state = Default::default();
                }
            }
            _ => match &self.screen {
                Screen::MainMenu => self.handle_main_menu_action(action).await,
                Screen::ContainerList => self.handle_container_list_action(action).await,
                Screen::ContainerLogs => self.handle_container_logs_action(action).await,
                Screen::ContainerDetails => self.handle_details_action(action),
                Screen::ContainerExplorer => self.handle_container_explorer_action(action).await,
                Screen::VolumeList => self.handle_volume_list_action(action).await,
                Screen::VolumeDetails => self.handle_details_action(action),
                Screen::ImageList => self.handle_image_list_action(action).await,
                Screen::ImageDetails => self.handle_image_details_action(action).await,
                Screen::NetworkList => self.handle_network_list_action(action).await,
                Screen::NetworkDetails => self.handle_network_details_action(action).await,
                Screen::StackList => self.handle_stack_list_action(action).await,
                Screen::StackContainers => self.handle_stack_containers_action(action).await,
            },
        }
    }

    pub(crate) fn current_help_text(&self) -> Option<&'static str> {
        match self.screen {
            Screen::MainMenu => Some(resources::MAIN_MENU_HELP),
            Screen::ContainerList => Some(resources::CONTAINER_LIST_HELP),
            Screen::ContainerLogs => Some(resources::CONTAINER_LOGS_HELP),
            Screen::ContainerDetails => Some(resources::CONTAINER_DETAILS_HELP),
            Screen::ContainerExplorer => Some(resources::CONTAINER_EXPLORER_HELP),
            Screen::VolumeList => Some(resources::VOLUME_LIST_HELP),
            Screen::VolumeDetails => Some(resources::VOLUME_DETAILS_HELP),
            Screen::ImageList => Some(resources::IMAGE_LIST_HELP),
            Screen::ImageDetails => Some(resources::IMAGE_DETAILS_HELP),
            Screen::NetworkList => Some(resources::NETWORK_LIST_HELP),
            Screen::NetworkDetails => Some(resources::NETWORK_DETAILS_HELP),
            Screen::StackList => Some(resources::STACK_LIST_HELP),
            Screen::StackContainers => Some(resources::STACK_CONTAINERS_HELP),
        }
    }

    pub(crate) async fn handle_main_menu_action(&mut self, action: AppAction) {
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
                        4 => {
                            self.screen = Screen::NetworkList;
                            self.load_networks().await;
                        }
                        5 => self.should_quit = true,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    pub(crate) async fn handle_container_list_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => self.should_quit = true,
            AppAction::Back => {
                self.stop_container_stats_subscription();
                self.container_presenter.clear_runtime_stats();
                self.screen = Screen::MainMenu;
            }
            AppAction::NavigateUp => self.container_presenter.navigate_up(),
            AppAction::NavigateDown => self.container_presenter.navigate_down(),
            AppAction::ViewLogs => {
                if let Some(container) = self.container_presenter.selected_container().cloned() {
                    self.container_presenter.deactivate_log_search();
                    self.load_container_logs(&container).await;
                    if self.container_presenter.logs.is_some() {
                        self.logs_return_screen = Some(Screen::ContainerList);
                        self.screen = Screen::ContainerLogs;
                    }
                }
            }
            AppAction::Select | AppAction::ViewDetails => {
                if let Some(container) = self.container_presenter.selected_container().cloned() {
                    self.load_container_details(&container.id).await;
                    if self.container_presenter.selected_container.is_some() {
                        self.details_return_screen = Some(Screen::ContainerList);
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
                    self.confirm_dialog = Some((ConfirmAction::ContainerDelete(force), true));
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
                self.confirm_dialog = Some((ConfirmAction::ContainersPrune, true));
            }
            AppAction::ActivateFilter => {
                self.container_presenter.activate_filter();
            }
            AppAction::Exec => self.open_exec_shell_dialog(),
            AppAction::Add => self.open_add_dialog(),
            AppAction::ToggleSort => self.container_presenter.toggle_sort(),
            AppAction::BrowseFiles => {
                if let Some(container) = self.container_presenter.selected_container().cloned() {
                    self.open_container_explorer(&container.id, &container.name)
                        .await;
                }
            }
            _ => {}
        }
    }

    pub(crate) async fn handle_container_logs_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit | AppAction::Back => {
                self.container_presenter.deactivate_log_search();
                self.container_presenter.clear_logs();
                self.screen = self
                    .logs_return_screen
                    .take()
                    .unwrap_or(Screen::ContainerList);
            }
            AppAction::ActivateLogSearch => self.container_presenter.activate_log_search(),
            AppAction::ExportLogs => {
                if let Some(logs) = &self.container_presenter.logs {
                    let container_id = logs.container_id.clone();
                    let container_name = logs.container_name.clone();
                    match self
                        .container_actions
                        .export_logs_by(&container_id, &container_name)
                        .await
                    {
                        Ok(path) => {
                            self.popup_message =
                                Some(PopupMessage::Info(resources::logs_export_success_message(
                                    &path.file_name().unwrap_or_default().to_string_lossy(),
                                )));
                        }
                        Err(e) => {
                            self.popup_message = Some(PopupMessage::Error(
                                resources::logs_export_error_message(&e.to_string()),
                            ));
                        }
                    }
                }
            }
            AppAction::CopyLogLine => {
                let line = self
                    .container_presenter
                    .selected_log_line()
                    .map(|line| line.display_line());
                match line.as_deref().map(copy_text) {
                    Some(Ok(())) => {
                        self.popup_message =
                            Some(PopupMessage::Info(resources::logs_copy_success_message()));
                    }
                    Some(Err(e)) => {
                        self.popup_message =
                            Some(PopupMessage::Error(resources::logs_copy_error_message(&e)));
                    }
                    None => {}
                }
            }
            AppAction::NextLogMatch => self.container_presenter.next_log_match(),
            AppAction::PreviousLogMatch => self.container_presenter.previous_log_match(),
            AppAction::NavigateUp | AppAction::ScrollUp => {
                self.container_presenter.previous_logs_line()
            }
            AppAction::NavigateDown | AppAction::ScrollDown => {
                self.container_presenter.next_logs_line()
            }
            AppAction::PageUp => self.container_presenter.previous_logs_page(10),
            AppAction::PageDown => self.container_presenter.next_logs_page(10),
            AppAction::Select => {
                if let Some(index) = self.container_presenter.selected_logs_line() {
                    self.expanded_log_line = Some(index);
                }
            }
            _ => {}
        }
    }

    pub(crate) async fn handle_container_explorer_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => self.should_quit = true,
            AppAction::Back => {
                if self.container_explorer.is_filter_active() {
                    self.container_explorer.deactivate_filter();
                } else {
                    self.container_explorer.clear();
                    self.screen = self
                        .explorer_return_screen
                        .take()
                        .unwrap_or(Screen::ContainerList);
                }
            }
            AppAction::NavigateUp => self.container_explorer.navigate_up(),
            AppAction::NavigateDown => self.container_explorer.navigate_down(),
            AppAction::PageUp => {
                for _ in 0..10 {
                    self.container_explorer.navigate_up();
                }
            }
            AppAction::PageDown => {
                for _ in 0..10 {
                    self.container_explorer.navigate_down();
                }
            }
            AppAction::Select => {
                if let Some(dir_path) = self.container_explorer.selected_dir_path() {
                    self.load_explorer_listing(&dir_path).await;
                }
            }
            AppAction::GoToParent => {
                if self.container_explorer.can_go_up() {
                    let parent = self.container_explorer.current_path();
                    let parent = crate::application::container::parent_path(&parent);
                    self.container_explorer.go_to_parent();
                    if let Some(parent) = parent {
                        self.load_explorer_listing(&parent).await;
                    }
                }
            }
            AppAction::ActivateFilter => self.container_explorer.activate_filter(),
            AppAction::ToggleSort => self.container_explorer.toggle_sort(),
            AppAction::Refresh => {
                let path = self.container_explorer.current_path();
                self.refresh_explorer_listing(&path).await;
            }
            AppAction::ViewFile => {
                if let Some(entry) = self.container_explorer.selected_entry().cloned() {
                    if !entry.is_dir() {
                        self.load_file_view(&entry.path).await;
                    }
                }
            }
            AppAction::CopyFile => {
                if let Some(entry) = self.container_explorer.selected_entry().cloned() {
                    self.copy_entry_to_host(&entry.path).await;
                }
            }
            _ => {}
        }
    }

    pub(crate) async fn open_container_explorer(
        &mut self,
        container_id: &str,
        container_name: &str,
    ) {
        self.container_explorer.open(container_id, container_name);
        self.explorer_return_screen = Some(match self.screen {
            Screen::StackContainers => Screen::StackContainers,
            _ => Screen::ContainerList,
        });
        self.screen = Screen::ContainerExplorer;
        self.load_explorer_listing("/").await;
    }

    async fn load_explorer_listing(&mut self, path: &str) {
        self.container_explorer.set_loading(true);
        let container_id = self.container_explorer.container_id.clone();
        let container_name = self.container_explorer.container_name.clone();
        match self
            .container_actions
            .list_files(&container_id, &container_name, path)
            .await
        {
            Ok(listing) => self.container_explorer.set_listing(listing),
            Err(e) => {
                self.popup_message = Some(PopupMessage::Error(e.to_string()));
                self.container_explorer.go_to_parent();
            }
        }
        self.container_explorer.set_loading(false);
    }

    async fn refresh_explorer_listing(&mut self, path: &str) {
        self.container_explorer.set_loading(true);
        let container_id = self.container_explorer.container_id.clone();
        let container_name = self.container_explorer.container_name.clone();
        match self
            .container_actions
            .list_files(&container_id, &container_name, path)
            .await
        {
            Ok(listing) => self.container_explorer.refresh_listing(listing),
            Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
        }
        self.container_explorer.set_loading(false);
    }

    async fn load_file_view(&mut self, path: &str) {
        self.container_explorer.set_loading(true);
        let container_id = self.container_explorer.container_id.clone();
        match self
            .container_actions
            .read_file(&container_id, path, FILE_VIEW_MAX_BYTES)
            .await
        {
            Ok(content) => self.container_explorer.show_file(path, content),
            Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
        }
        self.container_explorer.set_loading(false);
    }

    async fn copy_entry_to_host(&mut self, path: &str) {
        self.container_explorer.set_loading(true);
        let container_id = self.container_explorer.container_id.clone();
        match self
            .container_actions
            .copy_to_host(&container_id, path)
            .await
        {
            Ok(dest) => {
                self.popup_message = Some(PopupMessage::Info(
                    resources::file_copy_success_message(&dest.to_string_lossy()),
                ));
            }
            Err(e) => {
                self.popup_message = Some(PopupMessage::Error(resources::file_copy_error_message(
                    &e.to_string(),
                )));
            }
        }
        self.container_explorer.set_loading(false);
    }

    pub(crate) fn handle_details_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit | AppAction::Back => match &self.screen {
                Screen::ContainerDetails => {
                    self.container_presenter.clear_details();
                    self.screen = self
                        .details_return_screen
                        .take()
                        .unwrap_or(Screen::ContainerList);
                }
                Screen::VolumeDetails => {
                    self.volume_presenter.clear_details();
                    self.screen = Screen::VolumeList;
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub(crate) async fn handle_image_details_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit | AppAction::Back => {
                self.image_presenter.clear_details();
                self.image_presenter.clear_pull_progress();
                self.screen = Screen::ImageList;
            }
            AppAction::Refresh => {
                self.load_images().await;
                self.restore_selected_image_details();
            }
            AppAction::PullImages => self.pull_selected_image().await,
            _ => {}
        }
    }

    pub(crate) fn restore_selected_image_details(&mut self) {
        let full_name = self
            .image_presenter
            .selected_image
            .as_ref()
            .map(|image| image.full_name.clone());
        match full_name {
            Some(full_name) => {
                let updated = self
                    .image_presenter
                    .items
                    .iter()
                    .find(|image| image.full_name == full_name)
                    .cloned();
                if let Some(updated) = updated {
                    self.image_presenter.set_details(updated);
                } else {
                    self.image_presenter.clear_details();
                    self.screen = Screen::ImageList;
                }
            }
            None => {
                self.image_presenter.clear_details();
                self.screen = Screen::ImageList;
            }
        }
    }

    pub(crate) async fn handle_volume_list_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => self.should_quit = true,
            AppAction::Back => self.screen = Screen::MainMenu,
            AppAction::NavigateUp => self.volume_presenter.navigate_up(),
            AppAction::NavigateDown => self.volume_presenter.navigate_down(),
            AppAction::Select | AppAction::ViewDetails => {
                if let Some(volume) = self.volume_presenter.selected_volume().cloned() {
                    self.volume_presenter.set_details(volume);
                    self.screen = Screen::VolumeDetails;
                }
            }
            AppAction::Delete => {
                if let Some(volume) = self.volume_presenter.selected_volume() {
                    if volume.can_delete {
                        self.confirm_dialog = Some((ConfirmAction::VolumeDelete, true));
                    } else {
                        self.popup_message = Some(PopupMessage::Error(
                            resources::VOLUME_IN_USE_DELETE_MESSAGE.to_string(),
                        ));
                    }
                }
            }
            AppAction::Refresh => self.load_volumes().await,
            AppAction::Prune => {
                self.confirm_dialog = Some((ConfirmAction::VolumesPrune, true));
            }
            AppAction::ActivateFilter => {
                self.volume_presenter.activate_filter();
            }
            AppAction::ToggleSort => self.volume_presenter.toggle_sort(),
            _ => {}
        }
    }

    pub(crate) async fn handle_image_list_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => self.should_quit = true,
            AppAction::Back => self.screen = Screen::MainMenu,
            AppAction::NavigateUp => self.image_presenter.navigate_up(),
            AppAction::NavigateDown => self.image_presenter.navigate_down(),
            AppAction::Select | AppAction::ViewDetails => {
                if let Some(image) = self.image_presenter.selected_image().cloned() {
                    self.image_presenter.set_details(image);
                    self.screen = Screen::ImageDetails;
                }
            }
            AppAction::Delete => {
                if let Some(image) = self.image_presenter.selected_image() {
                    let force = image.in_use;
                    self.confirm_dialog = Some((ConfirmAction::ImageDelete(force), true));
                }
            }
            AppAction::Refresh => self.load_images().await,
            AppAction::Prune => {
                self.confirm_dialog = Some((ConfirmAction::ImagesPrune, true));
            }
            AppAction::ActivateFilter => {
                self.image_presenter.activate_filter();
            }
            AppAction::Add => self.open_add_dialog(),
            AppAction::ToggleSort => self.image_presenter.toggle_sort(),
            _ => {}
        }
    }

    async fn pull_selected_image(&mut self) {
        if self.pending_image_pull.is_some() {
            return;
        }
        let Some(image) = self.image_presenter.selected_image().cloned() else {
            return;
        };
        if image.is_dangling || image.repository == "<none>" {
            self.popup_message = Some(PopupMessage::Error(
                resources::image_pull_dangling_message().to_string(),
            ));
            return;
        }
        let image_refs = vec![image.full_name.clone()];
        self.start_image_pull(image_refs).await;
    }

    pub(crate) async fn handle_network_list_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => self.should_quit = true,
            AppAction::Back => self.screen = Screen::MainMenu,
            AppAction::NavigateUp => self.network_presenter.navigate_up(),
            AppAction::NavigateDown => self.network_presenter.navigate_down(),
            AppAction::Select | AppAction::ViewDetails => {
                if let Some(network) = self.network_presenter.selected_network().cloned() {
                    self.network_presenter.set_details(network);
                    self.screen = Screen::NetworkDetails;
                }
            }
            AppAction::Delete => {
                if let Some(network) = self.network_presenter.selected_network() {
                    if network.builtin {
                        self.popup_message = Some(PopupMessage::Error(
                            resources::NETWORK_BUILTIN_DELETE_MESSAGE.to_string(),
                        ));
                    } else if !network.can_delete {
                        self.popup_message = Some(PopupMessage::Error(
                            resources::NETWORK_IN_USE_DELETE_MESSAGE.to_string(),
                        ));
                    } else {
                        self.confirm_dialog = Some((ConfirmAction::NetworkDelete, true));
                    }
                }
            }
            AppAction::Refresh => self.load_networks().await,
            AppAction::Prune => {
                self.confirm_dialog = Some((ConfirmAction::NetworksPrune, true));
            }
            AppAction::ActivateFilter => {
                self.network_presenter.activate_filter();
            }
            AppAction::ToggleSort => self.network_presenter.toggle_sort(),
            _ => {}
        }
    }

    pub(crate) async fn handle_network_details_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit | AppAction::Back => {
                self.network_presenter.clear_details();
                self.screen = Screen::NetworkList;
            }
            AppAction::NavigateUp => self.network_presenter.navigate_container_up(),
            AppAction::NavigateDown => self.network_presenter.navigate_container_down(),
            AppAction::DisconnectContainer => {
                if self
                    .network_presenter
                    .selected_network_container()
                    .is_some()
                {
                    self.confirm_dialog = Some((ConfirmAction::NetworkDisconnect, true));
                }
            }
            AppAction::Refresh => self.refresh_network_details().await,
            _ => {}
        }
    }

    /// Reloads networks and restores the details view for the currently
    /// selected network, falling back to the list screen if it disappeared.
    pub(crate) async fn refresh_network_details(&mut self) {
        let selected_name = self
            .network_presenter
            .selected_network
            .as_ref()
            .map(|network| network.name.clone());

        self.load_networks().await;

        match selected_name {
            Some(name)
                if self
                    .network_presenter
                    .items
                    .iter()
                    .any(|network| network.name == name) =>
            {
                self.network_presenter.select_network_by_name(&name);
            }
            Some(_) => {
                self.network_presenter.clear_details();
                self.screen = Screen::NetworkList;
            }
            None => {}
        }
        self.network_presenter.set_loading(false);
    }

    pub(crate) async fn execute_confirm_action(&mut self, action: ConfirmAction) {
        // Set loading state based on current screen
        match self.screen {
            Screen::ContainerList => {
                self.container_presenter.set_loading(true);
            }
            Screen::StackContainers => {
                self.stack_presenter.containers.set_loading(true);
            }
            Screen::VolumeList => {
                self.volume_presenter.set_loading(true);
            }
            Screen::ImageList => {
                self.image_presenter.set_loading(true);
            }
            Screen::StackList => {
                self.stack_presenter.set_loading(true);
            }
            Screen::NetworkList | Screen::NetworkDetails => {
                self.network_presenter.set_loading(true);
            }
            _ => {}
        }

        match action {
            ConfirmAction::ContainerDelete(force) => {
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
                        self.set_container_list_loading(false);
                    } else if matches!(self.screen, Screen::StackContainers) {
                        self.refresh_stack_containers().await;
                    } else {
                        self.load_containers().await;
                    }
                } else {
                    self.set_container_list_loading(false);
                }
            }
            ConfirmAction::VolumeDelete => {
                if let Some(volume) = self.volume_presenter.selected_volume().cloned() {
                    if let Err(e) = self.volume_actions.delete_volume(&volume.name).await {
                        self.popup_message = Some(PopupMessage::Error(e.to_string()));
                        self.volume_presenter.set_loading(false);
                    } else {
                        self.load_volumes().await;
                    }
                } else {
                    self.volume_presenter.set_loading(false);
                }
            }
            ConfirmAction::ImageDelete(force) => {
                if let Some(image) = self.image_presenter.selected_image().cloned() {
                    if let Err(e) = self.image_actions.delete_image(&image.id, force).await {
                        self.popup_message = Some(PopupMessage::Error(e.to_string()));
                        self.image_presenter.set_loading(false);
                    } else {
                        self.load_images().await;
                    }
                } else {
                    self.image_presenter.set_loading(false);
                }
            }
            ConfirmAction::NetworkDelete => {
                if let Some(network) = self.network_presenter.selected_network().cloned() {
                    if let Err(e) = self.network_actions.delete_network(&network.name).await {
                        self.popup_message = Some(PopupMessage::Error(e.to_string()));
                        self.network_presenter.set_loading(false);
                    } else {
                        self.load_networks().await;
                    }
                } else {
                    self.network_presenter.set_loading(false);
                }
            }
            ConfirmAction::NetworkDisconnect => {
                let target = self
                    .network_presenter
                    .selected_network
                    .as_ref()
                    .map(|network| network.name.clone())
                    .zip(
                        self.network_presenter
                            .selected_network_container()
                            .map(|container| container.name.clone()),
                    );

                if let Some((network_name, container_name)) = target {
                    if let Err(e) = self
                        .network_actions
                        .disconnect_container(&network_name, &container_name)
                        .await
                    {
                        self.popup_message = Some(PopupMessage::Error(e.to_string()));
                        self.network_presenter.set_loading(false);
                    } else {
                        self.refresh_network_details().await;
                    }
                } else {
                    self.network_presenter.set_loading(false);
                }
            }
            ConfirmAction::ContainersPrune => {
                self.set_container_list_loading(true);
                match self.container_actions.prune_containers().await {
                    Ok(result) => {
                        self.popup_message =
                            Some(PopupMessage::Info(resources::prune_result_message(
                                "container",
                                result.deleted_count,
                                &format_bytes(result.space_freed),
                            )));
                        if matches!(self.screen, Screen::StackContainers) {
                            self.refresh_stack_containers().await;
                        } else {
                            self.load_containers().await;
                        }
                    }
                    Err(e) => {
                        self.popup_message = Some(PopupMessage::Error(e.to_string()));
                        self.set_container_list_loading(false);
                    }
                }
            }
            ConfirmAction::VolumesPrune => {
                self.volume_presenter.set_loading(true);
                match self.volume_actions.prune_volumes().await {
                    Ok(result) => {
                        self.popup_message =
                            Some(PopupMessage::Info(resources::prune_result_message(
                                "volume",
                                result.deleted_count,
                                &format_bytes(result.space_freed),
                            )));
                        self.load_volumes().await;
                    }
                    Err(e) => {
                        self.popup_message = Some(PopupMessage::Error(e.to_string()));
                        self.volume_presenter.set_loading(false);
                    }
                }
            }
            ConfirmAction::ImagesPrune => {
                self.image_presenter.set_loading(true);
                match self.image_actions.prune_images().await {
                    Ok(result) => {
                        self.popup_message =
                            Some(PopupMessage::Info(resources::prune_result_message(
                                "image",
                                result.deleted_count,
                                &format_bytes(result.space_freed),
                            )));
                        self.load_images().await;
                    }
                    Err(e) => {
                        self.popup_message = Some(PopupMessage::Error(e.to_string()));
                        self.image_presenter.set_loading(false);
                    }
                }
            }
            ConfirmAction::NetworksPrune => {
                self.network_presenter.set_loading(true);
                match self.network_actions.prune_networks().await {
                    Ok(result) => {
                        self.popup_message = Some(PopupMessage::Info(
                            resources::prune_networks_result_message(result.deleted_count),
                        ));
                        self.load_networks().await;
                    }
                    Err(e) => {
                        self.popup_message = Some(PopupMessage::Error(e.to_string()));
                        self.network_presenter.set_loading(false);
                    }
                }
            }
            ConfirmAction::StackContainersRemoveAll => {
                let ids: Vec<String> = self
                    .stack_presenter
                    .selected_stack()
                    .map(|stack| stack.containers.iter().map(|c| c.id.clone()).collect())
                    .unwrap_or_default();
                match self.stack_actions.remove_all(&ids).await {
                    Ok(_) => {
                        self.load_stacks().await;
                    }
                    Err(e) => {
                        self.popup_message = Some(PopupMessage::Error(e.to_string()));
                        self.stack_presenter.set_loading(false);
                    }
                }
            }
        }
    }

    pub(crate) fn is_filter_active(&self) -> bool {
        match self.screen {
            Screen::ContainerList => self.container_presenter.is_filter_active(),
            Screen::VolumeList => self.volume_presenter.is_filter_active(),
            Screen::ImageList => self.image_presenter.is_filter_active(),
            Screen::NetworkList => self.network_presenter.is_filter_active(),
            Screen::StackList => self.stack_presenter.is_filter_active(),
            Screen::StackContainers => self.stack_presenter.containers.is_filter_active(),
            Screen::ContainerLogs => self.container_presenter.is_log_search_active(),
            Screen::ContainerExplorer => self.container_explorer.is_filter_active(),
            _ => false,
        }
    }

    /// Returns true if the key was consumed by the filter input.
    pub fn handle_filter_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Esc => {
                match self.screen {
                    Screen::ContainerList => self.container_presenter.deactivate_filter(),
                    Screen::VolumeList => self.volume_presenter.deactivate_filter(),
                    Screen::ImageList => self.image_presenter.deactivate_filter(),
                    Screen::NetworkList => self.network_presenter.deactivate_filter(),
                    Screen::StackList => self.stack_presenter.deactivate_filter(),
                    Screen::StackContainers => self.stack_presenter.containers.deactivate_filter(),
                    Screen::ContainerLogs => self.container_presenter.deactivate_log_search(),
                    Screen::ContainerExplorer => self.container_explorer.deactivate_filter(),
                    _ => {}
                }
                true
            }
            KeyCode::Backspace => {
                match self.screen {
                    Screen::ContainerList => self.container_presenter.pop_filter_char(),
                    Screen::VolumeList => self.volume_presenter.pop_filter_char(),
                    Screen::ImageList => self.image_presenter.pop_filter_char(),
                    Screen::NetworkList => self.network_presenter.pop_filter_char(),
                    Screen::StackList => self.stack_presenter.pop_filter_char(),
                    Screen::StackContainers => self.stack_presenter.containers.pop_filter_char(),
                    Screen::ContainerLogs => self.container_presenter.pop_log_search_char(),
                    Screen::ContainerExplorer => self.container_explorer.pop_filter_char(),
                    _ => {}
                }
                true
            }
            KeyCode::Char(c) => {
                match self.screen {
                    Screen::ContainerList => self.container_presenter.push_filter_char(c),
                    Screen::VolumeList => self.volume_presenter.push_filter_char(c),
                    Screen::ImageList => self.image_presenter.push_filter_char(c),
                    Screen::NetworkList => self.network_presenter.push_filter_char(c),
                    Screen::StackList => self.stack_presenter.push_filter_char(c),
                    Screen::StackContainers => self.stack_presenter.containers.push_filter_char(c),
                    Screen::ContainerLogs => self.container_presenter.push_log_search_char(c),
                    Screen::ContainerExplorer => self.container_explorer.push_filter_char(c),
                    _ => {}
                }
                true
            }
            _ => false,
        }
    }

    pub(crate) async fn handle_stack_list_action(&mut self, action: AppAction) {
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
                    self.refresh_container_stats_subscription().await;
                }
            }
            AppAction::StartStop => {
                self.stack_presenter.set_loading(true);
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
                            self.stack_presenter.set_loading(false);
                        } else {
                            self.load_stacks().await;
                        }
                    } else {
                        self.stack_presenter.set_loading(false);
                    }
                } else {
                    self.stack_presenter.set_loading(false);
                }
            }
            AppAction::StopAll => {
                self.stack_presenter.set_loading(true);
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
                            self.stack_presenter.set_loading(false);
                        } else {
                            self.load_stacks().await;
                        }
                    } else {
                        self.stack_presenter.set_loading(false);
                    }
                } else {
                    self.stack_presenter.set_loading(false);
                }
            }
            AppAction::RemoveAll => {
                if let Some(stack) = self.stack_presenter.selected_stack() {
                    if !stack.containers.is_empty() {
                        self.confirm_dialog = Some((ConfirmAction::StackContainersRemoveAll, true));
                    }
                }
            }
            AppAction::Refresh => self.load_stacks().await,
            AppAction::ActivateFilter => {
                self.stack_presenter.activate_filter();
            }
            AppAction::ToggleSort => self.stack_presenter.toggle_sort(),
            _ => {}
        }
    }

    pub(crate) async fn handle_stack_containers_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => self.should_quit = true,
            AppAction::Back => {
                self.stop_container_stats_subscription();
                self.stack_presenter.clear_runtime_stats();
                self.screen = Screen::StackList;
                self.stack_presenter.set_stack_containers(vec![]);
                self.load_stacks().await;
            }
            AppAction::NavigateUp => self.stack_presenter.navigate_container_up(),
            AppAction::NavigateDown => self.stack_presenter.navigate_container_down(),
            AppAction::ViewLogs => {
                if let Some(container) = self.stack_presenter.selected_stack_container().cloned() {
                    self.container_presenter.deactivate_log_search();
                    self.load_container_logs_by(&container.id, &container.name)
                        .await;
                    if self.container_presenter.logs.is_some() {
                        self.logs_return_screen = Some(Screen::StackContainers);
                        self.screen = Screen::ContainerLogs;
                    }
                }
            }
            AppAction::Select | AppAction::ViewDetails => {
                if let Some(container) = self.stack_presenter.selected_stack_container().cloned() {
                    self.load_container_details(&container.id).await;
                    if self.container_presenter.selected_container.is_some() {
                        self.details_return_screen = Some(Screen::StackContainers);
                        self.screen = Screen::ContainerDetails;
                    }
                }
            }
            AppAction::StartStop => {
                if let Some(container) = self.stack_presenter.selected_stack_container().cloned() {
                    self.stack_presenter.containers.set_loading(true);
                    let result = if container.can_stop {
                        self.container_actions.stop_container(&container.id).await
                    } else if container.can_start {
                        self.container_actions.start_container(&container.id).await
                    } else {
                        self.stack_presenter.containers.set_loading(false);
                        return;
                    };
                    match result {
                        Ok(_) => self.refresh_stack_containers().await,
                        Err(e) => {
                            self.popup_message = Some(PopupMessage::Error(e.to_string()));
                            self.stack_presenter.containers.set_loading(false);
                        }
                    }
                }
            }
            AppAction::Delete => {
                if let Some(container) = self.stack_presenter.selected_stack_container() {
                    let force = container.can_stop;
                    self.confirm_dialog = Some((ConfirmAction::ContainerDelete(force), true));
                }
            }
            AppAction::PauseUnpause => {
                if let Some(container) = self.stack_presenter.selected_stack_container().cloned() {
                    self.stack_presenter.containers.set_loading(true);
                    let result = if container.can_pause {
                        self.container_actions.pause_container(&container.id).await
                    } else if container.can_unpause {
                        self.container_actions
                            .unpause_container(&container.id)
                            .await
                    } else {
                        self.stack_presenter.containers.set_loading(false);
                        return;
                    };
                    match result {
                        Ok(_) => self.refresh_stack_containers().await,
                        Err(e) => {
                            self.popup_message = Some(PopupMessage::Error(e.to_string()));
                            self.stack_presenter.containers.set_loading(false);
                        }
                    }
                }
            }
            AppAction::Restart => {
                if let Some(container) = self.stack_presenter.selected_stack_container().cloned() {
                    if !container.can_restart {
                        return;
                    }
                    self.stack_presenter.containers.set_loading(true);
                    match self
                        .container_actions
                        .restart_container(&container.id)
                        .await
                    {
                        Ok(_) => self.refresh_stack_containers().await,
                        Err(e) => {
                            self.popup_message = Some(PopupMessage::Error(e.to_string()));
                            self.stack_presenter.containers.set_loading(false);
                        }
                    }
                }
            }
            AppAction::Refresh => self.refresh_stack_containers().await,
            AppAction::Prune => {
                self.confirm_dialog = Some((ConfirmAction::ContainersPrune, true));
            }
            AppAction::ActivateFilter => {
                self.stack_presenter.containers.activate_filter();
            }
            AppAction::Exec => self.open_exec_shell_dialog(),
            AppAction::Add => self.open_add_dialog(),
            AppAction::ToggleSort => self.stack_presenter.containers.toggle_sort(),
            AppAction::BrowseFiles => {
                if let Some(container) = self.stack_presenter.selected_stack_container().cloned() {
                    self.open_container_explorer(&container.id, &container.name)
                        .await;
                }
            }
            _ => {}
        }
    }
}

pub(crate) fn normalize_image_ref(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() || trimmed.contains(':') {
        trimmed.to_string()
    } else {
        format!("{trimmed}:latest")
    }
}

impl App {
    pub fn has_modal_overlay(&self) -> bool {
        self.popup_message.is_some()
            || self.confirm_dialog.is_some()
            || self.exec_shell_dialog.is_some()
            || self.add_dialog.is_some()
            || self.help_popup_open
    }

    pub(crate) fn handle_add_dialog_key(&mut self, code: KeyCode) -> bool {
        let Some(dialog) = self.add_dialog.as_mut() else {
            return false;
        };
        match code {
            KeyCode::Tab => dialog.state.focus_next(),
            KeyCode::BackTab => dialog.state.focus_previous(),
            KeyCode::Backspace => dialog.state.focused_mut().pop_char(),
            KeyCode::Char(c) => dialog.state.focused_mut().push_char(c),
            _ => return false,
        }
        true
    }

    pub(crate) async fn handle_add_dialog_action(&mut self, action: AppAction) {
        match action {
            AppAction::Back | AppAction::Quit => self.add_dialog = None,
            AppAction::NavigateUp => {
                if let Some(dialog) = self.add_dialog.as_mut() {
                    dialog.state.focus_previous();
                }
            }
            AppAction::NavigateDown => {
                if let Some(dialog) = self.add_dialog.as_mut() {
                    dialog.state.focus_next();
                }
            }
            AppAction::Select => self.confirm_add_dialog().await,
            _ => {}
        }
    }

    async fn confirm_add_dialog(&mut self) {
        let Some(dialog) = self.add_dialog.as_ref() else {
            return;
        };
        match dialog.kind {
            AddDialogKind::Image => {
                let image_ref = normalize_image_ref(dialog.state.values()[0]);
                if image_ref.is_empty() {
                    self.popup_message = Some(PopupMessage::Error(
                        resources::add_image_missing_message().to_string(),
                    ));
                    return;
                }
                self.add_dialog = None;
                self.start_image_pull(vec![image_ref]).await;
            }
            AddDialogKind::Container => {
                let values = dialog.state.values();
                let name = values[0].trim().to_string();
                let image = normalize_image_ref(values[1]);
                if name.is_empty() || image.is_empty() {
                    self.popup_message = Some(PopupMessage::Error(
                        resources::add_container_missing_fields_message().to_string(),
                    ));
                    return;
                }
                let port_mappings = match parse_port_mappings(values[2]) {
                    Ok(mappings) => mappings,
                    Err(e) => {
                        self.popup_message = Some(PopupMessage::Error(e.to_string()));
                        return;
                    }
                };
                self.add_dialog = None;
                self.start_container_pull_then_create(CreateContainerRequest::new(
                    &name,
                    &image,
                    &port_mappings,
                ))
                .await;
            }
        }
    }

    pub(crate) fn open_add_dialog(&mut self) {
        if self.pending_image_pull.is_some() {
            return;
        }
        let kind = match self.screen {
            Screen::ImageList | Screen::ImageDetails => AddDialogKind::Image,
            Screen::ContainerList | Screen::StackContainers => AddDialogKind::Container,
            _ => return,
        };
        let field_count = match kind {
            AddDialogKind::Image => 1,
            AddDialogKind::Container => 3,
        };
        self.add_dialog = Some(AddDialog {
            kind,
            state: InputDialogState::new(field_count),
        });
    }

    async fn start_image_pull(&mut self, image_refs: Vec<String>) {
        if self.pending_image_pull.is_some() {
            return;
        }
        self.image_presenter.set_loading(true);
        match self.image_actions.pull_images(&image_refs).await {
            Ok(subscription) => {
                self.image_presenter.clear_pull_progress();
                self.pending_image_pull = Some(PendingImagePull { subscription });
            }
            Err(e) => {
                self.image_presenter.set_loading(false);
                self.popup_message = Some(PopupMessage::Error(e.to_string()));
            }
        }
    }

    async fn start_container_pull_then_create(&mut self, request: CreateContainerRequest) {
        self.pending_create_after_pull = Some(request);
        self.container_presenter.set_loading(true);
        self.start_image_pull_for_create().await;
    }

    async fn start_image_pull_for_create(&mut self) {
        if self.pending_image_pull.is_some() {
            return;
        }
        let Some(request) = self.pending_create_after_pull.clone() else {
            return;
        };
        let image_ref = normalize_image_ref(&request.image);
        self.image_presenter.set_loading(true);
        match self.image_actions.pull_images(&[image_ref]).await {
            Ok(subscription) => {
                self.image_presenter.clear_pull_progress();
                self.pending_image_pull = Some(PendingImagePull { subscription });
            }
            Err(e) => {
                self.pending_create_after_pull = None;
                self.container_presenter.set_loading(false);
                self.image_presenter.set_loading(false);
                self.popup_message = Some(PopupMessage::Error(e.to_string()));
            }
        }
    }

    pub(crate) fn drain_container_stats(&mut self) {
        let mut drained_events = Vec::new();
        let mut disconnected = false;

        if let Some(subscription) = self.container_stats_subscription.as_mut() {
            loop {
                match subscription.try_recv() {
                    Ok(event) => drained_events.push(event),
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        disconnected = true;
                        break;
                    }
                }
            }
        }

        if disconnected {
            self.container_stats_subscription = None;
            self.container_presenter.clear_runtime_stats();
            self.stack_presenter.clear_runtime_stats();
        }

        for event in drained_events {
            match event {
                ContainerStatsEvent::Update(update) => {
                    self.container_presenter.apply_stats_update(update.clone());
                    self.stack_presenter.apply_stats_update(update);
                }
                ContainerStatsEvent::Error {
                    container_id,
                    message,
                } => {
                    self.popup_message = Some(PopupMessage::Error(format!(
                        "Stats stream failed for {container_id}: {message}"
                    )));
                }
            }
        }
    }
}
