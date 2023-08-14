use crate::{Result, SledKvStore, KvStore}; //type in error.rs
use async_trait::async_trait;
use std::marker::Sync;

/*
* Client::get(&mut self, key: String) -> Box<Future<Item = Option<String>, Error = Error>
* Client::get(&mut self, key: String) -> future::SomeExplicitCombinator<...>
* Client::get(&mut self, key: String) -> impl Future<Item = Option<String>, Error = Error>
*/

/*
* fn set(&self, key: String, value: String) -> Box<dyn Future<Item = (), Error = KvsError> + Send>;
* fn get(&self, key: String) -> Box<dyn Future<Item = Option<String>, Error = KvsError> + Send>;
* fn remove(&self, key: String) -> Box<dyn Future<Item = (), Error = KvsError> + Send>;
*/

    /*using async keywords in rewriting trait methods */

#[async_trait]
pub trait KvsEngine: Clone + Send + 'static {
    async fn set(&self, key: String, value: String) -> Result<()>;
    async fn get(&self, key: String) -> Result<Option<String>>;
    async fn remove(&self, key: String) -> Result<()>;
}

