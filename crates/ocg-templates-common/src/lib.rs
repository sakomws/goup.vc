//! Shared Askama template filters, helpers, and authentication templates.

pub mod auth;
pub mod filters;
pub mod helpers;

pub use ocg_core::shared::PageId;

pub(crate) mod router {
    #[allow(unused_imports)]
    pub(crate) use ocg_assets::COMMIT_SHA;
}

pub(crate) mod services {
    pub(crate) mod images {
        #[allow(unused_imports)]
        pub(crate) use ocg_core::shared::{OPEN_GRAPH_IMAGE_HEIGHT, OPEN_GRAPH_IMAGE_WIDTH};
    }
}

pub(crate) mod templates {
    #[allow(unused_imports)]
    pub(crate) use crate::helpers;
}

#[allow(unused_imports)]
pub(crate) use ocg_core::types;
#[allow(unused_imports)]
pub(crate) use ocg_core::validation;
