mod youtube;

use error::Result;
use hyper;
use video::Video;

pub use client::youtube::YoutubeClient;

pub trait Client {
    type Video: Video;
    fn query(&self, uri: &str, client: &hyper::Client) -> Result<Self::Video>;
}
