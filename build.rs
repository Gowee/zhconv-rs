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
/// - Generates output files:
///   - `.from.vzv` and `.to.vzv`: one monolithic [`VarZeroVec`](https://docs.rs/zerovec)<`str`,
///     `Index32`> store per script side (`HANS_ALL` = hans ++ cn-extras,
///     `HANT_ALL` = hant ++ tw-extras ++ hk-extras), full strings parsed
///     zero-copy at runtime (zstd-compressed when `compress` is on)
///   - `.daac`: Serialized Aho-Corasick automaton per target variant, whose
///     values index into the side store (remapped for non-prefix segments)
///   - `table_meta.rs`: element boundaries into the stores for views
///   - `cjk_norm.rs` (with `cjk-compat`): Sorted CJK compatibility pairs for
///     the converter pre-pass (no automaton; see below)
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
use std::path::Path;

use daachorse::{CharwiseDoubleArrayAhoCorasickBuilder, MatchKind};
use vergen::EmitBuilder;

fn main() -> io::Result<()> {
    #[cfg(all(
        feature = "opencc-twp",
        not(any(feature = "opencc-tw", feature = "opencc-cn"))
    ))]
    panic!("opencc-twp should only be enabled together with opencc-tw or opencc-cn");
    #[cfg(all(
        feature = "opencc-hkp",
        not(any(feature = "opencc-hk", feature = "opencc-cn"))
    ))]
    panic!("opencc-hkp should only be enabled together with opencc-hk or opencc-cn");

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

        // Note: OpenCC applies dict groups multi-pass with per-group `match_policy`
        // (`short_circuit` = first dict with any prefix wins, `union` = longest
        // across all wins) plus `mmseg` pre-segmentation
        // (`union[STPhrases, Generated]` keeps regional phrases whole; unmatched
        // runs stay grouped; we observed 2-char keys of STPhrases segs incorrectly).
        // For efficiency we merge and flatten to one
        // LeftmostLongest AC, i.e. `union` semantics, no inline override by design.
        // Audited against official configs: every forward `short_circuit`
        // group ends in a single-character dict (len-1 keys cannot have a
        // proper prefix, so nothing there can need pruning), and no phrase
        // key extends an earlier dict's key (the tw2sp reverse pair has 2
        // such cases, both identity mappings, hence benign). So on current
        // data, union-flattening coincides with `short_circuit`.
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
                // config: s2hk & t2hk (+ s2hkp with opencc-hkp)
                zhconv_data_opencc::load_hk_pairs(pairs, cfg!(feature = "opencc-hkp"));
            }
            #[cfg(feature = "opencc-cn")]
            "ZH_TO_CN" => {
                zhconv_data_opencc::load_cn_pairs(
                    pairs,
                    cfg!(feature = "opencc-twp"),
                    cfg!(feature = "opencc-hkp"),
                );
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
    let hant_pairs = zhconvs.remove("ZH_TO_HANT").unwrap();
    let mut cn_pairs = zhconvs.remove("ZH_TO_CN").unwrap();
    let mut tw_pairs = zhconvs.remove("ZH_TO_TW").unwrap();
    let mut hk_pairs = zhconvs.remove("ZH_TO_HK").unwrap();

    // Monolithic side stores: each variant's DAAC addresses one shared
    // store, so target words are borrowed once instead of copied per
    // variant. Hans-side: M = HANS ++ CN'; hant-side: M = HANT ++ TW' ++
    // HK'. Tails are included only when their features are on, so
    // feature-gated builds carry no foreign data.
    // Chaining (hans+cn tables reused across converters) does not work
    // for daac, so we build one automaton per target variant, tolerating
    // the redundancy there — same as before, only the word store is now
    // shared instead of split per table.

    // ---- hans side ----
    let mut m_hans = hans_pairs;
    let hans_len = m_hans.len();
    if cfg!(any(feature = "mediawiki-cn", feature = "opencc-cn")) {
        let hans_map: HashMap<&str, &str> = m_hans
            .iter()
            .map(|(f, t)| (f.as_str(), t.as_str()))
            .collect();
        cn_pairs.retain(|p| hans_map.get(p.0.as_str()).copied() != Some(p.1.as_str()));
        // A3: extras hold no base-identical pairs (else dead weight + remap noise).
        assert!(
            cn_pairs
                .iter()
                .all(|p| hans_map.get(p.0.as_str()).copied() != Some(p.1.as_str())),
            "CN extras overlap base identicals"
        );
        m_hans.extend(cn_pairs);
    }
    let hans_total = m_hans.len();
    // A1: boundaries sane.
    assert!(hans_len <= hans_total, "hans split past store");
    let (m_hans_froms, m_hans_tos): (Vec<&str>, Vec<&str>) =
        m_hans.iter().map(|(f, t)| (f.as_str(), t.as_str())).unzip();
    let mut hans_to_vzv_bytes = Vec::new();
    if cfg!(any(
        feature = "mediawiki-hans",
        feature = "opencc-hans",
        feature = "mediawiki-cn",
        feature = "opencc-cn"
    )) {
        hans_to_vzv_bytes = write_vzv_file("ZH_TO_HANS_ALL", &m_hans)?;
        assert!(
            !hans_to_vzv_bytes.is_empty(),
            "ZH_TO_HANS_ALL store bytes must be initialized"
        );
        // HANS values are positional: pairs == store prefix.
        write_daac_file(
            "ZH_TO_HANS",
            &m_hans[..hans_len],
            hans_len,
            hans_len as u32,
            &m_hans_froms,
            &m_hans_tos,
            &hans_to_vzv_bytes,
        )?;
    }
    if cfg!(any(feature = "mediawiki-cn", feature = "opencc-cn")) {
        assert!(
            !hans_to_vzv_bytes.is_empty(),
            "ZH_TO_HANS_ALL store bytes must be initialized"
        );
        // HANS_CN values are positional: pairs == whole store.
        write_daac_file(
            "ZH_TO_HANS_CN",
            &m_hans[..],
            hans_total,
            hans_total as u32,
            &m_hans_froms,
            &m_hans_tos,
            &hans_to_vzv_bytes,
        )?;
        log_diag!("ZH_TO_HANS_CN: final.len = {}\n", hans_total)?;
    }

    // ---- hant side ----
    let mut m_hant = hant_pairs;
    let hant_len = m_hant.len();
    let mut tw_end = hant_len;
    if cfg!(any(feature = "mediawiki-tw", feature = "opencc-tw")) {
        {
            let hant_map: HashMap<&str, &str> = m_hant
                .iter()
                .map(|(f, t)| (f.as_str(), t.as_str()))
                .collect();
            tw_pairs.retain(|p| hant_map.get(p.0.as_str()).copied() != Some(p.1.as_str()));
            // A3: same contract as CN.
            assert!(
                tw_pairs
                    .iter()
                    .all(|p| hant_map.get(p.0.as_str()).copied() != Some(p.1.as_str())),
                "TW extras overlap base identicals"
            );
        }
        m_hant.extend(tw_pairs);
        tw_end = m_hant.len();
    }
    if cfg!(any(feature = "mediawiki-hk", feature = "opencc-hk")) {
        {
            // Base only: hk extras identical to tw extras must be kept —
            // the TW segment is not in HK's automaton, so comparing
            // against it would silently drop HK rules.
            let hant_map: HashMap<&str, &str> = m_hant[..hant_len]
                .iter()
                .map(|(f, t)| (f.as_str(), t.as_str()))
                .collect();
            hk_pairs.retain(|p| hant_map.get(p.0.as_str()).copied() != Some(p.1.as_str()));
            // A3: same contract as CN.
            assert!(
                hk_pairs
                    .iter()
                    .all(|p| hant_map.get(p.0.as_str()).copied() != Some(p.1.as_str())),
                "HK extras overlap base identicals"
            );
        }
        m_hant.extend(hk_pairs);
    }
    let hant_total = m_hant.len();
    // A1: boundaries sane and ordered.
    assert!(
        hant_len <= tw_end && tw_end <= hant_total,
        "hant splits out of order"
    );
    let (m_hant_froms, m_hant_tos): (Vec<&str>, Vec<&str>) =
        m_hant.iter().map(|(f, t)| (f.as_str(), t.as_str())).unzip();
    let mut hant_to_vzv_bytes = Vec::new();
    if cfg!(any(
        feature = "mediawiki-hant",
        feature = "opencc-hant",
        feature = "mediawiki-tw",
        feature = "opencc-tw",
        feature = "mediawiki-hk",
        feature = "opencc-hk"
    )) {
        hant_to_vzv_bytes = write_vzv_file("ZH_TO_HANT_ALL", &m_hant)?;
        assert!(
            !hant_to_vzv_bytes.is_empty(),
            "ZH_TO_HANT_ALL store bytes must be initialized"
        );
        // HANT values are positional: pairs == store prefix.
        write_daac_file(
            "ZH_TO_HANT",
            &m_hant[..hant_len],
            hant_len,
            hant_len as u32,
            &m_hant_froms,
            &m_hant_tos,
            &hant_to_vzv_bytes,
        )?;
    }
    // Here, ZH_TO_HANT | ZH_TO_TW => ZH_TO_HANT_TW, etc. In other places, ZH_TO_TW might imply ZH_TO_HANT_TW.
    if cfg!(any(feature = "mediawiki-tw", feature = "opencc-tw")) {
        assert!(
            !hant_to_vzv_bytes.is_empty(),
            "ZH_TO_HANT_ALL store bytes must be initialized"
        );
        // HANT_TW values are positional: pairs == store prefix.
        write_daac_file(
            "ZH_TO_HANT_TW",
            &m_hant[..tw_end],
            tw_end,
            tw_end as u32,
            &m_hant_froms,
            &m_hant_tos,
            &hant_to_vzv_bytes,
        )?;
        log_diag!("ZH_TO_HANT_TW: final.len = {}\n", tw_end)?;
    }
    if cfg!(any(feature = "mediawiki-hk", feature = "opencc-hk")) {
        assert!(
            !hant_to_vzv_bytes.is_empty(),
            "ZH_TO_HANT_ALL store bytes must be initialized"
        );
        // HK pairs are discontiguous in the store (base ++ hk-extras past
        // the tw segment), so the tail remaps past it. Only segment with
        // non-identity values; A2 below pins every slot.
        let hant_hk_pairs: Vec<(String, String)> = m_hant[..hant_len]
            .iter()
            .chain(m_hant[tw_end..].iter())
            .cloned()
            .collect();
        write_daac_file(
            "ZH_TO_HANT_HK",
            &hant_hk_pairs,
            hant_len,
            tw_end as u32,
            &m_hant_froms,
            &m_hant_tos,
            &hant_to_vzv_bytes,
        )?;
        log_diag!(
            "ZH_TO_HANT_HK: final.len = {}\n",
            hant_len + (hant_total - tw_end)
        )?;
    }

    // Element boundaries into the monolithic stores, for runtime table
    // views (`Table.ranges` in tables.rs): base lengths and segment ends.
    // DAAC value remaps use the same numbers via locals above, not these
    // consts — both derive from one assembly, so they cannot drift.
    // Always emitted; tables.rs references each under matching cfgs.
    std::fs::write(
        Path::new(&env::var_os("OUT_DIR").unwrap()).join("table_meta.rs"),
        format!(
            "pub const HANS_ALL_HANS_LEN: usize = {hans_len};\n\
             pub const HANS_ALL_TOTAL: usize = {hans_total};\n\
             pub const HANT_ALL_HANT_LEN: usize = {hant_len};\n\
             pub const HANT_ALL_TW_END: usize = {tw_end};\n\
             pub const HANT_ALL_TOTAL: usize = {hant_total};\n"
        ),
    )?;

    log_diag!("Built in: {:?}\n=== DONE ===\n", start_time.elapsed())?;

    // CJK compat table for the converter pre-pass (see `normalize_cjk_compat`):
    // emitted as a key-sorted static array (`cjk_norm.rs`) for direct binary
    // search — no automaton, since every entry is 1-char -> 1-char. The CJK
    // dict stays out of the conversion chain: merging it made reverse lookups
    // yield compat variants (`函數`) instead of simplified ones (`函数`) and
    // added ~21% dead keys. Only emitted with `cjk-compat`.
    #[cfg(feature = "cjk-compat")]
    {
        let s = zhconv_data_opencc::raw("CJK_Compatibility_Ideographs.txt");
        let mut pairs: Vec<(u32, u32)> = Vec::new();
        for line in s.lines().map(|l| l.trim()) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (k, vs) = line.split_once(char::is_whitespace).expect("well-formed");
            let mut vs = vs.split_whitespace();
            let (v, rest) = (vs.next().expect("well-formed"), vs.next());
            assert!(rest.is_none(), "CJK dict is 1-char -> 1-char");
            let (kc, vc) = (
                k.chars().next().expect("well-formed"),
                v.chars().next().expect("well-formed"),
            );
            assert!(
                k.chars().count() == 1 && v.chars().count() == 1,
                "CJK dict is 1-char -> 1-char"
            );
            pairs.push((kc as u32, vc as u32));
            // The fast-path byte ranges in `normalize_cjk_compat` must cover exactly these.
            assert!(
                matches!(kc, '\u{F900}'..='\u{FAFF}' | '\u{2F800}'..='\u{2FA1D}'),
                "CJK key outside fast-path ranges"
            );
        }
        pairs.sort_unstable();
        pairs.dedup();
        let mut out = String::from("pub const CJK_NORM_PAIRS: &[(u32, u32)] = &[");
        for (k, v) in &pairs {
            out.push_str(&format!("({k},{v}),"));
        }
        out.push_str("];\n");
        std::fs::write(
            Path::new(&env::var_os("OUT_DIR").unwrap()).join("cjk_norm.rs"),
            out,
        )?;
        log_diag!("CJK_NORM_PAIRS.len = {}\n", pairs.len())?;
    }

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

