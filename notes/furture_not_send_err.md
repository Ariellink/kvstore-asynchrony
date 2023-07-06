
```rust
pub struct KvServer<E>
where
    E: KvsEngine + Sync,
{
    engine: E,
}

impl<E: KvsEngine + Sync> KvServer<E> {
    // construct
    pub fn new(engine: E) -> Self {
        KvServer { engine }
    }

    //serve and listen at addr
    //循环处理每一个stream
    impl<E: KvsEngine + Sync> KvServer<E> {
    // construct
    pub fn new(engine: E) -> Self {
        KvServer { engine }
    }

    //serve and listen at addr
    //循环处理每一个stream
    pub async fn serve(&mut self, addr: &String) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        info!("serving request and listening on [{}]", addr);
        for (stream, _) in listener.accept().await {
            let engine = self.engine.clone();
            tokio::spawn(async move {
                    if let Err(e) = handle_connection(engine, stream).await {
                        error!("Unexpected error occurs when handling connection: {:?}", e);}
                }
            );
        }
        Ok(())
    }
}

```

## Compiler error
报错代码：
```rust
tokio::spawn(async move {
                    if let Err(e) = handle_connection(engine, stream).await {
                        error!("Unexpected error occurs when handling connection: {:?}", e);}
                }
            );
```
报错信息：
```rust
error: future cannot be sent between threads safely
   --> src/server.rs:50:26
    |
50  |               tokio::spawn(async move {
    |  __________________________^
51  | |                      if let Err(e) = handle_connection(engine, stream).await {
52  | |                          error!("Unexpected error occurs when handling connection: {:?}", e);
53  | |                       //handle_connection(engine, stream).await;
54  | |                     }
55  | |                 }
    | |_________________^ future created by async block is not `Send`
    |
note: future is not `Send` as this value is used across an await
   --> src/server.rs:90:51
    |
90  |         Request::GET(key) => match engine.get(key).await {
    |                                    ------         ^^^^^^ await occurs here, with `engine` maybe used later
    |                                    |
    |                                    has type `&E` which is not `Send`
...
93  |         },
    |         - `engine` is later dropped here
help: consider moving this into a `let` binding to create a shorter lived borrow
   --> src/server.rs:90:36
    |
90  |         Request::GET(key) => match engine.get(key).await {
    |                                    ^^^^^^^^^^^^^^^
note: required by a bound in `tokio::spawn`
   --> /home/chenxi0912/.cargo/registry/src/index.crates.io-6f17d22bba15001f/tokio-1.26.0/src/task/spawn.rs:163:21
    |
163 |         T: Future + Send + 'static,
    |                     ^^^^ required by this bound in `spawn`
help: consider further restricting this bound
    |
36  | impl<E: KvsEngine + std::marker::Sync /*+ Sync*/> KvServer<E> {
    |                   +++++++++++++++++++
```

## 解决方案
```rust
pub struct KvServer<E>
where
    E: KvsEngine + Sync,
{
    engine: E,
}

impl<E: KvsEngine + Sync> KvServer<E> {
    // construct
    pub fn new(engine: E) -> Self {
        KvServer { engine }
    }

```

## 可能的问题
E 实现了 Send 并不意味着 &E 自动实现了 Send。虽然 &E 引用了实现了 Send 的类型 E，但是在 Rust 中，引用类型的 Send 特性实现还需要满足额外的条件。

在 Rust 中，Send 特性表示类型是安全跨线程传递的。对于引用类型来说，引用的值必须满足以下两个条件才能实现 Send：

    1. 引用的值本身实现了 Send，即所引用的类型是线程安全的。
    2. 引用是 'static 生命周期，即它没有任何借用关系，可以在任何线程上安全地使用。

所以，对于 &E 类型，即使 E 实现了 Send，&E 并不一定满足 Send 的要求，因为它的生命周期可能不是 'static。

在您的代码中，编译器报错指出 &E 的生命周期过长，不满足 Send 的要求。这是因为在 await 操作之后，编译器无法确定 engine 的引用是否仍然有效，因此无法保证安全地跨线程传递。

为了解决这个问题，您可以将 engine 绑定到一个局部变量中，将其生命周期缩小到 handle_connection 函数内部，并确保它满足 Send 的要求。

综上所述，E 实现了 Send 并不意味着 &E 自动实现了 Send，引用类型的 Send 特性还需要满足额外的条件，包括引用的值本身是线程安全的并且引用的生命周期是 'static。