pub mod confirm;
pub mod login;
pub mod oauth;
pub mod signup;

use axum::routing::{get, post};

use crate::Config;

use super::Router;

pub fn router(config: &Config) -> Router {
    let mut router = Router::new()
        .route("/login", post(login::login))
        // .route("/login-retry", get(login::login_retry))
        // .route("/verify", post(verify::verify))
        .route("/logout", get(login::logout))
        .route("/signup", post(signup::signup))
        .route("/confirm/:key", get(confirm::confirm));

    if config.oauth.enabled {
        router = router.merge(oauth::router());
    }

    router
}
