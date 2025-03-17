use std::ops::Add;

use chrono::{DateTime, Utc};
use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::Collectable;
use crate::payment::{self, Payment};
use crate::product::Product;
use crate::Database;
use crate::Result;
use crate::{db::Identifiable, user, user::UserId};

pub type OrderCost = Decimal;
pub type OrderId = Uuid;

/// Single order describes an event where credits are to be subtracted from
/// user in exchange for a number of items.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Order {
    /// Unique identifier for the order
    pub id: OrderId,
    /// Id of the user connected to the order
    pub user: UserId,
    /// Time at which the order was initiated
    pub time: DateTime<Utc>,
    /// Current status of the order
    pub status: OrderStatus,
    /// Description of how the order was initiated
    pub mode: OrderMode,
    /// List of product items included in the order
    pub items: Vec<Product>,
}

impl Collectable for Order {
    fn get_collection_name() -> &'static str {
        "orders"
    }
}

impl Identifiable for Order {
    fn get_id(&self) -> uuid::Uuid {
        self.id
    }
}

impl Order {
    /// Calculates total cost of all order items.
    pub fn total_cost(&self) -> Decimal {
        let mut delta = Decimal::zero();
        for item in &self.items {
            delta += item.cost();
        }
        delta
    }

    /// Start fulfilling the order.
    pub async fn fulfill(mut self, db: &Database) -> Result<()> {
        self.status = OrderStatus::Processing;
        db.set(&self)?;

        for item in &self.items {
            item.realize_for(self.user, db).await?;
        }

        self.status = OrderStatus::Completed { time: Utc::now() };
        db.set(&self)?;

        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum OrderStatus {
    /// The order was initiated, waiting on payment
    Initiated,
    /// Payment was successful, waiting on fulfillment
    Processing,
    /// Failed and couldn't recover (e.g. re-init payment)
    Failed,
    /// Completed the order successfully
    Completed { time: DateTime<Utc> },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum OrderMode {
    /// User initiated the order manually through the web UI
    Manual,
    /// Order was initiated through the API
    Api,
    /// Order was was initiated automatically based on a user-defined
    /// plan
    Auto,
}
