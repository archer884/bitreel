mod youtube;

pub use video::youtube::YoutubeVideo;

pub trait Video {
    fn title(&self) -> &str;
    fn uri(&mut self) -> &str;
}