extern crate bitreel;

use bitreel::client::{Client, YoutubeClient};

fn main() {
    let query_uris = &["0NM0vznAgwg"];
    let client = YoutubeClient::new();
    for uri in query_uris {
        println!("{:#?}", client.query(uri));
    }
}
