#[derive(Debug, Clone)]
pub struct VolumeDto {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    pub size: String,
    pub created: String,
    pub in_use: bool,
    pub linked_containers: Vec<String>,
    pub can_delete: bool,
}
