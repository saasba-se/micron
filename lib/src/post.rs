use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    db::{Collectable, Identifiable},
    ImageId, UserId,
};

#[derive(Clone, Debug, Default, Serialize, Deserialize, strum::Display)]
pub enum Status {
    Public,
    Private,
    #[default]
    Draft,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Post {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    pub owner: UserId,

    pub date: DateTime<Utc>,

    pub title: String,
    pub lead: String,
    pub slug: String,
    pub category: String,

    pub markdown: String,

    pub image: ImageId,

    pub featured: Option<u8>,

    pub likes: Vec<UserId>,

    pub status: Status,
}

impl Default for Post {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            owner: Uuid::nil(),
            date: Utc::now(),
            title: "title".to_string(),
            lead: "lead".to_string(),
            slug: "slug".to_string(),
            category: "category".to_string(),
            markdown: "content".to_string(),
            image: Uuid::nil(),
            featured: None,
            likes: Vec::new(),
            status: Status::Draft,
        }
    }
}

impl Collectable for Post {
    fn get_collection_name() -> &'static str {
        "post"
    }
}

impl Identifiable for Post {
    fn get_id(&self) -> Uuid {
        self.id
    }
}
