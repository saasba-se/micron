#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Head {
    pub title: String,
}

impl Default for Head {
    fn default() -> Self {
        Self {
            title: "Dashboard".to_string(),
        }
    }
}
