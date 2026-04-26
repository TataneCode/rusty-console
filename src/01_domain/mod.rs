//! Transitional bridge for the target domain layer.
//!
//! The directory is intentionally prefixed with `01_` so the refactored layer
//! order is easy to read in the filesystem. The public Rust module name stays
//! `domain` because module identifiers cannot start with a digit.
//!
//! Boundary rules for this layer:
//! - Keep only business concepts and business invariants here.
//! - Do not depend on application services, infrastructure adapters, or
//!   presentation concerns.
//! - Shared pure business primitives belong here first unless they are clearly
//!   application-only transport types.

pub mod container;

pub mod image;

pub mod volume;

pub mod stack;

pub mod error;

pub mod shared {
    pub type ByteSize = crate::shared::ByteSize;
}
