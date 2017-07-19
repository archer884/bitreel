use error::*;
use std::collections::hash_map::*;
use std::str;

// Not sure this stream key concept is the right way to go, even if you forget about the problem 
// of the terrible name. This is because these three "keys" can't support all the available 
// formats (for obvious reasons). It could be that what I want is to go through all available 
// formats and choose some subset of those to present to the user, a process I could represent 
// with some struct that provides the results of that.

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum StreamKey { LD, SD, HD }

impl str::FromStr for StreamKey {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "small" => Ok(StreamKey::LD),
            "sd_src" | "medium" => Ok(StreamKey::SD),
            "hd_src" | "hd720" | "large" => Ok(StreamKey::HD),

            _ => Err(Error::video("Invalid stream key")),
        }
    }
}

pub struct Video {
    identifier: String,
    streams: HashMap<StreamKey, String>
}

impl Video {
    pub fn new<T: Into<String>>(identifier: T, streams: HashMap<StreamKey, String>) -> Video {
        Video {
            identifier: identifier.into(),
            streams
        }
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn get_stream(&self, key: StreamKey) -> Option<&str> {
        self.streams.get(&key).map(|s| s.as_ref())
    }

    pub fn best_stream(&self) -> Option<&str> {
        unimplemented!()
    }

    pub fn streams(&self) -> Iter<StreamKey, String> {
        self.streams.iter()
    }
}
