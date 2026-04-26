#![allow(unused_imports)]

//! Transitional bridge for the target presentation layer.
//!
//! Boundary rules for this layer:
//! - Own terminal rendering, input mapping, presenters, and UI orchestration.
//! - Depend on application services and DTOs, not on infrastructure adapters.
//! - Translate user intent into application calls without embedding Docker or
//!   persistence knowledge.

pub mod tui;
