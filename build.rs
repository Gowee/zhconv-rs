/// Generates conversion tables and data structures for Chinese character/phrase conversion.
///
/// This build script:
/// - Loads MediaWiki conversion rulesets via `zhconv-data-mediawiki`
/// - Optionally merges OpenCC rulesets via `zhconv-data-opencc`
///   (each data crate parses its raw dataset internally and returns
///   structured pairs; merging, sorting and codegen stay here so the
///   per-target output remains a single automaton)
/// - Sorts conversion pairs by length (longest first) and lexicographically
/// - Deduplicates pairs, retaining only the first rule for each source mapping
/// - Generates three types of output files:
///   - `.from.conv` and `.to.conv`: Compressed pair format for direct lookup
///   - `.daac`: Serialized Aho-Corasick automaton for efficient pattern matching
///
/// The conversion rulesets are processed for the following targets:
/// - `ZH_TO_HANS`: Simplified Chinese
/// - `ZH_TO_HANT`: Traditional Chinese
/// - `ZH_TO_CN`: Mainland China variant (Hans + CN-specific rules)
/// - `ZH_TO_TW`: Taiwan variant (Hant + TW-specific rules)
/// - `ZH_TO_HK`: Hong Kong variant (Hant + HK-specific rules)
/// - `ZH_TO_MO`, `ZH_TO_SG`, `ZH_TO_MY`: Regional variants
///
/// **Note**: Earlier rules take precedence over later ones. When multiple rules apply to the
/// same source string, the first occurrence is retained.
use std::collections::HashMap;

use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io;
use std::io::Write;
use std::iter;
use std::path::Path;

use daachorse::{CharwiseDoubleArrayAhoCorasickBuilder, MatchKind};
use vergen::EmitBuilder;

const DELIMITER: &str = "|";

