//! https://github.com/programatik29/tokio-rusqlite

use tokio_rusqlite::Connection;

use crate::Result;

use super::Store;

#[derive(Clone, Debug)]
pub struct SqliteDb {
    connection: Connection,
}

impl SqliteDb {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            inner: Connection::open_in_memory().await.unwrap(),
        })
    }
}

impl Store for SqliteDb {
    fn get_collection<T: serde::de::DeserializeOwned>(
        &self,
        collection: &str,
    ) -> crate::Result<Vec<T>> {
        todo!()
    }

    async fn get<T: serde::de::DeserializeOwned>(
        &self,
        collection: &str,
        id: uuid::Uuid,
    ) -> crate::Result<T> {
        let result: T = self
            .connection
            .call(|conn| {
                conn.execute().unwrap();
            })
            .await
            .unwrap();

        Ok(result)
    }

    fn set<T: serde::Serialize + super::Identifiable>(
        &self,
        collection: &str,
        value: &T,
    ) -> crate::Result<()> {
        todo!()
    }

    fn set_raw<T: serde::Serialize>(
        &self,
        collection: &str,
        value: &T,
        id: uuid::Uuid,
    ) -> crate::Result<()> {
        todo!()
    }

    fn remove(&self, collection: &str, id: uuid::Uuid) -> crate::Result<()> {
        todo!()
    }
}
