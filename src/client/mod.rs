mod youtube;

use video::Video;

pub use client::youtube::YoutubeClient;

pub trait Client {
    type Video: Video;

    fn query(&self, uri: &str) -> Vec<Self::Video>;
}