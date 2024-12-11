use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sled::{IVec, Tree};
use uuid::Uuid;

use crate::{error::ErrorKind, Result};

use super::{decode, encode, Collectable, Identifiable};

#[derive(Clone, Debug)]
pub struct SledDb {
    inner: sled::Db,
}

impl SledDb {
    pub fn new() -> Result<Self> {
        let inner = sled::Config::default()
            // TODO: specify this path better, perhaps relative to project root
            // or relative to `app` dir root
            .path("./db")
            .open()
            .expect("failed to open db");
        Ok(Self { inner })
    }

    pub fn get_collection<T: DeserializeOwned + Collectable>(&self) -> Result<Vec<T>> {
        let tree = self.inner.open_tree(T::get_collection_name())?;
        let mut out = Vec::new();
        for entry in tree.iter() {
            let (_, value_bytes) = entry?;
            let value: T = decode(&value_bytes)?;
            out.push(value);
        }
        Ok(out)
    }

    pub fn len<T: Collectable>(&self) -> Result<usize> {
        Ok(self.inner.open_tree(T::get_collection_name())?.len())
    }

    pub fn get_collection_raw<T: Collectable>(&self) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let tree = self.inner.open_tree(T::get_collection_name())?;
        let mut out = Vec::new();
        for entry in tree.iter() {
            let (id, value) = entry?;
            out.push((id.to_vec(), value.to_vec()));
        }
        Ok(out)
    }

    pub fn get<T: DeserializeOwned + Collectable>(&self, id: Uuid) -> Result<T> {
        self.get_at(T::get_collection_name(), id)
    }

    /// Supplants an arbitrary id
    pub fn get_at<T: DeserializeOwned>(&self, collection: &str, id: Uuid) -> Result<T> {
        let tree = self.inner.open_tree(collection)?;
        for entry in tree.iter() {
            let (id_bytes, value_bytes) = entry?;
            let _id = Uuid::from_slice(&id_bytes)?;
            if _id == id {
                let value: T = decode(&value_bytes)?;
                return Ok(value);
            }
        }
        Err(ErrorKind::DbError(format!(
            "entity with id '{}' not found in collection {}",
            id, collection
        ))
        .into())
    }

    /// Convenience function providing initializing a default if the target
    /// collection element is not found in the db.
    pub fn get_or_create<T: Serialize + DeserializeOwned + Identifiable + Collectable + Default>(
        &self,
        id: Uuid,
    ) -> Result<T> {
        self.get::<T>(id).or_else(|_| {
            let default = T::default();
            self.set(&default).map(|_| default)
        })
    }

    pub fn set<T: Serialize + Identifiable + Collectable>(&self, value: &T) -> Result<()> {
        self.set_raw_at(T::get_collection_name(), value, value.get_id())?;
        Ok(())
    }

    pub fn set_raw_at<T: Serialize>(&self, collection: &str, value: &T, id: Uuid) -> Result<()> {
        let tree = self.inner.open_tree(collection)?;
        let encoded = encode(value)?;
        tree.insert(id, encoded)?;
        Ok(())
    }

    pub fn remove<T: Identifiable + Collectable>(&self, value: &T) -> Result<()> {
        let tree = self.inner.open_tree(T::get_collection_name())?;
        tree.remove(value.get_id())?;
        Ok(())
    }
}
