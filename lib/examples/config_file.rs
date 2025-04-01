//! Shows how one can load configuration from file.

use micron::{axum::Router, config, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration from file
    let config: Config = config::load_from(&format!(
        "{}/../examples/saas/micron.toml",
        env!("CARGO_MANIFEST_DIR")
    ))?;

    // Start the application
    micron::axum::start(Router::new(), config).await?;

    Ok(())
}
