#![allow(unused_imports)]

pub mod domain {
    pub use crate::domain::volume::*;

    pub mod entity {
        pub use crate::domain::volume::entity::*;
    }

    pub mod value_objects {
        pub use crate::domain::volume::value_objects::*;
    }
}

pub mod application {
    pub use crate::application::volume::*;

    pub mod dto {
        pub use crate::application::volume::dto::*;
    }

    pub mod mapper {
        pub use crate::application::volume::mapper::*;
    }

    pub mod service {
        pub use crate::application::volume::service::*;
    }

    pub mod traits {
        pub use crate::application::volume::traits::*;
    }
}

pub mod infrastructure {
    pub use crate::infrastructure::docker::volume::*;

    pub mod adapter {
        pub use crate::infrastructure::docker::volume::adapter::*;
    }

    pub mod mapper {
        pub use crate::infrastructure::docker::volume::mapper::*;
    }
}

pub mod ui {
    pub use crate::presentation::tui::volume::*;

    pub mod actions {
        pub use crate::presentation::tui::volume::actions::*;
    }

    pub mod presenter {
        pub use crate::presentation::tui::volume::presenter::*;
    }

    pub mod view {
        pub use crate::presentation::tui::volume::view::*;
    }
}
