use video::Video;
use std::collections::HashMap;

#[derive(Debug)]
pub struct YoutubeVideo {
    title: String,
    streams: HashMap<String, String>,
}

impl YoutubeVideo {
    pub fn new(title: String, streams: HashMap<String, String>) -> YoutubeVideo {
        YoutubeVideo { title, streams }
    }
}

impl Video for YoutubeVideo {
    fn title(&self) -> &str {
        &self.title
    }
    
    fn format(&self, f: &str) -> Option<&str> {
        self.streams.get(&f.to_lowercase()).map(|s| s.as_ref())
    }
}
