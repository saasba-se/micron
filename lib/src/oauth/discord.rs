use std::collections::HashMap;

use axum::extract::{Query, State};
use axum::response::{IntoResponse, Redirect};
use axum::Extension;
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::PrivateCookieJar;
use mime::Mime;
use oauth2::reqwest::async_http_client;
use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse};

use serde::Deserialize;
use tracing::debug;
use uuid::Uuid;

use platform_core::auth::TokenMeta;
use platform_core::data::user::User;
use platform_core::db::{decode, encode, Database};

use crate::error::{ErrorKind, Result};
use crate::oauth::OauthClients;
use crate::{routes, AppConfig, Error};

/// User information to be retrieved from the Discord API.
///
/// # API documentation
///
/// Based on https://discord.com/developers/docs/resources/user#user-object
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
struct UserInfo {
    username: String,
    avatar_hash: String,
    bot: bool,
    verified: bool,
    email: String,
}

/// Calls `get_redirect`, which sets up a token request and returns
/// a `Redirect` to the authorization endpoint.
pub async fn login(
    State(clients): State<OauthClients>,
    State(config): State<AppConfig>,
) -> impl IntoResponse {
    if !config.registration {
        return Redirect::to(routes::SIGN_UP);
    }

    let (mut auth_url, _csrf_token) = clients
        .discord
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identify".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .url();

    // Redirect to oauth service
    Redirect::to(&auth_url.to_string())
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AuthRequest {
    code: String,
}

pub async fn login_authorized(
    cookies: PrivateCookieJar,
    Query(auth_req): Query<AuthRequest>,
    // Extension(store): Extension<MemoryStore>,
    State(oauth_clients): State<OauthClients>,
    State(db): State<Database>,
) -> Result<(PrivateCookieJar, Redirect)> {
    println!("start login callback, auth_request: {:?}", auth_req);
    // Get an auth token
    let token = oauth_clients
        .discord
        .exchange_code(AuthorizationCode::new(auth_req.code))
        .request_async(async_http_client)
        .await
        .unwrap();

    // Fetch user data
    let client = reqwest::Client::new();
    let response = client
        .get("https://discord.com/api/users/@me")
        .bearer_auth(token.access_token().secret())
        .header(USER_AGENT, format!("{}", config.name))
        .send()
        .await
        .unwrap();

    if !response.status().is_success() {
        // TODO return a proper error
        println!("response status: {:?}", response.status());
        println!("response: {:?}", response);
        // panic!();
        // return Err(anyhow::anyhow!(
        //     "got non-success status {}",
        //     response.status
        // ))?;
    }
    // println!("response: {:?}", response.text().await.unwrap());

    let user_info = response.json::<UserInfo>().await.unwrap();

    println!("user_info: {:?}", user_info);

    let mut matched_user = None;
    let mut login_user = None;

    // determine if it's a new user logging in, or if we've already seen them
    for entry in db.users.iter() {
        let (_id, _user) = entry?;
        let user_id = Uuid::from_slice(&_id)?;
        let user: User = decode(&_user)?;
        if user.email == user_info.email {
            // found user with matching email
            matched_user = Some((user_id, user));
        }
    }

    if let Some((user_id, user)) = matched_user {
        // user appears in the db (matching email)
        if !user.email_confirmed {
            // user email was not confirmed, we will overwrite that user
            // with a new one based on the oauth provider info
            let user = new_user_from_oauth(&db, user_info)?;
            db.add_user(user_id, &user)?;
            login_user = Some(user_id);
        } else {
            // user is confirmed the owner of the email, it must be the
            // same person, log in as the existing user
            //
            // add any additional information provided by oauth provider
            // to the user account
            //
            // finally let the user in
            login_user = Some(user_id);
        }
    } else {
        // user email doesn't appear in the db, treat this login as a new user
        let user = new_user_from_oauth(&db, user_info)?;
        db.add_user(user.id, &user)?;
        login_user = Some(user.id);
    }

    // log the user in
    if let Some(user_id) = login_user {
        // check if an active token exists for user
        for token in db.access_tokens.iter() {
            let (auth_token_id_bytes, auth_token) = token?;
            let auth_token_id = Uuid::from_slice(&auth_token_id_bytes)?;
            let auth_token: TokenMeta = decode(&auth_token)?;
            if auth_token.user_id == user_id {
                println!("active token exists for user");
                let updated_cookies = cookies.add(
                    Cookie::build("token", auth_token_id.to_string())
                        .same_site(SameSite::Lax)
                        .path("/")
                        .secure(true)
                        .finish(),
                );
                return Ok((updated_cookies, Redirect::to("/")));
            }
        }

        // no active token for user, generate token and set the private cookie
        let auth_token = TokenMeta::new(user_id);
        let auth_token_id = Uuid::new_v4();
        db.access_tokens.insert(auth_token_id, encode(&auth_token)?);
        let updated_cookies = cookies.add(
            Cookie::build("token", auth_token_id.to_string())
                .same_site(SameSite::Lax)
                .path("/")
                .secure(true)
                .finish(),
        );
        println!("added token cookie");
        return Ok((updated_cookies, Redirect::to("/")));
    }

    println!("cookies not updated");
    Ok((cookies, Redirect::to("/")))
}

fn new_user_from_oauth(db: &Database, user_info: UserInfo) -> Result<User> {
    let mut user = User::new(db)?;
    user.email = user_info.email.clone();
    user.email_confirmed = user_info.verified;
    user.is_disabled = false;
    user.full_name = user_info.username.clone();
    user.display_name = user_info.username.clone();
    Ok(user)
}
