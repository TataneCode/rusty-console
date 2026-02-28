#[derive(Debug, Clone)]
pub struct ImageDto {
    pub id: String,
    pub short_id: String,
    pub repository: String,
    pub tag: String,
    pub full_name: String,
    pub size: String,
    pub created: String,
    pub in_use: bool,
    pub is_dangling: bool,
    pub can_delete: bool,
}
