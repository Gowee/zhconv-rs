//! Raw OpenCC dictionary dataset and parser.
//!
//! License note: the Rust parsing/staging code in this file is licensed under
//! MIT OR Apache-2.0, same as the parent `zhconv` crate. It is independent
//! of the bundled `*.txt` dictionaries, which are licensed under
//! Apache-2.0 (upstream OpenCC).
//! Staging/flattening follows OpenCC's `data/config/*.json` multi-pass dict
//! groups, pre-flattened here so the parent crate can build a single
//! automaton; see the parent crate's `build.rs`.

use std::collections::HashMap;
use std::sync::LazyLock;

use daachorse::{CharwiseDoubleArrayAhoCorasick, CharwiseDoubleArrayAhoCorasickBuilder, MatchKind};
use hex_literal::hex;

// To update upstream rulesets, run `data/update_basic.py` and `cargo fmt`.
pub const OPENCC_COMMIT: &str = "26753884f1984add422f3b0249ccee8613deaff6";
pub const OPENCC_SHA256: [(&str, [u8; 32]); 9] = [
    (
        "HKVariants.txt",
        hex!("e5cd4345303224587102f2c9e4d2b67d2b7e349c6ce9152e4a118f4656cf7302"),
    ),
    (
        "HKVariantsRevPhrases.txt",
        hex!("35352aef4833c2631b2144bc85623cc44d5a09221dda9c32178ea024300d34d3"),
    ),
    (
        "STCharacters.txt",
        hex!("a0ca1601c70648cf48b33c3c6210ccbecc5c7eead4b4c3daf76587ba2c03582b"),
    ),
    (
        "STPhrases.txt",
        hex!("f6eab5e5c6dd7640597878d3dfc6599ee1279d2bc91561eadd8e114194e2925a"),
    ),
    (
        "TSCharacters.txt",
        hex!("737c21c66f55a419dd6956cb3089476cdefc5a36877452631617696df1e5d925"),
    ),
    (
        "TSPhrases.txt",
        hex!("362fa1b9a7d6edd04b462a32e12c9fef3adae822ab1dee9c83561cc37c06cb1f"),
    ),
    (
        "TWPhrases.txt",
        hex!("bcb435b744ee3e522beb9b18fcc5486a36ed4763c6aa642ce18112fb5d604e31"),
    ),
    (
        "TWVariants.txt",
        hex!("e187278e119c427ca561180ac5da5b20e9f8681190458f35c327ce499e95a6a5"),
    ),
    (
        "TWVariantsRevPhrases.txt",
        hex!("5ebfb4bdc938c2b14e01ace378988d5d3dc12462b3496ef1d424951ccd371256"),
    ),
];

pub static OPENCC_SHA256_MAP: LazyLock<HashMap<String, [u8; 32]>> = LazyLock::new(|| {
    OPENCC_SHA256
        .into_iter()
        .map(|(n, s)| (n.to_owned(), s))
        .collect()
});

fn sha256(text: &str) -> [u8; 32] {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hasher.finalize().into()
}

fn raw_unchecked(name: &str) -> &'static str {
    match name {
        "HKVariants.txt" => include_str!("../HKVariants.txt"),
        "HKVariantsRevPhrases.txt" => include_str!("../HKVariantsRevPhrases.txt"),
        "STCharacters.txt" => include_str!("../STCharacters.txt"),
        "STPhrases.txt" => include_str!("../STPhrases.txt"),
        "TSCharacters.txt" => include_str!("../TSCharacters.txt"),
        "TSPhrases.txt" => include_str!("../TSPhrases.txt"),
        "TWPhrases.txt" => include_str!("../TWPhrases.txt"),
        "TWVariants.txt" => include_str!("../TWVariants.txt"),
        "TWVariantsRevPhrases.txt" => include_str!("../TWVariantsRevPhrases.txt"),
        _ => panic!("unknown OpenCC dictionary: {}", name),
    }
}

/// Return a raw dictionary text after validating its SHA256 checksum.
pub fn raw(name: &str) -> &'static str {
    let s = raw_unchecked(name);
    let expected = OPENCC_SHA256_MAP
        .get(name)
        .unwrap_or_else(|| panic!("{} not found in OPENCC_SHA256", name));
    assert_eq!(&sha256(s), expected, "Validating the checksum of {}", name);
    s
}

