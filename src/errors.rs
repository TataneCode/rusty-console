#![allow(unused_imports)]

pub mod application {
    pub use crate::application::error::*;
}

pub mod domain {
    pub use crate::domain::error::*;
}

pub mod infrastructure {
    pub use crate::infrastructure::error::*;
}

pub use application::AppError;
pub use domain::DomainError;
pub use infrastructure::InfraError;
