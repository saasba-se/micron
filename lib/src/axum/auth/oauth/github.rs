// GitHub: "https://github.com/login/oauth/authorize", "https://github.com/login/oauth/access_token",

use axum::{
    extract::Query,
    response::{AppendHeaders, IntoResponse, Redirect, Response},
    Extension,
};
use axum_extra::extract::{CookieJar, PrivateCookieJar};
use http::{header::SET_COOKIE, HeaderMap};
use mime::Mime;
use oauth2::{reqwest::async_http_client, AuthorizationCode, CsrfToken, Scope};

use crate::Result;
use crate::{
    axum::{ConfigExt, DbExt},
    oauth::{self, Link},
    ErrorKind,
};

/// Initiates oauth2 randevous with github. Results in a redirect to provider
/// service.
pub async fn initiate(headers: HeaderMap, Extension(config): ConfigExt) -> Result<Response> {
    let config = &config;

    let client = crate::oauth::client(
        &config.oauth.github,
        config.domain.clone(),
        "github".to_string(),
        crate::oauth::github::AUTH_URL.to_string(),
        crate::oauth::github::TOKEN_URL.to_string(),
    )?;

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("user:read".to_string()))
        .url();

    // Redirect to oauth service
    if let Some(referer) = headers.get("Referer") {
        if let Ok(referer_str) = referer.to_str() {
            return Ok((
                AppendHeaders([(
                    SET_COOKIE,
                    format!("next={};SameSite=Lax;Secure;Path=/", referer_str),
                )]),
                Redirect::to(&auth_url.to_string()),
            )
                .into_response());
        }
    }
    Ok(Redirect::to(&auth_url.to_string()).into_response())
}

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    code: Option<String>,
    state: String,
    error: Option<String>,
}

/// Callback executed after provider service is done with it's part.
pub async fn callback(
    mut private_cookies: PrivateCookieJar,
    Query(query): Query<AuthRequest>,
    Extension(config): ConfigExt,
    Extension(db): DbExt,
) -> Result<Response> {
    if let Some(code) = query.code {
        match crate::oauth::github::get_user_info(code, &config, &db).await {
            Ok(user_info) => {
                let (user_id, cookie) =
                    crate::oauth::login_or_register(user_info.clone(), &db, &config).await?;

                // Link the account
                let mut user = db.get::<crate::User>(user_id)?;
                if user.linked_accounts.github.is_none() {
                    user.linked_accounts.github = Some(Link {
                        email: user_info.email,
                        handle: user_info.handle.ok_or(ErrorKind::Other(format!(
                            "github provider did not provide user handle"
                        )))?,
                    });
                    db.set(&user)?;
                }

                // Update cookies to actually log the user in
                private_cookies = private_cookies.add(cookie);

                return Ok((private_cookies, Redirect::to("/redir")).into_response());
            }
            Err(e) => {
                log::warn!("unsuccessful github oauth2: {}", e);
                Ok((private_cookies, Redirect::to("/")).into_response())
            }
        }
    } else {
        if let Some(e) = query.error {
            log::warn!("unsuccessful github oauth2: {}", e);
        }
        Ok((private_cookies, Redirect::to("/")).into_response())
    }
}
