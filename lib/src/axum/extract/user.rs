use std::backtrace::Backtrace;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;
use std::sync::Arc;

use axum::extract::{FromRef, FromRequest, FromRequestParts};
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::http::{Request, StatusCode};
use axum::response::Redirect;
use axum::{async_trait, Extension};
use axum_auth::AuthBearer;
use axum_extra::extract::cookie::Key as CookieKey;
use axum_extra::extract::PrivateCookieJar;
use chrono::Utc;
use log::debug;
use tracing::{error, warn};
use uuid::Uuid;

use crate::auth::TokenMeta;
use crate::db::{decode, Database};
use crate::error::{Error, ErrorKind};
use crate::user::User as RawUser;
use crate::util::token_expired;
use crate::Config;

#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub struct User(pub RawUser);

impl Deref for User {
    type Target = RawUser;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for User {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<RawUser> for User {
    fn from(u: RawUser) -> Self {
        Self(u)
    }
}

impl Into<RawUser> for User {
    fn into(self) -> RawUser {
        self.0
    }
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for User
where
    // Database: FromRef<S>,
    // Config: FromRef<S>,
    CookieKey: FromRef<S>,
{
    type Rejection = Error;

    async fn from_request_parts(mut parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let db = parts
            .extensions
            .get::<Arc<Database>>()
            .expect("database extension unavailable")
            .clone();
        let config = parts
            .extensions
            .get::<Arc<Config>>()
            .expect("config extension unavailable")
            .clone();

        // autologin functionality for faster development, can be set in config
        if let Some(autologin_email) = &config.dev.autologin {
            debug!("attempting autologin, uri: {}", parts.uri);
            let users = db.get_collection::<RawUser>()?;
            if let Some(user) = users.into_iter().find(|u| &u.email == autologin_email) {
                return Ok(User(user));
            } else {
                return Err(ErrorKind::AuthFailed(format!(
                    "autologin: provided user email that doesn't exist: {}",
                    autologin_email
                ))
                .into());
            }
        }

        // first see if the bearer token is presented with authorization header
        let token = if let Ok(token) = AuthBearer::from_request_parts(parts, state).await {
            token.0
        } else {
            // otherwise try accessing cookie jar and extracting the token cookie
            let jar: PrivateCookieJar<CookieKey> =
                PrivateCookieJar::from_request_parts(&mut parts, state)
                    .await
                    .unwrap();

            let cookie = jar
                .get("token")
                .ok_or(ErrorKind::FailedGettingTokenCookie(parts.uri.clone()))?;

            cookie.value().to_string()
        };

        let token = db.get::<TokenMeta>(Uuid::from_str(&token)?).map_err(|_| {
            Error::new(ErrorKind::AuthFailed(
                "failed getting token meta from db".to_string(),
            ))
        })?;

        // check if token hasn't expired
        if token_expired(&db, &token) {
            return Err(ErrorKind::AuthFailed("token expired".to_string()).into());
        }

        db.get::<RawUser>(token.user_id).map(|u| User(u))
    }
}

pub struct UserId {
    pub id: Uuid,
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for UserId
where
    Database: FromRef<S>,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts.headers.get(AUTHORIZATION.as_str());
        // access token as seen in the auth header
        let token = if let Some(header) = auth_header {
            let header_str = String::from_utf8_lossy(&header.as_bytes());

            // perhaps header is still "Bearer [token]", we need to split it
            let split = header_str.split(' ').collect::<Vec<&str>>();
            Uuid::from_str(&split[1])?

            // Uuid::from_str(&header_str)?
        } else {
            return Err(ErrorKind::AuthFailed("auth header not present".to_string()).into());
        };

        // let db = parts
        //     .extensions
        //     .get::<Database>()
        //     .expect("failed getting db");
        let db = Database::from_ref(state);

        // expand the token to include its meta information
        let token = db.get::<TokenMeta>(token)?;

        // check if token hasn't expired
        if token_expired(&db, &token) {
            return Err(ErrorKind::AuthFailed("token expired".to_string()).into());
        }

        let user_pointer = UserId { id: token.user_id };

        Ok(user_pointer)
    }
}
