//! Database storage based on `redb`.
//!
//! `redb` design document: https://github.com/cberner/redb/blob/master/docs/design.md

use std::str::FromStr;
use std::sync::Arc;

use redb::{Database, ReadableTable, TableDefinition, TableHandle};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{ErrorKind, Result};

use super::{decode, encode, Identifiable};

#[derive(Clone, Debug)]
pub struct ReDb {
    db: Arc<Database>,
}

impl ReDb {
    pub async fn new() -> Result<Self> {
        let db = Database::create("db.redb")?;

        let mut wx = db.begin_write().unwrap();
        wx.open_table(TableDefinition::<&[u8], &[u8]>::new("access_tokens"))
            .unwrap();
        wx.commit().unwrap();

        Ok(Self { db: Arc::new(db) })
    }

    pub fn get_collection<T: DeserializeOwned>(&self, collection: &str) -> Result<Vec<T>> {
        let rd = self.db.begin_read().unwrap();
        let mut out = Vec::new();
        {
            let table = rd
                .open_table(redb::TableDefinition::<&[u8], &[u8]>::new(collection))
                .unwrap();
            for entry in table.iter().unwrap() {
                let (_, bytes) = entry.unwrap();
                let value: T = decode(&bytes.value())?;
                out.push(value);
            }
        }
        Ok(out)
    }

    pub fn get<T: DeserializeOwned>(&self, collection: &str, id: Uuid) -> Result<T> {
        let rd = self.db.begin_read().unwrap();

        if !rd.list_tables().unwrap().any(|th| th.name() == collection) {
            // create the table first
            rd.close();
            let wx = self.db.begin_write().unwrap();
            wx.open_table(redb::TableDefinition::<&[u8], &[u8]>::new(collection))
                .unwrap();
            wx.commit().unwrap();
        }

        let rd = self.db.begin_read().unwrap();
        {
            let table = rd
                .open_table(redb::TableDefinition::<&[u8], &[u8]>::new(collection))
                .unwrap();
            for entry in table.iter().unwrap() {
                let (id_bytes, value_bytes) = entry.unwrap();
                let _id = Uuid::from_slice(&id_bytes.value())?;
                if _id == id {
                    let value: T = decode(&value_bytes.value())?;
                    return Ok(value);
                }
            }
            Err(ErrorKind::DbError(format!(
                "entity with id '{}' not found in collection {}",
                id, collection
            ))
            .into())
        }
    }

    pub fn set<T: Serialize + Identifiable>(&self, collection: &str, value: &T) -> Result<()> {
        self.set_raw(collection, value, value.get_id())?;
        Ok(())
    }

    pub fn set_raw<T: Serialize>(&self, collection: &str, value: &T, id: Uuid) -> Result<()> {
        let wx = self.db.begin_write().unwrap();
        {
            let mut table = wx
                .open_table(redb::TableDefinition::<&[u8], &[u8]>::new(collection))
                .unwrap();
            let encoded = encode(value)?;
            table
                .insert(id.as_bytes().as_slice(), encoded.as_slice())
                .unwrap();
        }

        wx.commit().unwrap();

        Ok(())
    }

    pub fn remove(&self, collection: &str, id: Uuid) -> Result<()> {
        let wx = self.db.begin_write().unwrap();
        {
            let mut table = wx
                .open_table(redb::TableDefinition::<&[u8], &[u8]>::new(collection))
                .unwrap();
            table.remove(id.as_bytes().as_slice()).unwrap();
        }

        Ok(())
    }
}
