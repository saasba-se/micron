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

    pub fn trees_for<T: Collectable>(&self) -> Result<Vec<String>> {
        Ok(self
            .inner
            .tree_names()
            .into_iter()
            .map(|s| String::from_utf8_lossy(&s).into_owned())
            .filter(|t| t.contains(T::get_collection_name()))
            .collect::<Vec<_>>())
    }

    pub fn get_collection<T: DeserializeOwned + Collectable>(&self) -> Result<Vec<T>> {
        self.get_collection_at(T::get_collection_name())
    }

    /// Gets a collection of entries of the same type from the collection
    /// specified by name.
    pub fn get_collection_at<T: DeserializeOwned + Collectable>(
        &self,
        name: impl AsRef<[u8]>,
    ) -> Result<Vec<T>> {
        let tree = self.inner.open_tree(name)?;
        let mut out = Vec::new();
        for entry in tree.iter() {
            let (_, value_bytes) = entry?;
            let value: T = decode(&value_bytes).unwrap();
            out.push(value);
        }
        Ok(out)
    }

    /// Returns the length of the collection as defined for the specified type.
    pub fn len<T: Collectable>(&self) -> Result<usize> {
        Ok(self.inner.open_tree(T::get_collection_name())?.len())
    }

    /// Gets a collection of entries of the same type. Returns a raw key-value
    /// pair for each entry.
    pub fn get_collection_raw<T: Collectable>(&self) -> Result<Vec<(Vec<u8>, Vec<u8>)>> {
        let tree = self.inner.open_tree(T::get_collection_name())?;
        let mut out = Vec::new();
        for entry in tree.iter() {
            let (key, value) = entry?;
            out.push((key.to_vec(), value.to_vec()));
        }
        Ok(out)
    }

    /// Gets an item from the collection defined for the item type.
    pub fn get<T: DeserializeOwned + Collectable>(&self, id: Uuid) -> Result<T> {
        self.get_at(T::get_collection_name(), id)
    }

    /// Gets an item by id from the collection specified by name.
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
    /// TODO: currently this doesn't set the id of the new item to the id
    /// provided to the function. It could be done by expanding the
    /// Identifiable trait to include ability to also set the id.
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
        self.set_at(T::get_collection_name(), value)?;
        Ok(())
    }

    pub fn set_at<T: Serialize + Identifiable + Collectable>(
        &self,
        collection: impl AsRef<[u8]>,
        value: &T,
    ) -> Result<()> {
        self.set_raw_at(collection, value, value.get_id())?;
        Ok(())
    }

    pub fn set_raw_at<T: Serialize>(
        &self,
        collection: impl AsRef<[u8]>,
        value: &T,
        id: Uuid,
    ) -> Result<()> {
        let tree = self.inner.open_tree(collection)?;
        let encoded = encode(value)?;
        tree.insert(id, encoded)?;
        Ok(())
    }

    pub fn remove<T: Identifiable + Collectable>(&self, value: &T) -> Result<()> {
        self.remove_at(T::get_collection_name(), value)
    }

    pub fn remove_at<T: Identifiable + Collectable>(
        &self,
        collection: impl AsRef<[u8]>,
        value: &T,
    ) -> Result<()> {
        let tree = self.inner.open_tree(collection)?;
        tree.remove(value.get_id())?;
        Ok(())
    }

    pub fn clear_at(&self, collection: &str) -> Result<()> {
        let tree = self.inner.open_tree(collection)?;
        tree.clear()?;
        Ok(())
    }

    pub fn clear<T: Collectable>(&self) -> Result<()> {
        let tree = self.inner.open_tree(T::get_collection_name())?;
        tree.clear()?;
        Ok(())
    }
}
