use std::sync::Arc;

// use axum::extract::{Path, Query, Request};
// use axum::http::HeaderMap;
// use axum::response::{AppendHeaders, Html, IntoResponse, Redirect};
// use axum::{Extension, Form};
// use axum_extra::extract::PrivateCookieJar;
use cookie::{Cookie, CookieJar, PrivateJar, SameSite};
use serde_json::json;
use uuid::Uuid;

use crate::db::{decode, encode};
use crate::error::{ErrorKind, Result};
use crate::{util, Config, Database, User, UserId};

use super::TokenMeta;

/// Generates a cookie for logging in user with user email.
pub fn log_in_user_email<'c>(user_email: &str, db: &Database) -> Result<Cookie<'c>> {
    let users = db.get_collection::<User>()?;

    let mut matched_user_id = users.iter().find(|u| &u.email == user_email).map(|u| u.id);

    if let Some(user_id) = matched_user_id {
        return log_in_user_id(&user_id, db);
    } else {
        return Err(ErrorKind::UserNotFound(format!("email: {}", user_email)).into());
    }
}

/// Generates a cookie for logging in user by user id.
pub fn log_in_user_id<'c>(user_id: &UserId, db: &Database) -> Result<Cookie<'c>> {
    let tokens = db.get_collection::<TokenMeta>()?;

    // check if an active token exists for user
    for auth_token in tokens {
        if &auth_token.user_id == user_id {
            return Ok(Cookie::build(("token", auth_token.id.to_string()))
                .same_site(SameSite::Lax)
                .path("/")
                .secure(true)
                .finish());
        }
    }

    // no active token for user, generate token and create the cookie
    let auth_token = TokenMeta::new(user_id.clone());

    db.set(&auth_token)?;

    return Ok(Cookie::build(("token", auth_token.id.to_string()))
        .same_site(SameSite::Lax)
        .path("/")
        .secure(true)
        .finish());
}
