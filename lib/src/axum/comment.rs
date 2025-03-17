use axum::{
    extract::{Path, Query},
    response::IntoResponse,
    routing::post,
    Extension, Form,
};
use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::{Comment, Result};

use super::{extract, ConfigExt, DbExt, Router};

pub fn router() -> Router {
    Router::new().route("/comment/:parent", post(add_comment))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommentForm {
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommentQuery {
    pub reply: Option<Uuid>,
}

pub async fn add_comment(
    Path(parent): Path<Uuid>,
    user: extract::User,
    Extension(db): DbExt,
    Extension(config): ConfigExt,
    Query(query): Query<CommentQuery>,
    Form(form): Form<CommentForm>,
) -> Result<impl IntoResponse> {
    if let Some(rate_limit_secs) = config.comments.rate_limit {
        if !user.is_admin {
            // TODO: this is really inefficient. Instead we should probably add
            // a separate data table for rate-limit "locks" per user.
            let mut comments = Comment::collection_at(parent, &db)?
                .into_iter()
                .filter(|c| c.owner == user.id)
                .collect::<Vec<_>>();
            comments.sort_by_key(|c| c.published_time);
            if let Some(comment) = comments.last() {
                if comment.published_time + Duration::seconds(rate_limit_secs as i64) > Utc::now() {
                    return Ok("Posting too frequently");
                }
            }
        }
    }

    if form.text.len() > 2000 {
        return Ok("Comment too long");
    }

    let comment = Comment {
        owner: user.id,
        parent,
        is_reply: query.reply,
        content: form.text,
        ..Default::default()
    };
    comment.store_at(parent, &db)?;

    Ok("Sent")
}
