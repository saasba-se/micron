pub mod confirm;
pub mod login;
pub mod oauth;
pub mod signup;

use axum::{
    response::IntoResponse,
    routing::{get, post},
};

use crate::{Config, Result};

use super::Router;

pub fn router(config: &Config) -> Router {
    let mut router = Router::new()
        .route("/login", post(login::login))
        // .route("/login-retry", get(login::login_retry))
        // .route("/verify", post(verify::verify))
        .route("/redir", get(redir))
        .route("/logout", get(login::logout))
        .route("/signup", post(signup::signup))
        .route("/confirm/:key", get(confirm::confirm));

    if config.oauth.enabled {
        router = router.merge(oauth::router());
    }

    router
}

/// Perform lingering cookie-based redirect if it was scheduled.
pub async fn redir(
    mut cookies: axum_extra::extract::CookieJar,
) -> Result<axum::response::Response> {
    // Process possible cookie-based redirect
    let redir = if let Some(redir) = cookies.clone().get("next") {
        cookies = cookies.remove(cookie::Cookie::named("next"));
        redir.value().to_string()
    } else {
        "/".to_string()
    };

    Ok((cookies, axum::response::Redirect::to(&redir)).into_response())
}
