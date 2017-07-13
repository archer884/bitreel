mod youtube;

pub use video::youtube::YoutubeVideo;

pub trait Video {
    /// Gets the title of the video.
    fn title(&self) -> &str;

    /// Attempts to get a url for the video in the requested format.
    fn get_url(&self, format: &str) -> Option<&str>;

    /// Lists all available formats for the video.
    fn list_formats<'a>(&'a self) -> Box<Iterator<Item=&'a str> + 'a>;
}
