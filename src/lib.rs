//#![deny(missing_docs)]
/*!
The KvStore store key/value pairs.
*/

mod engine;
mod errors;
mod request;
mod response;
mod server;
pub mod thread_pool;

pub use engine::Command;
pub use engine::KvStore;
pub use engine::KvsEngine;
pub use engine::SledKvStore;
pub use errors::{KVStoreError, Result};
pub use request::Request;
pub use response::Response;
pub use server::{EngineType, KvServer};