fn write_vzv_file(name: &str, pairs: &[(String, String)]) -> io::Result<Vec<u8>> {
    use zerovec::vecs::{Index32, VarZeroVecOwned};
    let out_dir = env::var_os("OUT_DIR").unwrap();
    // {from, to}.vzv hold VarZeroVec<str, Index32> bytes: full strings,
    // borrowed zero-copy at runtime (validated in debug builds).
    // Index32: tables exceed the u16 range. zstd-compressed when
    // `compress` is on (same settings as `.daac`).
    let froms: Vec<&str> = pairs.iter().map(|(f, _)| f.as_str()).collect();
    let tos: Vec<&str> = pairs.iter().map(|(_, t)| t.as_str()).collect();
    let from_bytes = VarZeroVecOwned::<str, Index32>::try_from_elements(&froms)
        .expect("VZV-encode froms")
        .as_bytes()
        .to_vec();
    let to_bytes = VarZeroVecOwned::<str, Index32>::try_from_elements(&tos)
        .expect("VZV-encode tos")
        .as_bytes()
        .to_vec();
    let uncompressed_to_bytes = to_bytes.clone();
    #[cfg(feature = "compress")]
    let from_bytes = zstd_compress(&from_bytes)?;
    #[cfg(feature = "compress")]
    let to_bytes = zstd_compress(&to_bytes)?;
    std::fs::write(
        Path::new(&out_dir).join(format!("{name}.from.vzv")),
        from_bytes,
    )?;
    std::fs::write(Path::new(&out_dir).join(format!("{name}.to.vzv")), to_bytes)?;
    Ok(uncompressed_to_bytes)
}

