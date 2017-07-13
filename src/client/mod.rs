mod youtube;

use error::Result;
use std::error;
use std::result;
use video::Video;

pub use client::youtube::YoutubeClient;

pub trait Client {
    type Video: Video;
    fn query(&self, identifier: &str) -> Result<Self::Video>;
}

pub trait ClientConnector {
    type Err: error::Error + 'static;
    fn download_string(&self, uri: &str) -> result::Result<String, Self::Err>;
}
