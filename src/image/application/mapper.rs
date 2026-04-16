use crate::image::application::dto::ImageDto;
use crate::image::domain::Image;

pub struct ImageMapper;

impl ImageMapper {
    pub fn to_dto(image: &Image) -> ImageDto {
        ImageDto {
            id: image.id().to_string(),
            short_id: image.id().short().to_string(),
            repository: image.repository().to_string(),
            tag: image.tag().to_string(),
            full_name: image.full_name(),
            size: image.size_display(),
            created: image.created().format("%Y-%m-%d %H:%M:%S").to_string(),
            in_use: image.in_use(),
            is_dangling: image.is_dangling(),
            can_delete: image.can_be_deleted(),
        }
    }

    pub fn to_dto_list(images: &[Image]) -> Vec<ImageDto> {
        images.iter().map(Self::to_dto).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::domain::{ImageId, ImageSize};
    use chrono::{TimeZone, Utc};

    fn create_test_image() -> Image {
        Image::new(
            ImageId::new("sha256:abcdef1234567890fedcba").unwrap(),
            "nginx",
            "latest",
            ImageSize::new(142_000_000),
            Utc.with_ymd_and_hms(2024, 3, 10, 12, 0, 0).unwrap(),
        )
    }

    #[test]
    fn to_dto_maps_basic_fields() {
        let image = create_test_image();
        let dto = ImageMapper::to_dto(&image);

        assert_eq!(dto.id, "sha256:abcdef1234567890fedcba");
        assert_eq!(dto.short_id, "abcdef123456");
        assert_eq!(dto.repository, "nginx");
        assert_eq!(dto.tag, "latest");
        assert_eq!(dto.full_name, "nginx:latest");
        assert_eq!(dto.size, "135.42 MB");
        assert_eq!(dto.created, "2024-03-10 12:00:00");
    }

    #[test]
    fn to_dto_not_in_use_can_delete() {
        let image = create_test_image().with_in_use(false);
        let dto = ImageMapper::to_dto(&image);

        assert!(!dto.in_use);
        assert!(dto.can_delete);
    }

    #[test]
    fn to_dto_in_use_cannot_delete() {
        let image = create_test_image().with_in_use(true);
        let dto = ImageMapper::to_dto(&image);

        assert!(dto.in_use);
        assert!(!dto.can_delete);
    }

    #[test]
    fn to_dto_dangling_image() {
        let image = Image::new(
            ImageId::new("sha256:deadbeef1234567890ab").unwrap(),
            "<none>",
            "<none>",
            ImageSize::new(50_000_000),
            Utc::now(),
        );
        let dto = ImageMapper::to_dto(&image);

        assert!(dto.is_dangling);
        assert_eq!(dto.full_name, "<none>");
    }

    #[test]
    fn to_dto_non_dangling_image() {
        let dto = ImageMapper::to_dto(&create_test_image());
        assert!(!dto.is_dangling);
    }

    #[test]
    fn to_dto_list_maps_multiple_images() {
        let images = vec![
            create_test_image(),
            Image::new(
                ImageId::new("sha256:second123456789012").unwrap(),
                "redis",
                "7-alpine",
                ImageSize::new(30_000_000),
                Utc::now(),
            ),
        ];
        let dtos = ImageMapper::to_dto_list(&images);

        assert_eq!(dtos.len(), 2);
        assert_eq!(dtos[0].repository, "nginx");
        assert_eq!(dtos[1].repository, "redis");
        assert_eq!(dtos[1].full_name, "redis:7-alpine");
    }

    #[test]
    fn to_dto_list_empty_input() {
        let dtos = ImageMapper::to_dto_list(&[]);
        assert!(dtos.is_empty());
    }
}
