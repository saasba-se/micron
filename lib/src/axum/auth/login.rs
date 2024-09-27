use axum::http::HeaderMap;
use axum::response::{AppendHeaders, Html, IntoResponse};
use axum::routing::{get, post};
use axum::{extract::Request, response::Redirect, Extension};
use axum::{Form, Router};
use axum_extra::extract::PrivateCookieJar;
use cookie::Cookie;

use crate::auth::TokenMeta;
use crate::error::{ErrorKind, Result};
use crate::{util, Config};

use super::super::{ConfigExt, DbExt};

/// Logout handler. Removes the token cookie and redirects to home page.
pub async fn logout(cookies: PrivateCookieJar, request: Request) -> (PrivateCookieJar, Redirect) {
    let updated_cookies = cookies.remove(Cookie::named("token"));
    (updated_cookies, Redirect::to("/"))
}

#[derive(Debug, Deserialize)]
pub struct LoginUserData {
    email: String,
    password: String,
}

pub async fn login_submit(
    Extension(db): DbExt,
    headers: HeaderMap,
    cookies: PrivateCookieJar,
    Form(user_data): Form<LoginUserData>,
) -> Result<(PrivateCookieJar, impl IntoResponse)> {
    let user = match util::find_user_by_email(&db, &user_data.email) {
        Ok(u) => u,
        Err(e) => return Ok((cookies, Html("Invalid credentials").into_response())),
    };
    // println!(
    //     "validating {:?}\n{}\n{:?}",
    //     user_id, user_data.password, user.password_hash
    // );
    if user.password_hash == None {
        // println!("passwd hash is none");
        // return Err(Error::AuthFailed("".to_string()));
        return Err(ErrorKind::AuthFailed("".to_string()).into());
    }
    if let Err(e) = crate::auth::validate_password(
        user_data.password.as_bytes(),
        &user.password_hash.clone().unwrap(),
    ) {
        // password invalid
        // println!("password didn't match: {:?}", e);
        Ok((cookies, Html("Invalid credentials").into_response()))
    } else {
        // don't let disabled users log in
        if user.is_disabled {
            // println!("user is disabled!");
            return Ok((cookies, Html("Account disabled").into_response()));
        }

        let token_meta = TokenMeta::new(user.id);
        db.set(&token_meta);

        let redir = if let Some(redir) = cookies.get("redir") {
            redir.value().to_string()
        } else {
            "/".to_string()
        };

        let new_cookies = cookies.add(Cookie::new("token", token_meta.id.to_string()));
        let new_cookies = new_cookies.remove(Cookie::named("redir"));

        Ok((
            new_cookies,
            AppendHeaders([("HX-Redirect", &redir)]).into_response(),
        ))
    }
}
