use std::collections::HashMap;

use std::collections::HashSet;
use std::env;
use std::fs::{self, File};
use std::io;
use std::io::Write;
use std::iter;
use std::path::Path;

use daachorse::{CharwiseDoubleArrayAhoCorasickBuilder, MatchKind};
use hex_literal::hex;
use regex::Regex;
use sha2::{Digest, Sha256};
use vergen::EmitBuilder;

#[cfg(feature = "opencc")]
use self::opencc::load_opencc_to;

// To update upstream dataset: manually update commits here and run data/update_basic.py
const MEDIAWIKI_COMMIT: &str = "7e8ae4dd01a659ecda127088bd1d227f4a1a1c68";
const MEDIAWIKI_SHA256: [u8; 32] =
    hex!("128e3240f31ad69513ec26b38c04b9ab485508429a2b9b4877e0926a0155d6a8");

#[cfg(feature = "opencc")]
const OPENCC_COMMIT: &str = "2c7187e33b77bd8a356c676843cda69d2fccf887";
#[cfg(feature = "opencc")]
const OPENCC_SHA256: [(&str, [u8; 32]); 11] = [
    (
        "HKVariants.txt",
        hex!("c3c93c35885902ba2b12a3235a7761b00fb2b027f36aa8314db2f6b6ad51d374"),
    ),
    (
        "HKVariantsRevPhrases.txt",
        hex!("c2da309afa7fdd9061f204664039d33b000a4dca0ecae4e7480dcbf9e20f658e"),
    ),
    (
        "STCharacters.txt",
        hex!("ed1d268e0ad028511dcf5b0089faed0a980ad332449ec11d481ceefde6879f41"),
    ),
    (
        "STPhrases.txt",
        hex!("3e24d511fac0fc293e41ea116c7b2f2ecbfb1f594ea741f54e051c9b986a908e"),
    ),
    (
        "TSCharacters.txt",
        hex!("6b5a0a799bea2bb22c001f635eaa3fc2904310f0c08addbff275477a80ecf09a"),
    ),
    (
        "TSPhrases.txt",
        hex!("504169029c43f7f234b8e2ae470720af3657675c5574ff8aa0feb257e1dc5ce2"),
    ),
    (
        "TWPhrasesIT.txt",
        hex!("3a4a2ad207f3a9442eb8f399630cf982c2ffc561df9c58b2ee352cfa023915c1"),
    ),
    (
        "TWPhrasesName.txt",
        hex!("76e643569a30ea54e7ab6e52621fd4c396e01ee6dc2d15b7d25bf23addf4438a"),
    ),
    (
        "TWPhrasesOther.txt",
        hex!("6d0365fd180283f3e14b44f63d19d1aca045d60b0e000765902ad889a90d7a33"),
    ),
    (
        "TWVariants.txt",
        hex!("30e6f8395edbfdd74e293fd8b9c62105d787c849fbb208d2a7832eac696734d7"),
    ),
    (
        "TWVariantsRevPhrases.txt",
        hex!("bef60ceb4e57b6b062351406cb5d4644875574231d64787e03711317b7e773f3"),
    ),
];

