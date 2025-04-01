use http::header::USER_AGENT;
use oauth2::reqwest::async_http_client;
use oauth2::{AuthorizationCode, TokenResponse};

use crate::Result;
use crate::{Config, Database};

use super::UserInfo;

pub const AUTH_URL: &str = "https://discordapp.com/api/oauth2/authorize";
pub const TOKEN_URL: &str = "https://discordapp.com/api/oauth2/token";

/// User information to be retrieved from the Discord API.
///
/// # API documentation
///
/// Based on https://discord.com/developers/docs/resources/user#user-object
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct DiscordUserInfo {
    username: String,
    // This is a hash for a specific avatar belonging to the user
    avatar: String,
    bot: bool,
    verified: bool,
    email: String,
    // We need the id to retrieve user avatar image
    id: String,
}

impl Into<UserInfo> for DiscordUserInfo {
    fn into(self) -> UserInfo {
        UserInfo {
            email: self.email,
            // With discord we never get reliable name information
            full_name: None,
            handle: Some(self.username),
            avatar_url: Some(format!(
                "https://cdn.discordapp.com/avatars/{}/{}.png?size=160",
                self.id, self.avatar
            )),
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
        &config.oauth.facebook,
        config.domain.clone(),
        "discord".to_string(),
        AUTH_URL.to_string(),
        TOKEN_URL.to_string(),
    )?;
    let token = client
        .exchange_code(AuthorizationCode::new(auth_code.clone()))
        .request_async(async_http_client)
        .await
        .unwrap();

    // Fetch user data
    let client = reqwest::Client::new();
    let user_info = client
        .get("https://discord.com/api/users/@me")
        .bearer_auth(token.access_token().secret())
        .header(USER_AGENT, format!("{}", config.domain))
        .send()
        .await?
        .json::<DiscordUserInfo>()
        .await?;

    println!("user_info: {:?}", user_info);

    Ok(user_info.into())
}
