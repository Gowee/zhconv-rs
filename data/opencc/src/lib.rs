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
use std::collections::HashSet;
use std::sync::LazyLock;

use daachorse::{CharwiseDoubleArrayAhoCorasick, CharwiseDoubleArrayAhoCorasickBuilder, MatchKind};
use hex_literal::hex;

// To update upstream rulesets, run `data/update_basic.py` and `cargo fmt`.
pub const OPENCC_COMMIT: &str = "3ac34aa439a9908dd49fa92b5174b46314787ac2";
pub const OPENCC_SHA256: [(&str, [u8; 32]); 15] = [
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
        hex!("9ff46a7d30e5765375eb13d33f2b03a34d298913caf2b120380679f33ae1642d"),
    ),
    (
        "TSPhrases.txt",
        hex!("9a23666e95c97dbf8668b5d71ca19f09f33b4f3a3aa9e05677dab4b608cef102"),
    ),
    (
        "TWPhrases.txt",
        hex!("bcb435b744ee3e522beb9b18fcc5486a36ed4763c6aa642ce18112fb5d604e31"),
    ),
    (
        "TWVariants.txt",
        hex!("245b94eb5842957e735dd44b7e7d4ff469a3643126cc8fa511adda5281e9cb86"),
    ),
    (
        "TWVariantsRevPhrases.txt",
        hex!("5ebfb4bdc938c2b14e01ace378988d5d3dc12462b3496ef1d424951ccd371256"),
    ),
    (
        "CJK_Compatibility_Ideographs.txt",
        hex!("e9623acec48d384f99d37d2760bd14a00b6d0cddad7707290dbb0b8c97a2904a"),
    ),
    (
        "TWVariantsPhrases.txt",
        hex!("36df033675a2e9152927fa8419f0732a061cb4909c5060f25a5cfc9b08ffff06"),
    ),
    (
        "HKVariantsPhrases.txt",
        hex!("e23019c35405065d7ea174fe7487e0bde064b1af56532b3986033c9bb98e555c"),
    ),
    (
        "HKPhrases.txt",
        hex!("de3ed57532c9e57e73998f696ec8afe49307b44aa01a8342aa9ae6ed1f6dd9da"),
    ),
    (
        "HKPhrasesRev.txt",
        hex!("081b5b8908b4f7217d1f6b3c11be21e02210b6753c2b0d8d6379fe21c1a40a20"),
    ),
    (
        "TWPhrasesRev.txt",
        hex!("2c13969308cf4b9216cfb0145cefb9da96c5d0329a393850c2a63b341caaf4bd"),
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
        "CJK_Compatibility_Ideographs.txt" => {
            include_str!("../CJK_Compatibility_Ideographs.txt")
        }
        "TWVariantsPhrases.txt" => include_str!("../TWVariantsPhrases.txt"),
        "HKVariantsPhrases.txt" => include_str!("../HKVariantsPhrases.txt"),
        "HKPhrases.txt" => include_str!("../HKPhrases.txt"),
        "HKPhrasesRev.txt" => include_str!("../HKPhrasesRev.txt"),
        "TWPhrasesRev.txt" => include_str!("../TWPhrasesRev.txt"),
        _ => panic!("unknown OpenCC dictionary: {}", name),
    }
}

/// Return a raw dictionary text after validating its SHA256 checksum.
///
/// The generated dictionary is served from memory instead (derived from
/// validated inputs, so no checksum of its own).
pub fn raw(name: &str) -> &'static str {
    // Mirroring upstream's `STPhrases_GeneratedFromRegionalPhrases.txt` (built,
    // not committed, by OpenCC). Usable in stage lists like any committed file.
    if name == "STPhrases_GeneratedFromRegionalPhrases.txt" {
        return &STPHRASES_GENERATED_TEXT;
    }
    let s = raw_unchecked(name);
    let expected = OPENCC_SHA256_MAP
        .get(name)
        .unwrap_or_else(|| panic!("{} not found in OPENCC_SHA256", name));
    assert_eq!(&sha256(s), expected, "Validating the checksum of {}", name);
    s
}

