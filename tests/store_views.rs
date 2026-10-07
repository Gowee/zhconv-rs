//! Store/view consistency: custom builds from table views must behave
//! exactly like the builtin borrowing converters, under any feature set.
//!
//! Complements the build-time index assertions (which pin one compilation)
//! by pinning the runtime contract across CI feature combos. Asserts only
//! feature-agnostic properties — never specific outputs, which vary by
//! ruleset (MediaWiki vs OpenCC).
use zhconv::{
    expand_table, get_builtin_converter, get_builtin_table, Variant, ZhConverterBuilder,
    ENABLED_TARGET_VARIANTS,
};

const TEXTS: &[&str] = &[
    "",
    "hello",
    "天干物燥 小心火烛",
    "阿拉伯联合酋长国",
    "服务器",
    "出租车",
    "鼠标",
    "软件",
    "函數",
    "鼠曲草",
    "内存条",
    "甲-{zh-hans:乙;}-丙",
    "中文混English混合テキスト",
];

#[test]
fn custom_build_matches_builtin() {
    for &v in ENABLED_TARGET_VARIANTS {
        let builtin = get_builtin_converter(v);
        let custom = ZhConverterBuilder::new()
            .target(v)
            .table(get_builtin_table(v))
            .build();
        for t in TEXTS {
            assert_eq!(
                builtin.convert(t),
                custom.convert(t),
                "{v} diverged on {t:?}"
            );
        }
    }
}

#[test]
fn table_views_well_formed() {
    for &v in ENABLED_TARGET_VARIANTS {
        let pairs = expand_table(get_builtin_table(v));
        if v == Variant::Zh {
            assert!(pairs.is_empty(), "blank view must stay empty");
            continue;
        }
        assert!(!pairs.is_empty(), "{v} view empty");
        for (f, t) in &pairs {
            // Empty sources break the automaton. Duplicate sources are
            // legitimate (regional override); `build_mapping` folds them.
            assert!(!f.is_empty(), "{v} has empty from");
            let _ = t;
        }
    }
}

#[test]
fn blank_table_expand() {
    assert!(zhconv::expand_table(zhconv::ZH_TABLE).is_empty());
}

/// Regional aliases share their target's table; a split would be a
/// deliberate product decision, not a silent drift.
#[cfg(any(feature = "mediawiki-hk", feature = "opencc-hk"))]
#[test]
fn mo_aliases_hk_table() {
    assert_eq!(zhconv::ZH_HANT_MO_TABLE, zhconv::ZH_HANT_HK_TABLE);
}

/// Same as above, for the hans side.
#[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
#[test]
fn sg_my_alias_cn_table() {
    assert_eq!(zhconv::ZH_HANS_SG_TABLE, zhconv::ZH_HANS_CN_TABLE);
    assert_eq!(zhconv::ZH_HANS_MY_TABLE, zhconv::ZH_HANS_CN_TABLE);
}

/// HK extras identical to TW extras must survive the retain: the TW
/// segment is not in HK's automaton, so comparing against it silently
/// drops HK rules (HK fell back to script-level output).
#[cfg(feature = "mediawiki-hk")]
#[test]
fn hk_keeps_tw_shared_regionals() {
    let hk = zhconv::get_builtin_converter(zhconv::Variant::ZhHK);
    for (from, to) in [
        ("计算机程序", "電腦程式"),
        ("泰坦尼克号", "鐵達尼號"),
        ("乔治·奥威尔", "喬治·歐威爾"),
    ] {
        assert_eq!(hk.convert(from), to, "HK lost regional {from:?}");
    }
}

/// Exhaustive rule firing: every distinct key of every view must convert
/// to its mapped word through the shipped automaton+store.
///
/// Catches dropped/misaligned rules — the failure class differentials
/// between two consumers of the same views cannot see. A failure means
/// the shipped automaton and its word store disagree: suspect stale
/// build artifacts first, then view ranges and value remapping.
#[test]
fn all_view_keys_fire() {
    use std::collections::HashMap;
    for &v in ENABLED_TARGET_VARIANTS {
        let mut map: HashMap<String, String> = HashMap::new();
        for (f, t) in expand_table(get_builtin_table(v)) {
            map.insert(f, t); // later (regional) wins, like build_mapping
        }
        let c = get_builtin_converter(v);
        for (f, t) in &map {
            assert_eq!(&c.convert(f), t, "{v} dropped/misaligned {f:?}");
        }
    }
}
