//! Example showing off basics of working with oauth2 to easily provide ability
//! for users to both register and log in using external oauth2 providers.
//!
//! # Setup
//!
//! To make this example work it's necessary to first:
//! 1. Create an oauth app with github, set the callback url to
//!    `http://{domain}/auth/github`. Setting `domain` to `127.0.0.1:8000`
//!    works fine in this case.
//! 2. Input proper `client_id` and `client_secret` values into the config.

use axum::{
    response::{Html, IntoResponse, Response},
    routing::get,
};

use saasbase::{config, Config};

#[tokio::main]
async fn main() {
    let config = Config {
        domain: "127.0.0.1:8000".to_string(),
        registration: config::Registration {
            enabled: true,
            oauth: true,
            ..Default::default()
        },
        auth: config::Auth {},
        oauth: config::Oauth {
            enabled: true,
            github: config::OauthEntry {
                client_id: "c31f7946cd7ccf47d3d2".to_string(),
                client_secret: "8a1b86e774f42222d26ed1c08e970f3dad58c167".to_string(),
            },
            ..Default::default()
        },
        ..Default::default()
    };

    // main application router
    let mut router = saasbase::axum::Router::new().route("/", get(home));

    // attach saasbase routes
    router = saasbase::axum::router(router, &config);

    // start the application
    saasbase::axum::start(router, config).await.expect("failed")
}

async fn home(user: Option<saasbase::axum::extract::User>) -> Response {
    if let Some(user) = user {
        Html(format!(
            "welcome {}! | credits: {} | <a href=\"/logout\">log out</a>",
            user.email, user.credits.available
        ))
        .into_response()
    } else {
        Html("landing page | <a href=\"/login/github\">log in with github</a>").into_response()
    }
}
