use oauth2::reqwest::async_http_client;
use oauth2::{AuthorizationCode, TokenResponse};

use crate::Result;
use crate::{Config, Database};

use super::UserInfo;

pub const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
pub const TOKEN_URL: &str = "https://www.googleapis.com/oauth2/v4/token";

/// User information to be retrieved from the Google API.
///
/// # API documentation
///
/// Based on https://developers.google.com/people/api/rest/v1/people#Person
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct GoogleUserInfo {
    names: Vec<GoogleName>,
    email_addresses: Vec<GoogleEmail>,
    photos: Vec<GooglePhoto>,
}

impl Into<UserInfo> for GoogleUserInfo {
    fn into(self) -> UserInfo {
        UserInfo {
            email: self.email_addresses.first().unwrap().value.clone(),
            full_name: Some(format!(
                "{} {}",
                self.names.first().unwrap().given_name,
                self.names.first().unwrap().family_name,
            )),
            avatar_url: self.photos.first().map(|p| p.url.clone()),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct GoogleMetadata {
    pub primary: bool,
    pub verified: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct GoogleName {
    pub metadata: GoogleMetadata,
    pub display_name: String,
    pub given_name: String,
    pub family_name: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct GoogleEmail {
    pub metadata: GoogleMetadata,
    pub value: String,
    pub r#type: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct GooglePhoto {
    pub metadata: GoogleMetadata,
    pub url: String,
    pub default: bool,
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
        &config.oauth.google,
        config.domain.clone(),
        "google".to_string(),
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
    let user_info: GoogleUserInfo = client
        .get("https://people.googleapis.com/v1/people/me?personFields=names,emailAddresses,photos")
        .bearer_auth(token.access_token().secret())
        .send()
        .await
        .unwrap()
        .json::<GoogleUserInfo>()
        .await
        .unwrap();

    println!("user_info: {:?}", user_info);

    Ok(user_info.into())
}
