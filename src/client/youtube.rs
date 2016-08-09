use client::Client;
use http;
use regex::Regex;
use video::YoutubeVideo;

lazy_static! {
    static ref JS: Regex = Regex::new(r#""js":"[^"]+""#).unwrap();
    static ref MAP: Regex = Regex::new(r#""url_encoded_fmt_stream_map":"[^"]+""#).unwrap();
    static ref FMT: Regex = Regex::new(r#""adaptive_fmts":"[^"]+""#).unwrap();
    static ref DASHMPD: Regex = Regex::new(r#""dashmpd":"[^"]+""#).unwrap();
    static ref TITLE: Regex = Regex::new(r#"<title>(.*)</title>"#).unwrap();
}

pub struct YoutubeClient;

impl YoutubeClient {
    pub fn new() -> YoutubeClient {
        YoutubeClient
    }
}

impl Client for YoutubeClient {
    type Video = YoutubeVideo;

    fn query(&self, uri: &str) -> Vec<Self::Video> {
        // todo: create error type
        let response = http::download_string(uri).expect(&format!("failed to download uri ({:?}); beware of redirects", uri));
        
        parse_videos(&response)
    } 
}

impl Default for YoutubeClient {
    fn default() -> Self {
        YoutubeClient
    }
}

fn parse_videos(source: &str) -> Vec<YoutubeVideo> {
    let title = TITLE.captures(source).and_then(|cap| cap.at(1)).unwrap_or("untitled");
    let js_player = JS.captures(source).and_then(|cap| cap.at(0)).expect("failed to find jsplayer");
    let queries = read_queries(source);

    queries.iter().map(|query| YoutubeVideo::new(title, query, js_player)).collect()
}

fn read_queries(source: &str) -> Vec<String> {
    let map = MAP.captures(source).and_then(|cap| cap.at(0)).unwrap_or("");
    map.split(',').map(|query| unscramble(query)).collect()    
}

fn unscramble(s: &str) -> String {
    s.replace("\\\\u0026", "&")
}

#[cfg(test)]
mod tests {
    #[test]
    fn regex_patterns_work() {
        use client::youtube::{JS, MAP, FMT, DASHMPD, TITLE};

        JS.is_match("");
        MAP.is_match("");
        FMT.is_match("");
        DASHMPD.is_match("");
        TITLE.is_match("");
    }
}