fn main() -> io::Result<()> {
    #[cfg(all(
        feature = "opencc-twp",
        not(any(feature = "opencc-tw", feature = "opencc-cn"))
    ))]
    panic!("opencc-twp should only be enabled together with opencc-tw or opencc-cn");

    let mut diagnostics_file =
        File::create(Path::new(&env::var_os("OUT_DIR").unwrap()).join("zhconv-diagnostics.txt"))?;
    macro_rules! log_diag {
        ($fmt:expr $(, $arg:expr)*) => {
            write!(diagnostics_file, $fmt $(, $arg)*)
        };
    }
    log_diag!("=== BUILDING ===\n")?;
    for (k, _v) in std::env::vars() {
        if k.starts_with("CARGO_FEATURE_") {
            log_diag!("feature: {}\n", k.strip_prefix("CARGO_FEATURE_").unwrap())?;
        }
    }
    #[cfg(feature = "_mediawiki-base")]
    log_diag!(
        "MEDIAWIKI_COMMIT={}\n",
        zhconv_data_mediawiki::MEDIAWIKI_COMMIT
    )?;
    #[cfg(feature = "_opencc-base")]
    log_diag!("OPENCC_COMMIT={}\n", zhconv_data_opencc::OPENCC_COMMIT)?;
    let start_time = std::time::Instant::now();

    // Load MediaWiki rulesets (parsed inside the data crate)
    #[cfg(feature = "_mediawiki-base")]
    let mut zhconvs = zhconv_data_mediawiki::parse();
    #[cfg(not(feature = "_mediawiki-base"))]
    let mut zhconvs: HashMap<String, Vec<(String, String)>> = HashMap::new();

    for name in [
        "ZH_TO_HANT",
        "ZH_TO_TW",
        "ZH_TO_HK",
        "ZH_TO_HANS",
        "ZH_TO_CN",
    ] {
        #[allow(unused_mut)]
        let mut pairs = zhconvs.entry(name.to_owned()).or_default();
        log_diag!("Processing {}: MediaWiki.len = {}", name, pairs.len())?;
        // Load and append OpenCC dicts (staged/flattened inside the data crate).
        // The per-target config mapping below mirrors OpenCC's
        // `data/config/*.json`; only the call site lives here so feature
        // gating stays with the main crate.
        // ref: https://github.com/BYVoid/OpenCC/blob/29d33fb8edb8c95e34691c8bd1ef76a50d0b5251/data/config/

        // Note: The conversion of OpenCC takes multi-pass for applying dict groups step by step.
        // For efficiency and reusing the existing implementation, we merge and flatten dict groups
        // in advance.
        // The conversion results may differ from the stock OpenCC implementation considering
        // that some conversion pairs span over the border of several natural phrases while not
        // covering them in whole.
        #[cfg(feature = "_opencc-base")]
        match name {
            // Used when targeting either zh-hans or zh-cn
            #[cfg(any(feature = "opencc-hans", feature = "opencc-cn"))]
            "ZH_TO_HANS" => {
                // config: t2s
                zhconv_data_opencc::load_hans_pairs(pairs);
            }
            // Used when targeting either zh-hant, zh-hk or zh-tw
            #[cfg(any(feature = "opencc-hant", feature = "opencc-tw", feature = "opencc-hk"))]
            "ZH_TO_HANT" => {
                // config: s2t
                zhconv_data_opencc::load_hant_pairs(pairs);
            }
            #[cfg(feature = "opencc-tw")]
            "ZH_TO_TW" => {
                zhconv_data_opencc::load_tw_pairs(pairs, cfg!(feature = "opencc-twp"));
            }
            #[cfg(feature = "opencc-hk")]
            "ZH_TO_HK" => {
                // config: s2hk & t2hk
                zhconv_data_opencc::load_hk_pairs(pairs);
            }
            #[cfg(feature = "opencc-cn")]
            "ZH_TO_CN" => {
                zhconv_data_opencc::load_cn_pairs(pairs, cfg!(feature = "opencc-twp"));
            }
            // "ZH_TO_MO" => {}
            // "ZH_TO_SG" => {}
            // "ZH_TO_MY" => {}
            _ => (),
        }
        log_diag!(", withOpenCC.len = {}", pairs.len())?;

        // Longer phrases and lexicographically smaller phrases appear earlier and hence take
        // precedence in the final conversion table.
        sort_and_dedup(pairs);

        log_diag!(", sortedAndDeduped.len = {}\n", pairs.len())?;
    }

    let hans_pairs = zhconvs.remove("ZH_TO_HANS").unwrap();
    if cfg!(any(
        feature = "mediawiki-hans",
        feature = "opencc-hans",
        feature = "mediawiki-cn",
        feature = "opencc-cn"
    )) {
        write_conv_file("ZH_TO_HANS", &hans_pairs)?;
        write_daac_file("ZH_TO_HANS", &hans_pairs)?;
    }

    let hant_pairs = zhconvs.remove("ZH_TO_HANT").unwrap();
    if cfg!(any(
        feature = "mediawiki-hant",
        feature = "opencc-hant",
        feature = "mediawiki-tw",
        feature = "opencc-tw",
        feature = "mediawiki-hk",
        feature = "opencc-hk"
    )) {
        write_conv_file("ZH_TO_HANT", &hant_pairs)?;
        write_daac_file("ZH_TO_HANT", &hant_pairs)?;
    }

    // The complete table for cn (normalized as hans-cn) are formed by chaining hans table and
    // cn-specific table, so that hans table can be reused for both hans and hans-cn converters,
    // thus reducing the bundled asset size.
    // Chaining does not work for daac, so we have to build complete daac for each target variant,
    // tolerating the redundancy.
    // The same logic applies to tw (hant-tw) and hk (hant-hk).
    let mut cn_pairs = zhconvs.remove("ZH_TO_CN").unwrap();
    if cfg!(any(feature = "mediawiki-cn", feature = "opencc-cn")) {
        let hans_map: HashMap<_, _> = hans_pairs.iter().cloned().collect();
        cn_pairs.retain(|(from, to)| hans_map.get(from.as_str()) != Some(to));
        write_conv_file("ZH_TO_CN", &cn_pairs)?;
        let mut hans_cn_pairs = hans_pairs;
        hans_cn_pairs.extend(cn_pairs);
        write_daac_file("ZH_TO_HANS_CN", &hans_cn_pairs)?;
        log_diag!("ZH_TO_HANS_CN: final.len = {}\n", hans_cn_pairs.len())?;
    }

    // Here, ZH_TO_HANT | ZH_TO_TW => ZH_TO_HANT_TW, etc. In other places, ZH_TO_TW might imply ZH_TO_HANT_TW.

    if cfg!(any(
        feature = "mediawiki-tw",
        feature = "opencc-tw",
        feature = "mediawiki-hk",
        feature = "opencc-hk"
    )) {
        let hant_map: HashMap<_, _> = hant_pairs.iter().cloned().collect();

        let mut tw_pairs = zhconvs.remove("ZH_TO_TW").unwrap();
        if cfg!(any(feature = "mediawiki-tw", feature = "opencc-tw")) {
            tw_pairs.retain(|(from, to)| hant_map.get(from.as_str()) != Some(to));
            write_conv_file("ZH_TO_TW", &tw_pairs)?;
            let mut hant_tw_pairs = hant_pairs.clone();
            hant_tw_pairs.extend(tw_pairs);
            write_daac_file("ZH_TO_HANT_TW", &hant_tw_pairs)?;
            log_diag!("ZH_TO_HANT_TW: final.len = {}\n", hant_tw_pairs.len())?;
        }

        let mut hk_pairs = zhconvs.remove("ZH_TO_HK").unwrap();
        if cfg!(any(feature = "mediawiki-hk", feature = "opencc-hk")) {
            hk_pairs.retain(|(from, to)| hant_map.get(from.as_str()) != Some(to));
            write_conv_file("ZH_TO_HK", &hk_pairs)?;
            let mut hant_hk_pairs = hant_pairs;
            hant_hk_pairs.extend(hk_pairs);
            write_daac_file("ZH_TO_HANT_HK", &hant_hk_pairs)?;
            log_diag!("ZH_TO_HANT_HK: final.len = {}\n", hant_hk_pairs.len())?;
        }
    }

    log_diag!("Built in: {:?}\n=== DONE ===\n", start_time.elapsed())?;

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
    #[cfg(feature = "_mediawiki-base")]
    println!(
        "cargo:rustc-env=MEDIAWIKI_COMMIT_HASH={}",
        zhconv_data_mediawiki::MEDIAWIKI_COMMIT
    );
    #[cfg(feature = "_opencc-base")]
    println!(
        "cargo:rustc-env=OPENCC_COMMIT_HASH={}",
        zhconv_data_opencc::OPENCC_COMMIT
    );
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    Ok(())
}

