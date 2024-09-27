use axum::routing::get;

use super::Router;

mod facebook;
mod github;

pub fn router() -> Router {
    Router::new()
        .route("/login/facebook", get(facebook::initiate))
        .route("/auth/facebook", get(facebook::callback))
        .route("/login/github", get(github::initiate))
        .route("/auth/github", get(github::callback))
    // .route("/login/discord", get(discord::login))
    // .route("/auth/discord", get(discord::login_authorized))
    // .route("/login/google", get(google::login))
    // .route("/auth/google", get(google::login_authorized))
}
