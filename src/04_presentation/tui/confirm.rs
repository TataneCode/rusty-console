use crate::presentation::tui::common::resources;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmAction {
    ContainerDelete(bool),
    VolumeDelete,
    ImageDelete(bool),
    ContainersPrune,
    VolumesPrune,
    ImagesPrune,
    StackContainersRemoveAll,
}

pub fn confirm_message(action: ConfirmAction) -> &'static str {
    match action {
        ConfirmAction::ContainerDelete(force) => {
            if force {
                resources::FORCE_DELETE_CONTAINER_MESSAGE
            } else {
                resources::DELETE_CONTAINER_MESSAGE
            }
        }
        ConfirmAction::VolumeDelete => resources::DELETE_VOLUME_MESSAGE,
        ConfirmAction::ImageDelete(force) => {
            if force {
                resources::FORCE_DELETE_IMAGE_MESSAGE
            } else {
                resources::DELETE_IMAGE_MESSAGE
            }
        }
        ConfirmAction::ContainersPrune => resources::PRUNE_CONTAINERS_MESSAGE,
        ConfirmAction::VolumesPrune => resources::PRUNE_VOLUMES_MESSAGE,
        ConfirmAction::ImagesPrune => resources::PRUNE_IMAGES_MESSAGE,
        ConfirmAction::StackContainersRemoveAll => resources::REMOVE_ALL_STACK_CONTAINERS_MESSAGE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_message_delete_container() {
        assert_eq!(
            confirm_message(ConfirmAction::ContainerDelete(false)),
            resources::DELETE_CONTAINER_MESSAGE
        );
        assert_eq!(
            confirm_message(ConfirmAction::ContainerDelete(true)),
            resources::FORCE_DELETE_CONTAINER_MESSAGE
        );
    }

    #[test]
    fn test_confirm_message_prunes() {
        assert_eq!(
            confirm_message(ConfirmAction::ContainersPrune),
            resources::PRUNE_CONTAINERS_MESSAGE
        );
        assert_eq!(
            confirm_message(ConfirmAction::VolumesPrune),
            resources::PRUNE_VOLUMES_MESSAGE
        );
        assert_eq!(
            confirm_message(ConfirmAction::ImagesPrune),
            resources::PRUNE_IMAGES_MESSAGE
        );
    }

    #[test]
    fn test_confirm_message_delete_volume_and_stack() {
        assert_eq!(
            confirm_message(ConfirmAction::VolumeDelete),
            resources::DELETE_VOLUME_MESSAGE
        );
        assert_eq!(
            confirm_message(ConfirmAction::StackContainersRemoveAll),
            resources::REMOVE_ALL_STACK_CONTAINERS_MESSAGE
        );
    }

    #[test]
    fn test_confirm_message_delete_image() {
        assert_eq!(
            confirm_message(ConfirmAction::ImageDelete(false)),
            resources::DELETE_IMAGE_MESSAGE
        );
        assert_eq!(
            confirm_message(ConfirmAction::ImageDelete(true)),
            resources::FORCE_DELETE_IMAGE_MESSAGE
        );
    }
}
