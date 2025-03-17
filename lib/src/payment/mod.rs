#[cfg(feature = "stripe")]
pub mod stripe;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    db::{Collectable, Identifiable},
    order::OrderId,
    Result, UserId,
};

pub type PaymentId = uuid::Uuid;

/// Payment instance object.
///
/// Payment is attached to each order and enables tracking of whether user has
/// paid for the order they made.
///
///
/// # Prepaid credits
///
/// The most basic payment kind is credits-based one. To some extent it allows
/// handling payments without third-party payment processors.
///
///
/// # Stripe payments
///
/// `micron` payments are currently inextricably linked to stripe payments
/// processor.
///
/// Each initiated payment is translated to a stripe checkout session. In the
/// background every possibly-paying user is also "mirrored" with the stripe
/// system as a *customer*.
///
/// Each payment can be translated to a stripe checkout session. Stripe's
/// checkout sessions also contain lots of information not directly related to
/// the payment itself; that's where we plug in payment-related order.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Payment {
    pub id: PaymentId,
    pub order: OrderId,

    pub status: Status,

    #[cfg(feature = "stripe")]
    pub stripe_session_id: Option<String>,
}

impl Collectable for Payment {
    fn get_collection_name() -> &'static str {
        "payments"
    }
}

impl Identifiable for Payment {
    fn get_id(&self) -> uuid::Uuid {
        self.id
    }
}

impl Payment {
    pub fn new(order: Uuid) -> Result<Self> {
        Ok(Self {
            id: Uuid::new_v4(),
            order,
            status: Status::Pending,
            #[cfg(feature = "stripe")]
            stripe_session_id: None,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Status {
    /// Waiting for payment
    Pending,
    /// Payment was canceled
    Canceled,
    /// An unrecoverable problem occured during payment processing
    Error(String),
    /// Payment was confirmed to be successful
    Successful { time: DateTime<Utc> },
}
