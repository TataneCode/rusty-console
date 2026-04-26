#![allow(unused_imports)]

pub mod domain {
    pub use crate::domain::stack::*;

    pub mod container {
        pub use crate::domain::stack::container::*;
    }

    pub mod entity {
        pub use crate::domain::stack::entity::*;
    }

    pub mod state {
        pub use crate::domain::stack::state::*;
    }

    pub mod value_objects {
        pub use crate::domain::stack::value_objects::*;
    }
}

pub mod application {
    pub use crate::application::stack::*;

    pub mod dto {
        pub use crate::application::stack::dto::*;
    }

    pub mod mapper {
        pub use crate::application::stack::mapper::*;
    }

    pub mod service {
        pub use crate::application::stack::service::*;
    }

    pub mod traits {
        pub use crate::application::stack::traits::*;
    }
}

pub mod infrastructure {
    pub use crate::infrastructure::docker::stack::*;

    pub mod adapter {
        pub use crate::infrastructure::docker::stack::adapter::*;
    }

    pub mod mapper {
        pub use crate::infrastructure::docker::stack::mapper::*;
    }
}

pub mod ui {
    pub use crate::presentation::tui::stack::*;

    pub mod actions {
        pub use crate::presentation::tui::stack::actions::*;
    }

    pub mod presenter {
        pub use crate::presentation::tui::stack::presenter::*;
    }

    pub mod view {
        pub use crate::presentation::tui::stack::view::*;
    }
}
