use rust_decimal::Decimal;

/// Subscription plan is defined very loosely here such that applications
/// can freely define whatever is needed.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Plan {
    pub name: String,
    pub price: Decimal,
}

impl Plan {
    pub fn free() -> Self {
        Self {
            name: "free".to_string(),
            price: Decimal::ZERO,
        }
    }
}
