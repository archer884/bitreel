#![feature(conservative_impl_trait, question_mark)]

#[macro_use] extern crate lazy_static;

extern crate hyper;
extern crate regex;
extern crate urlparse;

pub mod client;
pub mod video;

mod http;
