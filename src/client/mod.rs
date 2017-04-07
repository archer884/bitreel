mod youtube;

use error::Result;
use video::Video;

pub use client::youtube::YoutubeClient;

pub trait Client {
    type Video: Video;
    fn query(&self, uri: &str) -> Result<Self::Video>;
}
