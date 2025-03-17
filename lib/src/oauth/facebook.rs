use oauth2::reqwest::async_http_client;
use oauth2::{AuthorizationCode, TokenResponse};

use crate::Result;
use crate::{Config, Database};

use super::UserInfo;

pub const AUTH_URL: &str = "https://www.facebook.com/v3.1/dialog/oauth";
pub const TOKEN_URL: &str = "https://graph.facebook.com/v3.1/oauth/access_token";

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct FacebookPicture {
    data: FacebookPictureData,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct FacebookPictureData {
    height: usize,
    width: usize,
    url: String,
}

/// User information to be retrieved from the Facebook API.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct FacebookUserInfo {
    email: String,
    name: Option<String>,
    picture: Option<FacebookPicture>,
}

impl Into<UserInfo> for FacebookUserInfo {
    fn into(self) -> UserInfo {
        UserInfo {
            email: self.email,
            full_name: self.name,
            // location: self.location,
            avatar_url: self.picture.map(|p| p.data.url),
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
        "facebook".to_string(),
        AUTH_URL.to_string(),
        TOKEN_URL.to_string(),
    )?;
    let token = client
        .exchange_code(AuthorizationCode::new(auth_code.clone()))
        .request_async(async_http_client)
        .await
        .unwrap();

    // Q: do we need to set MIME?

    // Fetch user data
    let client = reqwest::Client::new();
    let user_info: FacebookUserInfo = client
        .get("https://graph.facebook.com/me?fields=id,name,email,picture.type(large)")
        .bearer_auth(token.access_token().secret())
        .send()
        .await
        .unwrap()
        .json::<FacebookUserInfo>()
        .await
        .unwrap();

    println!("user_info: {:?}", user_info);

    Ok(user_info.into())
}
