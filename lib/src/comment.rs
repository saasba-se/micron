use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::db::{Collectable, Identifiable};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Comment {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,

    /// Owner of the comment is the user who published it.
    pub owner: Uuid,

    /// Parent can be anything that can be referenced with a uuid.
    ///
    /// Retrieving comment parents and building reply trees is entirely in the
    /// application domain.
    pub parent: Uuid,

    /// Content is just plain text. Depending on application it might be
    /// markdown or even html.
    pub content: String,

    pub published_time: DateTime<Utc>,
}

impl Default for Comment {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            owner: Uuid::nil(),
            parent: Uuid::nil(),
            content: "".to_string(),
            published_time: Utc::now(),
        }
    }
}

impl Collectable for Comment {
    fn get_collection_name() -> &'static str {
        "comment"
    }
}

impl Identifiable for Comment {
    fn get_id(&self) -> Uuid {
        self.id
    }
}