macro_rules! load_opencc_to {
    ( @read_to $out_conv: expr, $out_revconv: expr, $name: ident) => {
        let s = crate::raw(concat!(stringify!($name), ".txt"));
        crate::parse_opencc_to($out_conv, $out_revconv, s);
    };
    ( @parse_to $out_conv: expr, $out_revconv: expr, $name: ident, $($remainings: tt)* ) => {
        load_opencc_to!(@read_to $out_conv, $out_revconv, $name);
        load_opencc_to!(@parse_to $out_conv, $out_revconv, $($remainings)*);
    };
    ( @parse_to $out_conv: expr, $out_revconv: expr, $name: ident ) => {
        load_opencc_to!(@read_to $out_conv, $out_revconv, $name);
    };
    ( @parse_to $out_conv: expr, $out_revconv: expr, ! $name: ident, $($remainings: tt)* ) => {
        load_opencc_to!(@read_to $out_revconv, $out_conv, $name);
        load_opencc_to!(@parse_to $out_conv, $out_revconv, $($remainings)*);
    };
    ( @parse_to $out_conv: expr, $out_revconv: expr, ! $name: ident ) => {
        load_opencc_to!(@read_to $out_revconv, $out_conv, $name);
    };
    ( @load_stage $out: expr, $prev_stage: ident, [ $($rule: tt)+ ] ) => {
        let (mut prev_convs, prev_revconvs): (HashMap<String, String>, HashMap<String, String>) = $prev_stage.unwrap_or_else(|| (HashMap::new(), HashMap::new()));
        let mut convs: HashMap<String, String> = HashMap::new();
        let mut revconvs: HashMap<String, String> = HashMap::new();
        load_opencc_to!(@parse_to &mut convs, &mut revconvs, $($rule)*);
        let conver: crate::SimpleConverter = convs.clone().into();
        let prev_revconver: crate::SimpleConverter = prev_revconvs.clone().into();
        for (_f, t) in prev_convs.iter_mut() {
            *t = conver.convert(t);
        }
        for (f, t) in convs.iter() {
            prev_convs.insert(f.clone(), t.clone());
            let ff = prev_revconver.convert(f);
            if &ff != f && &ff != t /* ? */ {
                prev_convs.insert(ff.to_owned(), t.to_owned());
            }
        }
        for (_f, t) in revconvs.iter_mut() {
            *t = prev_revconver.convert(t);
        }
        revconvs.extend(prev_revconvs.iter().map(|(f, t)| (conver.convert(f), t.to_owned())));
        revconvs.extend(prev_revconvs.iter().map(|(f, t)| (f.to_owned(), t.to_owned())));
        $prev_stage = Some((prev_convs, revconvs));
    };
    ( $out: expr, $($stage: tt),+ ) => {
        let mut prev_stage = None;
        $(load_opencc_to!(@load_stage $out, prev_stage, $stage);)*
        let (convs, _) = prev_stage.unwrap();
        $out.extend(convs.into_iter());
    };
}

pub fn parse_opencc_to(
    out_conv: &mut HashMap<String, String>,
    out_revconv: &mut HashMap<String, String>,
    s: &str,
) {
    // Strip BOM if present,
    // matching https://github.com/BYVoid/OpenCC/blob/master/src/Lexicon.cpp#L88
    let s = s.strip_prefix('\u{feff}').unwrap_or(s);
    for line in s
        .lines()
        .map(|l| l.trim())
        // Ignore #-prefixed comment lines, but no trailing comments stripping,
        // matching https://github.com/BYVoid/OpenCC/pull/1016
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        if let Some((f, ts)) = line.split_once(char::is_whitespace) {
            if f.is_empty() || ts.is_empty() {
                continue;
            }
            let ts: Vec<_> = ts.split_whitespace().collect();
            if !(ts.len() > 1 && ts.contains(&f)) {
                // be conservative when converting
                // e.g. 范 -> 範 范 can be simply eliminated
                out_conv.insert(f.to_owned(), ts[0].to_owned());
            }
            for t in ts {
                if !out_revconv.contains_key(t) {
                    out_revconv.insert(t.to_owned(), f.to_owned());
                }
            }
        }
    }
}

/// Simplified `ZhConverter` implementation for pre-processing rulesets from OpenCC
pub struct SimpleConverter {
    automaton: Option<CharwiseDoubleArrayAhoCorasick<usize>>,
    target_words: Vec<String>,
}