macro_rules! load_chain_to {
    ( @load_dict_to $out_mapping: expr, $out_rev_mapping: expr, $name: ident) => {
        let s = crate::raw(concat!(stringify!($name), ".txt"));
        crate::load_dict_to($out_mapping, $out_rev_mapping, s);
    };
    ( @load_dicts_to $out_mapping: expr, $out_rev_mapping: expr, $name: ident, $($remainings: tt)* ) => {
        load_chain_to!(@load_dict_to $out_mapping, $out_rev_mapping, $name);
        load_chain_to!(@load_dicts_to $out_mapping, $out_rev_mapping, $($remainings)*);
    };
    ( @load_dicts_to $out_mapping: expr, $out_rev_mapping: expr, $name: ident ) => {
        load_chain_to!(@load_dict_to $out_mapping, $out_rev_mapping, $name);
    };
    ( @load_dicts_to $out_mapping: expr, $out_rev_mapping: expr, ! $name: ident, $($remainings: tt)* ) => {
        load_chain_to!(@load_dict_to $out_rev_mapping, $out_mapping, $name);
        load_chain_to!(@load_dicts_to $out_mapping, $out_rev_mapping, $($remainings)*);
    };
    ( @load_dicts_to $out_mapping: expr, $out_rev_mapping: expr, ! $name: ident ) => {
        load_chain_to!(@load_dict_to $out_rev_mapping, $out_mapping, $name);
    };
    // One conversion-chain link: fold this stage's dicts into the flattened
    // mappings, emulating OpenCC's multi-pass application of chain links.
    // `$out` is only the final sink (see the top-level arm below); all
    // chaining state threads through `$chain_mappings: Option<(chain_mapping, chain_rev_mapping)>`.
    ( @load_stage $out: expr, $chain_mappings: ident, [ $($dict: tt)+ ] ) => {
        // forward and backward mappings, flattened/aggregated so far
        let (mut chain_mapping, chain_rev_mapping): (HashMap<String, String>, HashMap<String, String>) = $chain_mappings.unwrap_or_else(|| (HashMap::new(), HashMap::new()));
        // build forward & backward mappings of all dicts of this stage merged together
        // (OpenCC short_circuit match_policy is infeasible anyway with our single-pass AC, our
        // merging is functionally equivalent to union match_policy)
        let mut stage_mapping: HashMap<String, String> = HashMap::new();
        let mut stage_rev_mapping: HashMap<String, String> = HashMap::new();
        load_chain_to!(@load_dicts_to &mut stage_mapping, &mut stage_rev_mapping, $($dict)*);
        let stage_conver: crate::SimpleConverter = stage_mapping.clone().into();
        let chain_revconver: crate::SimpleConverter = chain_rev_mapping.clone().into();

        // Chain forward: re-convert previous target words through this stage,
        // e.g. `内存条 -> 內存條` ----> `内存条 -> 記憶體模組`.
        for (_f, t) in chain_mapping.iter_mut() {
            *t = stage_conver.convert(t);
        }
        for (f, t) in stage_mapping.iter() {
            // Absorb all pairs of this stage into the chain mapping.
            // TODO: prefer earlier or later (cross-stage collisions measure 0
            // on current data, reported by agent today).
            // TODO: log dups
            chain_mapping.insert(f.clone(), t.clone());
            // Chain backward: reverse-convert source words of this stage
            // through the reverse mapping of earlier stages,
            // e.g. `內存條 -> 記憶體模組` --rev--> `內存条 -> 記憶體模組`
            let ff = chain_revconver.convert(f);
            if &ff != f && &ff != t /* ? */ {
                // TODO: log dups
                chain_mapping.insert(ff.to_owned(), t.to_owned());
            }
        }
        // Chain forward & backward and absorb reverse pairs of this stage, for the reverse (backward)
        // mapping, which is only used internally to the chain, not exposed to the caller (the
        // caller only sees the final forward mapping).
        for (_f, t) in stage_rev_mapping.iter_mut() {
            *t = chain_revconver.convert(t);
        }
        stage_rev_mapping.extend(chain_rev_mapping.iter().map(|(f, t)| (stage_conver.convert(f), t.to_owned())));
        stage_rev_mapping.extend(chain_rev_mapping.iter().map(|(f, t)| (f.to_owned(), t.to_owned())));
        $chain_mappings = Some((chain_mapping, stage_rev_mapping));
    };
    // Entry point: run each `[...]` stage in order, then move the final forward
    // chain mapping into the caller vec. Reverse maps are chain-internal only and dropped.
    ( $out: expr, $($stage: tt),+ ) => {
        let mut chain_mappings = None;
        $(load_chain_to!(@load_stage $out, chain_mappings, $stage);)*
        let (chain_mapping, _) = chain_mappings.unwrap();
        $out.extend(chain_mapping.into_iter());
    };
}

