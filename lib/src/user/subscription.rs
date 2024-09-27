use rust_decimal::Decimal;

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
