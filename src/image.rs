#![allow(unused_imports)]

pub mod domain {
    pub use crate::domain::image::*;

    pub mod entity {
        pub use crate::domain::image::entity::*;
    }

    pub mod value_objects {
        pub use crate::domain::image::value_objects::*;
    }
}

pub mod application {
    pub use crate::application::image::*;

    pub mod dto {
        pub use crate::application::image::dto::*;
    }

    pub mod mapper {
        pub use crate::application::image::mapper::*;
    }

    pub mod service {
        pub use crate::application::image::service::*;
    }

    pub mod traits {
        pub use crate::application::image::traits::*;
    }
}

pub mod infrastructure {
    pub use crate::infrastructure::docker::image::*;

    pub mod adapter {
        pub use crate::infrastructure::docker::image::adapter::*;
    }

    pub mod mapper {
        pub use crate::infrastructure::docker::image::mapper::*;
    }
}

pub mod ui {
    pub use crate::presentation::tui::image::*;

    pub mod actions {
        pub use crate::presentation::tui::image::actions::*;
    }

    pub mod presenter {
        pub use crate::presentation::tui::image::presenter::*;
    }

    pub mod view {
        pub use crate::presentation::tui::image::view::*;
    }
}
