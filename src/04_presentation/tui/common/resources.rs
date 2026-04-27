pub const FILTER_CURSOR: &str = "▏";
pub const TRUNCATION_MARKER: &str = "...";

pub const MAIN_MENU_TITLE: &str = "Rusty Console - Docker Manager";
pub const MAIN_MENU_BLOCK_TITLE: &str = " Menu ";
pub const MAIN_MENU_HELP: &str = " j/k: Navigate | Enter: Select | q: Quit ";
pub const MAIN_MENU_HIGHLIGHT_SYMBOL: &str = "> ";
pub const MAIN_MENU_ITEMS: [&str; 5] = [
    "  Containers",
    "  Volumes",
    "  Images",
    "  Stacks",
    "  Quit",
];

pub const CONFIRM_TITLE: &str = " Confirm ";
pub const ERROR_TITLE: &str = " Error ";
pub const INFO_TITLE: &str = " Info ";
pub const CONFIRM_YES_LABEL: &str = "  [Yes]";
pub const CONFIRM_NO_LABEL: &str = "  [No]";
pub const CONFIRM_YES_BUTTON_WIDTH: u16 = 7;
pub const CONFIRM_NO_BUTTON_WIDTH: u16 = 6;
pub const CONFIRM_BUTTON_SPACING: u16 = 9;

pub const CONTAINER_TITLE: &str = "Containers";
pub const CONTAINER_HEADERS: [&str; 5] = ["Name", "Image", "State", "Status", "Ports"];
pub const CONTAINER_LIST_HELP: &str =
    " q: Quit | /: Filter | j/k: Navigate | l: Logs | e: Exec | s: Start/Stop | d: Delete | c: Details | r: Refresh | p: Pause | R: Restart | X: Prune";
pub const CONTAINER_DETAILS_TITLE: &str = " Container Details ";
pub const CONTAINER_DETAILS_HELP: &str = " Esc/q: Back ";
pub const CONTAINER_LOGS_HELP: &str = " Esc/q: Back | Ctrl+u/d: Scroll ";
pub const CONTAINER_DETAILS_ENV_VARS_LABEL: &str = "Environment Variables";
pub const EXEC_SHELL_DIALOG_TITLE: &str = " Exec Shell ";
pub const EXEC_SHELL_DIALOG_HELP: &str = " j/k: Navigate | Enter: Select | Esc/q: Cancel ";

pub const VOLUME_TITLE: &str = "Volumes";
pub const VOLUME_HEADERS: [&str; 5] = ["Name", "Driver", "Size", "In Use", "Created"];
pub const VOLUME_LIST_HELP: &str =
    " q: Quit | /: Filter | j/k: Navigate | d: Delete | r: Refresh | X: Prune | Esc: Back ";

pub const IMAGE_TITLE: &str = "Images";
pub const IMAGE_HEADERS: [&str; 6] = ["Repository", "Tag", "ID", "Size", "In Use", "Created"];
pub const IMAGE_LIST_HELP: &str =
    " q: Quit | /: Filter | j/k: Navigate | d: Delete | c: Details | r: Refresh | X: Prune | Esc: Back ";
pub const IMAGE_DETAILS_TITLE: &str = " Image Details ";
pub const IMAGE_DETAILS_HELP: &str = " Esc/q: Back ";

pub const STACK_TITLE: &str = "Stacks";
pub const STACK_HEADERS: [&str; 3] = ["Stack", "Containers", "Running"];
pub const STACK_LIST_HELP: &str =
    " q: Quit | /: Filter | j/k: Navigate | Enter: Drill-down | s: Start All | S: Stop All | r: Refresh | Esc: Back ";
pub const STACK_CONTAINER_HEADERS: [&str; 5] = ["Name", "Image", "State", "Status", "Ports"];
pub const STACK_CONTAINERS_HELP: &str =
    " Esc/q: Back | j/k: Navigate | e: Exec | s: Start/Stop | S: Stop All | Ctrl+S: Start All | D: Remove All | d: Delete | r: Refresh ";

pub const LABEL_ID: &str = "ID:";
pub const LABEL_NAME: &str = "Name:";
pub const LABEL_IMAGE: &str = "Image:";
pub const LABEL_STATE: &str = "State:";
pub const LABEL_STATUS: &str = "Status:";
pub const LABEL_CREATED: &str = "Created:";
pub const LABEL_PORTS: &str = "Ports:";
pub const LABEL_NETWORKS: &str = "Networks:";
pub const LABEL_REPOSITORY: &str = "Repository:";
pub const LABEL_TAG: &str = "Tag:";
pub const LABEL_FULL_NAME: &str = "Full Name:";
pub const LABEL_SIZE: &str = "Size:";
pub const LABEL_IN_USE: &str = "In Use:";
pub const LABEL_DANGLING: &str = "Dangling:";
pub const VALUE_YES: &str = "Yes";
pub const VALUE_NO: &str = "No";

pub const DELETE_CONTAINER_MESSAGE: &str = "Delete this container?";
pub const FORCE_DELETE_CONTAINER_MESSAGE: &str = "Force delete this container?";
pub const DELETE_VOLUME_MESSAGE: &str = "Delete this volume?";
pub const DELETE_IMAGE_MESSAGE: &str = "Delete this image?";
pub const FORCE_DELETE_IMAGE_MESSAGE: &str = "Force delete this image?";
pub const PRUNE_CONTAINERS_MESSAGE: &str = "Prune all stopped containers?";
pub const PRUNE_VOLUMES_MESSAGE: &str = "Prune all unused volumes?";
pub const PRUNE_IMAGES_MESSAGE: &str = "Prune all dangling images?";
pub const REMOVE_ALL_STACK_CONTAINERS_MESSAGE: &str =
    "Remove ALL containers in this stack? (force)";

pub const CONTAINER_NOT_FOUND_MESSAGE: &str = "Container not found";
pub const VOLUME_IN_USE_DELETE_MESSAGE: &str = "Cannot delete volume: it is in use";

pub fn filter_prompt_title(base_title: &str, active_filter: Option<&str>) -> String {
    match active_filter {
        Some(filter) => format!(" {base_title} [/: {filter}{FILTER_CURSOR}] "),
        None => format!(" {base_title} "),
    }
}

pub fn logs_title(container_name: &str) -> String {
    format!(" Logs: {container_name} ")
}

pub fn stack_containers_title(stack_name: &str) -> String {
    format!(" Stack: {stack_name} ")
}

pub fn prune_result_message(resource_name: &str, deleted_count: u32, freed: &str) -> String {
    format!("Pruned {deleted_count} {resource_name}(s), freed {freed}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_prompt_title_without_filter() {
        assert_eq!(filter_prompt_title("Images", None), " Images ");
    }

    #[test]
    fn test_filter_prompt_title_with_filter() {
        assert_eq!(
            filter_prompt_title("Volumes", Some("data")),
            format!(" Volumes [/: data{FILTER_CURSOR}] ")
        );
    }

    #[test]
    fn test_logs_title_formats_name() {
        assert_eq!(logs_title("web"), " Logs: web ");
    }

    #[test]
    fn test_stack_containers_title_formats_name() {
        assert_eq!(
            stack_containers_title("compose-app"),
            " Stack: compose-app "
        );
    }

    #[test]
    fn test_prune_result_message_formats_result() {
        assert_eq!(
            prune_result_message("image", 3, "5.0 MB"),
            "Pruned 3 image(s), freed 5.0 MB"
        );
    }
}
