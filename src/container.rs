#![allow(unused_imports)]

pub mod domain {
    pub use crate::domain::container::*;

    pub mod entity {
        pub use crate::domain::container::entity::*;
    }

    pub mod state {
        pub use crate::domain::container::state::*;
    }

    pub mod value_objects {
        pub use crate::domain::container::value_objects::*;
    }
}

pub mod application {
    pub use crate::application::container::*;

    pub mod dto {
        pub use crate::application::container::dto::*;
    }

    pub mod mapper {
        pub use crate::application::container::mapper::*;
    }

    pub mod service {
        pub use crate::application::container::service::*;
    }

    pub mod traits {
        pub use crate::application::container::traits::*;
    }
}

pub mod infrastructure {
    pub use crate::infrastructure::docker::container::*;

    pub mod adapter {
        pub use crate::infrastructure::docker::container::adapter::*;
    }

    pub mod mapper {
        pub use crate::infrastructure::docker::container::mapper::*;
    }
}

pub mod ui {
    pub use crate::presentation::tui::container::*;

    pub mod actions {
        pub use crate::presentation::tui::container::actions::*;
    }

    pub mod presenter {
        pub use crate::presentation::tui::container::presenter::*;
    }

    pub mod view {
        pub use crate::presentation::tui::container::view::*;
    }
}
