extern crate bitreel;

use bitreel::client::{Client, YoutubeClient};

fn main() {
    let query_uris = &[
        "https://www.youtube.com/watch?v=IB3lcPjvWLA",
        "https://www.youtube.com/watch?v=BgpXMA_M98o",
        "https://www.youtube.com/watch?v=nfWlot6h_JM",
        "https://www.youtube.com/watch?v=EphGWZKtXvE", // Without adaptive map
        "http://youtube.com/watch?v=IB3lcPjvWLA",
        "https://www.youtube.com/watch?v=kp8u_Yrw76Q", //private
        "https://www.youtube.com/watch?v=09R8_2nJtjg", //encrypted
        "https://www.youtube.com/watch?v=ZAqC3Qh_oUs",
        "https://www.youtube.com/watch?v=pG_euGOe0ww",
    ];

    let client = YoutubeClient::new();
    for uri in query_uris {
        println!("{:?}", client.query(uri));
    }
}
