use axum::http::HeaderMap;
use axum::response::{AppendHeaders, Html, IntoResponse};
use axum::routing::{get, post};
use axum::{extract::Request, response::Redirect, Extension};
use axum::{Form, Router};
use axum_extra::extract::PrivateCookieJar;
use cookie::Cookie;

use crate::auth::TokenMeta;
use crate::error::{ErrorKind, Result};
use crate::{util, Config, Error};

use super::super::{ConfigExt, DbExt};

/// Logout handler. Removes the token cookie and redirects to home page.
pub async fn logout(
    mut cookies: PrivateCookieJar,
    request: Request,
) -> (PrivateCookieJar, Redirect) {
    cookies = cookies.remove(Cookie::named("token"));
    (cookies, Redirect::to("/"))
}

#[derive(Debug, Deserialize)]
pub struct LoginData {
    email: String,
    password: String,
}

/// Processes login form data and logs the user in.
pub async fn login(
    Extension(db): DbExt,
    headers: HeaderMap,
    mut cookies: PrivateCookieJar,
    Form(user_data): Form<LoginData>,
) -> Result<(PrivateCookieJar, impl IntoResponse)> {
    let user = match util::find_user_by_email(&db, &user_data.email) {
        Ok(u) => u,
        Err(e) => {
            log::trace!("didn't find user by email: {e}, trying to find by handle...");
            match util::find_user_by_handle(&db, &user_data.email) {
                Ok(u) => u,
                Err(e) => return Err(ErrorKind::InvalidCredentials.into()),
            }
        }
    };
    if user.password_hash == None {
        return Err(Error::new_with(
            ErrorKind::PasswordNotSet,
            None,
            Some(user.id),
        ));
    }
    if let Err(e) = crate::auth::validate_password(
        user_data.password.as_bytes(),
        &user.password_hash.clone().unwrap(),
    ) {
        Err(ErrorKind::InvalidCredentials.into())
    } else {
        // don't let disabled users log in
        if user.is_disabled {
            return Err(Error::new_with(
                ErrorKind::AccountDisabled,
                None,
                Some(user.id),
            ));
        }

        let redir = if let Some(redir) = cookies.get("redir") {
            redir.value().to_string()
        } else {
            "/".to_string()
        };

        cookies = cookies.add(crate::auth::login::log_in_user_id(&user.id, &db)?);
        cookies = cookies.remove(Cookie::named("redir"));

        Ok((
            cookies,
            AppendHeaders([("HX-Redirect", &redir)]).into_response(),
        ))
    }
}
