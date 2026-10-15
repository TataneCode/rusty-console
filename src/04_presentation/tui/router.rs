//! Screen routing: maps user actions to screen-specific handlers.

use crate::application::container::ContainerStatsEvent;
use crate::presentation::tui::app::{format_bytes, App, PendingImagePull, Screen};
use crate::presentation::tui::common::{resources, AppAction, PopupMessage};
use crate::presentation::tui::confirm::ConfirmAction;
use crossterm::event::KeyCode;
use std::collections::BTreeSet;
use tokio::sync::mpsc::error::TryRecvError;

impl App {
    pub(crate) async fn handle_action(&mut self, action: AppAction) {
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

        match &self.screen {
            Screen::MainMenu => self.handle_main_menu_action(action).await,
            Screen::ContainerList => self.handle_container_list_action(action).await,
            Screen::ContainerLogs => self.handle_container_logs_action(action).await,
            Screen::ContainerDetails => self.handle_details_action(action),
            Screen::VolumeList => self.handle_volume_list_action(action).await,
            Screen::VolumeDetails => self.handle_details_action(action),
            Screen::ImageList => self.handle_image_list_action(action).await,
            Screen::ImageDetails => self.handle_details_action(action),
            Screen::NetworkList => self.handle_network_list_action(action).await,
            Screen::NetworkDetails => self.handle_network_details_action(action).await,
            Screen::StackList => self.handle_stack_list_action(action).await,
            Screen::StackContainers => self.handle_stack_containers_action(action).await,
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
            _ => {}
        }
    }