fn main() -> io::Result<()> {
    let zhconv = read_and_validate_file("data/ZhConversion.php", &MEDIAWIKI_SHA256);
    let mut zhconvs = parse_mediawiki(&zhconv);
    #[allow(unused_variables, unused_mut)]
    for (name, mut pairs) in zhconvs.iter_mut() {
        // Load and append OpenCC rulesets to the Mediawiki ones
        // ref: https://github.com/BYVoid/OpenCC/blob/29d33fb8edb8c95e34691c8bd1ef76a50d0b5251/data/config/

        // Note: OpenCC conversion procededures take multi-pass for chaining rulesets.
        // For efficiency and re-using the existing implementation, we chain the rulesets
        // straightforward by chaining conversion pairs at different level in advance.
        // It may result in conversion results different to the stock OpenCC implementation
        // considering that some conversion pairs span over the border of several natural phrases
        // while not covering them in whole.
        #[cfg(feature = "opencc")]
        match name.as_ref() {
            "ZH_TO_HANS" => {
                // hk2s & tw2s & t2s
                load_opencc_to!(
                    &mut pairs,
                    [HKVariantsRevPhrases, !HKVariants],
                    [TSCharacters, TSPhrases]
                );
                load_opencc_to!(
                    &mut pairs,
                    [TWVariantsRevPhrases, !TWVariants],
                    [TSCharacters, TSPhrases]
                );
            }
            "ZH_TO_HANT" => {
                // s2t & hk2t & tw2t
                load_opencc_to!(&mut pairs, [HKVariantsRevPhrases, !HKVariants]);
                load_opencc_to!(&mut pairs, [TWVariantsRevPhrases, !TWVariants]);
                load_opencc_to!(&mut pairs, [STCharacters, STPhrases]);
            }
            "ZH_TO_TW" => {
                // s2twp & t2tw
                load_opencc_to!(
                    &mut pairs,
                    [STPhrases, STCharacters],
                    [TWPhrasesIT, TWPhrasesName, TWPhrasesOther],
                    [TWVariants]
                );
            }
            "ZH_TO_HK" => {
                // s2hk & t2hk
                load_opencc_to!(&mut pairs, [STPhrases, STCharacters], [HKVariants]);
            }
            "ZH_TO_MO" => {}
            "ZH_TO_CN" => {
                // tw2sp & hk2s
                load_opencc_to!(
                    &mut pairs,
                    [
                        !TWPhrasesIT,
                        !TWPhrasesName,
                        !TWPhrasesOther,
                        TWVariantsRevPhrases,
                        !TWVariants
                    ],
                    [TSPhrases, TSCharacters]
                );
                load_opencc_to!(
                    &mut pairs,
                    [HKVariantsRevPhrases, !HKVariants],
                    [TSPhrases, TSCharacters]
                );
            }
            "ZH_TO_SG" => {}
            "ZH_TO_MY" => {}
            _ => (),
        }

        // longer phrases come first; lexicographically smaller phrases come first
        sort_and_dedup(pairs);
    }

    let hans_pairs = zhconvs.remove("ZH_TO_HANS").unwrap();
    write_conv_file("ZH_TO_HANS", &hans_pairs)?;
    // let hans_pairs: HashMap<String, String> = hans_pairs.into_iter().collect();
    write_daac_file("ZH_TO_HANS", &hans_pairs)?;
    let hans_map: HashMap<_, _> = hans_pairs.iter().cloned().collect();

    let hant_pairs = zhconvs.remove("ZH_TO_HANT").unwrap();
    write_conv_file("ZH_TO_HANT", &hant_pairs)?;
    // let hant_pairs: HashMap<String, String> = hant_pairs.into_iter().collect();
    write_daac_file("ZH_TO_HANT", &hant_pairs)?;
    let hant_map: HashMap<_, _> = hant_pairs.iter().cloned().collect();

    let mut cn_pairs = zhconvs.remove("ZH_TO_CN").unwrap();
    cn_pairs.retain(|(from, to)| hans_map.get(from.as_str()) != Some(to));
    // write_conv_file("ZH_TO_CN", &cn_pairs)?;
    // cn_pairs.extend();
    write_conv_file("ZH_TO_CN", &cn_pairs)?;
    let mut hans_cn_pairs = hans_pairs;
    hans_cn_pairs.extend(cn_pairs);
    // sort_and_dedup(&mut hans_cn_pairs);
    write_daac_file("ZH_TO_HANS_CN", &hans_cn_pairs)?;

    // FIXME: doc
    // Here, ZH_TO_HANT | ZH_TO_TW => ZH_TO_HANT_TW, etc. In other places, ZH_TO_TW might imply ZH_TO_HANT_TW.

    let mut tw_pairs = zhconvs.remove("ZH_TO_TW").unwrap();
    tw_pairs.retain(|(from, to)| hant_map.get(from.as_str()) != Some(to));
    // write_conv_file("ZH_TO_TW", &tw_pairs)?;
    // tw_pairs.extend(.into_iter());
    write_conv_file("ZH_TO_TW", &tw_pairs)?;
    let mut hant_tw_pairs = hant_pairs.clone();
    hant_tw_pairs.extend(tw_pairs);
    // sort_and_dedup(&mut hant_tw_pairs);
    write_daac_file("ZH_TO_HANT_TW", &hant_tw_pairs)?;

    let mut hk_pairs = zhconvs.remove("ZH_TO_HK").unwrap();
    hk_pairs.retain(|(from, to)| hant_map.get(from.as_str()) != Some(to));
    // write_conv_file("ZH_TO_HK", &hk_pairs)?;
    // hk_pairs.extend(zhconvs.remove("ZH_TO_HK").unwrap().into_iter());
    write_conv_file("ZH_TO_HK", &hk_pairs)?;
    let mut hant_hk_pairs = hant_pairs;
    hant_hk_pairs.extend(hk_pairs);
    // sort_and_dedup(&mut hant_hk_pairs);
    write_daac_file("ZH_TO_HANT_HK", &hant_hk_pairs)?;

    if std::env::var("DOCS_RS").is_err() {
        // vergen panics in docs.rs. It is only used by wasm.rs for now.
        // So it is ok to disable it in docs.rs.

        // Note: conditional compilation tricks won't be effective since it is cross compiling here.
        // Ref:
        //   https://kazlauskas.me/entries/writing-proper-buildrs-scripts
        //   https://github.com/rust-lang/cargo/issues/4302
        // #[cfg(target_arch = "wasm32")] #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        if env::var("CARGO_CFG_TARGET_ARCH") == Ok("wasm32".to_owned()) {
            EmitBuilder::builder()
                .all_build()
                .all_git()
                .emit()
                .unwrap_or_else(|e| println!("cargo:warning=vergen failed: {:?}", e));
        }
    }
    println!("cargo:rustc-env=MEDIAWIKI_COMMIT_HASH={}", MEDIAWIKI_COMMIT);
    #[cfg(feature = "opencc")]
    println!("cargo:rustc-env=OPENCC_COMMIT_HASH={}", OPENCC_COMMIT);
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=data/ZhConversion.php");
    #[cfg(feature = "opencc")]
    for (opencc, _) in OPENCC_SHA256.iter() {
        println!("cargo:rerun-if-changed=data/{}", opencc);
    }
    println!("cargo:rerun-if-changed=Cargo.toml");

    Ok(())
}

