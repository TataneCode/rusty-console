use crate::application::container::{
    ContainerDto, ContainerStatsSubscription, CreateContainerRequest,
};
use crate::application::image::{ImagePullEvent, ImagePullOutcome, ImagePullSubscription};
use crate::application::volume::VolumeDto;
use crate::presentation::tui::common::{
    dim_background, map_key_to_action_for_context, render_breadcrumb, render_confirm_dialog,
    render_expanded_log_line, render_input_dialog, render_main_menu, render_popup_message,
    render_selection_dialog, resources, split_app_area, AppAction, InputDialogState, InputField,
    KeyContext, PopupMessage, Spinner,
};
use crate::presentation::tui::confirm::{confirm_message, ConfirmAction};
use crate::presentation::tui::container::{
    render_container_details, render_container_list, render_container_logs, ContainerActions,
    ContainerPresenter, LogSearchView,
};
use crate::presentation::tui::event::{AppEvent, EventHandler};
use crate::presentation::tui::exec::{
    build_exec_command, build_external_exec_command, finalize_exec_command_result,
    find_terminal_emulator, format_exec_error, ExecCommandConfig, ExecCommandResult,
    ExecRefreshTarget, ExecShellDialog, PendingExec, EXEC_SHELL_OPTIONS,
};
use crate::presentation::tui::image::{
    filter_images, render_image_details, render_image_list, ImageActions, ImageHelpBarState,
    ImagePresenter,
};
use crate::presentation::tui::network::{
    filter_networks, render_network_details, render_network_list, NetworkActions, NetworkPresenter,
};
use crate::presentation::tui::stack::{
    filter_stacks, render_stack_containers, render_stack_list, StackActions, StackHelpBarState,
    StackPresenter,
};
use crate::presentation::tui::volume::{
    render_volume_details, render_volume_list, VolumeActions, VolumePresenter,
};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, widgets::ListState, Frame, Terminal};
use std::{collections::HashSet, io};
use tokio::sync::mpsc::error::TryRecvError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    MainMenu,
    ContainerList,
    ContainerLogs,
    ContainerDetails,
    VolumeList,
    VolumeDetails,
    ImageList,
    ImageDetails,
    NetworkList,
    NetworkDetails,
    StackList,
    StackContainers,
}

pub enum AddDialogKind {
    Image,
    Container,
}

pub(crate) struct AddDialog {
    pub(crate) kind: AddDialogKind,
    pub(crate) state: InputDialogState,
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
    pub network_presenter: NetworkPresenter,
    pub container_actions: ContainerActions,
    pub volume_actions: VolumeActions,
    pub image_actions: ImageActions,
    pub stack_actions: StackActions,
    pub network_actions: NetworkActions,
    pub confirm_dialog: Option<(ConfirmAction, bool)>,
    pub expanded_log_line: Option<usize>,
    pub popup_message: Option<PopupMessage>,
    pub spinner: Spinner,
    exec_command_config: ExecCommandConfig,
    pub(crate) exec_shell_dialog: Option<ExecShellDialog>,
    pending_exec: Option<PendingExec>,
    pub(crate) container_stats_subscription: Option<ContainerStatsSubscription>,
    pub(crate) pending_image_pull: Option<PendingImagePull>,
    pub(crate) add_dialog: Option<AddDialog>,
    pub(crate) pending_create_after_pull: Option<CreateContainerRequest>,
}

pub(crate) struct PendingImagePull {
    pub(crate) subscription: ImagePullSubscription,
}

fn record_cleanup_error(cleanup_error: &mut Option<io::Error>, result: io::Result<()>) {
    if let Err(err) = result {
        if cleanup_error.is_none() {
            *cleanup_error = Some(err);
        }
    }
}

fn finalize_run_result(result: io::Result<()>, cleanup_error: Option<io::Error>) -> io::Result<()> {
    match (result, cleanup_error) {
        (Ok(()), None) => Ok(()),
        (Ok(()), Some(cleanup_err)) => Err(cleanup_err),
        (Err(err), None) => Err(err),
        (Err(err), Some(cleanup_err)) => Err(io::Error::new(
            err.kind(),
            format!("{err}; cleanup also failed: {cleanup_err}"),
        )),
    }
}

