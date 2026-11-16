//! Static asset embedding and build metadata for OCG.

use rust_embed::Embed;

/// Static file embedder using `rust-embed`.
#[derive(Embed)]
#[folder = "../../ocg-server/dist/static"]
pub struct StaticFile;

/// Current application commit SHA embedded at build time.
pub const COMMIT_SHA: &str = env!("OCG_COMMIT_SHA");
