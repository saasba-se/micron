use std::collections::HashSet;

use axum::{
    extract::{Path, Query},
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Extension, Form,
};
use http::HeaderMap;
use uuid::Uuid;

use crate::{auth::ConfirmationKey, email::list::Subscriber, ErrorKind};
use crate::{Result, Router};

use super::{ConfigExt, DbExt};

pub fn router() -> Router {
    Router::new()
        .route("/mailing/subscribe", post(subscribe))
        .route("/mailing/unsubscribe", get(unsubscribe))
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SubscribeForm {
    pub email: String,
    pub lists: Option<HashSet<String>>,
}

pub async fn subscribe(
    Extension(db): DbExt,
    Extension(config): ConfigExt,
    Form(form): Form<SubscribeForm>,
) -> Result<impl IntoResponse> {
    let mut subscriber = Subscriber::new(form.email);
    // Consent is given as the caller is actively trying to register to a list
    subscriber.marketing_consent = true;
    // Add subscriber to provided selection of lists or to all of them if no
    // selection is specified
    subscriber.lists = form.lists.unwrap_or(config.mailing.lists.clone());

    db.set(&subscriber)?;

    // Perform email confirmation if required
    if config.mailing.confirmation {
        // send subscription confirmation email
        crate::email::mailing_confirmation(subscriber.address, subscriber.id.to_string(), &config)?;
    }

    Ok(())
}

/// Verifies the provided key, which is also the subscriber id.
pub async fn confirm(
    Extension(db): DbExt,
    headers: HeaderMap,
    Path(key): Path<Uuid>,
) -> Result<impl IntoResponse> {
    // verify the key
    let mut subscriber = db
        .get::<Subscriber>(key)
        .map_err(|e| ErrorKind::Other("mailing subscriber verification failed".to_string()))?;

    // set the subscriber as confirmed
    subscriber.confirmed = true;

    // TODO: show notification to user that verification was successful
    Ok(Redirect::to("/"))
}

pub async fn unsubscribe(
    Extension(db): DbExt,
    Path((subscriber, lists)): Path<(Uuid, Option<HashSet<String>>)>,
) -> Result<impl IntoResponse> {
    let mut sub: Subscriber = db.get(subscriber)?;

    // Remove subscriber from selected lists
    if let Some(lists) = lists {
        sub.lists.retain(|list| !lists.contains(list));
        db.set(&sub)?;
    }
    // If no lists are speficied in the request then remove the subscriber
    else {
        db.remove(&sub)?;
    }

    Ok(())
}
