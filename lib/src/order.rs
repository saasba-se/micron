use std::ops::Add;

use chrono::{DateTime, Utc};
use rust_decimal::prelude::Zero;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::Collectable;
use crate::payment::Payment;
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
    pub user_id: UserId,
    /// Time at which the order took place
    pub time: DateTime<Utc>,
    /// Description of how the order was initiated
    pub mode: OrderMode,
    /// List of items included in the order
    pub items: Vec<OrderItem>,

    /// Payment attached to the order, tracking payment details.
    pub payment: Payment,
}

impl Collectable for Order {
    fn get_collection_name() -> &'static str {
        "order"
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

    /// Returns status of the order based on payment status.
    // TODO: take into account the item delivery status.
    pub fn status(&self) -> OrderStatus {
        match self.payment.status {
            crate::payment::Status::Pending => OrderStatus::Processing,
            crate::payment::Status::Canceled => OrderStatus::Failed,
            crate::payment::Status::Successful { time } => OrderStatus::Completed { time },
            crate::payment::Status::Error(_) => todo!(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum OrderStatus {
    Initiated,
    Processing,
    Failed,
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum OrderItem {
    Credits(u32),
    // CloudNodeHours(CloudNodeHours),
    // SharedNodeMonths(u32),
    SubscriptionPlanMonths(user::Plan, u32),
}

impl OrderItem {
    pub fn amount(&self) -> u32 {
        match self {
            Self::Credits(amount) => *amount,
            Self::SubscriptionPlanMonths(plan, amount) => *amount,
        }
    }

    pub fn cost(&self) -> OrderCost {
        match self {
            Self::Credits(amount) => Decimal::new(*amount as i64, 1),
            // Self::SharedNodeMonths(item) => item.into(),
            Self::SubscriptionPlanMonths(plan, months) => {
                plan.price * Decimal::from_f32_retain(*months as f32).unwrap()
            }
            _ => unimplemented!(),
        }
    }
}

impl std::fmt::Display for OrderItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderItem::Credits(amount) => write!(f, "Credits ({amount})"),
            OrderItem::SubscriptionPlanMonths(plan, months) => {
                write!(f, "{} subscription plan ({} months)", plan.name, months)
            }
            _ => unimplemented!(),
        }
    }
}
