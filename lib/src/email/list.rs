//! Mailing list module.
//!
//! Provides functionality for easily gathering email information and sending
//! emails to multiple subscribers.
//!
//! Note that although for registered users we store emails differently, the
//! same mailing list mechanism can be used to send emails to both
//! non-registered subscribers and registered users at the same time.

use std::collections::HashSet;

use uuid::Uuid;

use crate::db::{Collectable, Identifiable};

/// A standard subscriber definition.
///
/// It can be used to store an email address of someone who would like to
/// receive emails from about the application (e.g. a newsletter). Some
/// optional information like a potential marketing consent is stored as well.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Subscriber {
    pub id: Uuid,

    pub address: String,

    /// If confirmation is required, unconfirmed subscribers will be cleaned
    /// regularly.
    pub confirmed: bool,

    /// All the lists that the subscriber is going to receive emails from.
    pub lists: HashSet<String>,

    /// Explicit mark that the subscriber has agreed to receive marketing
    /// messages, or they actively pursued subscription even though there was
    /// no consent checkmark.
    pub marketing_consent: bool,

    /// Additional information about the subscriber.
    pub notes: String,
}

impl Subscriber {
    pub fn new(address: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            address,
            confirmed: false,
            lists: Default::default(),
            marketing_consent: false,
            notes: String::new(),
        }
    }
}

impl Collectable for Subscriber {
    fn get_collection_name() -> &'static str {
        "email_subscriptions"
    }
}

impl Identifiable for Subscriber {
    fn get_id(&self) -> Uuid {
        self.id
    }
}
