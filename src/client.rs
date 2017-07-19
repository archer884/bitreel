#[cfg(feature = "default-connector")]
use reqwest;

use error::*;
use regex::Regex;
use std::collections::HashMap;
use std::str;
use video::*;

#[cfg(feature = "default-connector")]
pub struct Client<T: Connector=DefaultConnector> { connector: T }

#[cfg(feature = "default-connector")]
impl Default for Client<DefaultConnector> {
    fn default() -> Client<DefaultConnector> {
        Client { connector: DefaultConnector }
    }
}

#[cfg(not(feature = "default-connector"))]
pub struct Client<T: Connector> { connector: T }

#[cfg(feature = "default-connector")]
impl Client<DefaultConnector> {
    pub fn new() -> Client<DefaultConnector> {
        Client::default()
    }
}

impl<T: Connector> Client<T> {
    pub fn with_connector(connector: T) -> Client<T> {
        Client { connector }
    }

    pub fn query(&self, url: &str) -> Result<Video> {
        let query_type = url.parse::<QueryType>()?;

        match query_type {
            QueryType::Facebook => {
                let id = extract_facebook_id(url)?;
                let info = self.connector.download_string(&url)
                    .ok_or_else(|| Error::network("Unable to download resource"))?;

                Ok(Video::new(id, parse_facebook_streams(&info)?))
            }

            QueryType::YouTube => {
                let id = extract_youtube_id(url)?;
                println!("{}", id);
                let url = format!("http://youtube.com/get_video_info?video_id={}", id);
                let info = self.connector.download_string(&url)
                    .ok_or_else(|| Error::network("Unable to download resource"))?;

                Ok(Video::new(id, parse_youtube_streams(&info)?))
            }
        }
    }
}

pub trait Connector {
    fn download_string(&self, s: &str) -> Option<String>;
}

#[cfg(feature = "default-connector")]
pub struct DefaultConnector;

#[cfg(feature = "default-connector")]
impl Connector for DefaultConnector {
    fn download_string(&self, s: &str) -> Option<String> {
        use std::io::Read;

        let mut response = match reqwest::get(s).ok() {
            None => return None,
            Some(response) => response,
        };

        let mut buf = String::new();
        match response.read_to_string(&mut buf) {
            Err(_) => None,
            Ok(_) => Some(buf),
        }
    }
}

enum QueryType {
    Facebook,
    YouTube,
}

impl str::FromStr for QueryType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        // Here's the issue: because you need to authenticate to access facebook, we do not
        // support downloading facebook videos right now. I may be wrong about this. There
        // might be some videos you can get to on facebook *without* logging in, in which case
        // commenting this out is a dumb idea--but I'm assuming this is correct for right now.

        // if s.contains("facebook.com") {
        //     return Ok(QueryType::Facebook);
        // }

        if s.contains("youtu.be") || s.contains("youtube.com") {
            return Ok(QueryType::YouTube);
        }

        Err(Error::unknown_query_type())
    }
}

fn extract_youtube_id(s: &str) -> Result<&str> {
    // Let's not optimize this prematurely. The odds of these regular expressions
    // being required more than once per execution is next to nil anyway.
    let short_pattern = Regex::new(r#"youtu.be/([^/]*)$"#).unwrap();
    let long_pattern = Regex::new(r#"v=([^&]*)"#).unwrap();

    if let Some(cap) = short_pattern.captures(s) {
        return Ok(cap.get(1).unwrap().as_str())
    }

    if let Some(cap) = long_pattern.captures(s) {
        return Ok(cap.get(1).unwrap().as_str())
    }

    Err(Error::unknown_query_type())
}

fn extract_facebook_id(s: &str) -> Result<&str> {
    let pattern = Regex::new(r#"/([^/]+)/?$"#).unwrap();
    match pattern.captures(s) {
        None => Err(Error::unknown_query_type()),
        Some(cap) => Ok(cap.get(1).unwrap().as_str())
    }
}

fn parse_youtube_streams(info: &str) -> Result<HashMap<StreamKey, String>> {

    fn map_url(url: &str) -> HashMap<String, String> {
        use url::Url;

        // In theory, the video-info repsonse from youtube contains information about the video encoded as a url
        // for no goddamn reason. This imaginary url (based on work by a guy named smoqadam) allows me to use
        // a standard issue URL parser to find out what's in it.

        let fake_url = format!("http://thanks-smoqadam.com?{}", url);
        let parsed_url = Url::parse(&fake_url).expect("invalid url");

        parsed_url.query_pairs().into_owned().collect()
    }

    let map = map_url(info);
    let streams = map.get("url_encoded_fmt_stream_map").ok_or(Error::video("playback restricted"))?
        .split(',')
        .map(|s| {
            let map = map_url(s);
            map.get("quality").and_then(|q| q.to_lowercase().parse::<StreamKey>().ok()).ok_or(Error::video("quality not present"))
                .and_then(|q| map.get("url").map(|u| (q, u.to_string())).ok_or(Error::video("url not present")))
        })
        .collect::<Result<HashMap<_, _>>>()?;

    Ok(streams)
}

fn parse_facebook_streams(s: &str) -> Result<HashMap<StreamKey, String>> {
    let sd_pattern = Regex::new(r#"sd_src:"([^"]+)""#).unwrap();
    let hd_pattern = Regex::new(r#"hd_src:"([^"]+)""#).unwrap();

    // Here we employ determinant parsing for the Facebook streams. I feel like we should do the
    // same thing for YouTube, because I think that would be a good deal easier to read, but...
    // /shrug

    let mut streams = HashMap::new();

    if let Some(cap) = sd_pattern.captures(s) {
        streams.insert(StreamKey::SD, cap.get(1).unwrap().as_str().into());
    }

    if let Some(cap) = hd_pattern.captures(s) {
        streams.insert(StreamKey::HD, cap.get(1).unwrap().as_str().into());
    }

    Ok(streams)
}
