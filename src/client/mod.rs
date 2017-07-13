mod youtube;

use error::Result;
use std::error;
use std::result;
use video::Video;

pub use client::youtube::YoutubeClient;

pub trait Client {
    type Video: Video;

    /// Queries the service for a video matching the provided identifier.
    fn query(&self, identifier: &str) -> Result<Self::Video>;
}

pub trait ClientConnector {
    type Err: error::Error + 'static;

    /// Downloads the provided uri as a string.
    ///
    /// This is provided to allow the client to download video information, but bitreel does not 
    /// provide an implementation so that consumers can use whatever network implementation they 
    /// prefer.
    fn download_string(&self, uri: &str) -> result::Result<String, Self::Err>;
}