/// Write DAAC automaton file.
///
/// Pattern values directly encode `(offset << 10) | len` pointing into the side store's
/// `to.vzv` byte slice, bypassing runtime VarZeroVec index lookups.
///
/// # Architecture Decision: Uniform Packing vs. Inlining
/// We experimentally evaluated inlining short targets (1-char / 2-char words) directly into
/// the `u32` DAAC value:
/// - In prototype benchmarks, tagged inlining did not show consistent throughput advantages
///   over uniform `(offset << 10) | len` (and showed regressions on sparse/mixed texts).
/// - Suspected / theoretical reasons for the lack of improvement (hypotheses): tag checks add
///   branching in the hot loop, runtime UTF-8 re-encoding of packed BMP codepoints adds ALU overhead,
///   and widening to `u64` doubles automaton value memory which may degrade CPU cache locality.
///
/// Therefore, uniform `(offset << 10) | len` was selected for its branchless value unpacking,
/// simpler code structure, and ability to leverage single-instruction 16B SIMD copying from
/// pre-encoded UTF-8 memory.
///
/// Value remap into the monolithic side store: the first `base_len` pairs
/// keep positional values; the tail maps to `tail_base + (i - base_len)`.
/// Prefix cases pass `tail_base == base_len` (identity).
fn write_daac_file(
    name: &str,
    pairs: &[(String, String)],
    base_len: usize,
    tail_base: u32,
    store_froms: &[&str],
    store_tos: &[&str],
    store_to_bytes: &[u8],
) -> io::Result<()> {
    use zerovec::vecs::Index32;
    use zerovec::VarZeroSlice;

    assert!(base_len <= pairs.len(), "{name}: base past pairs");
    assert!(
        pairs.iter().all(|(f, _)| !f.is_empty()),
        "{name}: empty from poisons the automaton"
    );
    assert!(
        store_to_bytes.len() <= 0x3FFFFF,
        "{name}: total store bytes {} exceeds 22-bit addressable range (4MB)",
        store_to_bytes.len()
    );

    let to_slice = VarZeroSlice::<str, Index32>::parse_bytes(store_to_bytes)
        .expect("store_to_bytes must be valid VarZeroSlice");
    assert_eq!(
        to_slice.len(),
        store_tos.len(),
        "{name}: store slice count mismatch"
    );

    let slot_of = |i: usize| -> usize {
        if i < base_len {
            i
        } else {
            (tail_base as usize) + (i - base_len)
        }
    };

    // A2: every pair's remapped slot holds exactly that pair.
    for (i, (f, t)) in pairs.iter().enumerate() {
        let v = slot_of(i);
        assert!(v < store_tos.len(), "{name}: value {v} out of store");
        assert!(
            store_froms[v] == f && store_tos[v] == t,
            "{name}: remap mismatch at pair {i}"
        );
    }

    let packed_val_of = |slot: usize| -> u32 {
        let word = to_slice.get(slot).unwrap();
        let offset = word.as_ptr() as usize - store_to_bytes.as_ptr() as usize;
        let len = word.len();
        assert!(
            len <= 0x3FF,
            "{name}: target word {word:?} length {len} exceeds 10-bit limit (1023 bytes)"
        );
        assert!(
            offset <= 0x3FFFFF,
            "{name}: target word {word:?} offset {offset} exceeds 22-bit limit (4MB)"
        );
        let packed = ((offset as u32) << 10) | (len as u32);
        assert_eq!(
            (packed >> 10) as usize,
            offset,
            "{name}: offset unpack mismatch"
        );
        assert_eq!(
            (packed & 0x3FF) as usize,
            len,
            "{name}: len unpack mismatch"
        );
        assert_eq!(
            &store_to_bytes[offset..offset + len],
            word.as_bytes(),
            "{name}: target slice mismatch"
        );
        packed
    };

    let mut seen = HashSet::new();
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path_daac = Path::new(&out_dir).join(format!("{name}.daac"));
    let automaton = CharwiseDoubleArrayAhoCorasickBuilder::new()
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
                    let slot = slot_of(i);
                    Some((f, packed_val_of(slot)))
                }
            },
        ))
        .expect(name);

    // A4: end-to-end through the real matcher. Last-wins expectations mirror
    // the rev-dedup above; each distinct key must resolve to its word.
    {
        let mut expected: HashMap<&str, &str> = HashMap::new();
        for (f, t) in pairs {
            expected.insert(f.as_str(), t.as_str());
        }
        for (&f, &t) in &expected {
            let v = automaton
                .leftmost_find_iter(f)
                .next()
                .unwrap_or_else(|| panic!("{name}: key {f:?} unfindable"))
                .value();
            let offset = (v >> 10) as usize;
            let len = (v & 0x3FF) as usize;
            assert_eq!(
                &store_to_bytes[offset..offset + len],
                t.as_bytes(),
                "{name}: match mismatch for {f:?}"
            );
        }
    }
    let daac = automaton.serialize();

    #[cfg(feature = "compress")]
    let daac = zstd_compress(&daac)?;

    File::create(dest_path_daac)?.write_all(&daac)
}

/// Shared zstd settings for bundled artifacts (level 19, no checksum):
/// sized window for the payload, pledged size up front.
#[cfg(feature = "compress")]
fn zstd_compress(data: &[u8]) -> io::Result<Vec<u8>> {
    let window_log = (data.len().max(1).next_power_of_two().trailing_zeros()).clamp(17, 22);
    let mut encoder = zstd::stream::Encoder::new(Vec::new(), 19)?;
    encoder.set_pledged_src_size(Some(data.len() as u64))?;
    encoder.window_log(window_log)?;
    encoder.include_checksum(false)?;
    use std::io::Write;
    encoder.write_all(data)?;
    encoder.finish()
}

fn sort_and_dedup(pairs: &mut Vec<(String, String)>) {
    // earlier rules take precedence
    pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then(a.0.cmp(&b.0)));
    pairs.dedup_by(|a, b| a.0 == b.0);
}
