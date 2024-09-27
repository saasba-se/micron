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
    // main application router
    let mut router = Router::new().route("/", get(home));

    saasbase::axum::start(router, config::load()?).await?;

    Ok(())
}

#[derive(Template)]
#[template(path = "pages/summary.html")]
pub struct Summary {
    head: partial::Head,
    config: saasbase::Config,

    pub user: saasbase::User,
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
    if let Some(user) = user {
        HtmlTemplate(Summary {
            head: Head {
                title: "Summary".to_string(),
            },
            config: config.as_ref().to_owned(),
            user: user.0,
        })
        .into_response()
    } else {
        HtmlTemplate(Login {
            head: Head {
                title: "Login".to_string(),
            },
            config: config.as_ref().to_owned(),
        })
        .into_response()
    }
}