    pub(crate) async fn handle_container_logs_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit | AppAction::Back => {
                self.container_presenter.deactivate_log_search();
                self.container_presenter.clear_logs();
                self.screen = Screen::ContainerList;
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

    pub(crate) fn handle_details_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit | AppAction::Back => match &self.screen {
                Screen::ContainerDetails => {
                    self.container_presenter.clear_details();
                    self.screen = Screen::ContainerList;
                }
                Screen::VolumeDetails => {
                    self.volume_presenter.clear_details();
                    self.screen = Screen::VolumeList;
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

    pub(crate) async fn handle_volume_list_action(&mut self, action: AppAction) {
        match action {
            AppAction::Quit => self.should_quit = true,
            AppAction::Back => self.screen = Screen::MainMenu,
            AppAction::NavigateUp => self.volume_presenter.navigate_up(),
            AppAction::NavigateDown => self.volume_presenter.navigate_down(),
            AppAction::ViewDetails => {
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
            _ => {}
        }
    }

    pub(crate) async fn handle_image_list_action(&mut self, action: AppAction) {
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
            _ => {}
        }
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
                    .networks
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
            Screen::ContainerList | Screen::StackContainers => {
                self.container_presenter.set_loading(true);
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
                        self.container_presenter.set_loading(false);
                    } else if matches!(self.screen, Screen::StackContainers) {
                        self.refresh_stack_containers().await;
                    } else {
                        self.load_containers().await;
                    }
                } else {
                    self.container_presenter.set_loading(false);
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
                self.container_presenter.set_loading(true);
                match self.container_actions.prune_containers().await {
                    Ok(result) => {
                        self.popup_message =
                            Some(PopupMessage::Info(resources::prune_result_message(
                                "container",
                                result.deleted_count,
                                &format_bytes(result.space_freed),
                            )));
                        self.load_containers().await;
                    }
                    Err(e) => {
                        self.popup_message = Some(PopupMessage::Error(e.to_string()));
                        self.container_presenter.set_loading(false);
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
                self.stack_presenter.set_loading(true);
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
            Screen::ContainerLogs => self.container_presenter.is_log_search_active(),
            _ => false,
        }
    }

    /// Returns true if the key was consumed by the filter input.
    pub(crate) fn handle_filter_key(&mut self, code: KeyCode) -> bool {
        match code {
            KeyCode::Esc => {
                match self.screen {
                    Screen::ContainerList => self.container_presenter.deactivate_filter(),
                    Screen::VolumeList => self.volume_presenter.deactivate_filter(),
                    Screen::ImageList => self.image_presenter.deactivate_filter(),
                    Screen::NetworkList => self.network_presenter.deactivate_filter(),
                    Screen::StackList => self.stack_presenter.deactivate_filter(),
                    Screen::ContainerLogs => self.container_presenter.deactivate_log_search(),
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
                    Screen::ContainerLogs => self.container_presenter.pop_log_search_char(),
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
                    Screen::ContainerLogs => self.container_presenter.push_log_search_char(c),
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
            AppAction::PullImages => self.pull_selected_stack_images(false).await,
            AppAction::Refresh => self.load_stacks().await,
            AppAction::ActivateFilter => {
                self.stack_presenter.activate_filter();
            }
            _ => {}
        }
    }

    pub(crate) async fn handle_stack_containers_action(&mut self, action: AppAction) {
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
                    self.stack_presenter.set_loading(true);
                    let result = if container.can_stop {
                        self.container_actions.stop_container(&container.id).await
                    } else if container.can_start {
                        self.container_actions.start_container(&container.id).await
                    } else {
                        self.stack_presenter.set_loading(false);
                        return;
                    };
                    match result {
                        Ok(_) => self.refresh_stack_containers().await,
                        Err(e) => {
                            self.popup_message = Some(PopupMessage::Error(e.to_string()));
                            self.stack_presenter.set_loading(false);
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
            AppAction::StopAll => {
                self.stack_presenter.set_loading(true);
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
                            Err(e) => {
                                self.popup_message = Some(PopupMessage::Error(e.to_string()));
                                self.stack_presenter.set_loading(false);
                            }
                        }
                    } else {
                        self.stack_presenter.set_loading(false);
                    }
                } else {
                    self.stack_presenter.set_loading(false);
                }
            }
            AppAction::StartAll => {
                self.stack_presenter.set_loading(true);
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
                            Err(e) => {
                                self.popup_message = Some(PopupMessage::Error(e.to_string()));
                                self.stack_presenter.set_loading(false);
                            }
                        }
                    } else {
                        self.stack_presenter.set_loading(false);
                    }
                } else {
                    self.stack_presenter.set_loading(false);
                }
            }
            AppAction::RemoveAll if !self.stack_presenter.stack_containers.is_empty() => {
                self.confirm_dialog = Some((ConfirmAction::StackContainersRemoveAll, true));
            }
            AppAction::PullImages => self.pull_selected_stack_images(true).await,
            AppAction::Refresh => self.refresh_stack_containers().await,
            AppAction::Exec => self.open_exec_shell_dialog(),
            _ => {}
        }
    }

    fn selected_stack_pull_data(&self) -> Option<(String, Vec<String>)> {
        let stack = self.stack_presenter.selected_stack()?;
        let images = stack
            .containers
            .iter()
            .map(|container| container.image.clone())
            .filter(|image| !image.trim().is_empty() && image.as_str() != "unknown")
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();

        Some((stack.name.clone(), images))
    }

    async fn pull_selected_stack_images(&mut self, refresh_stack_containers: bool) {
        if self.pending_image_pull.is_some() {
            return;
        }

        let Some((stack_name, images)) = self.selected_stack_pull_data() else {
            return;
        };

        if images.is_empty() {
            self.popup_message = Some(PopupMessage::Info(resources::stack_pull_empty_message(
                &stack_name,
            )));
            return;
        }

        // Set loading state - spinner will be shown
        self.stack_presenter.set_loading(true);

        match self.stack_actions.pull_images(&images).await {
            Ok(subscription) => {
                self.stack_presenter.clear_pull_progress();
                self.pending_image_pull = Some(PendingImagePull {
                    subscription,
                    stack_name,
                    image_count: images.len(),
                    refresh_containers: refresh_stack_containers,
                });
            }
            Err(e) => {
                self.stack_presenter.set_loading(false);
                self.popup_message = Some(PopupMessage::Error(e.to_string()));
            }
        }
    }

    pub(crate) fn has_modal_overlay(&self) -> bool {
        self.popup_message.is_some()
            || self.confirm_dialog.is_some()
            || self.exec_shell_dialog.is_some()
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
        }

        for event in drained_events {
            match event {
                ContainerStatsEvent::Update(update) => {
                    self.container_presenter.apply_stats_update(update);
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
