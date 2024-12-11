// use std::collections::HashMap;

// use axum::extract::{Query, State};
// use axum::response::{IntoResponse, Redirect};
// use axum::Extension;
// use axum_extra::extract::cookie::{Cookie, SameSite};
// use axum_extra::extract::PrivateCookieJar;
// use mime::Mime;
// use oauth2::reqwest::async_http_client;
// use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse};

// use serde::Deserialize;
// use uuid::Uuid;

// use crate::auth::TokenMeta;
// use crate::db::{decode, encode, Database};
// use crate::error::{Error, Result};
// use crate::oauth::OauthClients;
// // use crate::user::User;
// use crate::Config;

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

// /// Calls `get_redirect`, which sets up a token request and returns
// /// a `Redirect` to the authorization endpoint.
// pub async fn login(
//     State(clients): State<OauthClients>,
//     State(config): State<Config>,
// ) -> impl IntoResponse {
//     if !config.registration {
//         return Redirect::to("sign_up");
//     }

//     let (mut auth_url, _csrf_token) = clients
//         .google
//         .authorize_url(CsrfToken::new_random)
//         .add_scope(Scope::new(
//             "https://www.googleapis.com/auth/userinfo.email".to_string(),
//         ))
//         .add_scope(Scope::new(
//             "https://www.googleapis.com/auth/userinfo.profile".to_string(),
//         ))
//         .url();

//     // if config.development {
//     //     auth_url.set_host(Some("127.0.0.1"));
//     // }
//     // println!("redirecting to: {}", auth_url);

//     // Redirect to oauth service
//     Redirect::to(&auth_url.to_string())
// }

// #[derive(Debug, Deserialize)]
// #[allow(dead_code)]
// pub struct AuthRequest {
//     code: String,
//     prompt: String,
//     authuser: String,
//     state: String,
// }

// pub async fn login_authorized(
//     cookies: PrivateCookieJar,
//     Query(auth_req): Query<AuthRequest>,
//     // Extension(store): Extension<MemoryStore>,
//     State(oauth_clients): State<OauthClients>,
//     State(db): State<Database>,
// ) -> Result<(PrivateCookieJar, Redirect)> {
//     println!("start login callback, auth_request: {:?}", auth_req);
//     // Get an auth token
//     let token = oauth_clients
//         .google
//         .exchange_code(AuthorizationCode::new(auth_req.code))
//         .request_async(async_http_client)
//         .await
//         .unwrap();

//     // Fetch user data from Google
//     let client = reqwest::Client::new();
//     let response = client
//         .get("https://people.googleapis.com/v1/people/me?personFields=names,emailAddresses")
//         .bearer_auth(token.access_token().secret())
//         .header(USER_AGENT, format!("{}", config.name))
//         .send()
//         .await
//         .unwrap();

//     if !response.status().is_success() {
//         // TODO return a proper error
//         println!("response status: {:?}", response.status());
//         println!("response: {:?}", response);
//         // panic!();
//         // return Err(anyhow::anyhow!(
//         //     "got non-success status {}",
//         //     response.status
//         // ))?;
//     }
//     // println!("response: {:?}", response.text().await.unwrap());

//     let user_info = response.json::<GoogleUserInfo>().await.unwrap();

//     println!("user_info: {:?}", user_info);

//     let mut matched_user = None;
//     let mut login_user = None;

//     // determine if it's a new user logging in, or if we've already seen them
//     for entry in db.users.iter() {
//         let (_id, _user) = entry?;
//         let user_id = Uuid::from_slice(&_id)?;
//         let user: User = decode(&_user)?;
//         if user.email == user_info.email_addresses.first().unwrap().value {
//             // found user with matching email
//             matched_user = Some((user_id, user));
//         }
//     }

//     if let Some((user_id, user)) = matched_user {
//         // user appears in the db (matching email)
//         if !user.email_confirmed {
//             // user email was not confirmed, we will overwrite that user
//             // with a new one based on the oauth provider info
//             let user = new_user_from_oauth(&db, user_info)?;
//             db.add_user(user_id, &user)?;
//             login_user = Some(user_id);
//         } else {
//             // user is confirmed the owner of the email, it must be the
//             // same person, log in as the existing user
//             //
//             // add any additional information provided by oauth provider
//             // to the user account
//             //
//             // finally let the user in
//             login_user = Some(user_id);
//         }
//     } else {
//         // user email doesn't appear in the db, treat this login as a new user
//         let user = new_user_from_oauth(&db, user_info)?;
//         db.add_user(user.id, &user)?;
//         login_user = Some(user.id);
//     }

//     // log the user in
//     if let Some(user_id) = login_user {
//         // check if an active token exists for user
//         for token in db.access_tokens.iter() {
//             let (auth_token_id_bytes, auth_token) = token?;
//             let auth_token_id = Uuid::from_slice(&auth_token_id_bytes)?;
//             let auth_token: TokenMeta = decode(&auth_token)?;
//             if auth_token.user_id == user_id {
//                 println!("active token exists for user");
//                 let updated_cookies = cookies.add(
//                     Cookie::build("token", auth_token_id.to_string())
//                         .same_site(SameSite::Lax)
//                         .path("/")
//                         .secure(true)
//                         .finish(),
//                 );
//                 return Ok((updated_cookies, Redirect::to("/")));
//             }
//         }

//         // no active token for user, generate token and set the private cookie
//         let auth_token = TokenMeta::new(user_id);
//         let auth_token_id = Uuid::new_v4();
//         db.access_tokens.insert(auth_token_id, encode(&auth_token)?);
//         let updated_cookies = cookies.add(
//             Cookie::build("token", auth_token_id.to_string())
//                 .same_site(SameSite::Lax)
//                 .path("/")
//                 .secure(true)
//                 .finish(),
//         );
//         println!("added token cookie");
//         return Ok((updated_cookies, Redirect::to("/")));
//     }

//     println!("cookies not updated");
//     Ok((cookies, Redirect::to("/")))
// }

// fn new_user_from_oauth(db: &Database, user_info: GoogleUserInfo) -> Result<User> {
//     let mut user = User::new(db)?;
//     user.email = user_info.email_addresses.first().unwrap().value.clone();
//     // user.email_confirmed = true;
//     user.email_confirmed = user_info.email_addresses.first().unwrap().metadata.verified;
//     user.is_disabled = false;
//     let name = user_info
//         .names
//         .iter()
//         .find(|n| n.metadata.primary)
//         .ok_or::<Error>(ErrorKind::Other("no primary name for user".to_string()).into())?;
//     user.full_name = format!("{}{}", name.given_name, name.family_name);
//     user.display_name = user_info.names.first().unwrap().display_name.clone();
//     Ok(user)
// }
