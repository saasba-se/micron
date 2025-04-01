//! Example showing off embedding of assets into the application binary.
//!
//! Embedding strategy can be used to produce truly standalone binary artifacts
//! that don't need to go to the filesystem for static files at runtime.
//!
//! # Rebuilding on assets change
//!
//! Note that, as this is just an example that doesn't come with a build
//! script, you will be required to forcibly clean the artifact and rebuild
//! after you introduce changes to `examples/assets`.
//!
//! In a real project it's useful to establish a `build.rs` that prints the
//! following:
//!
//! ```rust
//! println!("cargo:rerun-if-changed=<PATH_TO_ASSETS>")
//! ```
//!
//! This will ensure rebuild on changes made to the directory to be embedded.

use include_dir::{include_dir, Dir};
use tower_serve_static::ServeDir;

use micron::Config;

// embed files from `examples/assets` directory into the binary
static ASSETS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/examples/assets");

#[tokio::main]
async fn main() {
    let mut config = Config::default();
    // disable serving `assets` from filesystem directory
    config.assets.serve = false;

    // main application router
    let mut router = micron::axum::Router::new().nest_service("/assets", ServeDir::new(&ASSETS));

    // attach micron routes
    router = micron::axum::router(router, &config);

    // start the application
    micron::axum::start(router, config).await.expect("failed")
}
