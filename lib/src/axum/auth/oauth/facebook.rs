use axum::{
    extract::Query,
    response::{IntoResponse, Redirect},
    Extension,
};
use axum_extra::extract::PrivateCookieJar;
use mime::Mime;
use oauth2::{reqwest::async_http_client, AuthorizationCode, CsrfToken, Scope};

use crate::axum::{ConfigExt, DbExt};
use crate::{routes, Result};

/// Initiates oauth2 randevous with facebook. Results in a redirect to provider
/// service.
pub async fn initiate(Extension(config): ConfigExt) -> Result<impl IntoResponse> {
    let config = &config;

    let client = crate::oauth::client(
        &config.oauth.facebook,
        config.domain.clone(),
        "facebook".to_string(),
        crate::oauth::facebook::AUTH_URL.to_string(),
        crate::oauth::facebook::TOKEN_URL.to_string(),
    )?;

    let (auth_url, _csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("public_profile".to_string()))
        .url();

    // Redirect to oauth service
    Ok(Redirect::to(&auth_url.to_string()))
}

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
    code: Option<String>,
    state: String,
    error: Option<String>,
}

/// Callback executed after provider service is done with it's part.
pub async fn callback(
    cookies: PrivateCookieJar,
    Query(query): Query<AuthRequest>,
    Extension(config): ConfigExt,
    Extension(db): DbExt,
) -> Result<(PrivateCookieJar, Redirect)> {
    if let Some(code) = query.code {
        if let Ok(user_info) = crate::oauth::facebook::get_user_info(code, &config, &db).await {
            if let Ok(cookie) = crate::oauth::login_or_register(user_info, &db, &config).await {
                let updated_cookies = cookies.add(cookie);
                return Ok((updated_cookies, Redirect::to("/")));
            } else {
                Ok((cookies, Redirect::to("/")))
            }
        } else {
            Ok((cookies, Redirect::to("/")))
        }
    } else {
        if let Some(e) = query.error {
            log::warn!("unsuccessful facebook oauth2: {}", e);
        }
        Ok((cookies, Redirect::to("/")))
    }
}
