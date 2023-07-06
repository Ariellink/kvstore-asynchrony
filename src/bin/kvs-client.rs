use clap::{arg, command, ArgMatches, Command};
use futures::{SinkExt, TryStreamExt}; //for iterators
use kvs::Result;
use tokio::{net::{TcpStream, tcp::{OwnedReadHalf, OwnedWriteHalf}}};
use tokio_serde::formats::SymmetricalJson;
use tokio_serde::SymmetricallyFramed;
use std::{env, process};
use kvs::{Request, Response};
use tokio_util::codec::{FramedRead,FramedWrite,LengthDelimitedCodec};


//build the Command instance
#[tokio::main]
async fn main() -> Result<()> {
    let matches = command!() // requires `cargo` feature
        .propagate_version(true)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("get")
                .about("get a vaule from a key: get <key>")
                .arg(arg!(<KEY>).help("A String key").required(true))
                .arg(
                    arg!(-a --addr <ipport> "example: 127.0.0.1:4000")
                        .required(true)
                        .default_value("127.0.0.1:4000"),
                ),
        )
        .subcommand(
            Command::new("set")
                .about("set a key/vaule pair: set <key> <vaule>")
                .arg(arg!(<KEY>).help("A String key").required(true))
                .arg(arg!(<VALUE>).help("A String vaule").required(true))
                .arg(
                    arg!(-a --addr <ipport> "example: 127.0.0.1:4000")
                        .required(true)
                        .default_value("127.0.0.1:4000"),
                ),
        )
        .subcommand(
            Command::new("rm")
                .about("remove the a key/vaule pair: rm <key>")
                .arg(arg!(<KEY>).help("A String key").required(true))
                .arg(
                    arg!(-a --addr <ipport> "example: 127.0.0.1:4000")
                        .required(true)
                        .default_value("127.0.0.1:4000"),
                ),
        )
        .get_matches(); //get the command struct

    if let Err(err) = send_request(matches).await {
        eprintln!("{:?}", err);
        process::exit(-1);
    }

    Ok(())
}


//fn send_request()
//对command本身进行模式匹配，根据不同的命令进行后续操作
async fn send_request(matches: ArgMatches) -> Result<()> {
    match matches.subcommand() {
        Some(("get", _matches)) => {
            let key = _matches.get_one::<String>("KEY").unwrap();
            let addr = _matches.get_one::<String>("addr").unwrap();
            //拿到了server ip和要查询的key
            //需要建立连接
            let mut client = Client::new(&addr).await?;
            match client.request(Request::GET(key.to_owned())).await? {
                Some(val) => println!("{}", val),
                None => println!("Key not found"),
            };
        }
        Some(("set", _matches)) => {
            let key = _matches.get_one::<String>("KEY").unwrap();
            let value = _matches.get_one::<String>("VALUE").unwrap();
            let addr = _matches.get_one::<String>("addr").unwrap();
            let mut client = Client::new(&addr).await?;
            client.request(Request::SET(key.to_owned(), value.to_owned())).await?;
        }
        Some(("rm", _matches)) => {
            let key = _matches.get_one::<String>("KEY").unwrap();
            let addr = _matches.get_one::<String>("addr").unwrap();
            let mut client = Client::new(&addr).await?;
            client.request(Request::RM(key.to_owned())).await?;
        }
        _ => process::exit(-1),
    }
    Ok(())
}

struct Client {
    //for response (server -> client)
    reader: SymmetricallyFramed<FramedRead<OwnedReadHalf,LengthDelimitedCodec>,Response,SymmetricalJson<Response>>,
    //for request (client -> server)
    writer: SymmetricallyFramed<FramedWrite<OwnedWriteHalf,LengthDelimitedCodec>,Request,SymmetricalJson<Request>>,
}

impl Client {
    async fn new(addr: &str) -> Result<Client> {
        let stream = TcpStream::connect(addr).await?;
        //(OwnedReadHalf,OwnedWriteHalf) = tokio::net::TcpStream::into_split()
        let(stream_read,stream_write) = stream.into_split();

        //construct the reader
        let length_delimited_read = FramedRead::new(stream_read,LengthDelimitedCodec::new());
        let reader: SymmetricallyFramed<FramedRead<OwnedReadHalf,LengthDelimitedCodec>,Response,SymmetricalJson<Response>> = SymmetricallyFramed::new(length_delimited_read,SymmetricalJson::<Response>::default());

        //construct the writer
        let length_delimited_write = FramedWrite::new(stream_write,LengthDelimitedCodec::new());
        let writer:SymmetricallyFramed<FramedWrite<OwnedWriteHalf,LengthDelimitedCodec>,Request,SymmetricalJson<Request>> = SymmetricallyFramed::new(length_delimited_write,SymmetricalJson::<Request>::default());

        Ok(Client {
            reader,
            writer,
        })
    }

    async fn request(&mut self, request: Request) -> Result<Option<String>> {
        // 把request序列化为JSON, 然后放进Client::writer (or IO stream)
        self.writer.send(request).await?;

        //client处理server发过来的respone, 从reader中使用迭代器读取
        //self.reader.try_next().await?返回类型是Option<Result>
        match self.reader.try_next().await? {
            Some(Response::Ok(val)) => Ok(val),
            Some(Response::Err(err)) => Err(kvs::KVStoreError::ServerError(err)),
            None => Ok(None),
        }
    }
}
