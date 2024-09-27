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

pub mod footer {
    pub fn year() -> String {
        use chrono::Datelike;
        chrono::Utc::now().year().to_string()
    }
}
