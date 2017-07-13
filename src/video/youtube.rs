use std::collections::HashMap;
use video::Video;

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
    
    fn get_url(&self, format: &str) -> Option<&str> {
        self.streams.get(&format.to_lowercase()).map(|s| s.as_ref())
    }

    fn list_formats<'a>(&'a self) -> Box<Iterator<Item=&'a str> + 'a> {
        Box::new(self.streams.keys().map(|s| s.as_ref()))
    }
}
