pub mod auth;
pub mod extract;
pub mod image;
pub mod mailing;
pub mod user;

#[cfg(feature = "askama")]
pub mod askama;

pub use extract::user::User;

use std::{net::ToSocketAddrs, sync::Arc};

use axum::Extension;

use crate::Result;
use crate::{Config, Database};

pub type Router = axum::Router<cookie::Key>;

pub type ConfigExt<C = Config> = Extension<Arc<C>>;
pub type DbExt = Extension<Arc<Database>>;

/// Registers saasbase routes on the provided router.
///
/// Meant to be used if there is a need to register custom middleware that will
/// run on saasbase routes.
pub fn router(router: Router, config: &Config) -> Router {
    // TODO: merge routers based on whether they are enabled in config
    router
        .merge(user::router())
        .merge(auth::router(&config))
        .merge(image::router())
}

/// Registers saasbase routes on the provided router, initializes application
/// state and starts the web server.
pub async fn start(mut router: Router, config: Config) -> Result<()> {
    start_with(Database::new()?, router, config).await
}

pub async fn start_with(db: Database, mut router: Router, config: Config) -> Result<()> {
    crate::tracing::init(&config).unwrap_or_else(|e| {
        log::warn!("failed to initialize tracing (perhaps it was already initialized?): {e}")
    });

    // Provide initial state as defined in config
    crate::init::initialize(&config, &db);

    // Generate mock data. Basically we want to be able to create a full
    // "synthetic" state consisting of all the different data items.
    if config.dev.enabled && config.dev.mock {
        crate::mock::generate(&config, &db)?;
    }

    // Generate the cookie key. We store the cookie key in state instead of
    // in the state extension because of how cookies extraction is
    // currently implemented in axum.
    let key = if config.dev.enabled {
        // In dev mode the cookie key is stored in memory and only persists
        // until application is rerun.
        cookie::Key::generate()
    } else {
        // Otherwise the cookie key is stored in the db and persisted
        // between application restarts. The key can be refreshed manually
        // using the cli tool.
        match db.get_at::<Vec<u8>>("cookie_keys", uuid::Uuid::nil()) {
            Ok(k) => cookie::Key::from(&k),
            Err(_) => {
                let k = cookie::Key::generate();
                db.set_raw_at("cookie_keys", &k.master(), uuid::Uuid::nil())?;
                k
            }
        }
    };

    if config.assets.serve {
        router = router.nest_service(
            "/assets",
            tower_http::services::ServeDir::new(&config.assets.path),
        );
    }

    // Encapsulate application state
    let addr = config.address;

    let mut router = router
        // Register common state extension for all routes
        .layer(Extension(Arc::new(config)))
        .layer(Extension(Arc::new(db)))
        .with_state(key);

    // Serve the application
    tracing::info!("starting server at {addr}");
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed binding to addr: {addr}");
    axum::serve(listener, router).await.map_err(|e| e.into())
}
