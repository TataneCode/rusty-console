//! Transitional bridge for the target application layer.
//!
//! Boundary rules for this layer:
//! - Orchestrate use cases around domain models.
//! - Define ports, DTOs, and mappers for application-facing workflows.
//! - Depend only on domain concepts and application-local transport types.
//! - Do not depend on ratatui, crossterm, bollard, or other outer-framework
//!   implementation details.

pub mod container {
    pub use crate::container::application::*;
}

pub mod image {
    pub use crate::image::application::*;
}

pub mod volume {
    pub use crate::volume::application::*;
}

pub mod stack {
    pub use crate::stack::application::*;
}

pub mod error {
    pub use crate::errors::application::AppError;
}

pub mod shared {
    pub use crate::shared::PruneResultDto;
}
