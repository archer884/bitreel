use error::*;
use hyper::Client;
use std::io::Read;

pub fn download_string(client: &Client, uri: &str) -> Result<String> {
    let mut response = client.get(uri).send()?;
    let mut buf = String::new();
    response.read_to_string(&mut buf).expect("unable to read response body");
    Ok(buf)
}