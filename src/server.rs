use crate::{KvsEngine, Request, Response, Result};
use tokio::net::{TcpListener, TcpStream};
use tokio_serde;
use tokio_util::codec::{FramedRead,FramedWrite,LengthDelimitedCodec};
//use serde::Deserialize;
use log::{debug, error, info};
use std::fmt;
use std::time::SystemTime;
use std::marker::Sync;

pub enum EngineType {
    KvStore,
    SledKvStore,
}

//for to_string() can be used on enum EngineType when combine the current dir in kvs_server.rs
impl fmt::Display for EngineType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EngineType::KvStore => write!(f, "kvs"),
            EngineType::SledKvStore => write!(f, "sled"),
        }
    }
}

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
    pub async fn serve(&mut self, addr: &String) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        info!("serving request and listening on [{}]", addr);
        while let Ok((stream, _)) = listener.accept().await {
            let engine = self.engine.clone();
            tokio::spawn(async move {
                    info!("get in to the tokio spawn");
                    if let Err(e) = handle_connection(engine, stream).await {
                        error!("Unexpected error occurs when handling connection: {:?}", e);}
                });
        }
        Ok(())
    }
}

//deserialize the stream to data gram strcut
//call from struct 
use futures::prelude::*;

async fn handle_connection<E: KvsEngine>(engine: E, stream: TcpStream) -> Result<()> {
    info!("tcpstream: {:?}", &stream);
    let (rd, wr) = stream.into_split();
    info!("OwnedReadHalf: {:?}", &rd);
    info!("OwnedWriteHalf: {:?}", &wr);
    let frame_read = FramedRead::new(rd, LengthDelimitedCodec::new());
    info!("frame_read: {:#?}", &frame_read);
    let mut read_json = tokio_serde::SymmetricallyFramed::new(
        frame_read,
        tokio_serde::formats::SymmetricalJson::<Request>::default(),
    );
    info!("read_json: {:#?}", &read_json);
    let request = read_json.try_next().await.unwrap().take().unwrap();
    info!("request: {:#?}", &request);
    // let request = Request::deserialize(&mut serde_json::Deserializer::from_reader(
    //     BufReader::new(&mut stream)
    // ))?;

    
    //let bufreader = BufReader::new(&mut stream);
    //info!("bufreader: {:?}", &bufreader);

    let now = SystemTime::now();
   

    let response = match request {
        Request::GET(key) => match engine.get(key).await{
            Ok(value) =>  Response::Ok(value),
            Err(err) => Response::Err(err.to_string()),
        },
        Request::SET(key, val) => match engine.set(key, val).await {
            Ok(()) => Response::Ok(None),
            Err(err) => Response::Err(err.to_string()),
        },
        Request::RM(key) => match engine.remove(key).await {
            Ok(()) => Response::Ok(None),
            Err(err) => Response::Err(err.to_string()),
        },
    };

    info!("Response: {:?},spent time: {:?}", &response, now.elapsed());

    //serde_json::to_writer(stream, &response)?;
    let frame_write = FramedWrite::new(wr, LengthDelimitedCodec::new());
    let mut write_json = tokio_serde::SymmetricallyFramed::new(frame_write,tokio_serde::formats::SymmetricalJson::default());

    write_json
        .send(response)
        .await
        .unwrap();

    Ok(())
}


