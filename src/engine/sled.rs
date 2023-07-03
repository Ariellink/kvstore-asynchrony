use async_trait::async_trait;
use log::error;
use tokio::sync::oneshot;
use crate::{KVStoreError, KvsEngine, Result};
use std::path::PathBuf;

#[derive(Clone)]
pub struct SledKvStore {
    inner: sled::Db,
}

impl SledKvStore {
    pub fn open(open_path: impl Into<PathBuf>) -> Result<SledKvStore> {
        let inner_sleddb = sled::open(open_path.into())?;

        Ok(SledKvStore {
            inner: inner_sleddb,
        })
    }
}

#[async_trait]
impl KvsEngine for SledKvStore {
    async fn set(&self, key: String, value: String) -> Result<()> {
        let db = self.inner.clone();
        let (tx,rx) = oneshot::channel();
        tokio::spawn(async move {
            let res = db.insert(key, value.into_bytes());
            if tx.send(res).is_err() {
                error!("receiving end was dropped during sled set operation");
            }
        });

        match rx.await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(KVStoreError::SledError(e)),
            Err(e) => Err(KVStoreError::ServerError(format!("{}",e))),
        }
    }

    async fn get(&self, key: String) -> Result<Option<String>> {
        let db = self.inner.clone();
        let (tx,rx) = oneshot::channel();
        tokio::spawn(async move {
            let res = tokio::task::spawn_blocking(move ||{
                let val = self
                .inner
                .get(key)?
                .map(|vec| vec.to_vec())
                .map(String::from_utf8)
                .transpose()?; //utf8 errors
                //Result::<_>::Ok(val)
                Result::<Option<String>>::Ok(val)
            })
            .await
            .map_err(|join_error| KVStoreError::ServerError(format!("JoinError: {}", join_error)));
             
            if tx.send(res).is_err() {
                error!("receiving end was dropped during sled get operation");
            }
        });

        match rx.await {
            Ok(Ok(val)) => val,
            Ok(Err(e)) => Err(e),
            Err(e) => Err(KVStoreError::ServerError(format!("{}",e))),
        }    
    }

    async fn remove(&self, key: String) -> Result<()> {
        // Db::remove only returns if it existed.
        let db = self.inner.clone();
        let (tx,rx) = oneshot::channel();
        tokio::spawn(async move {
             let res = tokio::task::spawn_blocking(move ||{
                db.remove(key)?.ok_or(KVStoreError::KeyNotFound)?;
                self.inner.flush()?;
                Result::<_>::Ok(())
            })
            .await
            .map_err(|join_error| KVStoreError::ServerError(format!("JoinError: {}", join_error)));
            if tx.send(res).is_err() {
                error!("receiving end was dropped during sled remove operation");
            }
        });

        match rx.await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(e) => Err(KVStoreError::ServerError(format!("{}",e))),
        }
    }
}
