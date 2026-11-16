//! Public Askama templates for site, alliance, group, and event pages.

pub mod alliance;
pub mod event;
pub mod group;
pub mod site;

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
    pub(crate) use crate::{alliance, site};
    #[allow(unused_imports)]
    pub(crate) use ocg_templates_common::helpers;
    #[allow(unused_imports)]
    pub(crate) use ocg_templates_dashboard::dashboard;
}

#[allow(unused_imports)]
pub(crate) use ocg_core::types;
#[allow(unused_imports)]
pub(crate) use ocg_core::validation;
