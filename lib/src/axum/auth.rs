use axum::routing::{get, post};

use crate::Config;

use super::Router;

pub mod login;
pub mod oauth;

pub fn router(config: &Config) -> Router {
    let mut router = Router::new()
        .route("/login", post(login::login_submit))
        // .route("/login-retry", get(login::login_retry))
        .route("/logout", get(login::logout));
    // .route("/sign-up", post(signup::sign_up_submit))
    // .route("/verify", post(verify::verify))

    if config.oauth.enabled {
        router = router.merge(oauth::router());
    }

    router
}
