use std::collections::HashMap;

use std::collections::HashSet;
use std::convert::TryInto;
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
const MEDIAWIKI_COMMIT: &str = "c40f34c8562e739d2c9baaec8543a968f29a4676";
const MEDIAWIKI_SHA256: [u8; 32] =
    hex!("37c17b6361ab774b0b7dab801aa7ff919f8efa6e2ad3f66ccb051e9c1a848f6e");

#[cfg(feature = "opencc")]
const OPENCC_COMMIT: &str = "5750d92a92ac1f2d64c880c1f6f1a5e382d7d199";
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
        hex!("9207708da9f2e2a248f39c457b2fccad26ec42e7efaf47a860e6900464f4cac5"),
    ),
    (
        "STPhrases.txt",
        hex!("a4de4d2471f73cdb7e5b1b22920139aa4e4bbb1ebeea8f1fc341f988aa75c586"),
    ),
    (
        "TSCharacters.txt",
        hex!("6b5a0a799bea2bb22c001f635eaa3fc2904310f0c08addbff275477a80ecf09a"),
    ),
    (
        "TSPhrases.txt",
        hex!("b2ef895dd4953b4bb77fc8ef8d26a2a9ca6d43a760ed9a1d767672cfafa6324f"),
    ),
    (
        "TWPhrasesIT.txt",
        hex!("8a129130a10c57278485c4b7a81c4c74a8242239576018d9bfd2149e2d3c2af6"),
    ),
    (
        "TWPhrasesName.txt",
        hex!("76e643569a30ea54e7ab6e52621fd4c396e01ee6dc2d15b7d25bf23addf4438a"),
    ),
    (
        "TWPhrasesOther.txt",
        hex!("06d9e1a24b1f87431e029d38cdf67a35d32b96a08df736cf1a362477dd39f7c7"),
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
            "zh2Hans" => {
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
            "zh2Hant" => {
                // s2t & hk2t & tw2t
                load_opencc_to!(&mut pairs, [HKVariantsRevPhrases, !HKVariants]);
                load_opencc_to!(&mut pairs, [TWVariantsRevPhrases, !TWVariants]);
                load_opencc_to!(&mut pairs, [STCharacters, STPhrases]);
            }
            "zh2TW" => {
                // s2twp & t2tw
                load_opencc_to!(
                    &mut pairs,
                    [STPhrases, STCharacters],
                    [TWPhrasesIT, TWPhrasesName, TWPhrasesOther],
                    [TWVariants]
                );
            }
            "zh2HK" => {
                // s2hk & t2hk
                load_opencc_to!(&mut pairs, [STPhrases, STCharacters], [HKVariants]);
            }
            "zh2MO" => {}
            "zh2CN" => {
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
            "zh2SG" => {}
            "zh2MY" => {}
            _ => (),
        }

        // longer phrases come first; lexicographically smaller phrases come first
        sort_and_dedup(pairs);

        // debug_assert_eq!(
        //     pairs.len(),
        //     pairs
        //         .iter()
        //         .map(|(from, _to_)| from)
        //         .collect::<HashSet<_>>()
        //         .len(),
        //     "deduping keys of {}",
        //     name
        // );
    }

    let hans_pairs = zhconvs.remove("zh2Hans").unwrap();
    write_conv_file("zh2Hans", &hans_pairs)?;
    // let hans_pairs: HashMap<String, String> = hans_pairs.into_iter().collect();
    write_daac_file("zh2Hans", &hans_pairs)?;
    let hans_map: HashMap<_, _> = hans_pairs.iter().cloned().collect();

    let hant_pairs = zhconvs.remove("zh2Hant").unwrap();
    write_conv_file("zh2Hant", &hant_pairs)?;
    // let hant_pairs: HashMap<String, String> = hant_pairs.into_iter().collect();
    write_daac_file("zh2Hant", &hant_pairs)?;
    let hant_map: HashMap<_, _> = hant_pairs.iter().cloned().collect();

    let mut cn_pairs = zhconvs.remove("zh2CN").unwrap();
    cn_pairs.retain(|(from, to)| hans_map.get(from.as_str()) != Some(to));
    // write_conv_file("zh2CN", &cn_pairs)?;
    // cn_pairs.extend();
    write_conv_file("zh2CN", &cn_pairs)?;
    let mut hans_cn_pairs = hans_pairs;
    hans_cn_pairs.extend(cn_pairs);
    // sort_and_dedup(&mut hans_cn_pairs);
    write_daac_file("zh2HansCN", &hans_cn_pairs)?;

    // Here, zh2Hant | zh2TW => zh2HantTW, etc. In other places, zh2TW might imply zh2HantTW.

    let mut tw_pairs = zhconvs.remove("zh2TW").unwrap();
    tw_pairs.retain(|(from, to)| hant_map.get(from.as_str()) != Some(to));
    // write_conv_file("zh2TW", &tw_pairs)?;
    // tw_pairs.extend(.into_iter());
    write_conv_file("zh2TW", &tw_pairs)?;
    let mut hant_tw_pairs = hant_pairs.clone();
    hant_tw_pairs.extend(tw_pairs);
    // sort_and_dedup(&mut hant_tw_pairs);
    write_daac_file("zh2HantTW", &hant_tw_pairs)?;

    let mut hk_pairs = zhconvs.remove("zh2HK").unwrap();
    hk_pairs.retain(|(from, to)| hant_map.get(from.as_str()) != Some(to));
    // write_conv_file("zh2HK", &hk_pairs)?;
    // hk_pairs.extend(zhconvs.remove("zh2HK").unwrap().into_iter());
    write_conv_file("zh2HK", &hk_pairs)?;
    let mut hant_hk_pairs = hant_pairs;
    hant_hk_pairs.extend(hk_pairs);
    // sort_and_dedup(&mut hant_hk_pairs);
    write_daac_file("zh2HantHK", &hant_hk_pairs)?;

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
    let patb = Regex::new(r"public static \$(\w+) = \[([^]]+)\]?;").unwrap();
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
    use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
    use lazy_static::lazy_static;
    use std::collections::HashMap;

    use super::OPENCC_SHA256;

    lazy_static! {
        pub static ref OPENCC_SHA256_MAP: HashMap<String, [u8; 32]> = OPENCC_SHA256
            .into_iter()
            .map(|(n, s)| (n.to_owned(), s))
            .collect();
    }

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
            let mut it = line.split_whitespace();
            if let (Some(f), Some(t)) = (it.next(), it.next()) {
                out_conv.insert(f.to_owned(), t.to_owned());
                out_revconv.insert(t.to_owned(), f.to_owned());
                for tt in it {
                    out_revconv.insert(tt.to_owned(), f.to_owned());
                }
            }
        }
    }

    /// Simplified `ZhConverter` implementation for pre-processing rulesets from OpenCC
    pub struct SimpleConverter {
        automaton: AhoCorasick,
        mapping: HashMap<String, String>,
    }

    impl From<HashMap<String, String>> for SimpleConverter {
        fn from(mapping: HashMap<String, String>) -> Self {
            let automaton = AhoCorasickBuilder::new()
                .match_kind(MatchKind::LeftmostLongest)
                .build(mapping.keys())
                .unwrap();
            Self { automaton, mapping }
        }
    }

    impl SimpleConverter {
        #[allow(dead_code)]
        pub fn build<'s>(pairs: impl Iterator<Item = (&'s str, &'s str)>) -> Self {
            let mapping = HashMap::from_iter(pairs.map(|(a, b)| (a.to_owned(), b.to_owned())));
            mapping.into()
        }

        pub fn convert(&self, text: &str) -> String {
            let mut output = String::new();
            let mut last = 0;
            // leftmost-longest matching
            for (s, e) in self.automaton.find_iter(text).map(|m| (m.start(), m.end())) {
                if s > last {
                    output.push_str(&text[last..s]);
                }
                output.push_str(self.mapping.get(&text[s..e]).unwrap());
                last = e;
            }
            output.push_str(&text[last..]);
            output
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
    hasher.finalize().try_into().unwrap()
}