fn parse_mediawiki(text: &str) -> HashMap<String, Vec<(String, String)>> {
    let patb = Regex::new(r"public const (\w+) = \[([^]]+)\]?;").unwrap();
    let patl = Regex::new(r"'(.+?)' *=> *'(.+?)' *,?\n").unwrap();
    let mut res = HashMap::new();

    for block in patb.captures_iter(text) {
        let name = block.get(1).unwrap().as_str();
        let body = block.get(2).unwrap().as_str();
        let mut pairs = vec![];
        for line in patl.captures_iter(body) {
            let from = line.get(1).unwrap().as_str();
            let to = line.get(2).unwrap().as_str();
            pairs.push((from.to_owned(), to.to_owned()));
        }
        assert!(res.insert(name.to_owned(), pairs).is_none());
    }
    res
}

fn write_conv_file(name: &str, pairs: &[(String, String)]) -> io::Result<()> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path_from = Path::new(&out_dir).join(format!("{}.from.conv", name));
    let dest_path_to = Path::new(&out_dir).join(format!("{}.to.conv", name));

    let mut ffrom = File::create(dest_path_from)?;
    let mut fto = File::create(dest_path_to)?;
    let mut it = pairs.iter().peekable();
    let mut last_from = "";
    while let Some((from, to)) = it.next().map(|(f, t)| (f, t)) {
        for c in pair_reduce(from.chars(), last_from.chars()) {
            write!(ffrom, "{}", c)?;
        }
        for c in pair_reduce(to.chars(), from.chars()) {
            write!(fto, "{}", c)?;
        }
        if it.peek().is_some() {
            write!(ffrom, "|")?;
            write!(fto, "|")?;
        }
        last_from = from;
    }

    Ok(())
}

fn write_daac_file(name: &str, pairs: &[(String, String)]) -> io::Result<()> {
    let mut seen = HashSet::new();
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path_daac = Path::new(&out_dir).join(format!("{}.daac", name));
    let daac = CharwiseDoubleArrayAhoCorasickBuilder::new()
        .match_kind(MatchKind::LeftmostLongest)
        .build_with_values::<_, _, u32>(pairs.iter().enumerate().rev().filter_map(
            |(i, (f, _t))| {
                // Note the rev here, which ensures later rules take precedence over earlier ones.
                if seen.contains(f) {
                    None
                } else {
                    seen.insert(f);
                    Some((f, i as u32))
                }
            },
        ))
        .expect(name)
        .serialize();

    #[cfg(feature = "compress")]
    let daac = zstd::bulk::Compressor::new(21)
        .unwrap()
        .compress(&daac)
        .unwrap();

    File::create(dest_path_daac)?.write_all(&daac)

    // let automaton: CharwiseDoubleArrayAhoCorasick<u32> = CharwiseDoubleArrayAhoCorasickBuilder::new().match_kind(MatchKind::LeftmostLongest).build(opairs.iter().map(|(f, t)|f)).unwrap();
    // let dest_path_daac = Path::new(&out_dir).join(format!("{}.daac", name));
    // let mut fdaac = File::create(dest_path_daac)?;
    // let daac = automaton.serialize();
    // // let mut compressed_daac = vec![0; snap::raw::max_compress_len(daac.len())];
    // // snap::raw::Encoder::new().compress(&daac, &mut compressed_daac).unwrap();
    // // let compressed_daac = lz4_flex::compress_prepend_size(&daac);
    // let compressed_daac =zstd::bulk::Compressor::new(3).unwrap().compress(&daac).unwrap();
    //  fdaac.write(&compressed_daac)?;

    // let automaton: CharwiseDoubleArrayAhoCorasick<String> = CharwiseDoubleArrayAhoCorasickBuilder::new().match_kind(MatchKind::LeftmostLongest).build_with_values(opairs.into_iter()).unwrap();
    // let dest_path_daac = Path::new(&out_dir).join(format!("{}.daccv", name));
    // let mut fdaac = File::create(dest_path_daac)?;
    // fdaac.write(&automaton.serialize())?;
}

