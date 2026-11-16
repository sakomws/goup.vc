//! Image storage service abstractions and shared types.

pub mod db;
pub mod s3;

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
#[cfg(any(test, feature = "test-utils"))]
use mockall::automock;

pub use db::DbImageStorage;
pub use ocg_core::shared::{Image, NewImage, OPEN_GRAPH_IMAGE_HEIGHT, OPEN_GRAPH_IMAGE_WIDTH};
pub use s3::S3ImageStorage;

/// Trait representing the capabilities required from an image storage provider.
#[async_trait]
#[cfg_attr(any(test, feature = "test-utils"), automock)]
pub trait ImageStorage {
    /// Retrieve an image by its file name from the storage provider.
    async fn get(&self, file_name: &str) -> Result<Option<Image>>;

    /// Save the provided image to the storage provider.
    async fn save(&self, image: &NewImage<'_>) -> Result<()>;
}

/// Trait object type for image storage providers.
pub type DynImageStorage = Arc<dyn ImageStorage + Send + Sync>;
