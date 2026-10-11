#[derive(Debug, Clone)]
pub struct PruneResultDto {
    pub deleted_count: u32,
    pub space_freed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prune_result_dto_fields() {
        let dto = PruneResultDto {
            deleted_count: 3,
            space_freed: 1024,
        };
        assert_eq!(dto.deleted_count, 3);
        assert_eq!(dto.space_freed, 1024);
    }
}