const SURROGATE_START: char = '\x00';
const SURROGATE_END: char = '\x20';

fn pair_reduce<'s>(
    mut s: impl Iterator<Item = char> + 's + Clone,
    mut base: impl Iterator<Item = char> + 's + Clone,
) -> impl Iterator<Item = char> + 's + Clone {
    let mut it = iter::from_fn(move || match (s.next(), base.next()) {
        (Some(a), Some(b)) if a == b => Some(SURROGATE_START),
        (Some(a), _) => Some(a),
        (None, _) => None,
    })
    .peekable();

    iter::from_fn(move || {
        it.next().map(|curr| {
            if curr == SURROGATE_START {
                let mut count = 1;
                while Some(&SURROGATE_START) == it.peek() {
                    if (SURROGATE_START as u32) + (count + 1) >= (SURROGATE_END as u32) {
                        break;
                    }
                    let _ = it.next();
                    count += 1;
                }
                char::from_u32(SURROGATE_START as u32 + count).unwrap()
            } else {
                curr
            }
        })
    })
}

fn sort_and_dedup(pairs: &mut Vec<(String, String)>) {
    pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then(a.0.cmp(&b.0)));
    pairs.dedup_by(|a, b| a.0 == b.0);
}

#[cfg(feature = "opencc")]
mod opencc {

    use daachorse::{
        CharwiseDoubleArrayAhoCorasick, CharwiseDoubleArrayAhoCorasickBuilder, MatchKind,
    };
    // use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
    use std::collections::HashMap;
    use std::sync::LazyLock;

    use super::OPENCC_SHA256;

    pub static OPENCC_SHA256_MAP: LazyLock<HashMap<String, [u8; 32]>> = LazyLock::new(|| {
        OPENCC_SHA256
            .into_iter()
            .map(|(n, s)| (n.to_owned(), s))
            .collect()
    });

    macro_rules! load_opencc_to {
        ( @read_to $out_conv: expr, $out_revconv: expr, $name: ident) => {
            let s = read_and_validate_file(concat!("data/", stringify!($name), ".txt"), crate::opencc::OPENCC_SHA256_MAP.get(stringify!($name.txt)).expect(stringify!($name.txt not found)));
            crate::opencc::parse_opencc_to($out_conv, $out_revconv, &s);
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
            let conver: crate::opencc::SimpleConverter = convs.clone().into();
            let prev_revconver: crate::opencc::SimpleConverter = prev_revconvs.clone().into();
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
    pub(crate) use load_opencc_to;
    pub fn parse_opencc_to(
        out_conv: &mut HashMap<String, String>,
        out_revconv: &mut HashMap<String, String>,
        s: &str,
    ) {
        for line in s.lines().map(|l| l.trim()).filter(|l| !l.is_empty()) {
            if let Some((f, ts)) = line.split_once(char::is_whitespace) {
                if f.is_empty() || ts.is_empty() {
                    continue;
                }
                let ts: Vec<_> = ts.split_whitespace().collect();
                if !(ts.len() > 1 && ts.iter().any(|&t| t == f)) {
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
}

fn read_and_validate_file(path: &str, sha256sum: &[u8; 32]) -> String {
    let data_dir = env::var_os("CARGO_MANIFEST_DIR").unwrap();
    let path = Path::new(&data_dir).join(path);
    let content = String::from_utf8(
        fs::read(&path).unwrap_or_else(|e| panic!("{} when reading {}", e, path.display())),
    )
    .unwrap_or_else(|e| panic!("{} is not in valid UTF-8 ({})", path.display(), e));
    assert_eq!(
        &sha256(&content),
        sha256sum,
        "Validating the checksum of {}",
        path.display()
    );
    content
}

fn sha256(text: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hasher.finalize().into()
}