/// Split raw dict text into `(key, values-part)` items, skipping blanks,
/// `#`-comments and malformed lines. Shared by [`load_dict_to`] and the
/// [`STPHRASES_GENERATED`] key extraction below (which needs keys even for lines
/// whose entry the conservative rule would skip, mirroring upstream
/// `Dict().iter`).
fn parse_dict(s: &str) -> impl Iterator<Item = (&str, impl Iterator<Item = &str>)> {
    // Strip BOM if present,
    // matching https://github.com/BYVoid/OpenCC/blob/master/src/Lexicon.cpp#L88
    let s = s.strip_prefix('\u{feff}').unwrap_or(s);
    s.lines().map(|l| l.trim()).filter_map(|l| {
        // Ignore #-prefixed comment lines, but no trailing comments stripping,
        // matching https://github.com/BYVoid/OpenCC/pull/1016
        if l.is_empty() || l.starts_with('#') {
            return None;
        }
        // TODO: split at tab only?
        let (key, ts) = l.split_once(char::is_whitespace)?;
        if key.is_empty() || ts.is_empty() {
            return None;
        }
        Some((key, ts.split_whitespace()))
    })
}

pub fn load_dict_to(
    out_mapping: &mut HashMap<String, String>,
    out_rev_mapping: &mut HashMap<String, String>,
    s: &str,
) {
    for (f, ts) in parse_dict(s) {
        let ts: Vec<_> = ts.collect();
        if !(ts.len() > 1 && ts.contains(&f)) {
            // be conservative when converting
            // e.g. 范 -> 範 范 can be simply eliminated
            // 1-char identity pairs like 范 -> 范 is meaningless in our leftmost-longest
            // matching, since longer source phrases containing the char always shadow it
            // TO: allow identity conversion?

            // Prefer earlier rules, matching the behavior of `union` match_policy in OpenCC.
            out_mapping.entry(f.to_owned()).or_insert(ts[0].to_owned());
        }
        for t in ts {
            if !out_rev_mapping.contains_key(t) {
                out_rev_mapping.insert(t.to_owned(), f.to_owned());
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

/// Simp-keyed entries generated from regional phrase keys via t2s, mirroring
/// `data/scripts/generate_st_phrases_from_regional_phrases.py`
/// (`--input HKPhrases.txt --input TWPhrases.txt`, stock `t2s.json`).
/// Keys converted with our own flattened t2s tables (a `SimpleConverter`
/// over `load_hans_pairs`, so CJK chaining is included); converted keys
/// shorter than 3 chars are skipped, same as upstream (`len < 3`).
/// Upstream FAILS the data build on key conflicts across inputs; here the
/// first input file wins silently (HKPhrases before TWPhrases, CMake order),
/// consistent with union first-wins elsewhere. No identity filtering, also
/// matching the script.
static STPHRASES_GENERATED: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    let mut t2s_pairs = Vec::new();
    load_hans_pairs(&mut t2s_pairs);
    let mut t2s_map = HashMap::new();
    for (f, t) in t2s_pairs {
        t2s_map.entry(f).or_insert(t);
    }
    let t2s = SimpleConverter::from(t2s_map);

    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for name in ["HKPhrases.txt", "TWPhrases.txt"] {
        for (key, _) in parse_dict(raw(name)) {
            let converted = t2s.convert(key);
            if converted.chars().count() < 3 {
                continue;
            }
            if seen.insert(converted.clone()) {
                out.push((converted, key.to_owned()));
            }
        }
    }
    out
});

/// [`STPHRASES_GENERATED`] rendered as standard dict-file text, so stages consume it
/// through [`load_dict_to`] exactly like a committed file.
static STPHRASES_GENERATED_TEXT: LazyLock<String> = LazyLock::new(|| {
    STPHRASES_GENERATED
        .iter()
        .map(|(f, t)| format!("{}\t{}\n", f, t))
        .collect()
});

// Ref: https://github.com/BYVoid/OpenCC/blob/29d33fb8edb8c95e34691c8bd1ef76a50d0b5251/data/config/
// Staging mirrors the upstream configs one-to-one; the parent crate decides
// per-target gating and merges the result with MediaWiki pairs.
// Each loader appends into the caller's vec (no intermediate allocation).
//
// Union semantics: files within a stage merge with earlier rules winning
// (`load_dict_to` first-wins, matching OpenCC's union/first-wins). File
// order follows upstream dict order. `short_circuit`'s first-dict-match-wins
// is not emulated: a single leftmost-longest automaton cannot express it.

// config: t2s (+ CJK pre-normalization, as in every upstream config)
pub fn load_hans_pairs(out: &mut Vec<(String, String)>) {
    load_chain_to!(
        out,
        [CJK_Compatibility_Ideographs],
        [TSPhrases, TSCharacters]
    );
}

// config: s2t (+ CJK pre-normalization)
pub fn load_hant_pairs(out: &mut Vec<(String, String)>) {
    load_chain_to!(
        out,
        [CJK_Compatibility_Ideographs],
        [
            STPhrases,
            STPhrases_GeneratedFromRegionalPhrases,
            STCharacters
        ]
    );
}

pub fn load_tw_pairs(out: &mut Vec<(String, String)>, twp: bool) {
    if twp {
        // config: s2tw & s2twp & t2tw. Upstream s2twp chains
        // short_circuit[TWPhrases, TWVariantsPhrases, TWVariants] as ONE link,
        // so all three merge in a single stage (no cross-reconversion between
        // TWPhrases targets and the variants dicts).
        load_chain_to!(
            out,
            [CJK_Compatibility_Ideographs],
            [
                STPhrases,
                STPhrases_GeneratedFromRegionalPhrases,
                STCharacters
            ],
            [TWPhrases, TWVariantsPhrases, TWVariants]
        );
    } else {
        // config: s2tw & t2tw
        load_chain_to!(
            out,
            [CJK_Compatibility_Ideographs],
            [
                STPhrases,
                STPhrases_GeneratedFromRegionalPhrases,
                STCharacters
            ],
            [TWVariantsPhrases, TWVariants]
        );
    }
}

pub fn load_hk_pairs(out: &mut Vec<(String, String)>, hkp: bool) {
    if hkp {
        // config: s2hk & s2hkp & t2hk
        load_chain_to!(
            out,
            [CJK_Compatibility_Ideographs],
            [
                STPhrases,
                STPhrases_GeneratedFromRegionalPhrases,
                STCharacters
            ],
            [HKPhrases, HKVariantsPhrases, HKVariants]
        );
    } else {
        // config: s2hk & t2hk
        load_chain_to!(
            out,
            [CJK_Compatibility_Ideographs],
            [
                STPhrases,
                STPhrases_GeneratedFromRegionalPhrases,
                STCharacters
            ],
            [HKVariantsPhrases, HKVariants]
        );
    }
}

pub fn load_cn_pairs(out: &mut Vec<(String, String)>, twp: bool, hkp: bool) {
    if twp {
        // config: tw2sp; "!TWVariants" deliberately omitted.
        // TWPhrasesRev.txt is the checked-in (and since diverged) reverse of
        // TWPhrases, so it replaces the derived `!TWPhrases` reversal here.
        load_chain_to!(
            out,
            [CJK_Compatibility_Ideographs],
            [TWPhrasesRev, TWVariantsRevPhrases],
            [TSPhrases, TSCharacters]
        );
    } else {
        // config: tw2s; "!TWVariants" deliberately omitted to prevent
        // misconversions like `么 -> 幺, 抬 -> 檯, 著 -> 着`
        load_chain_to!(
            out,
            [CJK_Compatibility_Ideographs],
            [TWVariantsRevPhrases],
            [TSPhrases, TSCharacters]
        );
    }
    if hkp {
        // config: hk2sp; "!HKVariants" deliberately omitted
        load_chain_to!(
            out,
            [HKPhrasesRev, HKVariantsRevPhrases],
            [TSPhrases, TSCharacters]
        );
    } else {
        // config: hk2s; "!HKVariants" deliberately omitted
        load_chain_to!(out, [HKVariantsRevPhrases], [TSPhrases, TSCharacters]);
    }
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

            for hkp in [false, true] {
                let mut cn = Vec::new();
                load_cn_pairs(&mut cn, twp, hkp);
                assert!(
                    !cn.is_empty(),
                    "cn_pairs({}, {}) should not be empty",
                    twp,
                    hkp
                );

                let mut hk = Vec::new();
                load_hk_pairs(&mut hk, hkp);
                assert!(!hk.is_empty(), "hk_pairs({}) should not be empty", hkp);
            }
        }
    }

    #[test]
    fn st_generated_covers_regional_phrases() {
        // 內存條 (TWPhrases + HKPhrases key) projects to simp 内存条 via t2s.
        assert!(
            STPHRASES_GENERATED
                .iter()
                .any(|(f, t)| f == "内存条" && t == "內存條"),
            "expected generated 内存条->內存條, got {} entries",
            STPHRASES_GENERATED.len()
        );
        // All generated keys are simplified-side, len>=3 chars.
        assert!(STPHRASES_GENERATED
            .iter()
            .all(|(f, _)| f.chars().count() >= 3));
    }
}
