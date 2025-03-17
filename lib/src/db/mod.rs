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

pub trait Identifiable {
    fn get_id(&self) -> Uuid;
}

pub trait Collectable {
    fn get_collection_name() -> &'static str;
}

pub trait CollectableAt {
    fn get_collection_name_at(keyset: Uuid) -> String;
}

pub trait Storable {
    fn store() -> Result<()>;
    fn restore() -> Result<()>;
}

pub fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    let t: T = pot::from_slice(bytes)?;
    Ok(t)
}

pub fn encode<T: serde::Serialize>(item: &T) -> Result<Vec<u8>> {
    let bytes = pot::to_vec(item)?;
    Ok(bytes)
}
