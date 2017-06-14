use client::*;
use error::*;
use std::collections::HashMap;
use video::YoutubeVideo;

#[derive(Default)]
pub struct YoutubeClient;

impl YoutubeClient {
    pub fn new() -> YoutubeClient {
        YoutubeClient
    }
}

impl Client for YoutubeClient {
    type Video = YoutubeVideo;

    fn query<C: ClientConnector>(&self, identifier: &str, connector: &C) -> Result<Self::Video> {
        let uri = format!("http://youtube.com/get_video_info?video_id={}", identifier);
        let info = connector.download_string(&uri)?;
        parse_videos(&info)
    }
}

fn parse_videos(info: &str) -> Result<YoutubeVideo> {
    let map = map_url(info);
    let title = map.get("title").ok_or(Error::video("title not found"))?.to_string();
    let streams = map.get("url_encoded_fmt_stream_map").ok_or(Error::video("playback restricted"))?
        .split(',')
        .map(|s| {
            let map = map_url(s);
            map.get("quality").map(|q| q.to_lowercase()).ok_or(Error::video("quality not present"))
                .and_then(|q| map.get("url").map(|u| (q, u.to_string())).ok_or(Error::video("url not present")))
        })
        .collect::<Result<HashMap<_, _>>>()?;

    Ok(YoutubeVideo::new(title, streams))
}

fn map_url(url: &str) -> HashMap<String, String> {
    use url::Url;

    // In theory, the video-info repsonse from youtube contains information about the video encoded as a url
    // for no goddamn reason. This imaginary url (based on work by a guy named smoqadam) allows me to use
    // a standard issue URL parser to find out what's in it.
    let fake_url = format!("http://thanks-smoqadam.com?{}", url);
    let parsed_url = Url::parse(&fake_url).expect("invalid url");

    parsed_url.query_pairs().into_owned().collect()
}
