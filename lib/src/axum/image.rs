use axum::extract::Path;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Extension;

use crate::{Image, Result};
use crate::{ImageId, Router};

use super::DbExt;

pub fn router() -> Router {
    Router::new().route("/image/:id", get(image))
}

pub async fn image(Path(id): Path<ImageId>, Extension(db): DbExt) -> Result<impl IntoResponse> {
    let image = db.get::<Image>(id)?;
    Ok((
        axum::response::AppendHeaders([(axum::http::header::CONTENT_TYPE, "image/jpeg")]),
        image.bytes,
    ))
}
