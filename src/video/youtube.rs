use video::Video;

#[derive(Debug)]
pub struct YoutubeVideo {
    title: String,
    uri: String,
    js_player: String,
}

impl YoutubeVideo {
    pub fn new(title: &str, uri: &str, js_player: &str) -> YoutubeVideo {
        YoutubeVideo {
            title: title.to_owned(),
            uri: uri.to_owned(),
            js_player: js_player.to_owned(),
        }
    }
}

impl Video for YoutubeVideo {
    fn title(&self) -> &str {
        &self.title
    }
    
    fn uri(&self) -> &str {
        &self.uri
    }
}