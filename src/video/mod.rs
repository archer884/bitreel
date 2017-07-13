mod youtube;

pub use video::youtube::YoutubeVideo;

pub trait Video {
    fn title(&self) -> &str;
    fn get_url(&self, format: &str) -> Option<&str>;
    fn list_formats<'a>(&'a self) -> Box<Iterator<Item=&'a str> + 'a>;
}
