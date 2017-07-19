#[cfg(feature = "default-connector")]
extern crate reqwest;

#[cfg(feature = "evil-mode")]
#[macro_use]
extern crate lazy_static;

#[cfg(feature = "evil-mode")]
extern crate urlparse;

extern crate regex;
extern crate url;

pub mod client;
pub mod error;
pub mod video;
