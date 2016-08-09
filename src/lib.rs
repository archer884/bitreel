#![feature(question_mark)]

#[macro_use] extern crate lazy_static;

extern crate hyper;
extern crate regex;

pub mod client;
pub mod video;

mod http;
