//! Transitional bridge for the target application layer.
//!
//! Boundary rules for this layer:
//! - Orchestrate use cases around domain models.
//! - Define ports, DTOs, and mappers for application-facing workflows.
//! - Depend only on domain concepts and application-local transport types.
//! - Do not depend on ratatui, crossterm, bollard, or other outer-framework
//!   implementation details.

pub mod container;

pub mod image;

pub mod volume;

pub mod stack;

pub mod error;

pub mod shared {
    pub type PruneResultDto = crate::shared::PruneResultDto;
}
