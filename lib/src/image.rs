//! Module handling *dynamic* images stored in the database, as opposed to
//! *static* image assets.

use serde::{Deserialize, Serialize};

use crate::db::{Collectable, Identifiable};

pub type ImageId = uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Image {
    pub id: ImageId,
    pub bytes: Vec<u8>,
}

impl Image {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            id: ImageId::new_v4(),
            bytes,
        }
    }
}

impl Collectable for Image {
    fn get_collection_name() -> &'static str {
        "images"
    }
}

impl Identifiable for Image {
    fn get_id(&self) -> uuid::Uuid {
        self.id
    }
}