fn write_conv_file(name: &str, pairs: &[(String, String)]) -> io::Result<()> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    // {from, to}.conv is just DELIMITER-separated list of {source, target} phrases.
    // source phrases, which are coded into daac, are only useful for building new daacs.
    // However, we still bundle it anyway for implementation convenience.
    let dest_path_from = Path::new(&out_dir).join(format!("{}.from.conv", name));
    let dest_path_to = Path::new(&out_dir).join(format!("{}.to.conv", name));

    let mut ffrom = File::create(dest_path_from)?;
    let mut fto = File::create(dest_path_to)?;
    let mut it = pairs.iter().peekable();
    let mut last_from = "";
    while let Some((from, to)) = it.next().map(|(f, t)| (f, t)) {
        debug_assert!(
            !from.contains(DELIMITER) && !to.contains(DELIMITER),
            "Unexpected delimiter {} in pair {} -> {}",
            DELIMITER,
            from,
            to
        );
        debug_assert!(
            !from
                .chars()
                .any(|c| (SURROGATE_START..SURROGATE_END).contains(&c))
                && !to
                    .chars()
                    .any(|c| (SURROGATE_START..SURROGATE_END).contains(&c)),
            "Unexpected surrogate char in pair {} -> {}",
            from,
            to
        );
        for c in pair_reduce(from.chars(), last_from.chars()) {
            write!(ffrom, "{}", c)?;
        }
        for c in pair_reduce(to.chars(), from.chars()) {
            write!(fto, "{}", c)?;
        }
        if it.peek().is_some() {
            write!(ffrom, "{}", DELIMITER)?;
            write!(fto, "{}", DELIMITER)?;
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
        // Disable prefilter: conversion tables have high text coverage, so prefiltering cannot skip ahead and only adds overhead.
        .use_prefilter(false)
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
    let daac = {
        let window_log = (daac.len().next_power_of_two().trailing_zeros()).clamp(17, 22);
        let mut encoder = zstd::stream::Encoder::new(Vec::new(), 19)?;
        encoder.set_pledged_src_size(Some(daac.len() as u64))?;
        encoder.window_log(window_log)?;
        encoder.include_checksum(false)?;
        use std::io::Write;
        encoder.write_all(&daac)?;
        encoder.finish()?
    };

    File::create(dest_path_daac)?.write_all(&daac)
}

const SURROGATE_START: char = '\x00';
const SURROGATE_END: char = '\x20'; // exclusive

// simple but efficient compression
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
    // earlier rules take precedence
    pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then(a.0.cmp(&b.0)));
    pairs.dedup_by(|a, b| a.0 == b.0);
}
