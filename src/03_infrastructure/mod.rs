#![allow(unused_imports)]

//! Transitional bridge for the target infrastructure layer.
//!
//! Boundary rules for this layer:
//! - Implement application ports and talk to external systems.
//! - Keep Docker/Bollard-specific code here.
//! - Depend inward on application and domain code, never the other way around.
//! - Presentation-specific concerns do not belong here.

pub mod docker;
pub mod error;

pub use error::InfraError;
