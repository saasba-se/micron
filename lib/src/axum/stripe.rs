use std::sync::Arc;

use axum::{
    async_trait,
    body::Body,
    extract::{FromRef, FromRequest},
    response::{IntoResponse, Response},
    routing::post,
    Extension,
};
use chrono::Utc;
use http::{Request, StatusCode};

use crate::{
    order::{self, Order},
    payment::Payment,
};
use crate::{payment, Result};

use super::{DbExt, Router};

struct StripeEvent(stripe::Event);

pub fn router() -> Router {
    Router::new().route("/events/stripe", post(webhook))
}

#[async_trait]
impl<S> FromRequest<S> for StripeEvent
where
    String: FromRequest<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(
        req: Request<Body>,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let signature = if let Some(sig) = req.headers().get("stripe-signature") {
            sig.to_owned()
        } else {
            return Err(StatusCode::BAD_REQUEST.into_response());
        };

        let config = req
            .extensions()
            .get::<Arc<crate::Config>>()
            .expect("failed getting config extension");

        let signing_secret = {
            if cfg!(debug_assertions) {
                config.payments.stripe.test_signing_secret.clone()
            } else {
                config.payments.stripe.signing_secret.clone()
            }
        };

        let payload = String::from_request(req, state)
            .await
            .map_err(IntoResponse::into_response)?;

        Ok(Self(
            stripe::Webhook::construct_event(
                &payload,
                signature.to_str().unwrap(),
                &signing_secret,
            )
            .map_err(|_| StatusCode::BAD_REQUEST.into_response())?,
        ))
    }
}

async fn webhook(Extension(db): DbExt, StripeEvent(event): StripeEvent) -> Result<()> {
    use stripe::{EventObject, EventType};

    match event.type_ {
        EventType::PaymentIntentSucceeded => {
            if let EventObject::PaymentIntent(intent) = event.data.object {
                println!(
                    "received payment intent succeeded webhook with id: {:?}",
                    intent.id
                )
            }
        }
        EventType::CheckoutSessionCompleted => {
            if let EventObject::CheckoutSession(session) = event.data.object {
                println!(
                    "Received checkout session completed webhook with id: {:?}",
                    session.id
                );
                let payments = db.get_collection::<Payment>()?;
                if let Some(ref mut payment) = payments.into_iter().find(|p| {
                    p.stripe_session_id
                        .as_ref()
                        .is_some_and(|s| s == &session.id.to_string())
                }) {
                    payment.status = payment::Status::Successful { time: Utc::now() };
                    db.set(payment)?;

                    // Start fulfiling the order payment is pointing at
                    let mut order: Order = db.get(payment.order)?;
                    order.fulfill(&db).await?;
                } else {
                    // There's no payments linked to the session we got the
                    // event for, weird!
                    log::warn!("received webhook event for stripe session not linked to any pending payment");
                }
            }
        }
        EventType::AccountUpdated => {
            if let EventObject::Account(account) = event.data.object {
                println!(
                    "Received account updated webhook for account: {:?}",
                    account.id
                );
            }
        }
        _ => println!("Unknown event encountered in webhook: {:?}", event.type_),
    }

    Ok(())
}
