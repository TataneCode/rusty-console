use crate::container::application::dto::ContainerDto;

#[derive(Debug, Clone)]
pub struct StackDto {
    pub name: String,
    pub container_count: usize,
    pub running_count: usize,
    pub containers: Vec<ContainerDto>,
}
