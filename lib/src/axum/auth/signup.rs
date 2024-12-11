use axum::response::{AppendHeaders, Html, IntoResponse};
use axum::{Extension, Form};
use axum_extra::extract::PrivateCookieJar;
use cookie::Cookie;
use http::HeaderMap;
use uuid::Uuid;
use validator::{ValidateEmail, ValidateLength};

use crate::auth::{ConfirmationKey, TokenMeta};
use crate::axum::{ConfigExt, DbExt};
use crate::{util, ErrorKind, Result, User};

#[derive(Debug, Deserialize)]
pub struct SignupUserData {
    email: String,
    password: String,
    #[serde(default)]
    consent: bool,
}

pub async fn signup(
    Extension(db): DbExt,
    Extension(config): ConfigExt,
    headers: HeaderMap,
    mut cookies: PrivateCookieJar,
    Form(user_data): Form<SignupUserData>,
) -> Result<(PrivateCookieJar, impl IntoResponse)> {
    // validate inputs
    if !user_data.email.validate_email() {
        return Err(ErrorKind::BadInput("invalid email".to_string()).into());
    }

    if !user_data.password.validate_length(Some(8), Some(24), None) {
        return Err(ErrorKind::BadInput("invalid password length".to_string()).into());
    }

    let mut user = User::new(&db)?;

    // create a new user entry with unverified email status
    user.email = user_data.email;
    user.password_hash = Some(crate::auth::hash_password(&user_data.password)?);
    db.set(&user)?;

    // create a new verification key item and store it
    let key = ConfirmationKey {
        user: user.id,
        key: Uuid::new_v4(),
    };
    db.set(&key)?;

    // send email with the code
    crate::email::confirmation(user.email, key.key.to_string(), &config)?;

    // depending on configuration let the user in or require verification
    if config.auth.require_confirmed_email {
        // redirect to the page instructing user to click the link from email
        Ok((cookies, AppendHeaders([("HX-Redirect", "/verify")])))
    } else {
        // login the user in
        cookies = cookies.add(crate::auth::login::log_in_user_id(&user.id, &db)?);
        Ok((cookies, AppendHeaders([("HX-Redirect", "/")])))
    }
}
