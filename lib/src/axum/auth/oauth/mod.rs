use axum::routing::get;

use super::Router;

// TODO: linkedin, reddit
mod discord;
mod facebook;
mod github;
mod google;

pub fn router() -> Router {
    Router::new()
        .route("/login/facebook", get(facebook::initiate))
        .route("/auth/facebook", get(facebook::callback))
        .route("/login/github", get(github::initiate))
        .route("/auth/github", get(github::callback))
        .route("/login/discord", get(discord::initiate))
        .route("/auth/discord", get(discord::callback))
        .route("/login/google", get(google::initiate))
        .route("/auth/google", get(google::callback))
}
