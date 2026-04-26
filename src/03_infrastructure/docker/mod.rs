#![allow(unused_imports)]

//! Transitional bridge for infrastructure Docker integration.
//!
//! `client.rs` is now the canonical home of the shared Docker client wrapper.
//! Feature-specific Docker adapters will move under this directory incrementally.

pub mod client;
pub mod container;

pub mod image;

pub mod volume;

pub mod stack;
