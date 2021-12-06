use std::collections::HashMap;

use lazy_static::lazy_static;
use regex::Regex;
use aho_corasick::AhoCorasick;

pub struct ZhConverter {
    // regex: Regex,
    ac: AhoCorasick,
    mapping: HashMap<String, String>, // Or str?
}

impl ZhConverter {
    pub fn new(ac: AhoCorasick, mapping: HashMap<String, String>) -> ZhConverter {
        ZhConverter { ac, mapping }
    }

    // pub fn from_pairs(mut pairs: Vec<(String, String)>) -> ZhConverter {
    //     pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    //     let mut pat = String::with_capacity(pairs.len() * 2 + 1);
    //     let mut mapping = HashMap::new();
    //     let mut it = pairs.into_iter().peekable();
    //     while let Some((from, to)) = it.next() {
    //         pat.push_str(&from);
    //         if it.peek().is_some() {
    //             pat.push_str("|");
    //         }
    //         mapping.insert(from, to);
    //     }
    //     ZhConverter {
    //         regex: Regex::new(&pat).unwrap(),
    //         mapping,
    //     }
    // }

    pub fn convert(&self, text: &str) -> String {
        // Ref: https://github.dev/rust-lang/regex/blob/5197f21287344d2994f9cf06758a3ea30f5a26c3/src/re_trait.rs#L192
        let mut converted = String::with_capacity(text.len());
        let mut last = 0;
        for (s, e) in self.ac.find_iter(text).map(|m| (m.start(), m.end())) {
            if s > last {
                converted.push_str(&text[last..s]);
            }
            converted.push_str(&self.mapping.get(&text[s..e]).unwrap());
            last = e;
        }
        converted.push_str(&text[last..]);
        converted
    }

    // pub fn convert

    // pub fn convert_inplace(&self, text: &mut String) {
    //     let tbp = VecDeque::<&str>::new(); // to be pushed
    //     let mut wi = 0; // writing index
    //     let mut ri = 0; // reading index
    //     while let Some((s, e)) = self.regex.find_at(text, ri).map(|m| (m.start(), m.end())) {
    //         while !tbp.is_empty() && s - wi >= tbp[0].len() {
    //             let raw = unsafe { text.as_bytes_mut() };
    //             raw[wi..wi + tbp[0].len()].clone_from_slice(tbp[0].as_bytes());
    //             tbp.pop_front();
    //         }
    //     }
    // }
}
