//! Full-blown demo showing capabilities of the library.
//!
//! Templates for this demo are based on the great
//! [`tabler`](https://tabler.io/) UI kit.

#![allow(warnings)]

#[macro_use]
extern crate serde_derive;

mod partial;

use askama::Template;
use axum::{
    response::{Html, IntoResponse, Response},
    routing::get,
    Extension,
};

use partial::Head;
use saasbase::{
    axum::{askama::HtmlTemplate, ConfigExt, Router},
    config, Config,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::load()?;

    let router = Router::new().route("/", get(home));
    let router = saasbase::router(router, &config);

    saasbase::axum::start(router, config).await?;

    Ok(())
}

#[derive(Template)]
#[template(path = "pages/home.html")]
pub struct Home {
    head: partial::Head,
    config: saasbase::Config,

    pub user: Option<saasbase::User>,
}

#[derive(Template)]
#[template(path = "pages/login.html")]
pub struct Login {
    head: Head,
    config: saasbase::Config,
}

async fn home(
    user: Option<saasbase::axum::extract::User>,
    Extension(config): ConfigExt,
) -> Response {
    HtmlTemplate(Home {
        head: Head {
            title: match &user {
                Some(user) => "Summary".to_string(),
                None => "Welcome to the demo".to_string(),
            },
        },
        config: config.as_ref().to_owned(),
        user: user.map(|u| u.0),
    })
    .into_response()
}
