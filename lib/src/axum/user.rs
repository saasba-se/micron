use std::sync::Arc;

use axum::extract::Path;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Extension;

use crate::db::Database;
use crate::Result;
use crate::{Image, User, UserId};

use super::extract;
use super::{DbExt, Router};

pub fn router() -> Router {
    Router::new()
        .route("/avatar", get(my_avatar))
        .route("/avatar/:user_id", get(avatar))
}

pub async fn my_avatar(user: extract::User, Extension(db): DbExt) -> Result<impl IntoResponse> {
    let image = db.get::<Image>(user.avatar)?;
    Ok((
        axum::response::AppendHeaders([(axum::http::header::CONTENT_TYPE, "image/png")]),
        image.bytes,
    ))
}

pub async fn avatar(
    Path(user_id): Path<UserId>,
    Extension(db): DbExt,
) -> Result<impl IntoResponse> {
    let user = db.get::<User>(user_id)?;
    let image = db.get::<Image>(user.avatar)?;
    Ok((
        axum::response::AppendHeaders([(axum::http::header::CONTENT_TYPE, "image/png")]),
        image.bytes,
    ))
}
