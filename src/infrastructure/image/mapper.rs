use crate::domain::{Image, ImageId, ImageSize};
use bollard::models::ImageSummary;
use chrono::{TimeZone, Utc};
use std::collections::HashSet;

pub struct ImageInfraMapper;

impl ImageInfraMapper {
    pub fn from_docker(
        summary: &ImageSummary,
        in_use_image_ids: &HashSet<String>,
    ) -> Option<Image> {
        let id = ImageId::new(&summary.id).ok()?;

        let (repository, tag) = summary
            .repo_tags
            .first()
            .map(|rt| {
                let parts: Vec<&str> = rt.rsplitn(2, ':').collect();
                if parts.len() == 2 {
                    (parts[1].to_string(), parts[0].to_string())
                } else {
                    (rt.clone(), "<none>".to_string())
                }
            })
            .unwrap_or_else(|| ("<none>".to_string(), "<none>".to_string()));

        let size = ImageSize::new(summary.size);

        let created = Utc
            .timestamp_opt(summary.created, 0)
            .single()
            .unwrap_or_else(Utc::now);

        let in_use = in_use_image_ids.contains(&summary.id);

        Some(Image::new(id, repository, tag, size, created).with_in_use(in_use))
    }
}
