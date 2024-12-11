// GitHub: "https://github.com/login/oauth/authorize", "https://github.com/login/oauth/access_token",

use std::sync::Arc;

use cookie::Cookie;
use http::header::{ACCEPT, USER_AGENT};
use mime::Mime;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use tracing::debug;

use serde::Deserialize;
use uuid::Uuid;

use crate::auth::login::log_in_user_id;
use crate::auth::TokenMeta;
use crate::config;
use crate::db::{decode, encode, Database};
use crate::error::Result;
use crate::user::User;
use crate::{routes, Config};

use super::UserInfo;

pub const AUTH_URL: &str = "https://github.com/login/oauth/authorize";
pub const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";

/// User information to be retrieved from the GitHub API.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct GitHubUserInfo {
    login: String,
    email: String,
    name: Option<String>,
    location: Option<String>,
    avatar_url: Option<String>,
}

impl Into<UserInfo> for GitHubUserInfo {
    fn into(self) -> UserInfo {
        UserInfo {
            email: self.email,
            handle: Some(self.login),
            full_name: self.name,
            location: self.location,
            avatar_url: self.avatar_url,
            ..Default::default()
        }
    }
}

/// Constructs common user info using auth code sent by the service provider.
///
/// Includes additional calls to target service to get user information.
pub async fn get_user_info<'c>(
    auth_code: String,
    config: &Config,
    db: &Database,
) -> Result<UserInfo> {
    // Get an auth token
    let client = crate::oauth::client(
        &config.oauth.github,
        config.domain.clone(),
        "github".to_string(),
        AUTH_URL.to_string(),
        TOKEN_URL.to_string(),
    )?;
    let token = client
        .exchange_code(AuthorizationCode::new(auth_code.clone()))
        .request_async(async_http_client)
        .await
        .unwrap();

    let mime: Mime = "application/vnd.github.v3+json"
        .parse()
        .expect("parse GitHub MIME type");

    // Fetch user data
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/user")
        .bearer_auth(token.access_token().secret())
        .header(ACCEPT, mime.essence_str())
        .header(USER_AGENT, format!("{}", config.name))
        .send()
        .await
        .unwrap();

    let user_info: GitHubUserInfo = response.json::<GitHubUserInfo>().await.unwrap();
    println!("user_info: {:?}", user_info);

    Ok(user_info.into())
}
