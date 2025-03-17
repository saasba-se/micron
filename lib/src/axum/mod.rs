pub mod auth;
pub mod comment;
pub mod extract;
pub mod image;
pub mod mailing;
pub mod user;

#[cfg(feature = "askama")]
pub mod askama;
#[cfg(feature = "stripe")]
pub mod stripe;

pub use extract::user::User;

use std::{net::ToSocketAddrs, sync::Arc};

use axum::Extension;

use crate::Result;
use crate::{Config, Database};

pub type Router = axum::Router<cookie::Key>;

pub type ConfigExt<C = Config> = Extension<Arc<C>>;
pub type DbExt = Extension<Arc<Database>>;

#[cfg(feature = "stripe")]
pub type StripeExt = Extension<Arc<::stripe::Client>>;

/// Registers micron routes on the provided router.
///
/// Meant to be used if there is a need to register custom middleware that will
/// run on micron routes.
///
/// # Configurable routes
///
/// It's possible to customize the routes registered with this function through
/// relevant config declarations. This is helpful in cases where we want to
/// still register the same route with the same micron handler but also add
/// a middleware layer on top of that route.
// TODO: allow more granular control over registered routes; pass config down
// to the individual module router generators; differentiate `module` vs
// `/route` identifiers
// TODO: could perhaps make this nicer by introducing a newtype for the router
// and implementing custom `route` and `merge`
pub fn router(mut router: Router, config: &Config) -> Router {
    #[cfg(feature = "stripe")]
    {
        router = conditional_merge("stripe", router, stripe::router(), config);
    }
    router = conditional_merge("user", router, user::router(), config);
    router = conditional_merge("comment", router, comment::router(), config);
    router = conditional_merge("mailing", router, mailing::router(), config);
    router = conditional_merge("auth", router, auth::router(config), config);
    conditional_merge("image", router, image::router(), config)
}

fn conditional_merge(route: &str, routera: Router, routerb: Router, config: &Config) -> Router {
    if config.routes.enable.contains(&route.to_string())
        || !config.routes.disable.contains(&route.to_string())
    {
        routera.merge(routerb)
    } else {
        routera
    }
}

/// Registers micron routes on the provided router, initializes application
/// state and starts the web server.
pub async fn start(mut router: Router, config: Config) -> Result<()> {
    start_with(Database::new()?, router, config).await
}

pub async fn start_with(db: Database, mut router: Router, config: Config) -> Result<()> {
    crate::tracing::init(&config).unwrap_or_else(|e| {
        log::warn!("failed to initialize tracing (perhaps it was already initialized?): {e}")
    });

    // Provide initial state as defined in config
    if config.init.enabled {
        crate::init::initialize(&config, &db)?;
    }

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

    #[cfg(feature = "stripe")]
    let mut router = {
        let secret = if cfg!(debug_assertions) {
            &config.payments.stripe.test_secret
        } else {
            &config.payments.stripe.secret
        };
        router.layer(Extension(Arc::new(::stripe::Client::new(secret))))
    };

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
