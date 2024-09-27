use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;

use crate::db::Database;
use crate::{order::Order, UserId};
use crate::{Result, User};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Credits {
    /// Total credits currently available
    pub available: Decimal,
    pub history: CreditsHistory,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct CreditsHistory {
    pub last_three: Vec<(DateTime<Utc>, Decimal)>,

    /// Credits values for last 24 hours by hour
    pub last_day: Vec<Decimal>,
    /// Credits values for last 7 days by day
    pub last_five_days: Vec<Decimal>,
    /// Credits values for last 30 days by day
    pub last_month: Vec<Decimal>,
    /// Credits values for last 6 months by month
    pub last_six_months: Vec<Decimal>,
    /// Credits values for last 12 months by month
    pub last_year: Vec<Decimal>,
}

/// Calculates user credits history. Saves the result in the database and
/// also returns it to caller.
pub fn calculate_history(user_id: UserId, db: &Database) -> Result<CreditsHistory> {
    let mut user = db.get::<User>(user_id)?;
    let mut orders = db.get_collection::<Order>()?;
    orders.retain(|p| p.user_id == user_id);

    let history = user.credits.calculate_history(orders);

    Ok(history)
}

impl Credits {
    /// Goes through the user order history and recalculates credits history
    /// accordingly.
    pub fn calculate_history(&mut self, mut orders: Vec<Order>) -> CreditsHistory {
        // sort from newest to oldest
        orders.sort_by(|a, b| b.time.cmp(&a.time));

        // println!("user_orders: {:?}", user_orders);

        let mut last_three = Vec::new();
        for order in orders.iter().take(3) {
            let delta = -order.total_cost();
            last_three.push((order.time, delta));
        }

        let mut _last_day = Vec::new();
        for order in orders {
            if order.time.signed_duration_since(Utc::now()) < Duration::days(1) {
                _last_day.push(order);
            }
        }

        // Sort orders
        _last_day.sort_by(|a, b| a.time.timestamp().cmp(&b.time.timestamp()));

        // println!("_last_day: {:?}", _last_day);

        // Calculate user available credits for each order point in time,
        // going backwards hour by hour from the newest order.
        let mut last_day = Vec::new();
        let mut credits = self.available;
        let mut hours = 24;

        for delta_hour in 1..hours + 1 {
            // println!("delta_hour: {}", delta_hour);
            let mut hour_delta = Decimal::ZERO;
            for order in &_last_day {
                let order_delta_hours = order
                    .time
                    .signed_duration_since(Utc::now())
                    .num_hours()
                    .abs();
                // println!("order_delta_hours: {:?}", order_delta_hours);
                if order_delta_hours < delta_hour && order_delta_hours >= delta_hour - 1 {
                    let delta = -order.total_cost();
                    hour_delta += delta;
                }
            }
            last_day.push(credits);
            credits -= hour_delta;
        }

        // println!("last_day: {:?}", last_day);

        CreditsHistory {
            last_three,
            last_day,
            last_five_days: vec![],
            last_month: vec![],
            last_six_months: vec![],
            last_year: vec![],
        }
    }
}
