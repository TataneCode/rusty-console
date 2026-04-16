#[derive(Debug, Clone)]
pub struct PruneResultDto {
    pub deleted_count: u32,
    pub space_freed: u64,
}
