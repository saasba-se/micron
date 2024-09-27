#[cfg(feature = "redb")]
mod redb;
#[cfg(feature = "sled")]
mod sled;
#[cfg(feature = "sqlite")]
mod sqlite;

use fnv::FnvHashMap;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::ErrorKind, Result};

#[cfg(feature = "redb")]
pub use redb::ReDb as Database;
#[cfg(feature = "sled")]
pub use sled::SledDb as Database;
#[cfg(feature = "sqlite")]
pub use sqlite::SqliteDb as Database;

// pub trait Store {
//     fn get_collection<T: DeserializeOwned>(&self, collection: &str) -> Result<Vec<T>>;
//     fn get<T: DeserializeOwned>(&self, collection: &str, id: Uuid) -> Result<T>;
//     fn set<T: Serialize + Identifiable>(&self, collection: &str, value: &T) -> Result<()>;
//     fn set_raw<T: Serialize>(&self, collection: &str, value: &T, id: Uuid) -> Result<()>;
//     fn remove(&self, collection: &str, id: Uuid) -> Result<()>;
// }

pub trait Identifiable {
    fn get_id(&self) -> Uuid;
}

pub trait Collectable {
    fn get_collection_name() -> &'static str;
}

pub fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    let t: T = pot::from_slice(bytes)?;
    Ok(t)
}

pub fn encode<T: serde::Serialize>(item: &T) -> Result<Vec<u8>> {
    let bytes = pot::to_vec(item)?;
    Ok(bytes)
}
