mod youtube;

pub use video::youtube::YoutubeVideo;

pub trait Video {
    fn title(&self) -> &str;
    fn format(&self, f: &str) -> Option<&str>;
}
