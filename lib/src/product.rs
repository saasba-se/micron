use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{
    db::{Collectable, Identifiable},
    user::Plan,
    Database, Result, User, UserId,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Product {
    pub id: Uuid,
    pub inner: ProductInner,
}

impl Collectable for Product {
    fn get_collection_name() -> &'static str {
        "products"
    }
}

impl Identifiable for Product {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ProductInner {
    Subscription {
        plan: Plan,
        months: usize,
        recurring: bool,
    },
    Credits {
        /// Exact amount of credits
        amount: usize,
        multiplier: Option<f32>,
    },
    Custom {
        name: String,
        /// Price defined as per amount == 1
        price: Decimal,
        amount: usize,
        // recurring: bool,
    },
}

impl Product {
    pub fn amount(&self) -> usize {
        match &self.inner {
            ProductInner::Credits { amount, .. } => *amount,
            ProductInner::Subscription { months, .. } => *months,
            ProductInner::Custom { amount, .. } => *amount,
        }
    }

    pub fn cost(&self) -> Decimal {
        match &self.inner {
            ProductInner::Credits { amount, multiplier } => {
                Decimal::new((*amount as f32 * multiplier.unwrap_or(1.)) as i64, 1)
            }
            ProductInner::Subscription {
                plan,
                months,
                recurring,
            } => plan.price * Decimal::new(*months as i64, 1),
            ProductInner::Custom {
                name,
                price,
                amount,
            } => price * Decimal::new(*amount as i64, 1),
        }
    }

    pub async fn realize_for(&self, user: UserId, db: &Database) -> Result<()> {
        match &self.inner {
            ProductInner::Credits { amount, multiplier } => {
                let mut user = db.get::<User>(user)?;
                user.credits.available += Decimal::new(*amount as i64, 1);
                db.set(&user)?;
            }
            ProductInner::Subscription {
                plan,
                months,
                recurring,
            } => {
                let mut user = db.get::<User>(user)?;
                user.plan = plan.clone();
                // TODO: add expiry date for subscription

                db.set(&user)?;
            }
            ProductInner::Custom {
                name,
                price,
                amount,
            } => {
                // NOTE: realization must be custom, probably triggered using
                // a separate task looking at recently fulfilled orders and
                // realizing them based on relevant business logic.
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for Product {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            ProductInner::Subscription { plan, months, .. } => {
                write!(f, "`{}` subscription ({} months)", plan.name, months)
            }
            ProductInner::Credits { amount, multiplier } => write!(f, "{} credits", amount),
            ProductInner::Custom {
                name,
                price,
                amount,
            } => write!(f, "{} ({}x)", name, amount),
        }
    }
}