impl From<HashMap<String, String>> for SimpleConverter {
    fn from(mapping: HashMap<String, String>) -> Self {
        let mut target_words = Vec::with_capacity(mapping.len());
        let automaton = if mapping.is_empty() {
            None
        } else {
            Some(
                CharwiseDoubleArrayAhoCorasickBuilder::new()
                    .match_kind(MatchKind::LeftmostLongest)
                    .build(mapping.into_iter().map(|(f, t)| {
                        target_words.push(t);
                        f
                    }))
                    .expect("Conversion table is valid"),
            )
        };
        Self {
            automaton,
            target_words,
        }
    }
}

impl SimpleConverter {
    #[allow(dead_code)]
    pub fn build<'s>(pairs: impl Iterator<Item = (&'s str, &'s str)>) -> Self {
        let mapping = HashMap::from_iter(pairs.map(|(a, b)| (a.to_owned(), b.to_owned())));
        mapping.into()
    }

    pub fn convert(&self, text: &str) -> String {
        match &self.automaton {
            Some(automaton) => {
                let mut output = String::new();
                let mut last = 0;
                // leftmost-longest matching
                for (s, e, ti) in automaton
                    .leftmost_find_iter(text)
                    .map(|m| (m.start(), m.end(), m.value()))
                {
                    if s > last {
                        output.push_str(&text[last..s]);
                    }
                    output.push_str(&self.target_words[ti]);
                    last = e;
                }
                output.push_str(&text[last..]);
                output
            }
            None => String::from(text),
        }
    }
}

// Ref: https://github.com/BYVoid/OpenCC/blob/29d33fb8edb8c95e34691c8bd1ef76a50d0b5251/data/config/
// Staging mirrors the previous `build.rs` match arms one-to-one; the parent
// crate decides per-target gating and merges the result with MediaWiki pairs.
// Each loader appends into the caller's vec (no intermediate allocation).

// config: t2s
pub fn load_hans_pairs(out: &mut Vec<(String, String)>) {
    load_opencc_to!(out, [TSCharacters, TSPhrases]);
}

// config: s2t
pub fn load_hant_pairs(out: &mut Vec<(String, String)>) {
    load_opencc_to!(out, [STCharacters, STPhrases]);
}

pub fn load_tw_pairs(out: &mut Vec<(String, String)>, twp: bool) {
    if twp {
        // config: s2tw & s2twp & t2tw
        load_opencc_to!(out, [STPhrases, STCharacters], [TWPhrases], [TWVariants]);
    } else {
        // config: s2tw & t2tw
        load_opencc_to!(out, [STPhrases, STCharacters], [TWVariants]);
    }
}

pub fn load_hk_pairs(out: &mut Vec<(String, String)>) {
    // config: s2hk & t2hk
    load_opencc_to!(out, [STPhrases, STCharacters], [HKVariants]);
}

pub fn load_cn_pairs(out: &mut Vec<(String, String)>, twp: bool) {
    if twp {
        // config: tw2sp; "!TWVariants" deliberately omitted
        load_opencc_to!(
            out,
            [!TWPhrases, TWVariantsRevPhrases],
            [TSPhrases, TSCharacters]
        );
    } else {
        // config: tw2s; "!TWVariants" deliberately omitted to prevent
        // misconversions like `么 -> 幺, 抬 -> 檯, 著 -> 着`
        load_opencc_to!(out, [TWVariantsRevPhrases], [TSPhrases, TSCharacters]);
    }
    // config: hk2s; "!HKVariants" deliberately omitted
    load_opencc_to!(out, [HKVariantsRevPhrases], [TSPhrases, TSCharacters]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_is_validated_and_nonempty() {
        assert!(!raw("STCharacters.txt").is_empty());
        assert!(!raw("TSPhrases.txt").is_empty());
    }

    #[test]
    fn loaders_return_nonempty_tables() {
        let mut hans = Vec::new();
        load_hans_pairs(&mut hans);
        assert!(!hans.is_empty());

        let mut hant = Vec::new();
        load_hant_pairs(&mut hant);
        assert!(!hant.is_empty());

        for twp in [false, true] {
            let mut tw = Vec::new();
            load_tw_pairs(&mut tw, twp);
            assert!(!tw.is_empty(), "tw_pairs({}) should not be empty", twp);

            let mut cn = Vec::new();
            load_cn_pairs(&mut cn, twp);
            assert!(!cn.is_empty(), "cn_pairs({}) should not be empty", twp);
        }

        let mut hk = Vec::new();
        load_hk_pairs(&mut hk);
        assert!(!hk.is_empty());
    }
}
