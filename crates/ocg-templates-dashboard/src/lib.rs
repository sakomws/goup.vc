//! Dashboard Askama templates for alliance, group, user, jobs, and opportunities.

pub mod dashboard;

pub use dashboard::*;

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
    pub(crate) use crate::dashboard;
    #[allow(unused_imports)]
    pub(crate) use ocg_templates_common::helpers;
}

#[allow(unused_imports)]
pub(crate) use ocg_core::types;
#[allow(unused_imports)]
pub(crate) use ocg_core::validation;
