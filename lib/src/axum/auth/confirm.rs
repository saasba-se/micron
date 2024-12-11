use axum::extract::{Path, Query};
use axum::response::Redirect;
use axum::{response::IntoResponse, Extension};
use axum_extra::extract::PrivateCookieJar;
use http::HeaderMap;
use uuid::Uuid;

use crate::auth::login::log_in_user_id;
use crate::auth::ConfirmationKey;
use crate::axum::DbExt;
use crate::{ErrorKind, Result, User};

#[derive(Debug, Deserialize)]
pub struct ConfirmData {
    key: Uuid,
}

/// Verifies the provided account confirmation token and logs the user in.
pub async fn confirm(
    Extension(db): DbExt,
    headers: HeaderMap,
    mut cookies: PrivateCookieJar,
    Path(key): Path<Uuid>,
) -> Result<(PrivateCookieJar, impl IntoResponse)> {
    // verify the key
    let key = db
        .get::<ConfirmationKey>(key)
        .map_err(|e| ErrorKind::Other("verification failed".to_string()))?;
    db.remove(&key)?;

    // set the user email as verified
    let mut user: User = db.get(key.user)?;
    user.email_confirmed = true;
    db.set(&user)?;

    // just confirming email is not enough to get the verified status
    // user.is_verified = true;

    // log the user in
    cookies = cookies.add(log_in_user_id(&user.id, &db)?);

    Ok((cookies, Redirect::to("/")))
}
