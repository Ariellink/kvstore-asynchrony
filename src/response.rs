use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    //1. for succeed request
    Ok(Option<String>),
    //2. for failed request
    Err(String),
}
