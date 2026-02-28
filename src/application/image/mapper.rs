use crate::application::image::dto::ImageDto;
use crate::domain::Image;

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
