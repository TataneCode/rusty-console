#[derive(Debug, Clone)]
pub struct NetworkContainerDto {
    pub name: String,
    pub ip: String,
}

#[derive(Debug, Clone)]
pub struct NetworkDto {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
    pub internal: bool,
    pub attachable: bool,
    pub subnet: Option<String>,
    pub gateway: Option<String>,
    pub created: String,
    pub builtin: bool,
    pub containers: Vec<NetworkContainerDto>,
    pub can_delete: bool,
    pub container_count: usize,
}
