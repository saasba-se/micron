//! Shows how to load configuration from file.

use micron::{axum::Router, config, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration from file
    let config: Config = config::load_from(&format!(
        "{}/examples/app/app.toml",
        env!("CARGO_MANIFEST_DIR")
    ))?;

    // Confirm the application was set up based on the config file
    tokio::spawn(async {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        // TODO: check for something defined in the config file that results
        // in certain app setup
    });

    // Start the application
    micron::axum::start(Router::new(), config).await?;

    Ok(())
}
