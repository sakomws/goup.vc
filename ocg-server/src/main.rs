//! Binary entry point for the Open Alliance Groups server.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    ocg_server::run().await
}
