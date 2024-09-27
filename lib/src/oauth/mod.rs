// pub mod discord;
pub mod facebook;
pub mod github;
// pub mod google;

use std::env;
use std::sync::Arc;

use axum::http::Uri;
use axum::routing::{get, post};
use axum::Router;
use cookie::Cookie;
use oauth2::basic::BasicClient;
use oauth2::url::Url;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};

use crate::auth::login::log_in_user_id;
use crate::Database;
use crate::{config, User};
use crate::{Config, ErrorKind, Result};

pub fn client(
    config: &config::OauthEntry,
    domain: String,
    provider: String,
    auth_url: String,
    token_url: String,
) -> Result<BasicClient> {
    let client = BasicClient::new(
        ClientId::new(config.client_id.to_owned()),
        Some(ClientSecret::new(config.client_secret.clone())),
        AuthUrl::new(auth_url)?,
        Some(TokenUrl::new(token_url)?),
    )
    .set_redirect_uri(RedirectUrl::new(format!(
        "https://{domain}/auth/{provider}"
    ))?);
    Ok(client)
}

/// Set of data points that can be extracted from oauth providers and
/// integrated into our model.
#[derive(Clone, Default)]
pub struct UserInfo {
    email: String,
    full_name: Option<String>,
    display_name: Option<String>,
    location: Option<String>,
    avatar_url: Option<String>,
}

/// Determines how to proceed after successful oauth procedure.
pub async fn login_or_register<'c>(
    user_info: UserInfo,
    db: &Database,
    config: &Config,
) -> Result<Cookie<'c>> {
    let mut matched_user = None;

    // determine if it's a new user logging in, or if we've already seen them
    for user in db.get_collection::<User>()? {
        if user.email == user_info.email {
            // found user with matching email
            matched_user = Some(user);
            break;
        }
    }

    // user appears in the db (matching email)
    if let Some(user) = matched_user {
        // user email was not confirmed, we will overwrite that user
        // with a new one based on the oauth provider info
        if !user.email_confirmed {
            let user = new_user_from_oauth(&db, user_info).await?;
            db.set(&user)?;

            return log_in_user_id(&user.id, db);
        } else {
            // user is confirmed the owner of the email, it must be the
            // same person, log in as the existing user

            // TODO: add any additional information provided by oauth provider
            // to the user account

            // let the user in
            println!("logging as the existing user: {:?}", user.id);
            return log_in_user_id(&user.id, db);
        }
    } else {
        // user email doesn't appear in the db, treat this login as a new user

        // return immediately if config dissalows registration in general, or
        // through oauth specifically
        if !config.registration.enabled || !config.registration.oauth {
            return Err(ErrorKind::RegistrationClosed(
                "can't create new user based on valid oauth process".to_string(),
            )
            .into());
        }

        let user = new_user_from_oauth(&db, user_info).await?;
        db.set(&user)?;
        return log_in_user_id(&user.id, db);
    }
}

/// Attempts to fit information from oauth provider into a new user structure.
pub async fn new_user_from_oauth(db: &Database, user_info: UserInfo) -> Result<User> {
    // let name_split = user_info
    //     .full_name
    //     .rsplitn(1, " ")
    //     .map(|s| s.to_string())
    //     .collect::<Vec<String>>();
    let mut user = User::new(db)?;
    user.email = user_info.email.clone();
    user.email_confirmed = true;
    user.is_disabled = false;
    user.full_name = user_info.full_name.unwrap_or("".to_string());
    user.display_name = user_info.display_name.unwrap_or(user_info.email);
    if let Some(avatar_url) = user_info.avatar_url {
        user.set_avatar_from_url(db, &avatar_url).await?;
    }
    Ok(user)
}

// Discord: "https://discordapp.com/api/oauth2/authorize", "https://discordapp.com/api/oauth2/token",
// Google: "https://accounts.google.com/o/oauth2/v2/auth", "https://www.googleapis.com/oauth2/v4/token",
// Microsoft: "https://login.microsoftonline.com/common/oauth2/v2.0/authorize", "https://login.microsoftonline.com/common/oauth2/v2.0/token",
// Reddit: "https://www.reddit.com/api/v1/authorize", "https://www.reddit.com/api/v1/access_token",
// Wikimedia: "https://meta.wikimedia.org/w/rest.php/oauth2/authorize", "https://meta.wikimedia.org/w/rest.php/oauth2/access_token",
// Yahoo: "https://api.login.yahoo.com/oauth2/request_auth", "https://api.login.yahoo.com/oauth2/get_token",
