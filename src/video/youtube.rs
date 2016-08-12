use http;
use regex::Regex;
use video::Video;

lazy_static! {
    static ref DECRYPT_FUNC_NAME: Regex = Regex::new(r#"\.sig\|\|(\w+[.])?(\w+)\("#).unwrap();
    static ref FUNC_ID: Regex = Regex::new(r#"(\w+)\("#).unwrap();
}

enum CryptOp {
    Reverse,        // reverses entire string
    Swap(usize),    // swaps character at idx with first character
    Slice(usize),   // removes characters up to idx
}

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
    let pattern = format!("{}=function\\(\\w\\)\\{{(.*?)\\}}", decrypt_function(js));
    let regex = Regex::new(&pattern).expect(&format!("dynamic regex failed to compile: {}", pattern));
    let function_body = regex.captures(js).and_then(|cap| cap.at(1)).expect("unable to capture decrypt function");

    let mut reverse_id = None;
    let mut swap_id = None;
    let mut slice_id = None;

    function_body.split(';').skip(1).filter(|line| !line.starts_with("return")).map(|line| {
        let id = get_id(&line);

        match reverse_id {
            None => if is_reverse_id(id, js) {
                reverse_id = Some(id.to_owned());
                return CryptOp::Reverse;
            },
            Some(ref reverse_id) => if reverse_id == id {
                return CryptOp::Reverse;
            }
        }

        match swap_id {
            None => if is_swap_id(id, js) {
                swap_id = Some(id.to_owned());
                return CryptOp::swap(line);
            },
            Some(ref swap_id) => if swap_id == id {
                return CryptOp::swap(line);
            }
        }

        match slice_id {
            None => if is_slice_id(&id, &js) {
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
    FUNC_ID.captures(s).and_then(|cap| cap.at(1)).unwrap_or("")
}

fn is_reverse_id(id: &str, js: &str) -> bool {
    Regex::new(&format!("{}:\\bfunction\\b\\(\\w+\\)", id))
        .map(|pattern| pattern.is_match(js))
        .unwrap_or(false)
}

fn is_swap_id(id: &str, js: &str) -> bool {
    Regex::new(&format!("{}\\bfunction\\b\\(\\w+\\,\\w\\).\\bvar\\b.\\bc=a\\b", id))
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
    // the C# program I'm porting had a custom-build dfa or state machine or 
    // whatever for this crap... I feel like rust's regex crate is good enough
    // I can just say fuck all that noise

    DECRYPT_FUNC_NAME.captures(js).and_then(|cap| cap.at(2)).expect("unable to find decrypt function")
}

#[cfg(test)]
mod tests {
    #[test]
    fn regex_patterns_compile() {
        use regex::Regex;
        use video::youtube::{
            DECRYPT_FUNC_NAME,
            FUNC_ID,
        };

        DECRYPT_FUNC_NAME.is_match("");
        FUNC_ID.is_match("");

        // this is an example of the dynamic regex used to find the decrypt function
        let pattern = r#"by=function\(\w\)\{(.*?)\}"#;
        Regex::new(pattern).unwrap();
    }

    #[test]
    fn swap() {
        use video::youtube::CryptOp;

        let op = CryptOp::Swap(4);
        let target = ['e', 'b', 'c', 'd', 'a'];
        let mut vec = vec!['a', 'b', 'c', 'd', 'e'];

        op.apply(&mut vec);
        assert_eq!(target, vec.as_ref());
    }

    #[test]
    fn slice() {
        use video::youtube::CryptOp;

        let op = CryptOp::Slice(2);
        let target = ['c', 'd', 'e'];
        let mut vec = vec!['a', 'b', 'c', 'd', 'e'];

        op.apply(&mut vec);
        assert_eq!(target, vec.as_ref());
    }

    #[test]
    fn reverse() {
        use video::youtube::CryptOp;

        let op = CryptOp::Reverse;
        let target = ['e', 'd', 'c', 'b', 'a'];
        let mut vec = vec!['a', 'b', 'c', 'd', 'e'];

        op.apply(&mut vec);
        assert_eq!(target, vec.as_ref());
    }
}