impl App {
    pub fn new(
        container_actions: ContainerActions,
        volume_actions: VolumeActions,
        image_actions: ImageActions,
        stack_actions: StackActions,
        network_actions: NetworkActions,
        exec_command_config: ExecCommandConfig,
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
            network_presenter: NetworkPresenter::new(),
            container_actions,
            volume_actions,
            image_actions,
            stack_actions,
            network_actions,
            confirm_dialog: None,
            expanded_log_line: None,
            popup_message: None,
            spinner: Spinner::new(),
            exec_command_config,
            exec_shell_dialog: None,
            pending_exec: None,
            container_stats_subscription: None,
            pending_image_pull: None,
            add_dialog: None,
            pending_create_after_pull: None,
        }
    }

    pub async fn run(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_inner(&mut terminal).await;
        let mut cleanup_error = None;

        record_cleanup_error(&mut cleanup_error, disable_raw_mode());
        record_cleanup_error(
            &mut cleanup_error,
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            ),
        );
        record_cleanup_error(&mut cleanup_error, terminal.show_cursor());

        finalize_run_result(result, cleanup_error)
    }

    async fn run_inner(
        &mut self,
        terminal: &mut ratatui::Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        let event_handler = EventHandler::default();

        loop {
            self.drain_container_stats();
            terminal.draw(|frame| self.render(frame))?;

            match event_handler.next()? {
                AppEvent::Key(key) => {
                    if self.add_dialog.is_some() && self.handle_add_dialog_key(key.code) {
                        // Key consumed by add dialog input
                    } else if !self.has_modal_overlay()
                        && self.is_filter_active()
                        && self.handle_filter_key(key.code)
                    {
                        // Key consumed by filter input
                    } else if let Some(action) = map_key_to_action_for_context(
                        key,
                        match self.screen {
                            Screen::ContainerLogs => KeyContext::ContainerLogs,
                            Screen::NetworkDetails => KeyContext::NetworkDetails,
                            _ => KeyContext::Global,
                        },
                    ) {
                        self.handle_action(action).await;
                    }
                }
                AppEvent::Tick => {}
            }

            if let Some(request) = self.pending_exec.take() {
                self.execute_pending_exec(terminal, request).await?;
            }

            self.drain_container_stats();
            self.drain_image_pull().await;

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    #[doc(hidden)]
    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Update spinner animation
        self.spinner.update();
        let spinner_char = self.spinner.current_char();

        let content_area = if self.screen == Screen::MainMenu {
            area
        } else {
            let [breadcrumb_area, rest] = split_app_area(area);
            let segments = self.breadcrumb_segments();
            let refs: Vec<&str> = segments.iter().map(String::as_str).collect();
            render_breadcrumb(frame, breadcrumb_area, &refs);
            rest
        };

        match &self.screen {
            Screen::MainMenu => render_main_menu(frame, content_area, &mut self.menu_state),
            Screen::ContainerList => {
                let presenter = &mut self.container_presenter;
                let is_loading = presenter.is_loading();
                let active_filter = presenter.active_filter().map(str::to_string);
                let containers: Vec<ContainerDto> = presenter
                    .filtered_containers()
                    .into_iter()
                    .cloned()
                    .collect();
                let selection_state = &mut presenter.selection.state;
                render_container_list(
                    frame,
                    content_area,
                    &containers,
                    selection_state,
                    active_filter.as_deref(),
                    is_loading,
                    spinner_char,
                );
            }
            Screen::ContainerLogs => {
                let presenter = &mut self.container_presenter;
                if let Some(logs) = presenter.logs.as_ref() {
                    let search_value = presenter.log_search_value().to_string();
                    let active_search = presenter.active_search_value().map(str::to_string);
                    let search = LogSearchView {
                        active: presenter.is_log_search_active(),
                        value: &search_value,
                        active_term: active_search.as_deref(),
                        counter: presenter.search_counter(),
                        matches: &presenter.log_search_results,
                    };
                    render_container_logs(
                        frame,
                        content_area,
                        logs,
                        &mut presenter.logs_state,
                        &search,
                    );
                }
            }
            Screen::ContainerDetails => {
                if let Some(container) = &self.container_presenter.selected_container {
                    render_container_details(frame, content_area, container);
                }
            }
            Screen::VolumeList => {
                let presenter = &mut self.volume_presenter;
                let is_loading = presenter.is_loading();
                let active_filter = presenter.active_filter().map(str::to_string);
                let volumes: Vec<VolumeDto> =
                    presenter.filtered_volumes().into_iter().cloned().collect();
                let selection_state = &mut presenter.selection.state;
                render_volume_list(
                    frame,
                    content_area,
                    &volumes,
                    selection_state,
                    active_filter.as_deref(),
                    is_loading,
                    spinner_char,
                );
            }
            Screen::VolumeDetails => {
                if let Some(volume) = &self.volume_presenter.selected_volume {
                    render_volume_details(frame, content_area, volume);
                }
            }
            Screen::ImageList => {
                let presenter = &mut self.image_presenter;
                let is_loading = presenter.is_loading();
                let active_filter = presenter.active_filter().map(str::to_string);
                let filter = presenter.filter.value().to_string();
                let images = &presenter.images;
                let selection_state = &mut presenter.selection.state;
                let pull_progress = presenter.pull_progress.as_ref();
                let filtered = filter_images(images, &filter);
                render_image_list(
                    frame,
                    content_area,
                    &filtered,
                    selection_state,
                    active_filter.as_deref(),
                    ImageHelpBarState {
                        is_loading,
                        spinner_char,
                        pull_progress,
                    },
                );
            }
            Screen::ImageDetails => {
                if let Some(image) = &self.image_presenter.selected_image {
                    render_image_details(
                        frame,
                        content_area,
                        image,
                        ImageHelpBarState {
                            is_loading: self.image_presenter.is_loading(),
                            spinner_char,
                            pull_progress: self.image_presenter.pull_progress.as_ref(),
                        },
                    );
                }
            }
            Screen::NetworkList => {
                let presenter = &mut self.network_presenter;
                let is_loading = presenter.is_loading();
                let active_filter = presenter.active_filter().map(str::to_string);
                let filter = presenter.filter.value().to_string();
                let networks = &presenter.networks;
                let selection_state = &mut presenter.selection.state;
                let filtered = filter_networks(networks, &filter);
                render_network_list(
                    frame,
                    content_area,
                    &filtered,
                    selection_state,
                    active_filter.as_deref(),
                    is_loading,
                    spinner_char,
                );
            }
            Screen::NetworkDetails => {
                let presenter = &self.network_presenter;
                if let Some(network) = &presenter.selected_network {
                    render_network_details(
                        frame,
                        content_area,
                        network,
                        presenter.container_selection.selected(),
                    );
                }
            }
            Screen::StackList => {
                let presenter = &mut self.stack_presenter;
                let is_loading = presenter.is_loading();
                let active_filter = presenter.active_filter().map(str::to_string);
                let filter = presenter.filter.value().to_string();
                let stacks = &presenter.stacks;
                let selection_state = &mut presenter.selection.state;
                let filtered = filter_stacks(stacks, &filter);
                render_stack_list(
                    frame,
                    content_area,
                    &filtered,
                    selection_state,
                    active_filter.as_deref(),
                    StackHelpBarState {
                        is_loading,
                        spinner_char,
                    },
                );
            }
            Screen::StackContainers => {
                let stack_name = self
                    .stack_presenter
                    .selected_stack()
                    .map(|s| s.name.as_str())
                    .unwrap_or("Stack")
                    .to_string();
                let is_loading = self.stack_presenter.is_loading();
                render_stack_containers(
                    frame,
                    content_area,
                    &stack_name,
                    &self.stack_presenter.stack_containers,
                    &mut self.stack_presenter.container_selection.state,
                    StackHelpBarState {
                        is_loading,
                        spinner_char,
                    },
                );
            }
        }

        let has_overlay = self.expanded_log_line.is_some()
            || self.confirm_dialog.is_some()
            || self.popup_message.is_some()
            || self.exec_shell_dialog.is_some()
            || self.add_dialog.is_some();
        if has_overlay {
            dim_background(frame);
        }
        if let Some(line) = self.container_presenter.logs.as_ref().and_then(|logs| {
            self.expanded_log_line
                .and_then(|index| logs.lines.get(index))
        }) {
            render_expanded_log_line(frame, &line.timestamp_display(), &line.message);
        }
        if let Some((action, selected_yes)) = &self.confirm_dialog {
            render_confirm_dialog(frame, confirm_message(*action), *selected_yes);
        }

        if let Some(message) = &self.popup_message {
            render_popup_message(frame, message);
        }

        if let Some(dialog) = &mut self.exec_shell_dialog {
            render_selection_dialog(
                frame,
                resources::EXEC_SHELL_DIALOG_TITLE,
                &EXEC_SHELL_OPTIONS,
                dialog.state_mut(),
                resources::EXEC_SHELL_DIALOG_HELP,
            );
        }
        if let Some(dialog) = self.add_dialog.as_ref() {
            let focused = dialog.state.focused_index();
            let values = dialog.state.values();
            let mut fields: Vec<InputField<'_>> = Vec::new();
            match dialog.kind {
                AddDialogKind::Image => fields.push(InputField {
                    label: resources::ADD_IMAGE_FIELD_LABEL,
                    value: values[0],
                    focused: focused == 0,
                }),
                AddDialogKind::Container => {
                    fields.push(InputField {
                        label: resources::ADD_CONTAINER_NAME_LABEL,
                        value: values[0],
                        focused: focused == 0,
                    });
                    fields.push(InputField {
                        label: resources::ADD_CONTAINER_IMAGE_LABEL,
                        value: values[1],
                        focused: focused == 1,
                    });
                    fields.push(InputField {
                        label: resources::ADD_CONTAINER_PORTS_LABEL,
                        value: values[2],
                        focused: focused == 2,
                    });
                }
            }
            let (title, help) = match dialog.kind {
                AddDialogKind::Image => (
                    resources::ADD_IMAGE_DIALOG_TITLE,
                    resources::ADD_IMAGE_DIALOG_HELP,
                ),
                AddDialogKind::Container => (
                    resources::ADD_CONTAINER_DIALOG_TITLE,
                    resources::ADD_CONTAINER_DIALOG_HELP,
                ),
            };
            render_input_dialog(
                frame,
                title,
                &fields,
                help,
                resources::INPUT_DIALOG_WIDTH_PERCENT,
            );
        }
    }

    fn breadcrumb_segments(&self) -> Vec<String> {
        let mut segments = vec![resources::APP_TITLE.to_string()];
        match self.screen {
            Screen::MainMenu => return segments,
            Screen::ContainerList => segments.push(resources::CONTAINER_TITLE.to_string()),
            Screen::ContainerLogs => {
                segments.push(resources::CONTAINER_TITLE.to_string());
                let name = self
                    .container_presenter
                    .logs
                    .as_ref()
                    .map(|logs| logs.container_name.clone())
                    .or_else(|| {
                        self.container_presenter
                            .selected_container
                            .as_ref()
                            .map(|container| container.name.clone())
                    });
                if let Some(name) = name {
                    segments.push(format!("{name} {}", resources::BREADCRUMB_LOGS_LABEL));
                }
            }
            Screen::ContainerDetails => {
                segments.push(resources::CONTAINER_TITLE.to_string());
                if let Some(container) = &self.container_presenter.selected_container {
                    segments.push(format!(
                        "{} {}",
                        container.name,
                        resources::BREADCRUMB_DETAILS_LABEL
                    ));
                }
            }
            Screen::VolumeList => segments.push(resources::VOLUME_TITLE.to_string()),
            Screen::VolumeDetails => {
                segments.push(resources::VOLUME_TITLE.to_string());
                if let Some(volume) = &self.volume_presenter.selected_volume {
                    segments.push(format!(
                        "{} {}",
                        volume.name,
                        resources::BREADCRUMB_DETAILS_LABEL
                    ));
                }
            }
            Screen::ImageList => segments.push(resources::IMAGE_TITLE.to_string()),
            Screen::ImageDetails => {
                segments.push(resources::IMAGE_TITLE.to_string());
                if let Some(image) = &self.image_presenter.selected_image {
                    segments.push(format!(
                        "{} {}",
                        image.full_name,
                        resources::BREADCRUMB_DETAILS_LABEL
                    ));
                }
            }
            Screen::NetworkList => segments.push(resources::NETWORK_TITLE.to_string()),
            Screen::NetworkDetails => {
                segments.push(resources::NETWORK_TITLE.to_string());
                if let Some(network) = &self.network_presenter.selected_network {
                    segments.push(format!(
                        "{} {}",
                        network.name,
                        resources::BREADCRUMB_DETAILS_LABEL
                    ));
                }
            }
            Screen::StackList => segments.push(resources::STACK_TITLE.to_string()),
            Screen::StackContainers => {
                segments.push(resources::STACK_TITLE.to_string());
                if let Some(stack) = self.stack_presenter.selected_stack() {
                    segments.push(format!(
                        "{} {}",
                        stack.name,
                        resources::BREADCRUMB_CONTAINERS_LABEL
                    ));
                }
            }
        }
        segments
    }

    pub(crate) async fn load_containers(&mut self) {
        self.container_presenter.set_loading(true);
        match self.container_actions.load_containers().await {
            Ok(containers) => {
                self.container_presenter.set_containers(containers);
                self.refresh_container_stats_subscription().await;
            }
            Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
        }
        self.container_presenter.set_loading(false);
    }

    pub(crate) async fn load_container_details(&mut self, id: &str) {
        self.container_presenter.set_loading(true);
        match self.container_actions.load_container_details(id).await {
            Ok(Some(container)) => self.container_presenter.set_details(container),
            Ok(None) => {
                self.popup_message = Some(PopupMessage::Error(
                    resources::CONTAINER_NOT_FOUND_MESSAGE.to_string(),
                ))
            }
            Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
        }
        self.container_presenter.set_loading(false);
    }

    pub(crate) async fn load_container_logs(&mut self, container: &ContainerDto) {
        self.container_presenter.set_loading(true);
        match self.container_actions.load_logs(container, Some(500)).await {
            Ok(logs) => self.container_presenter.set_logs(logs),
            Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
        }
        self.container_presenter.set_loading(false);
    }

    pub(crate) async fn toggle_container(&mut self, container: &ContainerDto) {
        self.container_presenter.set_loading(true);
        let result = if container.can_stop {
            self.container_actions.stop_container(&container.id).await
        } else if container.can_start {
            self.container_actions.start_container(&container.id).await
        } else {
            self.container_presenter.set_loading(false);
            return;
        };

        match result {
            Ok(_) => self.load_containers().await,
            Err(e) => {
                self.popup_message = Some(PopupMessage::Error(e.to_string()));
                self.container_presenter.set_loading(false);
            }
        }
    }

    pub(crate) async fn pause_unpause_container(&mut self, container: &ContainerDto) {
        self.container_presenter.set_loading(true);
        let result = if container.can_pause {
            self.container_actions.pause_container(&container.id).await
        } else if container.can_unpause {
            self.container_actions
                .unpause_container(&container.id)
                .await
        } else {
            self.container_presenter.set_loading(false);
            return;
        };

        match result {
            Ok(_) => self.load_containers().await,
            Err(e) => {
                self.popup_message = Some(PopupMessage::Error(e.to_string()));
                self.container_presenter.set_loading(false);
            }
        }
    }

    pub(crate) async fn restart_container(&mut self, container: &ContainerDto) {
        if !container.can_restart {
            return;
        }

        self.container_presenter.set_loading(true);
        match self
            .container_actions
            .restart_container(&container.id)
            .await
        {
            Ok(_) => self.load_containers().await,
            Err(e) => {
                self.popup_message = Some(PopupMessage::Error(e.to_string()));
                self.container_presenter.set_loading(false);
            }
        }
    }

    pub(crate) async fn load_volumes(&mut self) {
        self.volume_presenter.set_loading(true);
        match self.volume_actions.load_volumes().await {
            Ok(volumes) => self.volume_presenter.set_volumes(volumes),
            Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
        }
        self.volume_presenter.set_loading(false);
    }

    pub(crate) async fn load_images(&mut self) {
        self.image_presenter.set_loading(true);
        match self.image_actions.load_images().await {
            Ok(images) => self.image_presenter.set_images(images),
            Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
        }
        self.image_presenter.set_loading(false);
    }

    pub(crate) async fn load_networks(&mut self) {
        self.network_presenter.set_loading(true);
        match self.network_actions.load_networks().await {
            Ok(networks) => self.network_presenter.set_networks(networks),
            Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
        }
        self.network_presenter.set_loading(false);
    }

    pub(crate) async fn load_stacks(&mut self) {
        self.stack_presenter.set_loading(true);
        match self.stack_actions.load_stacks().await {
            Ok(stacks) => self.stack_presenter.set_stacks(stacks),
            Err(e) => self.popup_message = Some(PopupMessage::Error(e.to_string())),
        }
        self.stack_presenter.set_loading(false);
    }

    pub(crate) async fn refresh_stack_containers(&mut self) {
        self.stack_presenter.set_loading(true);
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
        self.stack_presenter.set_loading(false);
    }

    async fn refresh_container_stats_subscription(&mut self) {
        self.stop_container_stats_subscription();

        let active_container_ids: HashSet<String> = self
            .container_presenter
            .containers
            .iter()
            .filter(|container| container.state.is_active())
            .map(|container| container.id.clone())
            .collect();
        self.container_presenter
            .retain_runtime_stats(&active_container_ids);

        if active_container_ids.is_empty() {
            return;
        }

        match self
            .container_actions
            .subscribe_stats(active_container_ids.iter().cloned().collect())
            .await
        {
            Ok(subscription) => {
                self.container_stats_subscription = Some(subscription);
            }
            Err(error) => {
                self.popup_message = Some(PopupMessage::Error(error.to_string()));
            }
        }
    }

    pub(crate) fn stop_container_stats_subscription(&mut self) {
        if let Some(mut subscription) = self.container_stats_subscription.take() {
            subscription.abort();
        }
    }

    async fn drain_image_pull(&mut self) {
        let Some(pending) = self.pending_image_pull.as_mut() else {
            return;
        };
        let mut outcome: Option<Result<ImagePullOutcome, crate::application::error::AppError>> =
            None;
        loop {
            match pending.subscription.try_recv() {
                Ok(ImagePullEvent::Progress(progress)) => {
                    self.image_presenter.set_pull_progress(progress);
                }
                Ok(ImagePullEvent::Completed(result)) => {
                    outcome = Some(result);
                    break;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    outcome = Some(Err(crate::application::error::AppError::operation_failed(
                        "Image pull stream ended unexpectedly".to_string(),
                    )));
                    break;
                }
            }
        }
        let Some(result) = outcome else {
            return;
        };
        self.take_pending_image_pull();
        if self.pending_create_after_pull.is_some() {
            self.finish_pending_container_create(result.is_ok()).await;
            if let Err(e) = result {
                self.popup_message = Some(PopupMessage::Error(e.to_string()));
            }
            return;
        }
        match result {
            Ok(outcome) => {
                self.load_images().await;
                if self.screen == Screen::ImageDetails {
                    self.restore_selected_image_details();
                }
                self.popup_message = Some(PopupMessage::Info(
                    resources::image_pull_result_message(&outcome.pulled, &outcome.up_to_date),
                ));
            }
            Err(e) => {
                self.popup_message = Some(PopupMessage::Error(e.to_string()));
            }
        }
    }

    async fn finish_pending_container_create(&mut self, pull_ok: bool) {
        self.container_presenter.set_loading(false);
        let Some(request) = self.pending_create_after_pull.take() else {
            return;
        };
        if !pull_ok {
            return;
        }
        match self.container_actions.create_container(&request).await {
            Ok(id) => {
                if let Err(e) = self.container_actions.start_container(&id).await {
                    self.popup_message = Some(PopupMessage::Error(e.to_string()));
                }
                self.load_containers().await;
                if self.popup_message.is_none() {
                    self.popup_message = Some(PopupMessage::Info(
                        resources::container_created_message(&request.name),
                    ));
                }
            }
            Err(e) => {
                self.popup_message = Some(PopupMessage::Error(e.to_string()));
            }
        }
    }

    fn take_pending_image_pull(&mut self) {
        self.pending_image_pull.take();
        self.image_presenter.clear_pull_progress();
        self.image_presenter.set_loading(false);
    }

    fn selected_exec_target(&self) -> Option<(String, ExecRefreshTarget)> {
        match self.screen {
            Screen::ContainerList => self
                .container_presenter
                .selected_container()
                .map(|container| (container.id.clone(), ExecRefreshTarget::Containers)),
            Screen::StackContainers => self
                .stack_presenter
                .selected_stack_container()
                .map(|container| (container.id.clone(), ExecRefreshTarget::StackContainers)),
            _ => None,
        }
    }

    pub(crate) fn open_exec_shell_dialog(&mut self) {
        if let Some((container_id, refresh_target)) = self.selected_exec_target() {
            self.exec_shell_dialog = Some(ExecShellDialog::new(container_id, refresh_target));
        }
    }

    pub(crate) fn handle_exec_shell_action(&mut self, action: AppAction) {
        let Some(dialog) = &mut self.exec_shell_dialog else {
            return;
        };

        match action {
            AppAction::NavigateUp => dialog.navigate_up(),
            AppAction::NavigateDown => dialog.navigate_down(),
            AppAction::Select => {
                let request = dialog.build_request();
                self.exec_shell_dialog = None;
                self.pending_exec = Some(request);
            }
            AppAction::Back | AppAction::Quit => {
                self.exec_shell_dialog = None;
            }
            _ => {}
        }
    }

    async fn execute_pending_exec(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        request: PendingExec,
    ) -> io::Result<()> {
        // Clear loading state before exec as per user requirement
        // Exec spawns external process, spinner would be misleading
        self.container_presenter.set_loading(false);
        self.stack_presenter.set_loading(false);

        let result = if let Some(terminal_emulator) = find_terminal_emulator() {
            self.run_external_exec_command(&terminal_emulator, &request)
        } else {
            self.run_exec_command(terminal, &request)
        }?;

        match request.refresh_target {
            ExecRefreshTarget::Containers => self.load_containers().await,
            ExecRefreshTarget::StackContainers => self.refresh_stack_containers().await,
        }

        match result {
            ExecCommandResult::Spawned => {}
            ExecCommandResult::Status(status) if status.success() => {}
            ExecCommandResult::Status(status) => {
                self.popup_message = Some(PopupMessage::Error(format!(
                    "`docker exec` exited with status {status}"
                )));
            }
            ExecCommandResult::SpawnError(e) => {
                self.popup_message = Some(PopupMessage::Error(format_exec_error(&e)));
            }
        }

        Ok(())
    }

    fn run_external_exec_command(
        &mut self,
        terminal_emulator: &str,
        request: &PendingExec,
    ) -> io::Result<ExecCommandResult> {
        build_external_exec_command(&self.exec_command_config, request, terminal_emulator)
            .spawn()
            .map(|_| ExecCommandResult::Spawned)
    }

    fn run_exec_command(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        request: &PendingExec,
    ) -> io::Result<ExecCommandResult> {
        self.suspend_tui(terminal)?;
        let exec_result = build_exec_command(&self.exec_command_config, request).status();
        let resume_result = self.resume_tui(terminal);
        finalize_exec_command_result(exec_result, resume_result)
    }
    fn suspend_tui(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()
    }

    fn resume_tui(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> io::Result<()> {
        enable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            EnterAlternateScreen,
            EnableMouseCapture
        )?;
        terminal.hide_cursor()?;
        terminal.clear()
    }
}

