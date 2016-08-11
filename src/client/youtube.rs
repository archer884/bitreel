use client::Client;
use http;
use regex::Regex;
use urlparse;
use video::YoutubeVideo;

lazy_static! {
    static ref JS: Regex = Regex::new(r#""js":"([^"]+)""#).unwrap();
    static ref MAP: Regex = Regex::new(r#""url_encoded_fmt_stream_map":"([^"]+)""#).unwrap();
    // static ref FMT: Regex = Regex::new(r#""adaptive_fmts":"([^"]+)""#).unwrap();
    // static ref DASHMPD: Regex = Regex::new(r#""dashmpd":"([^"]+)""#).unwrap();
    static ref TITLE: Regex = Regex::new(r#"<title>(.*)</title>"#).unwrap();
    static ref SIGNATURE: Regex = Regex::new(r#"s=([^&]+)"#).unwrap();
    static ref URL: Regex = Regex::new(r#"url=([^&]+)"#).unwrap();
    static ref HOST: Regex = Regex::new(r#"fallback_host=([^&]+)"#).unwrap();
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
    let js_player = read_js_player(source);
    let uri_results = read_uris(source);

    uri_results.iter().map(
        |&(ref url, ref signature)| YoutubeVideo::new(title, url.as_ref(), &js_player, signature)
    ).collect()
}

// todo: write a test for this
fn read_js_player(source: &str) -> String {
    let player = JS.captures(source).and_then(|cap| cap.at(1)).expect("failed to find jsplayer");
    let player = player.replace("\\/", "/");
    let player = urlparse::unquote(player).expect("failed to read jsplayer as utf8");

    format!("http:{}", player)
}

// here we return the uri for the video and a boolean value
// indicating whether or not encryption is in use for the uri
fn read_uris(source: &str) -> Vec<(String, Option<String>)> {
    let map = MAP.captures(source).and_then(|cap| cap.at(1)).unwrap_or("");
    map.split(',').map(|query| {
        let query = query.replace("\\u0026", "&");
        let mut uri = URL.captures(&query).and_then(|cap| cap.at(1)).expect("query does not contain url").to_owned();

        if let Some(host) = HOST.captures(&query).and_then(|cap| cap.at(1)) {
            uri = uri + "&fallback_host=";
            uri = uri + host;
        }

        if !uri.contains("ratebypass") {
            uri = uri + "&ratebypass=yes";
        }

        (
            urlparse::unquote(uri).expect("failed to read uri as utf8")
                .replace("%2C", ",")
                .replace("%2F", "/"),

            SIGNATURE.captures(&query).and_then(|cap| cap.at(1)).map(|signature| signature.to_owned())
        )
    }).collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn regex_patterns_compile() {
        use client::youtube::{
            JS, 
            MAP, 
            // FMT, 
            // DASHMPD, 
            TITLE,
            SIGNATURE,
            URL,
            HOST,
        };

        JS.is_match("");
        MAP.is_match("");
        // FMT.is_match("");
        // DASHMPD.is_match("");
        TITLE.is_match("");
        SIGNATURE.is_match("");
        URL.is_match("");
        HOST.is_match("");
    }
}
