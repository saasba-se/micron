//! Corner-cutting example to show off the shortest path to a usable artifact.
//!
//! In practical terms it showcases very basic login functionality.

use std::str::FromStr;

use axum::{
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
    Extension,
};
use axum_extra::extract::PrivateCookieJar;
use uuid::Uuid;

use saasbase::{config, Config};

#[tokio::main]
async fn main() {
    let config = Config {
        address: std::net::SocketAddr::from_str("127.0.0.1:8001").unwrap(),
        users: vec![config::User {
            user: saasbase::User {
                email: "example@user.com".to_string(),
                ..Default::default()
            },
            ..Default::default()
        }],
        assets: config::Assets {
            serve: true,
            path: "examples/assets".to_string(),
        },
        ..Default::default()
    };

    // main application router
    let mut router = saasbase::axum::Router::new()
        .route("/", get(home))
        .route("/login", get(login));

    // attach saasbase routes
    router = saasbase::axum::router(router, &config);

    // start the application
    saasbase::axum::start(router, config).await.expect("failed")
}

async fn home(user: Option<saasbase::axum::extract::User>) -> Response {
    if let Some(user) = user {
        Html(format!(
            "welcome back {}! | credits: {} | <a href=\"/logout\">log out</a>",
            user.email, user.credits.available
        ))
        .into_response()
    } else {
        Html("landing page | <a href=\"/login\">log in</a>").into_response()
    }
}

async fn login(
    cookies: PrivateCookieJar,
    Extension(db): saasbase::axum::DbExt,
) -> (PrivateCookieJar, Response) {
    (
        cookies.add(
            saasbase::auth::login::log_in_user_id(&Uuid::nil(), &db)
                .expect("failed logging user in"),
        ),
        Redirect::to("/").into_response(),
    )
}