pub(crate) fn format_bytes(bytes: u64) -> String {
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
    use crossterm::event::KeyCode;

    struct CwdGuard {
        previous: std::path::PathBuf,
    }

    impl CwdGuard {
        fn new(dir: &std::path::Path) -> Self {
            let previous = std::env::current_dir().unwrap();
            std::env::set_current_dir(dir).unwrap();
            CwdGuard { previous }
        }
    }

    impl Drop for CwdGuard {
        fn drop(&mut self) {
            std::env::set_current_dir(&self.previous).unwrap();
        }
    }

    use super::{
        finalize_exec_command_result, finalize_run_result, format_bytes, App, ExecCommandConfig,
        ExecCommandResult, ExecRefreshTarget, Screen,
    };
    use crate::application::container::traits::MockContainerRepository;
    use crate::application::container::{
        ContainerDto, ContainerRuntimeStatsDto, ContainerService, ContainerStatsEvent,
        ContainerStatsSubscription, ContainerStatsUpdate,
    };
    use crate::application::image::traits::MockImageRepository;
    use crate::application::image::{
        ImageDto, ImagePullOutcome, ImagePullSubscription, ImageService,
    };
    use crate::application::network::traits::MockNetworkRepository;
    use crate::application::network::{NetworkContainerDto, NetworkDto, NetworkService};
    use crate::application::shared::PruneResultDto;
    use crate::application::stack::traits::MockStackRepository;
    use crate::application::stack::{StackContainerDto, StackDto, StackService};
    use crate::application::volume::traits::MockVolumeRepository;
    use crate::application::volume::{VolumeDto, VolumeService};
    use crate::domain::container::ContainerState;
    use crate::domain::image::{Image, ImageId, ImageSize};
    use crate::domain::stack::{Stack, StackContainerState, StackName};
    use crate::presentation::tui::common::PopupMessage;
    use crate::presentation::tui::common::{resources, AppAction};
    use crate::presentation::tui::confirm::{confirm_message, ConfirmAction};
    use crate::presentation::tui::container::ContainerActions;
    use crate::presentation::tui::exec::{
        build_exec_command, ExecShell, ExecShellDialog, PendingExec,
    };
    use crate::presentation::tui::image::ImageActions;
    use crate::presentation::tui::network::NetworkActions;
    use crate::presentation::tui::stack::StackActions;
    use crate::presentation::tui::volume::VolumeActions;
    use chrono::Utc;
    use ratatui::{backend::TestBackend, buffer::Buffer, Terminal};
    use std::os::unix::process::ExitStatusExt;
    use std::{io, sync::Arc};

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
            runtime_stats: Some(ContainerRuntimeStatsDto {
                cpu_percent: 12.5,
                memory_usage: crate::domain::shared::ByteSize::new(512 * 1024 * 1024),
                memory_limit: crate::domain::shared::ByteSize::new(1024 * 1024 * 1024),
                memory_percent: 50.0,
                network_rx: crate::domain::shared::ByteSize::new(2_048),
                network_tx: crate::domain::shared::ByteSize::new(1_024),
            }),
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
            linked_containers: if in_use {
                vec!["/web".to_string(), "/worker".to_string()]
            } else {
                Vec::new()
            },
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

    fn make_network_dto(builtin: bool, containers: usize) -> NetworkDto {
        NetworkDto {
            id: "net-1".to_string(),
            name: if builtin {
                "bridge".to_string()
            } else {
                "app-net".to_string()
            },
            driver: "bridge".to_string(),
            scope: "local".to_string(),
            internal: false,
            attachable: false,
            subnet: Some("172.20.0.0/16".to_string()),
            gateway: Some("172.20.0.1".to_string()),
            created: "2024-01-01".to_string(),
            builtin,
            containers: (0..containers)
                .map(|index| NetworkContainerDto {
                    name: format!("container-{index}"),
                    ip: format!("172.20.0.{index}/16"),
                })
                .collect(),
            can_delete: !builtin && containers == 0,
            container_count: containers,
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

    fn make_stack_container_with_image(
        image: &str,
        can_start: bool,
        can_stop: bool,
    ) -> StackContainerDto {
        StackContainerDto {
            image: image.to_string(),
            ..make_stack_container(can_start, can_stop)
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
        network_repo: MockNetworkRepository,
    ) -> App {
        let container_actions =
            ContainerActions::new(ContainerService::new(Arc::new(container_repo)));
        let volume_actions = VolumeActions::new(VolumeService::new(Arc::new(volume_repo)));
        let image_actions = ImageActions::new(ImageService::new(Arc::new(image_repo)));
        let stack_actions = StackActions::new(StackService::new(Arc::new(stack_repo)));
        let network_actions = NetworkActions::new(NetworkService::new(Arc::new(network_repo)));
        App::new(
            container_actions,
            volume_actions,
            image_actions,
            stack_actions,
            network_actions,
            ExecCommandConfig::new("unix:///var/run/docker.sock"),
        )
    }

    fn expect_empty_stats_subscription(mock: &mut MockContainerRepository, times: usize) {
        mock.expect_subscribe_stats()
            .times(times)
            .returning(|_| Ok(ContainerStatsSubscription::empty()));
    }

    fn empty_app() -> App {
        build_app(
            MockContainerRepository::new(),
            MockVolumeRepository::new(),
            MockImageRepository::new(),
            MockStackRepository::new(),
            MockNetworkRepository::new(),
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
        assert!(rendered.contains(resources::APP_TITLE));

        assert_eq!(
            confirm_message(ConfirmAction::ContainerDelete(false)),
            resources::DELETE_CONTAINER_MESSAGE
        );
        assert_eq!(
            confirm_message(ConfirmAction::ContainerDelete(true)),
            resources::FORCE_DELETE_CONTAINER_MESSAGE
        );
        assert_eq!(
            confirm_message(ConfirmAction::StackContainersRemoveAll),
            resources::REMOVE_ALL_STACK_CONTAINERS_MESSAGE
        );

        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(2 * 1024), "2.0 KB");
        assert_eq!(format_bytes(3 * 1024 * 1024), "3.0 MB");
        assert_eq!(format_bytes(4 * 1024 * 1024 * 1024), "4.0 GB");
    }

    #[test]
    fn test_finalize_run_result_returns_cleanup_error_after_success() {
        let err =
            finalize_run_result(Ok(()), Some(io::Error::other("show_cursor failed"))).unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::Other);
        assert_eq!(err.to_string(), "show_cursor failed");
    }

    #[test]
    fn test_finalize_run_result_preserves_run_error_and_attaches_cleanup_error() {
        let err = finalize_run_result(
            Err(io::Error::other("event loop failed")),
            Some(io::Error::other("disable_raw_mode failed")),
        )
        .unwrap_err();

        assert_eq!(err.kind(), io::ErrorKind::Other);
        assert_eq!(
            err.to_string(),
            "event loop failed; cleanup also failed: disable_raw_mode failed"
        );
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
                lines: vec![crate::application::container::LogLineDto {
                    timestamp: None,
                    message: "line1".to_string(),
                }],
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
        app.volume_presenter
            .set_details(make_volume_dto(true, false));
        app.screen = Screen::VolumeDetails;
        assert!(render_app(&mut app).contains("Volume Details"));
        app.screen = Screen::ImageList;
        assert!(render_app(&mut app).contains("Images"));
        app.screen = Screen::ImageDetails;
        assert!(render_app(&mut app).contains("Image Details"));
        app.screen = Screen::StackList;
        assert!(render_app(&mut app).contains("Stacks"));
        app.screen = Screen::StackContainers;
        assert!(render_app(&mut app).contains("compose-app"));

        app.confirm_dialog = Some((ConfirmAction::VolumeDelete, true));
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

        app.popup_message = None;
        app.exec_shell_dialog = Some(ExecShellDialog::new(
            container.id.clone(),
            ExecRefreshTarget::Containers,
        ));
        let rendered = render_app(&mut app);
        assert!(rendered.contains(resources::EXEC_SHELL_DIALOG_TITLE.trim()));
        assert!(rendered.contains("sh"));
        assert!(rendered.contains("bash"));
    }

    #[test]
    fn test_render_container_list_applies_sort_after_toggle() {
        let mut app = empty_app();
        let mut zeta = make_container_dto();
        zeta.name = "zeta".to_string();
        let mut alpha = make_container_dto();
        alpha.name = "alpha".to_string();
        app.container_presenter.set_containers(vec![zeta, alpha]);
        app.screen = Screen::ContainerList;

        app.container_presenter.toggle_sort();
        let rendered = render_app(&mut app);
        let alpha_pos = rendered.find("alpha").expect("alpha rendered");
        let zeta_pos = rendered.find("zeta").expect("zeta rendered");
        assert!(
            alpha_pos < zeta_pos,
            "container list should render name-ascending after first toggle"
        );

        app.container_presenter.toggle_sort();
        let rendered = render_app(&mut app);
        let alpha_pos = rendered.find("alpha").expect("alpha rendered");
        let zeta_pos = rendered.find("zeta").expect("zeta rendered");
        assert!(
            alpha_pos > zeta_pos,
            "container list should render name-descending after second toggle"
        );
    }

    #[test]
    fn test_render_volume_list_applies_sort_after_toggle() {
        let mut app = empty_app();
        let mut zeta = make_volume_dto(false, true);
        zeta.name = "zeta".to_string();
        let mut alpha = make_volume_dto(false, true);
        alpha.name = "alpha".to_string();
        app.volume_presenter.set_volumes(vec![zeta, alpha]);
        app.screen = Screen::VolumeList;

        app.volume_presenter.toggle_sort();
        let rendered = render_app(&mut app);
        let alpha_pos = rendered.find("alpha").expect("alpha rendered");
        let zeta_pos = rendered.find("zeta").expect("zeta rendered");
        assert!(
            alpha_pos < zeta_pos,
            "volume list should render name-ascending after first toggle"
        );

        app.volume_presenter.toggle_sort();
        let rendered = render_app(&mut app);
        let alpha_pos = rendered.find("alpha").expect("alpha rendered");
        let zeta_pos = rendered.find("zeta").expect("zeta rendered");
        assert!(
            alpha_pos > zeta_pos,
            "volume list should render name-descending after second toggle"
        );
    }

    #[test]
    fn test_breadcrumb_on_every_non_menu_screen() {
        let mut app = empty_app();
        let container = make_container_dto();
        let volume = make_volume_dto(false, true);
        let image = make_image_dto(true);
        let stack = make_stack_dto(vec![make_stack_container(false, true)]);

        app.container_presenter.set_details(container.clone());
        app.container_presenter
            .set_logs(crate::application::container::ContainerLogsDto {
                container_id: container.id.clone(),
                container_name: container.name.clone(),
                lines: vec![],
            });
        app.volume_presenter.set_details(volume);
        app.image_presenter.set_details(image);
        app.stack_presenter.set_stacks(vec![stack]);
        app.stack_presenter
            .set_stack_containers(vec![make_stack_container(false, true)]);

        app.screen = Screen::ContainerList;
        assert!(render_app(&mut app).contains("Rusty - docker manager > Containers"));
        app.screen = Screen::ContainerLogs;
        assert!(render_app(&mut app).contains("Rusty - docker manager > Containers > web logs"));
        app.screen = Screen::ContainerDetails;
        assert!(render_app(&mut app).contains("Rusty - docker manager > Containers > web details"));
        app.screen = Screen::VolumeList;
        assert!(render_app(&mut app).contains("Rusty - docker manager > Volumes"));
        app.screen = Screen::VolumeDetails;
        assert!(render_app(&mut app).contains("Rusty - docker manager > Volumes > db-data details"));
        app.screen = Screen::ImageList;
        assert!(render_app(&mut app).contains("Rusty - docker manager > Images"));
        app.screen = Screen::ImageDetails;
        assert!(
            render_app(&mut app).contains("Rusty - docker manager > Images > nginx:latest details")
        );
        app.screen = Screen::NetworkList;
        assert!(render_app(&mut app).contains("Rusty - docker manager > Networks"));
        app.network_presenter.set_details(make_network_dto(true, 2));
        app.screen = Screen::NetworkDetails;
        assert!(render_app(&mut app).contains("Rusty - docker manager > Networks > bridge details"));
        app.screen = Screen::StackList;
        assert!(render_app(&mut app).contains("Rusty - docker manager > Stacks"));
        app.screen = Screen::StackContainers;
        assert!(render_app(&mut app)
            .contains("Rusty - docker manager > Stacks > compose-app containers"));
    }

    #[test]
    fn test_main_menu_renders_title_without_breadcrumb_path() {
        let mut app = empty_app();
        let rendered = render_app(&mut app);
        assert!(rendered.contains(resources::APP_TITLE));
        assert!(!rendered.contains(&format!(
            "{}{}",
            resources::APP_TITLE,
            resources::BREADCRUMB_SEPARATOR
        )));
    }

    #[tokio::test]
    async fn test_handle_action_clears_popup_and_handles_confirm_dialog() {
        let mut app = empty_app();
        app.popup_message = Some(PopupMessage::Error("boom".to_string()));
        app.handle_action(AppAction::Quit).await;
        assert!(app.popup_message.is_none());
        assert!(!app.should_quit);

        app.confirm_dialog = Some((ConfirmAction::VolumeDelete, true));
        app.handle_action(AppAction::NavigateDown).await;
        assert_eq!(
            app.confirm_dialog,
            Some((ConfirmAction::VolumeDelete, false))
        );

        app.handle_action(AppAction::Back).await;
        assert!(app.confirm_dialog.is_none());

        app.confirm_dialog = Some((ConfirmAction::VolumeDelete, false));
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
        expect_empty_stats_subscription(&mut container_repo, 1);
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
        let mut network_repo = MockNetworkRepository::new();
        network_repo.expect_get_all().returning(|| {
            Ok(vec![crate::domain::network::Network::new(
                crate::domain::network::NetworkId::new("net1").unwrap(),
                "app-net",
                "bridge",
                crate::domain::network::NetworkScope::Local,
            )])
        });

        let mut app = build_app(
            container_repo,
            volume_repo,
            image_repo,
            stack_repo,
            network_repo,
        );

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

        app.screen = Screen::MainMenu;
        app.menu_state.select(Some(4));
        app.handle_main_menu_action(AppAction::Select).await;
        assert_eq!(app.screen, Screen::NetworkList);
        assert_eq!(app.network_presenter.networks.len(), 1);
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
        expect_empty_stats_subscription(&mut container_repo, 4);

        let mut app = build_app(
            container_repo,
            MockVolumeRepository::new(),
            MockImageRepository::new(),
            MockStackRepository::new(),
            MockNetworkRepository::new(),
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
            Some((ConfirmAction::ContainerDelete(true), true))
        );

        app.confirm_dialog = None;
        app.handle_container_list_action(AppAction::Prune).await;
        assert_eq!(
            app.confirm_dialog,
            Some((ConfirmAction::ContainersPrune, true))
        );

        app.confirm_dialog = None;
        app.handle_container_list_action(AppAction::Exec).await;
        assert!(app.exec_shell_dialog.is_some());

        app.handle_container_list_action(AppAction::Back).await;
        assert_eq!(app.screen, Screen::MainMenu);
        assert!(app.container_stats_subscription.is_none());
        assert!(app
            .container_presenter
            .containers
            .iter()
            .all(|container| container.runtime_stats.is_none()));
    }

    #[tokio::test]
    async fn test_handle_container_log_and_details_actions() {
        let mut app = empty_app();
        app.screen = Screen::ContainerLogs;
        app.container_presenter
            .set_logs(crate::application::container::ContainerLogsDto {
                container_id: "abc123".to_string(),
                container_name: "web".to_string(),
                lines: vec![crate::application::container::LogLineDto {
                    timestamp: None,
                    message: "line1".to_string(),
                }],
            });
        app.handle_container_logs_action(AppAction::ScrollDown)
            .await;
        app.handle_container_logs_action(AppAction::ScrollUp).await;
        app.handle_container_logs_action(AppAction::Back).await;
        assert_eq!(app.screen, Screen::ContainerList);
        assert!(app.container_presenter.logs.is_none());
    }

    #[tokio::test]
    async fn test_expanded_log_line_overlay_open_and_close() {
        let mut app = empty_app();
        app.screen = Screen::ContainerLogs;
        app.container_presenter
            .set_logs(crate::application::container::ContainerLogsDto {
                container_id: "abc123".to_string(),
                container_name: "web".to_string(),
                lines: vec![
                    crate::application::container::LogLineDto {
                        timestamp: None,
                        message: "line1".to_string(),
                    },
                    crate::application::container::LogLineDto {
                        timestamp: None,
                        message: "line2".to_string(),
                    },
                ],
            });
        app.handle_action(AppAction::Select).await;
        assert_eq!(app.expanded_log_line, Some(0));
        // While the overlay is open, screen actions are intercepted
        app.handle_action(AppAction::Back).await;
        assert_eq!(app.expanded_log_line, None);
        assert_eq!(app.screen, Screen::ContainerLogs);
        // Selection still intact after closing
        app.handle_container_logs_action(AppAction::NavigateDown)
            .await;
        assert_eq!(app.container_presenter.selected_logs_line(), Some(1));
        app.handle_action(AppAction::Select).await;
        assert_eq!(app.expanded_log_line, Some(1));
        app.handle_action(AppAction::Select).await;
        assert_eq!(app.expanded_log_line, None);
        app.expanded_log_line = Some(0);
        app.handle_action(AppAction::Quit).await;
        assert_eq!(app.expanded_log_line, None);
        assert!(!app.should_quit);

        app.screen = Screen::ContainerDetails;
        app.container_presenter.set_details(make_container_dto());
        app.handle_details_action(AppAction::Quit);
        assert_eq!(app.screen, Screen::ContainerList);

        app.screen = Screen::ImageDetails;
        app.image_presenter.set_details(make_image_dto(true));
        app.handle_image_details_action(AppAction::Back).await;
        assert_eq!(app.screen, Screen::ImageList);

        app.screen = Screen::VolumeDetails;
        app.volume_presenter
            .set_details(make_volume_dto(true, false));
        app.handle_details_action(AppAction::Back);
        assert_eq!(app.screen, Screen::VolumeList);
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
            MockNetworkRepository::new(),
        );

        app.screen = Screen::VolumeList;
        app.volume_presenter
            .set_volumes(vec![make_volume_dto(true, false)]);
        app.handle_volume_list_action(AppAction::ViewDetails).await;
        assert_eq!(app.screen, Screen::VolumeDetails);
        assert_eq!(
            app.volume_presenter
                .selected_volume
                .as_ref()
                .map(|volume| volume.name.as_str()),
            Some("db-data")
        );

        app.screen = Screen::VolumeList;
        app.volume_presenter.clear_details();
        app.handle_volume_list_action(AppAction::Delete).await;
        assert!(matches!(app.popup_message, Some(PopupMessage::Error(_))));
        assert!(app
            .popup_message
            .as_ref()
            .unwrap()
            .as_str()
            .contains(resources::VOLUME_IN_USE_DELETE_MESSAGE));
        app.popup_message = None;
        app.volume_presenter
            .set_volumes(vec![make_volume_dto(false, true)]);
        app.handle_volume_list_action(AppAction::Delete).await;
        assert_eq!(
            app.confirm_dialog,
            Some((ConfirmAction::VolumeDelete, true))
        );
        app.confirm_dialog = None;
        app.handle_volume_list_action(AppAction::Prune).await;
        assert_eq!(
            app.confirm_dialog,
            Some((ConfirmAction::VolumesPrune, true))
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
            Some((ConfirmAction::ImageDelete(true), true))
        );
        app.confirm_dialog = None;
        app.handle_image_list_action(AppAction::Prune).await;
        assert_eq!(app.confirm_dialog, Some((ConfirmAction::ImagesPrune, true)));
        app.confirm_dialog = None;
        app.handle_image_list_action(AppAction::ActivateFilter)
            .await;
        assert!(app.image_presenter.is_filter_active());
        app.handle_image_list_action(AppAction::Refresh).await;
    }

    #[tokio::test]
    async fn test_network_list_and_details_actions() {
        let mut network_repo = MockNetworkRepository::new();
        network_repo.expect_get_all().returning(|| {
            Ok(vec![crate::domain::network::Network::new(
                crate::domain::network::NetworkId::new("net1").unwrap(),
                "app-net",
                "bridge",
                crate::domain::network::NetworkScope::Local,
            )
            .with_containers(vec![
                crate::domain::network::NetworkContainer::new("container-0", "172.20.0.0/16"),
            ])])
        });

        let mut app = build_app(
            MockContainerRepository::new(),
            MockVolumeRepository::new(),
            MockImageRepository::new(),
            MockStackRepository::new(),
            network_repo,
        );

        // Menu entry loads the network list
        app.menu_state.select(Some(4));
        app.handle_main_menu_action(AppAction::Select).await;
        assert_eq!(app.screen, Screen::NetworkList);
        assert_eq!(app.network_presenter.networks.len(), 1);

        app.handle_network_list_action(AppAction::NavigateDown)
            .await;
        app.handle_network_list_action(AppAction::NavigateUp).await;
        app.handle_network_list_action(AppAction::ActivateFilter)
            .await;
        assert!(app.network_presenter.is_filter_active());
        assert!(app.handle_filter_key(crossterm::event::KeyCode::Esc));

        // Enter opens details, container navigation moves the selection
        app.handle_network_list_action(AppAction::Select).await;
        assert_eq!(app.screen, Screen::NetworkDetails);
        assert_eq!(
            app.network_presenter.container_selection.selected(),
            Some(0)
        );

        // 'd' on the details screen disconnects the selected container
        app.handle_network_details_action(AppAction::DisconnectContainer)
            .await;
        assert_eq!(
            app.confirm_dialog,
            Some((ConfirmAction::NetworkDisconnect, true))
        );
        app.confirm_dialog = None;

        // Back returns to the list and clears details
        app.handle_network_details_action(AppAction::Back).await;
        assert_eq!(app.screen, Screen::NetworkList);
        assert!(app.network_presenter.selected_network.is_none());

        // Delete on a network with connected containers is refused
        app.handle_network_list_action(AppAction::Delete).await;
        assert!(matches!(app.popup_message, Some(PopupMessage::Error(_))));
        assert!(app
            .popup_message
            .as_ref()
            .unwrap()
            .as_str()
            .contains(resources::NETWORK_IN_USE_DELETE_MESSAGE));
        app.popup_message = None;

        // Delete on a builtin network is refused
        app.network_presenter
            .set_networks(vec![make_network_dto(true, 0)]);
        app.handle_network_list_action(AppAction::Delete).await;
        assert!(app
            .popup_message
            .as_ref()
            .unwrap()
            .as_str()
            .contains(resources::NETWORK_BUILTIN_DELETE_MESSAGE));
        app.popup_message = None;

        // Delete on a deletable network asks for confirmation
        app.network_presenter
            .set_networks(vec![make_network_dto(false, 0)]);
        app.handle_network_list_action(AppAction::Delete).await;
        assert_eq!(
            app.confirm_dialog,
            Some((ConfirmAction::NetworkDelete, true))
        );
        app.confirm_dialog = None;

        // Prune asks for confirmation
        app.handle_network_list_action(AppAction::Prune).await;
        assert_eq!(
            app.confirm_dialog,
            Some((ConfirmAction::NetworksPrune, true))
        );
        app.confirm_dialog = None;
        app.handle_network_list_action(AppAction::Refresh).await;
        app.handle_network_list_action(AppAction::Back).await;
        assert_eq!(app.screen, Screen::MainMenu);
    }

    #[tokio::test]
    async fn test_execute_confirm_action_network_paths() {
        let mut network_repo = MockNetworkRepository::new();
        network_repo.expect_get_all().returning(|| Ok(vec![]));
        network_repo.expect_delete().returning(|_| Ok(()));
        network_repo.expect_prune().returning(|| {
            Ok(PruneResultDto {
                deleted_count: 2,
                space_freed: 0,
            })
        });
        network_repo
            .expect_disconnect()
            .withf(|network, container, force| {
                network == "app-net" && container == "container-0" && !force
            })
            .returning(|_, _, _| Ok(()));

        let mut app = build_app(
            MockContainerRepository::new(),
            MockVolumeRepository::new(),
            MockImageRepository::new(),
            MockStackRepository::new(),
            network_repo,
        );

        // Delete reloads the list after success
        app.screen = Screen::NetworkList;
        app.network_presenter
            .set_networks(vec![make_network_dto(false, 0)]);
        app.execute_confirm_action(ConfirmAction::NetworkDelete)
            .await;
        assert!(!app.network_presenter.is_loading());

        // Disconnect refreshes the details view of the same network
        app.screen = Screen::NetworkDetails;
        app.network_presenter
            .set_details(make_network_dto(false, 1));
        app.execute_confirm_action(ConfirmAction::NetworkDisconnect)
            .await;
        assert!(!app.network_presenter.is_loading());

        // Prune reports the deleted count
        app.screen = Screen::NetworkList;
        app.execute_confirm_action(ConfirmAction::NetworksPrune)
            .await;
        assert!(matches!(app.popup_message, Some(PopupMessage::Info(_))));
        assert!(app
            .popup_message
            .as_ref()
            .unwrap()
            .as_str()
            .contains("Pruned 2 network(s)"));
    }

    #[test]
    fn test_selected_exec_target_uses_current_screen_selection() {
        let mut app = empty_app();
        app.screen = Screen::ContainerList;
        app.container_presenter
            .set_containers(vec![make_container_dto()]);

        assert_eq!(
            app.selected_exec_target(),
            Some(("abc123".to_string(), ExecRefreshTarget::Containers))
        );

        app.screen = Screen::StackContainers;
        app.stack_presenter
            .set_stack_containers(vec![make_stack_container(false, true)]);

        assert_eq!(
            app.selected_exec_target(),
            Some(("stack-1".to_string(), ExecRefreshTarget::StackContainers))
        );
    }

    #[tokio::test]
    async fn test_drain_container_stats_applies_updates_and_surfaces_errors() {
        let mut app = empty_app();
        app.container_presenter
            .set_containers(vec![make_container_dto()]);
        let (sender, receiver) = tokio::sync::mpsc::channel(4);
        sender
            .send(ContainerStatsEvent::Update(ContainerStatsUpdate {
                container_id: "abc123".to_string(),
                stats: ContainerRuntimeStatsDto {
                    cpu_percent: 80.0,
                    memory_usage: crate::domain::shared::ByteSize::new(256),
                    memory_limit: crate::domain::shared::ByteSize::new(512),
                    memory_percent: 50.0,
                    network_rx: crate::domain::shared::ByteSize::new(64),
                    network_tx: crate::domain::shared::ByteSize::new(32),
                },
            }))
            .await
            .unwrap();
        sender
            .send(ContainerStatsEvent::Error {
                container_id: "abc123".to_string(),
                message: "boom".to_string(),
            })
            .await
            .unwrap();
        drop(sender);

        app.container_stats_subscription =
            Some(ContainerStatsSubscription::new(receiver, Vec::new()));
        app.drain_container_stats();

        assert_eq!(app.container_presenter.containers[0].cpu_display(), "80.0%");
        assert!(matches!(app.popup_message, Some(PopupMessage::Error(_))));
        assert!(app
            .popup_message
            .as_ref()
            .unwrap()
            .as_str()
            .contains("Stats stream failed for abc123"));
    }

    #[tokio::test]
    async fn test_drain_container_stats_clears_dead_subscription_on_disconnect() {
        let mut app = empty_app();
        app.container_presenter
            .set_containers(vec![make_container_dto()]);
        let (sender, receiver) = tokio::sync::mpsc::channel(1);
        drop(sender);

        app.container_stats_subscription =
            Some(ContainerStatsSubscription::new(receiver, Vec::new()));
        app.drain_container_stats();

        assert!(app.container_stats_subscription.is_none());
        assert!(app
            .container_presenter
            .containers
            .iter()
            .all(|container| container.runtime_stats.is_none()));
    }

    #[tokio::test]
    async fn test_handle_exec_shell_action_navigates_selects_and_cancels() {
        let mut app = empty_app();
        app.exec_shell_dialog = Some(ExecShellDialog::new(
            "abc123".to_string(),
            ExecRefreshTarget::Containers,
        ));

        app.handle_action(AppAction::NavigateDown).await;
        app.handle_action(AppAction::Select).await;

        assert!(app.exec_shell_dialog.is_none());
        assert_eq!(
            app.pending_exec,
            Some(PendingExec {
                container_id: "abc123".to_string(),
                shell: ExecShell::Bash,
                refresh_target: ExecRefreshTarget::Containers,
            })
        );

        app.exec_shell_dialog = Some(ExecShellDialog::new(
            "abc123".to_string(),
            ExecRefreshTarget::Containers,
        ));
        app.pending_exec = None;

        app.handle_action(AppAction::Quit).await;
        assert!(app.exec_shell_dialog.is_none());
        assert!(app.pending_exec.is_none());
        assert!(!app.should_quit);
    }

    #[test]
    fn test_build_exec_command_uses_configured_host_and_shell() {
        let config = ExecCommandConfig::new("unix:///var/run/docker.sock");
        let request = PendingExec {
            container_id: "abc123".to_string(),
            shell: ExecShell::Bash,
            refresh_target: ExecRefreshTarget::Containers,
        };
        let command = build_exec_command(&config, &request);
        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(command.get_program().to_string_lossy(), "docker");
        assert_eq!(
            args,
            vec![
                "--host",
                "unix:///var/run/docker.sock",
                "exec",
                "-it",
                "abc123",
                "bash",
            ]
        );
    }

    #[test]
    fn test_finalize_exec_command_result_treats_spawn_error_as_non_fatal() {
        let result = finalize_exec_command_result(
            Err(io::Error::new(io::ErrorKind::NotFound, "missing")),
            Ok(()),
        )
        .unwrap();

        assert!(matches!(result, ExecCommandResult::SpawnError(_)));
    }

    #[test]
    fn test_finalize_exec_command_result_treats_resume_error_as_fatal() {
        let result = finalize_exec_command_result(
            Ok(std::process::ExitStatus::from_raw(0)),
            Err(io::Error::other("resume failed")),
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "resume failed");
    }

    #[test]
    fn test_finalize_exec_command_result_prefers_resume_failure_when_both_fail() {
        let result = finalize_exec_command_result(
            Err(io::Error::new(io::ErrorKind::NotFound, "docker missing")),
            Err(io::Error::other("resume failed")),
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "resume failed; docker exec also failed: docker missing"
        );
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

        let mut app = build_app(
            container_repo,
            volume_repo,
            image_repo,
            stack_repo,
            MockNetworkRepository::new(),
        );
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

        app.execute_confirm_action(ConfirmAction::ContainerDelete(true))
            .await;
        app.execute_confirm_action(ConfirmAction::ContainersPrune)
            .await;
        assert!(matches!(app.popup_message, Some(PopupMessage::Info(_))));
        assert!(app
            .popup_message
            .as_ref()
            .unwrap()
            .as_str()
            .contains("Pruned 2 container(s)"));

        app.execute_confirm_action(ConfirmAction::VolumeDelete)
            .await;
        app.execute_confirm_action(ConfirmAction::VolumesPrune)
            .await;
        assert!(matches!(app.popup_message, Some(PopupMessage::Info(_))));
        assert!(app
            .popup_message
            .as_ref()
            .unwrap()
            .as_str()
            .contains("Pruned 1 volume(s)"));

        app.execute_confirm_action(ConfirmAction::ImageDelete(true))
            .await;
        app.execute_confirm_action(ConfirmAction::ImagesPrune).await;
        assert!(matches!(app.popup_message, Some(PopupMessage::Info(_))));
        assert!(app
            .popup_message
            .as_ref()
            .unwrap()
            .as_str()
            .contains("Pruned 3 image(s)"));

        app.screen = Screen::StackContainers;
        app.execute_confirm_action(ConfirmAction::ContainerDelete(true))
            .await;
        app.execute_confirm_action(ConfirmAction::StackContainersRemoveAll)
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

        let mut app = build_app(
            container_repo,
            volume_repo,
            image_repo,
            stack_repo,
            MockNetworkRepository::new(),
        );

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
        assert!(app
            .popup_message
            .as_ref()
            .unwrap()
            .as_str()
            .contains(resources::CONTAINER_NOT_FOUND_MESSAGE));
    }

    #[tokio::test]
    async fn test_loading_state_set_during_async_operations() {
        let mut container_repo = MockContainerRepository::new();
        container_repo.expect_get_all().returning(|| Ok(vec![]));
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
            .returning(|| Ok(vec![make_stack(0)]));

        let mut app = build_app(
            container_repo,
            volume_repo,
            image_repo,
            stack_repo,
            MockNetworkRepository::new(),
        );

        // Initially not loading
        assert!(!app.container_presenter.is_loading());
        assert!(!app.volume_presenter.is_loading());
        assert!(!app.image_presenter.is_loading());
        assert!(!app.stack_presenter.is_loading());

        // Load operations should set and clear loading state
        app.load_containers().await;
        assert!(!app.container_presenter.is_loading());

        app.load_volumes().await;
        assert!(!app.volume_presenter.is_loading());

        app.load_images().await;
        assert!(!app.image_presenter.is_loading());

        app.load_stacks().await;
        assert!(!app.stack_presenter.is_loading());
    }

    #[tokio::test]
    async fn test_image_details_pull_images_shows_progress_and_result() {
        let mut image_repo = MockImageRepository::new();
        image_repo
            .expect_get_all()
            .returning(|| Ok(vec![make_image()]));
        image_repo.expect_local_digest().returning(|_| Ok(None));
        image_repo.expect_registry_digest().returning(|_| Ok(None));
        image_repo
            .expect_pull_images()
            .withf(|image_refs| *image_refs == ["nginx:latest".to_string()])
            .returning(|_| {
                Ok(ImagePullSubscription::completed(Ok(ImagePullOutcome {
                    pulled: vec!["nginx:latest".to_string()],
                    up_to_date: vec![],
                })))
            });
        let mut app = build_app(
            MockContainerRepository::new(),
            MockVolumeRepository::new(),
            image_repo,
            MockStackRepository::new(),
            MockNetworkRepository::new(),
        );
        app.screen = Screen::ImageList;
        app.load_images().await;
        app.handle_image_list_action(AppAction::ViewDetails).await;
        assert_eq!(app.screen, Screen::ImageDetails);
        app.handle_image_details_action(AppAction::PullImages).await;
        assert!(app.image_presenter.is_loading());
        app.drain_image_pull().await;
        assert!(matches!(app.popup_message, Some(PopupMessage::Info(_))));
        let message = app.popup_message.as_ref().unwrap().as_str();
        assert!(message.contains("pulled 1 image(s)"), "{message}");
        assert!(!app.image_presenter.is_loading());
        assert!(app.image_presenter.pull_progress.is_none());
        assert_eq!(app.screen, Screen::ImageDetails);
        let details = app.image_presenter.selected_image.as_ref().unwrap();
        assert_eq!(details.full_name, "nginx:latest");
    }

    #[tokio::test]
    async fn test_image_details_pull_images_up_to_date_reports_no_download() {
        let mut image_repo = MockImageRepository::new();
        image_repo
            .expect_get_all()
            .returning(|| Ok(vec![make_image()]));
        image_repo
            .expect_local_digest()
            .returning(|_| Ok(Some("sha256:aaa".to_string())));
        image_repo
            .expect_registry_digest()
            .returning(|_| Ok(Some("sha256:aaa".to_string())));
        let mut app = build_app(
            MockContainerRepository::new(),
            MockVolumeRepository::new(),
            image_repo,
            MockStackRepository::new(),
            MockNetworkRepository::new(),
        );
        app.screen = Screen::ImageList;
        app.load_images().await;
        app.handle_image_list_action(AppAction::ViewDetails).await;
        app.handle_image_details_action(AppAction::PullImages).await;
        app.drain_image_pull().await;
        let message = app.popup_message.as_ref().unwrap().as_str();
        assert!(message.contains("1 already up to date"), "{message}");
    }

    #[tokio::test]
    async fn test_image_details_pull_dangling_image_shows_error_without_network_call() {
        let mut image_repo = MockImageRepository::new();
        image_repo.expect_get_all().returning(|| {
            Ok(vec![Image::new(
                ImageId::new("sha256:abc").unwrap(),
                "<none>",
                "latest",
                ImageSize::new(1_000_000),
                Utc::now(),
            )])
        });
        let mut app = build_app(
            MockContainerRepository::new(),
            MockVolumeRepository::new(),
            image_repo,
            MockStackRepository::new(),
            MockNetworkRepository::new(),
        );
        app.screen = Screen::ImageList;
        app.load_images().await;
        app.handle_image_list_action(AppAction::ViewDetails).await;
        app.handle_image_details_action(AppAction::PullImages).await;
        assert!(matches!(app.popup_message, Some(PopupMessage::Error(_))));
        let message = app.popup_message.as_ref().unwrap().as_str();
        assert!(message.contains("dangling"), "{message}");
        assert!(!app.image_presenter.is_loading());
        assert!(app.pending_image_pull.is_none());
    }

    #[tokio::test]
    async fn test_toggle_pause_restart_and_stack_actions() {
        let mut container_repo = MockContainerRepository::new();
        container_repo.expect_start().returning(|_| Ok(()));
        container_repo.expect_stop().returning(|_| Ok(()));
        container_repo.expect_pause().returning(|_| Ok(()));
        container_repo.expect_unpause().returning(|_| Ok(()));
        container_repo.expect_restart().returning(|_| Ok(()));
        container_repo.expect_get_all().returning(|| Ok(vec![]));
        let mut stack_repo = MockStackRepository::new();
        stack_repo
            .expect_get_all()
            .times(4)
            .returning(|| Ok(vec![make_stack(1)]));
        stack_repo.expect_stop_all().times(2).returning(|_| Ok(()));
        stack_repo.expect_start_all().returning(|_| Ok(()));
        let mut image_repo = MockImageRepository::new();
        image_repo.expect_get_all().returning(|| Ok(vec![]));

        let mut app = build_app(
            container_repo,
            MockVolumeRepository::new(),
            image_repo,
            stack_repo,
            MockNetworkRepository::new(),
        );

        app.toggle_container(&make_stopped_container_dto()).await;
        let mut unpause_container = make_stopped_container_dto();
        unpause_container.can_unpause = true;
        app.pause_unpause_container(&make_container_dto()).await;
        app.pause_unpause_container(&unpause_container).await;
        app.restart_container(&make_container_dto()).await;
        app.restart_container(&make_stopped_container_dto()).await;

        let stack = make_stack_dto(vec![
            make_stack_container_with_image("nginx:latest", true, false),
            make_stack_container_with_image("postgres:16", false, true),
            make_stack_container_with_image("nginx:latest", false, true),
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

        app.popup_message = None;
        app.stack_presenter.set_stacks(vec![stack.clone()]);
        app.stack_presenter
            .set_stack_containers(stack.containers.clone());
        app.screen = Screen::StackContainers;
        app.handle_stack_containers_action(AppAction::StartStop)
            .await;
        app.handle_stack_containers_action(AppAction::Delete).await;
        assert_eq!(
            app.confirm_dialog,
            Some((ConfirmAction::ContainerDelete(true), true))
        );
        app.confirm_dialog = None;
        app.handle_stack_containers_action(AppAction::StopAll).await;
        app.handle_stack_containers_action(AppAction::StartAll)
            .await;
        app.handle_stack_containers_action(AppAction::RemoveAll)
            .await;
        assert_eq!(
            app.confirm_dialog,
            Some((ConfirmAction::StackContainersRemoveAll, true))
        );
    }

    #[tokio::test]
    async fn test_log_search_workflow_via_actions() {
        let mut app = empty_app();
        app.screen = Screen::ContainerLogs;
        app.container_presenter
            .set_logs(crate::application::container::ContainerLogsDto {
                container_id: "abc123".to_string(),
                container_name: "web".to_string(),
                lines: vec![
                    crate::application::container::LogLineDto {
                        timestamp: None,
                        message: "starting app".to_string(),
                    },
                    crate::application::container::LogLineDto {
                        timestamp: None,
                        message: "ERROR disk full".to_string(),
                    },
                    crate::application::container::LogLineDto {
                        timestamp: None,
                        message: "error retrying".to_string(),
                    },
                ],
            });

        // '/' on the logs screen activates the log search
        app.handle_action(AppAction::ActivateLogSearch).await;
        assert!(app.container_presenter.is_log_search_active());

        // Typed characters are consumed by the input handler
        assert!(app.handle_filter_key(KeyCode::Char('e')));
        assert!(app.handle_filter_key(KeyCode::Char('r')));
        assert!(app.handle_filter_key(KeyCode::Char('r')));
        assert!(app.handle_filter_key(KeyCode::Char('o')));
        assert!(app.handle_filter_key(KeyCode::Char('r')));
        assert_eq!(app.container_presenter.log_search_value(), "error");

        // Enter confirms and jumps to the first match
        app.handle_action(AppAction::Select).await;
        assert!(!app.container_presenter.is_log_search_active());
        assert_eq!(app.container_presenter.log_search_results, vec![1, 2]);
        assert_eq!(app.container_presenter.selected_logs_line(), Some(1));

        // n / N navigate between matches with wrap-around
        app.handle_action(AppAction::NextLogMatch).await;
        assert_eq!(app.container_presenter.selected_logs_line(), Some(2));
        app.handle_action(AppAction::NextLogMatch).await;
        assert_eq!(app.container_presenter.selected_logs_line(), Some(1));
        app.handle_action(AppAction::PreviousLogMatch).await;
        assert_eq!(app.container_presenter.selected_logs_line(), Some(2));

        // Esc during input closes without validating
        app.handle_action(AppAction::ActivateLogSearch).await;
        assert!(app.handle_filter_key(KeyCode::Char('x')));
        assert!(app.handle_filter_key(KeyCode::Esc));
        assert!(!app.container_presenter.is_log_search_active());
        assert_eq!(app.container_presenter.log_search_value(), "");
        assert!(app.container_presenter.log_search_results.is_empty());

        // Leaving the screen clears the search
        app.handle_action(AppAction::ActivateLogSearch).await;
        app.handle_action(AppAction::Back).await;
        assert!(app.container_presenter.log_search_results.is_empty());
        assert!(app.container_presenter.logs.is_none());
    }

    #[tokio::test]
    async fn test_export_logs_action_success_and_error_feedback() {
        let mut mock = MockContainerRepository::new();
        mock.expect_export_logs()
            .withf(|id| id == "abc123")
            .returning(|_| Ok("2026-10-26T10:30:00Z line1\n".to_string()));
        let mut app = build_app(
            mock,
            MockVolumeRepository::new(),
            MockImageRepository::new(),
            MockStackRepository::new(),
            MockNetworkRepository::new(),
        );
        app.screen = Screen::ContainerLogs;
        app.container_presenter
            .set_logs(crate::application::container::ContainerLogsDto {
                container_id: "abc123".to_string(),
                container_name: "web".to_string(),
                lines: vec![crate::application::container::LogLineDto {
                    timestamp: None,
                    message: "line1".to_string(),
                }],
            });

        // No logs loaded -> E does nothing
        app.container_presenter.clear_logs();
        app.handle_action(AppAction::ExportLogs).await;
        assert!(app.popup_message.is_none());

        // With logs -> export succeeds, info popup with file name
        let dir =
            std::env::temp_dir().join(format!("rusty_app_test_{}_{}", std::process::id(), line!()));
        std::fs::create_dir_all(&dir).unwrap();
        let _cwd_guard = CwdGuard::new(&dir);
        app.container_presenter
            .set_logs(crate::application::container::ContainerLogsDto {
                container_id: "abc123".to_string(),
                container_name: "web".to_string(),
                lines: vec![crate::application::container::LogLineDto {
                    timestamp: None,
                    message: "line1".to_string(),
                }],
            });
        app.handle_action(AppAction::ExportLogs).await;
        match &app.popup_message {
            Some(PopupMessage::Info(msg)) => {
                assert!(msg.starts_with("Logs exported to web_"));
                assert!(msg.ends_with(".log"));
            }
            other => panic!("expected info popup, got {other:?}"),
        }
        app.popup_message = None;
        drop(_cwd_guard);
        std::fs::remove_dir_all(&dir).ok();

        // Error case -> error popup
        let mut mock = MockContainerRepository::new();
        mock.expect_export_logs().returning(|_| {
            Err(crate::application::error::AppError::repository(
                "docker down",
            ))
        });
        let mut app = build_app(
            mock,
            MockVolumeRepository::new(),
            MockImageRepository::new(),
            MockStackRepository::new(),
            MockNetworkRepository::new(),
        );
        app.screen = Screen::ContainerLogs;
        app.container_presenter
            .set_logs(crate::application::container::ContainerLogsDto {
                container_id: "abc123".to_string(),
                container_name: "web".to_string(),
                lines: vec![crate::application::container::LogLineDto {
                    timestamp: None,
                    message: "line1".to_string(),
                }],
            });
        app.handle_action(AppAction::ExportLogs).await;
        match &app.popup_message {
            Some(PopupMessage::Error(msg)) => assert!(msg.contains("docker down")),
            other => panic!("expected error popup, got {other:?}"),
        }
    }

    fn fill_add_dialog(app: &mut App, values: &[&str]) {
        for (index, value) in values.iter().enumerate() {
            while app.add_dialog.as_ref().unwrap().state.focused_index() < index {
                app.add_dialog.as_mut().unwrap().state.focus_next();
            }
            for c in value.chars() {
                app.add_dialog
                    .as_mut()
                    .unwrap()
                    .state
                    .focused_mut()
                    .push_char(c);
            }
        }
    }

    #[tokio::test]
    async fn test_add_key_opens_image_dialog_on_image_list() {
        let mut app = build_app(
            MockContainerRepository::new(),
            MockVolumeRepository::new(),
            MockImageRepository::new(),
            MockStackRepository::new(),
            MockNetworkRepository::new(),
        );
        app.screen = Screen::ImageList;
        app.handle_image_list_action(AppAction::Add).await;
        assert!(app.add_dialog.is_some());
        assert_eq!(app.add_dialog.as_ref().unwrap().state.values().len(), 1);
    }

    #[tokio::test]
    async fn test_add_key_opens_container_dialog_on_container_list() {
        let mut app = build_app(
            MockContainerRepository::new(),
            MockVolumeRepository::new(),
            MockImageRepository::new(),
            MockStackRepository::new(),
            MockNetworkRepository::new(),
        );
        app.screen = Screen::ContainerList;
        app.handle_container_list_action(AppAction::Add).await;
        assert!(app.add_dialog.is_some());
        assert_eq!(app.add_dialog.as_ref().unwrap().state.values().len(), 3);
    }

    #[tokio::test]
    async fn test_add_dialog_key_input_and_cancel() {
        let mut app = build_app(
            MockContainerRepository::new(),
            MockVolumeRepository::new(),
            MockImageRepository::new(),
            MockStackRepository::new(),
            MockNetworkRepository::new(),
        );
        app.screen = Screen::ImageList;
        app.handle_image_list_action(AppAction::Add).await;
        assert!(app.handle_add_dialog_key(crossterm::event::KeyCode::Char('n')));
        assert!(app.handle_add_dialog_key(crossterm::event::KeyCode::Char('g')));
        assert!(app.handle_add_dialog_key(crossterm::event::KeyCode::Backspace));
        assert_eq!(app.add_dialog.as_ref().unwrap().state.values(), vec!["n"]);
        assert!(!app.handle_add_dialog_key(crossterm::event::KeyCode::Enter));
        app.handle_add_dialog_action(AppAction::Back).await;
        assert!(app.add_dialog.is_none());
    }

    #[tokio::test]
    async fn test_add_dialog_container_tab_navigation_between_fields() {
        let mut app = build_app(
            MockContainerRepository::new(),
            MockVolumeRepository::new(),
            MockImageRepository::new(),
            MockStackRepository::new(),
            MockNetworkRepository::new(),
        );
        app.screen = Screen::ContainerList;
        app.handle_container_list_action(AppAction::Add).await;
        assert!(app.handle_add_dialog_key(crossterm::event::KeyCode::Char('w')));
        assert!(app.handle_add_dialog_key(crossterm::event::KeyCode::Tab));
        assert!(app.handle_add_dialog_key(crossterm::event::KeyCode::Char('x')));
        assert_eq!(
            app.add_dialog.as_ref().unwrap().state.values(),
            vec!["w", "x", ""]
        );
    }

    #[tokio::test]
    async fn test_add_image_dialog_validates_empty_input() {
        let mut app = build_app(
            MockContainerRepository::new(),
            MockVolumeRepository::new(),
            MockImageRepository::new(),
            MockStackRepository::new(),
            MockNetworkRepository::new(),
        );
        app.screen = Screen::ImageList;
        app.handle_image_list_action(AppAction::Add).await;
        app.handle_add_dialog_action(AppAction::Select).await;
        assert!(app.add_dialog.is_some());
        match &app.popup_message {
            Some(PopupMessage::Error(msg)) => assert!(msg.contains("enter an image")),
            other => panic!("expected error popup, got {other:?}"),
        }
        assert!(app.pending_image_pull.is_none());
    }

    #[tokio::test]
    async fn test_add_image_dialog_pulls_image_with_latest_default_tag() {
        let mut image_repo = MockImageRepository::new();
        image_repo.expect_get_all().returning(|| Ok(vec![]));
        image_repo.expect_local_digest().returning(|_| Ok(None));
        image_repo.expect_pull_images().returning(|refs| {
            assert_eq!(refs, vec!["nginx:latest".to_string()]);
            Ok(ImagePullSubscription::completed(Ok(ImagePullOutcome {
                pulled: vec!["nginx:latest".to_string()],
                up_to_date: vec![],
            })))
        });
        let mut app = build_app(
            MockContainerRepository::new(),
            MockVolumeRepository::new(),
            image_repo,
            MockStackRepository::new(),
            MockNetworkRepository::new(),
        );
        app.screen = Screen::ImageList;
        app.handle_image_list_action(AppAction::Add).await;
        fill_add_dialog(&mut app, &["nginx"]);
        app.handle_add_dialog_action(AppAction::Select).await;
        assert!(app.pending_image_pull.is_some());
        app.drain_image_pull().await;
        match &app.popup_message {
            Some(PopupMessage::Info(msg)) => assert!(msg.contains("pulled 1 image(s)")),
            other => panic!("expected info popup, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_add_container_dialog_validates_required_fields() {
        let mut app = build_app(
            MockContainerRepository::new(),
            MockVolumeRepository::new(),
            MockImageRepository::new(),
            MockStackRepository::new(),
            MockNetworkRepository::new(),
        );
        app.screen = Screen::ContainerList;
        app.handle_container_list_action(AppAction::Add).await;
        fill_add_dialog(&mut app, &["web", ""]);
        app.handle_add_dialog_action(AppAction::Select).await;
        assert!(app.add_dialog.is_some());
        match &app.popup_message {
            Some(PopupMessage::Error(msg)) => assert!(msg.contains("required")),
            other => panic!("expected error popup, got {other:?}"),
        }
        assert_eq!(
            app.add_dialog.as_ref().unwrap().state.values(),
            vec!["web", "", ""]
        );
    }

    #[tokio::test]
    async fn test_add_container_dialog_validates_port_mappings() {
        let mut app = build_app(
            MockContainerRepository::new(),
            MockVolumeRepository::new(),
            MockImageRepository::new(),
            MockStackRepository::new(),
            MockNetworkRepository::new(),
        );
        app.screen = Screen::ContainerList;
        app.handle_container_list_action(AppAction::Add).await;
        fill_add_dialog(&mut app, &["web", "nginx", "8080"]);
        app.handle_add_dialog_action(AppAction::Select).await;
        assert!(app.add_dialog.is_some());
        match &app.popup_message {
            Some(PopupMessage::Error(msg)) => assert!(msg.contains("invalid port mapping")),
            other => panic!("expected error popup, got {other:?}"),
        }
        assert_eq!(
            app.add_dialog.as_ref().unwrap().state.values(),
            vec!["web", "nginx", "8080"]
        );
        assert!(app.pending_create_after_pull.is_none());
    }

    #[tokio::test]
    async fn test_add_container_dialog_creates_and_starts_container_after_pull() {
        let mut container_repo = MockContainerRepository::new();
        container_repo.expect_get_all().returning(|| Ok(vec![]));
        container_repo
            .expect_create()
            .withf(|request| {
                request.name == "web"
                    && request.image == "nginx:latest"
                    && request.port_mappings.len() == 1
                    && request.port_mappings[0].host_port == 8080
                    && request.port_mappings[0].container_port == 80
            })
            .returning(|_| Ok("abc123".to_string()));
        container_repo
            .expect_start()
            .withf(|id| id == "abc123")
            .returning(|_| Ok(()));
        let mut image_repo = MockImageRepository::new();
        image_repo.expect_local_digest().returning(|_| Ok(None));
        image_repo.expect_pull_images().returning(|refs| {
            assert_eq!(refs, vec!["nginx:latest".to_string()]);
            Ok(ImagePullSubscription::completed(Ok(ImagePullOutcome {
                pulled: vec!["nginx:latest".to_string()],
                up_to_date: vec![],
            })))
        });
        let mut app = build_app(
            container_repo,
            MockVolumeRepository::new(),
            image_repo,
            MockStackRepository::new(),
            MockNetworkRepository::new(),
        );
        app.screen = Screen::ContainerList;
        app.handle_container_list_action(AppAction::Add).await;
        fill_add_dialog(&mut app, &["web", "nginx", "8080:80"]);
        app.handle_add_dialog_action(AppAction::Select).await;
        assert!(app.add_dialog.is_none());
        assert!(app.pending_create_after_pull.is_some());
        assert!(app.pending_image_pull.is_some());
        app.drain_image_pull().await;
        assert!(app.pending_create_after_pull.is_none());
        assert!(!app.container_presenter.is_loading());
        match &app.popup_message {
            Some(PopupMessage::Info(msg)) => assert!(msg.contains("Container web created")),
            other => panic!("expected info popup, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_add_container_dialog_reports_create_error() {
        let mut container_repo = MockContainerRepository::new();
        container_repo.expect_get_all().returning(|| Ok(vec![]));
        container_repo.expect_create().returning(|_| {
            Err(crate::application::error::AppError::repository(
                "name taken",
            ))
        });
        let mut image_repo = MockImageRepository::new();
        image_repo.expect_local_digest().returning(|_| Ok(None));
        image_repo.expect_pull_images().returning(|_| {
            Ok(ImagePullSubscription::completed(Ok(ImagePullOutcome {
                pulled: vec!["nginx:latest".to_string()],
                up_to_date: vec![],
            })))
        });
        let mut app = build_app(
            container_repo,
            MockVolumeRepository::new(),
            image_repo,
            MockStackRepository::new(),
            MockNetworkRepository::new(),
        );
        app.screen = Screen::ContainerList;
        app.handle_container_list_action(AppAction::Add).await;
        fill_add_dialog(&mut app, &["web", "nginx", ""]);
        app.handle_add_dialog_action(AppAction::Select).await;
        app.drain_image_pull().await;
        match &app.popup_message {
            Some(PopupMessage::Error(msg)) => assert!(msg.contains("name taken")),
            other => panic!("expected error popup, got {other:?}"),
        }
    }
}
