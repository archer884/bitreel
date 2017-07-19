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

            QueryType::YouTube => youtube::process_query(url, &self.connector),
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

#[cfg(not(feature = "evil-mode"))]
mod youtube {
    use client::{self, Connector};
    use error::*;
    use std::collections::HashMap;
    use video::*;

    pub fn process_query(url: &str, connector: &Connector) -> Result<Video> {
        let id = client::extract_youtube_id(url)?;
        let url = format!("http://youtube.com/get_video_info?video_id={}", id);
        let info = connector.download_string(&url)
            .ok_or_else(|| Error::network("Unable to download resource"))?;

        Ok(Video::new(id, parse_youtube_streams(&info)?))
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
}

#[cfg(feature = "evil-mode")]
mod youtube {
    use client::{self, Connector};
    use error::*;
    use regex::Regex;
    use std::collections::HashMap;
    use urlparse;
    use video::*;

    lazy_static! {
        static ref DECRYPT_FUNC_NAME: Regex = Regex::new(r#"\.sig\|\|(\w+[.])?(\w+)\("#).unwrap();
        static ref FUNC_ID: Regex = Regex::new(r#"(\w+)\("#).unwrap();
        static ref HOST: Regex = Regex::new(r#"fallback_host=([^&]+)"#).unwrap();
        static ref JS: Regex = Regex::new(r#""js":"([^"]+)""#).unwrap();
        static ref MAP: Regex = Regex::new(r#""url_encoded_fmt_stream_map":"([^"]+)""#).unwrap();
        static ref SIGNATURE: Regex = Regex::new(r#"s=([^&]+)"#).unwrap();
        static ref URL: Regex = Regex::new(r#"url=([^&]+)"#).unwrap();
    }

    /// A crypto operation used to descramble a YouTube key value.
    enum CryptOp {
        /// Reverses the entirety of the string
        Reverse,

        /// Swaps the string at `usize` with the first character in the string
        Swap(usize),

        /// Removes all characters up to `usize`
        Slice(usize),
    }

    // FIXME:
    // All this garbage just panics if anything goes wrong, because this version never got beyond
    // prototyping. You may want to fix that, because otherwise it's gonna be impossible to debug
    // this crap if it ever breaks.

    impl CryptOp {
        fn swap(s: &str) -> CryptOp {
            let numeric_values: String = s.chars().filter(|value| value.is_numeric()).collect();
            CryptOp::Swap(numeric_values.parse().expect(&format!("swap unable to parse {:?} {:?} as number", s,numeric_values)))
        }

        fn slice(s: &str) -> CryptOp {
            let numeric_values: String = s.chars().filter(|value| value.is_numeric()).collect();
            CryptOp::Slice(numeric_values.parse().expect(&format!("swap unable to parse {:?} {:?} as number", s,numeric_values)))
        }

        fn apply(&self, sig: &mut Vec<char>) {
            match *self {
                CryptOp::Reverse => sig.reverse(),
                CryptOp::Swap(idx) => sig.swap(0, idx),
                CryptOp::Slice(idx) => *sig = sig.split_off(idx),
            }
        }
    }

    pub fn process_query(url: &str, connector: &Connector) -> Result<Video> {
        let page_response = connector.download_string(url).ok_or_else(|| Error::network("Unable to load url"))?;
        let js_player = read_js_player(&page_response);
        let url_results = read_urls(&page_response)
            .inspect(|&(ref url, _)| println!("{}", url))
            .map(|(url, signature)| {
                match signature {
                    None => url,
                    Some(ref signature) => decrypt(&url, &js_player, signature, connector),
                }
            });

        for url in url_results {
            println!("{}", url);
        }

        unimplemented!()
    }

    fn read_js_player(s: &str) -> String {
        let player = JS.captures(s).and_then(|cap| cap.get(1)).expect("Unable to load javascript player");
        let player = player.as_str().replace("\\/", "/");
        let player = urlparse::unquote(player).expect("failed to parse url as utf8");
        "http:".to_owned() + &player
    }

    fn read_urls<'a>(source: &'a str) -> Box<Iterator<Item=(String, Option<String>)> + 'a> {
        let map = MAP.captures(source)
            .and_then(|cap| cap.get(1).map(|cap| cap.as_str()))
            .unwrap_or("");

        let uris = map.split(',')
            .map(|query| {
                let query = query.replace("\\u0026", "&");
                let mut uri = URL.captures(&query)
                    .and_then(|cap| cap.get(1)).expect("query does not contain url")
                    .as_str()
                    .to_owned();

                if let Some(host) = HOST.captures(&query).and_then(|cap| cap.get(1)) {
                    let host = host.as_str();
                    uri += "&fallback_host=";
                    uri += host;
                }

                if !uri.contains("ratebypass") {
                    uri += "&ratebypass=yes";
                }

                (
                    urlparse::unquote(uri).expect("failed to read uri as utf8")
                        .replace("%2C", ",")
                        .replace("%2F", "/"),

                    SIGNATURE.captures(&query)
                        .and_then(|cap| cap.get(1))
                        .map(|signature| signature.as_str().to_owned())
                )
            });

        Box::new(uris)
    }

    fn decrypt(uri: &str, js_player: &str, signature: &str, connector: &Connector) -> String {
        let js = connector.download_string(js_player).expect("failed to download js");
        let operations = discover_operations(&js);

        let mut signature: Vec<_> = signature.chars().collect();
        for op in operations {
            op.apply(&mut signature);
        }

        let signature: String = signature.iter().cloned().collect();
        format!("{}&signature={}", uri, signature)
    }

    fn discover_operations<'a>(js: &'a str) -> Box<Iterator<Item = CryptOp> + 'a> {
        let pattern = format!("{}=function\\(\\w\\)\\{{(.*?)\\}}", decrypt_function(js));
        let regex = Regex::new(&pattern).expect(&format!("dynamic regex failed to compile: {}", pattern));
        let function_body = regex.captures(js)
            .map(|cap| cap.get(1).unwrap().as_str())
            .expect("unable to capture decrypt function");

        let mut reverse_id = None;
        let mut swap_id = None;
        let mut slice_id = None;

        // TODO: remove this, or turn it into an error return thing...
        // this flat-map exists because I was considering just skipping invalid transforms
        let operations = function_body.split(';').skip(1).filter(|line| !line.starts_with("return")).flat_map(move |line| {
            let id = get_id(&line);

            match reverse_id {
                None => if is_reverse_id(id, js) {
                    reverse_id = Some(id.to_owned());
                    return Some(CryptOp::Reverse);
                },
                Some(ref reverse_id) => if reverse_id == id {
                    return Some(CryptOp::Reverse);
                }
            }

            match swap_id {
                None => if is_swap_id(id, js) {
                    swap_id = Some(id.to_owned());
                    return Some(CryptOp::swap(line));
                },
                Some(ref swap_id) => if swap_id == id {
                    return Some(CryptOp::swap(line));
                }
            }

            match slice_id {
                None => if is_slice_id(&id, &js) {
                    slice_id = Some(id.to_owned());
                    return Some(CryptOp::slice(line));
                },
                Some(ref slice_id) => if slice_id == id {
                    return Some(CryptOp::slice(line));
                }
            }

            panic!(format!("unknown transform: {} + {}", line, id));
        });

        Box::new(operations)
    }

    fn get_id(s: &str) -> &str {
        FUNC_ID.captures(s).map(|cap| cap.get(1).unwrap().as_str()).unwrap_or("")
    }

    fn is_reverse_id(id: &str, js: &str) -> bool {
        Regex::new(&format!("{}:\\bfunction\\b\\(\\w+\\)", id))
            .map(|pattern| pattern.is_match(js))
            .unwrap_or(false)
    }

    fn is_swap_id(id: &str, js: &str) -> bool {
        Regex::new(&format!("{}:\\bfunction\\(\\w+,\\w+\\).\\bvar", id))
            .map(|pattern| pattern.is_match(js))
            .unwrap_or(false)
    }

    fn is_slice_id(id: &str, js: &str) -> bool {
        Regex::new(&format!("{}:\\bfunction\\b\\([a],b\\).(\\breturn\\b)?.?\\w+\\.", id))
            .map(|pattern| pattern.is_match(js))
            .unwrap_or(false)
    }

    /// Returns the decryption function found in the javascript source code
    fn decrypt_function(js: &str) -> &str {
        // the C# program I'm porting had a custom-built dfa or state machine or
        // whatever for this crap... I feel like rust's regex crate is good enough
        // I can just say fuck all that noise

        DECRYPT_FUNC_NAME.captures(js)
            .and_then(|cap| cap.get(2)).expect("unable to find decrypt function")
            .as_str()
    }
}
