# kvstore-asynchrony

A multi-threaded, persistent key/value store server and client with asynchronous networking over a custom protocol.

- `KvsEngine` trait presents a futures-based API, by using `#[async_trait]` attribute macro to make async fn in traits work.
```rust
#[async_trait]
pub trait KvsEngine: Clone + Send + 'static {
    async fn set(&self, key: String, value: String) -> Result<()>;
    async fn get(&self, key: String) -> Result<Option<String>>;
    async fn remove(&self, key: String) -> Result<()>;
}
```
- Both the client and server are converted to asyncronous;
- `KvsServer` is based on the tokio runtime, which handles the distribution of asynchronous work to multiple threads on its own (tokio itself contains a thread pool).
  - Replaced the `serde` with `tokio-serde` for Tokio transport serialization and deserialization of frame values.
- `KvStore` handles file I/O using `tokio::spawn`;
