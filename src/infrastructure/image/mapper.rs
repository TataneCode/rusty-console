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

#[cfg(test)]
mod tests {
    use super::*;
    use bollard::models::ImageSummary;

    fn make_image_summary(id: &str, repo_tags: Vec<&str>, size: i64) -> ImageSummary {
        ImageSummary {
            id: id.to_string(),
            repo_tags: repo_tags.into_iter().map(|s| s.to_string()).collect(),
            size,
            created: 1_700_000_000,
            ..Default::default()
        }
    }

    #[test]
    fn from_docker_normal_image_with_repo_tag() {
        let summary = make_image_summary("sha256:abc123", vec!["nginx:latest"], 50_000_000);
        let in_use = HashSet::new();

        let image = ImageInfraMapper::from_docker(&summary, &in_use).unwrap();
        assert_eq!(image.repository(), "nginx");
        assert_eq!(image.tag(), "latest");
        assert_eq!(image.size().bytes(), 50_000_000);
        assert!(!image.in_use());
        assert!(!image.is_dangling());
    }

    #[test]
    fn from_docker_dangling_image() {
        let summary = make_image_summary("sha256:deadbeef", vec!["<none>:<none>"], 10_000);
        let in_use = HashSet::new();

        let image = ImageInfraMapper::from_docker(&summary, &in_use).unwrap();
        assert_eq!(image.repository(), "<none>");
        assert_eq!(image.tag(), "<none>");
        assert!(image.is_dangling());
    }

    #[test]
    fn from_docker_in_use_detection() {
        let summary = make_image_summary("sha256:abc123", vec!["myapp:v1"], 100_000);
        let mut in_use = HashSet::new();
        in_use.insert("sha256:abc123".to_string());

        let image = ImageInfraMapper::from_docker(&summary, &in_use).unwrap();
        assert!(image.in_use());
        assert!(!image.can_be_deleted());
    }
}
