use http;
use regex::Regex;
use video::Video;

lazy_static! {
    static ref DECRYPT_FUNC_NAME: Regex = Regex::new(r#"\.sig\|\|(.+?)\(.+?\)"#).unwrap();
}

enum CryptOp {
    Reverse,        // reverses entire string
    Swap(usize),    // swaps character at idx with first character
    Slice(usize),   // removes characters up to idx
}

impl CryptOp {
    fn swap(s: &str) -> CryptOp {
        unimplemented!()
    }

    fn slice(s: &str) -> CryptOp {
        unimplemented!()
    }

    fn apply(&self, sig: &mut Vec<char>) {
        match *self {
            CryptOp::Reverse => *sig = sig.iter().cloned().rev().collect(),
            CryptOp::Swap(idx) => sig.swap(0, idx),
            CryptOp::Slice(idx) => *sig = sig.split_off(idx),
        }
    }
}

#[derive(Debug)]
pub struct YoutubeVideo {
    title: String,
    uri: String,
    js_player: String,
}

impl YoutubeVideo {
    pub fn new(title: &str, uri: &str, js_player: &str, signature: &Option<String>) -> YoutubeVideo {
        let mut uri = uri.to_owned();
        if let &Some(ref signature) = signature {
            uri = decrypt(&uri, js_player, signature);
        }

        YoutubeVideo {
            title: title.to_owned(),
            uri: uri,
            js_player: js_player.to_owned(),
        }
    }
}

impl Video for YoutubeVideo {
    fn title(&self) -> &str {
        &self.title
    }
    
    fn uri(&mut self) -> &str {
        &self.uri
    }
}

fn decrypt(uri: &str, js_player: &str, signature: &str) -> String {
    let js = http::download_string(js_player).expect("failed to download js");
    let operations = discover_operations(&js);

    let mut signature: Vec<_> = signature.chars().collect();
    for op in operations {
        op.apply(&mut signature);
    }

    let signature: String = signature.iter().cloned().collect();
    format!("{}&signature={}", uri, signature)
}

fn discover_operations(js: &str) -> Vec<CryptOp> {
    let function_pattern = Regex::new(&format!("{}=function\\(\\w\\){{(.*?)}}", decrypt_function(js))).expect("dynamic regex failed to compile");
    let function_body = function_pattern.captures(js).and_then(|cap| cap.at(1)).expect("unable to capture decrypt function");

    let mut reverse_id = None;
    let mut swap_id = None;
    let mut slice_id = None;

    function_body.split(';').skip(1).filter(|line| !line.starts_with("return")).map(|line| {
        let id = get_id(&line);

        match reverse_id {
            None => if is_reverse_id(&id) {
                reverse_id = Some(id.to_owned());
                return CryptOp::Reverse;
            },
            Some(ref reverse_id) => if reverse_id == id {
                return CryptOp::Reverse;
            }
        }

        match swap_id {
            None => if is_swap_id(&id) {
                swap_id = Some(id.to_owned());
                return CryptOp::swap(line);
            },
            Some(ref swap_id) => if swap_id == id {
                return CryptOp::swap(line);
            }
        }

        match slice_id {
            None => if is_slice_id(&id) {
                slice_id = Some(id.to_owned());
                return CryptOp::slice(line);
            },
            Some(ref slice_id) => if slice_id == id {
                return CryptOp::slice(line);
            }
        }

        panic!("unknown crypto transform");
    }).collect()
}

fn get_id(s: &str) -> &str {
    unimplemented!()
}

fn is_reverse_id(id: &str) -> bool {
    unimplemented!()
}

fn is_swap_id(id: &str) -> bool {
    unimplemented!()
}

fn is_slice_id(id: &str) -> bool {
    unimplemented!()
}

/// Returns the decryption function found in the javascript source code
fn decrypt_function(js: &str) -> &str {
    // the C# program I'm porting had a custom-build dfa or state machine or 
    // whatever for this crap... I feel like rust's regex crate is good enough
    // I can just say fuck all that noise

    DECRYPT_FUNC_NAME.captures(js).and_then(|cap| cap.at(1)).expect("unable to find decrypt function")
}

#[cfg(test)]
mod tests {
    #[test]
    fn regex_patterns_work() {
        use video::youtube::{
            DECRYPT_FUNC_NAME
        };

        DECRYPT_FUNC_NAME.is_match("");
    } 
}
