//! Built-in conversion tables sourced from [zhConversion.php](https://github.com/wikimedia/mediawiki/blob/master/includes/Languages/Data/ZhConversion.php)
//! (maintained by MediaWiki and Chinese Wikipedia) and [OpenCC](https://github.com/BYVoid/OpenCC/tree/master/data/dictionary).
//!
//! One complete table per variant: regional tables include their base
//! entries, with region extras overriding base ones on duplicates.
//! Tables share their underlying storage; builtin converters borrow it.
//! (Storage layout and encoding are build internals; see `build.rs`.)

// Design notes:
// - One monolithic word store per script side: HANS_ALL = hans ++
//   cn-extras, HANT_ALL = hant ++ tw-extras ++ hk-extras.
// - Every table is a ranged view into its side store; builtin converters
//   borrow the whole store while their automata address their ranges.
// - Encoding: VarZeroVec<str, Index32> bytes per column (Index32 because
//   tables exceed the u16 range), zstd-compressed when `compress` is on.
// - Loads borrow zero-copy from the binary; compressed payloads decode
//   once into a leaked buffer. Views and builtin word lists read the same
//   columns, so they cannot diverge.
#[cfg(feature = "compress")]
use std::sync::LazyLock;

use zerovec::vecs::Index32;
use zerovec::VarZeroSlice;

use crate::Variant;

// Unused under partial features (only the enabled sides' consts are read).
#[allow(dead_code)]
mod meta {
    include!(concat!(env!("OUT_DIR"), "/table_meta.rs"));
}

/// A built-in conversion table: the complete conversion set for one
/// target variant, with region extras overriding base entries.
///
/// Opaque: never construct directly; use the `ZH_*_TABLE` constants or
/// [`get_builtin_table`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Table<'s> {
    /// Encoded `from` column of the backing side store.
    pub(crate) from: &'s [u8],
    /// Encoded `to` column of the backing side store.
    pub(crate) to: &'s [u8],
    /// Element intervals into the store forming this view, base-first.
    /// One interval per contiguous segment; later intervals win duplicates.
    /// E.g. `[(0, H)]` takes the base table alone, while `[(0, H), (T, N)]`
    /// appends the segment `[T, N)` after it (HK extras live past the TW
    /// segment, hence the gap).
    pub(crate) ranges: &'s [(usize, usize)],
}

impl<'s> std::fmt::Debug for Table<'s> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Table").finish_non_exhaustive()
    }
}
// pub(crate) const EMPTY_DAAC: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/empty.daac"));

/// Empty table
pub const ZH_TABLE: Table<'static> = Table {
    from: b"",
    to: b"",
    ranges: &[],
};

#[cfg(any(
    feature = "mediawiki-hans",
    feature = "opencc-hans",
    feature = "mediawiki-cn",
    feature = "opencc-cn",
))]
pub(crate) const HANS_ALL_FROM: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/ZH_TO_HANS_ALL.from.vzv"));
#[cfg(any(
    feature = "mediawiki-hans",
    feature = "opencc-hans",
    feature = "mediawiki-cn",
    feature = "opencc-cn",
))]
pub(crate) const HANS_ALL_TO: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/ZH_TO_HANS_ALL.to.vzv"));
#[cfg(any(
    feature = "mediawiki-hant",
    feature = "opencc-hant",
    feature = "mediawiki-tw",
    feature = "opencc-tw",
    feature = "mediawiki-hk",
    feature = "opencc-hk",
))]
pub(crate) const HANT_ALL_FROM: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/ZH_TO_HANT_ALL.from.vzv"));
#[cfg(any(
    feature = "mediawiki-hant",
    feature = "opencc-hant",
    feature = "mediawiki-tw",
    feature = "opencc-tw",
    feature = "mediawiki-hk",
    feature = "opencc-hk",
))]
pub(crate) const HANT_ALL_TO: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/ZH_TO_HANT_ALL.to.vzv"));

/// Simplified Chinese to Traditional Chinese conversion table, including no region-specific phrases
#[cfg(any(feature = "mediawiki-hant", feature = "opencc-hant"))]
pub const ZH_HANT_TABLE: Table<'static> = Table {
    from: HANT_ALL_FROM,
    to: HANT_ALL_TO,
    ranges: &[(0, meta::HANT_ALL_HANT_LEN)],
};
#[cfg(any(feature = "mediawiki-hant", feature = "opencc-hant"))]
#[doc(hidden)]
#[cfg(any(feature = "mediawiki-hant", feature = "opencc-hant"))]
pub(crate) const ZH_HANT_DAAC: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/ZH_TO_HANT.daac"));

/// Traditional Chinese to Simplified Chinese conversion table, including no region-specific phrases
#[cfg(any(feature = "mediawiki-hans", feature = "opencc-hans"))]
pub const ZH_HANS_TABLE: Table<'static> = Table {
    from: HANS_ALL_FROM,
    to: HANS_ALL_TO,
    ranges: &[(0, meta::HANS_ALL_HANS_LEN)],
};
#[cfg(any(feature = "mediawiki-hans", feature = "opencc-hans"))]
#[doc(hidden)]
#[cfg(any(feature = "mediawiki-hans", feature = "opencc-hans"))]
pub(crate) const ZH_HANS_DAAC: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/ZH_TO_HANS.daac"));
/// Taiwan-specific phrases conversion table, including `ZH_HANT_TABLE`
#[cfg(any(feature = "mediawiki-tw", feature = "opencc-tw"))]
pub const ZH_HANT_TW_TABLE: Table<'static> = Table {
    from: HANT_ALL_FROM,
    to: HANT_ALL_TO,
    ranges: &[(0, meta::HANT_ALL_TW_END)],
};
#[cfg(any(feature = "mediawiki-tw", feature = "opencc-tw"))]
#[doc(hidden)]
#[cfg(any(feature = "mediawiki-tw", feature = "opencc-tw"))]
pub(crate) const ZH_HANT_TW_DAAC: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/ZH_TO_HANT_TW.daac"));
/// Hong Kong-specific phrases conversion table, including `ZH_HANT_TABLE`
#[cfg(any(feature = "mediawiki-hk", feature = "opencc-hk"))]
pub const ZH_HANT_HK_TABLE: Table<'static> = Table {
    from: HANT_ALL_FROM,
    to: HANT_ALL_TO,
    ranges: &[
        (0, meta::HANT_ALL_HANT_LEN),
        (meta::HANT_ALL_TW_END, meta::HANT_ALL_TOTAL),
    ],
};
#[cfg(any(feature = "mediawiki-hk", feature = "opencc-hk"))]
#[doc(hidden)]
#[cfg(any(feature = "mediawiki-hk", feature = "opencc-hk"))]
pub(crate) const ZH_HANT_HK_DAAC: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/ZH_TO_HANT_HK.daac"));
/// Macao-specific phrases conversion table, same as `ZH_HANT_HK_TABLE`
#[cfg(any(feature = "mediawiki-hk", feature = "opencc-hk"))]
pub const ZH_HANT_MO_TABLE: Table<'static> = ZH_HANT_HK_TABLE;
#[cfg(any(feature = "mediawiki-hk", feature = "opencc-hk"))]
#[doc(hidden)]
#[cfg(any(feature = "mediawiki-hk", feature = "opencc-hk"))]
pub(crate) const ZH_HANT_MO_DAAC: &[u8] = ZH_HANT_HK_DAAC;
/// Mainland China-specific phrases conversion table, including `ZH_HANS_TABLE`
#[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
pub const ZH_HANS_CN_TABLE: Table<'static> = Table {
    from: HANS_ALL_FROM,
    to: HANS_ALL_TO,
    ranges: &[(0, meta::HANS_ALL_TOTAL)],
};
#[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
#[doc(hidden)]
#[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
pub(crate) const ZH_HANS_CN_DAAC: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/ZH_TO_HANS_CN.daac"));
/// Singapore-specific phrases conversion table, same as `ZH_HANS_CN_TABLE`
#[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
pub const ZH_HANS_SG_TABLE: Table<'static> = ZH_HANS_CN_TABLE;
#[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
#[doc(hidden)]
#[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
pub(crate) const ZH_HANS_SG_DAAC: &[u8] = ZH_HANS_CN_DAAC;
/// Malaysia-specific phrases conversion table, same as `ZH_HANS_CN_TABLE`
#[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
pub const ZH_HANS_MY_TABLE: Table<'static> = ZH_HANS_SG_TABLE;
#[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
#[doc(hidden)]
#[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
pub(crate) const ZH_HANS_MY_DAAC: &[u8] = ZH_HANS_SG_DAAC;

/// Borrow a side store's `to` column: zero-copy from the binary, or a
/// one-time decode into a leaked buffer when `compress` is on. The leak
/// is intentional and bounded (one buffer per store per process): no
/// adopt-bytes constructor exists, so borrowing needs a `'static` home,
/// and validation runs once here (plus always in debug builds).
#[cfg(any(
    feature = "mediawiki-hans",
    feature = "opencc-hans",
    feature = "mediawiki-cn",
    feature = "opencc-cn",
))]
pub(crate) fn hans_all_store() -> &'static VarZeroSlice<str, Index32> {
    #[cfg(feature = "compress")]
    {
        static STORE: LazyLock<&'static VarZeroSlice<str, Index32>> = LazyLock::new(|| {
            let raw = crate::utils::zstd_decompress(HANS_ALL_TO);
            debug_assert!(VarZeroSlice::<str, Index32>::parse_bytes(&raw).is_ok());
            // SAFETY: `raw` is lossless zstd output of `VarZeroVecOwned::as_bytes`
            // bytes, which cannot be malformed, with build-dep and
            // dependency versions pinned together in Cargo.lock — the same
            // trust basis as the `deserialize_unchecked` automaton.
            let leaked: &'static [u8] = Box::leak(raw.into_boxed_slice());
            unsafe { VarZeroSlice::from_bytes_unchecked(leaked) }
        });
        *STORE
    }
    #[cfg(not(feature = "compress"))]
    {
        debug_assert!(VarZeroSlice::<str, Index32>::parse_bytes(HANS_ALL_TO).is_ok());
        // SAFETY: `include_bytes!` of encoder output; see above.
        unsafe { VarZeroSlice::from_bytes_unchecked(HANS_ALL_TO) }
    }
}

/// Same as [`hans_all_store`], for the hant side.
#[cfg(any(
    feature = "mediawiki-hant",
    feature = "opencc-hant",
    feature = "mediawiki-tw",
    feature = "opencc-tw",
    feature = "mediawiki-hk",
    feature = "opencc-hk",
))]
pub(crate) fn hant_all_store() -> &'static VarZeroSlice<str, Index32> {
    #[cfg(feature = "compress")]
    {
        static STORE: LazyLock<&'static VarZeroSlice<str, Index32>> = LazyLock::new(|| {
            let raw = crate::utils::zstd_decompress(HANT_ALL_TO);
            debug_assert!(VarZeroSlice::<str, Index32>::parse_bytes(&raw).is_ok());
            // SAFETY: as in `hans_all_store`.
            let leaked: &'static [u8] = Box::leak(raw.into_boxed_slice());
            unsafe { VarZeroSlice::from_bytes_unchecked(leaked) }
        });
        *STORE
    }
    #[cfg(not(feature = "compress"))]
    {
        debug_assert!(VarZeroSlice::<str, Index32>::parse_bytes(HANT_ALL_TO).is_ok());
        // SAFETY: `include_bytes!` of encoder output; see above.
        unsafe { VarZeroSlice::from_bytes_unchecked(HANT_ALL_TO) }
    }
}

/// Expand a built-in conversion table into owned pairs.
///
/// Returns raw pairs in range order; a table may yield the same `from`
/// twice where a regional entry overrides a base one. Dedup is the
/// caller's job ([`ZhConverterBuilder`](crate::ZhConverterBuilder) folds
/// them later-wins). Cold path only.
pub fn expand_table(table: Table<'_>) -> Vec<(String, String)> {
    if table.ranges.is_empty() {
        return Vec::new();
    }
    #[cfg(feature = "compress")]
    let (from_buf, to_buf) = (
        crate::utils::zstd_decompress(table.from),
        crate::utils::zstd_decompress(table.to),
    );
    #[cfg(feature = "compress")]
    let (from_bytes, to_bytes) = (&from_buf[..], &to_buf[..]);
    #[cfg(not(feature = "compress"))]
    let (from_bytes, to_bytes) = (table.from, table.to);
    debug_assert!(VarZeroSlice::<str, Index32>::parse_bytes(from_bytes).is_ok());
    debug_assert!(VarZeroSlice::<str, Index32>::parse_bytes(to_bytes).is_ok());
    // SAFETY: build-written stores transported exactly; validated above
    // in debug builds, unchecked in release. Checked parsing here would
    // harden nothing: corruption implies DAAC corruption first, and the
    // DAAC stays unchecked by design — one policy for all three sites.
    let froms = unsafe { VarZeroSlice::<str, Index32>::from_bytes_unchecked(from_bytes) };
    let tos = unsafe { VarZeroSlice::<str, Index32>::from_bytes_unchecked(to_bytes) };
    debug_assert_eq!(froms.len(), tos.len(), "store columns diverge");
    let mut out = Vec::with_capacity(table.ranges.iter().map(|&(s, e)| e.saturating_sub(s)).sum());
    // Range-driven (not raw store order): output order always matches
    // range order, so documented later-wins precedence holds structurally.
    for &(s, e) in table.ranges {
        debug_assert!(e <= froms.len(), "view range past store");
        for i in s..e {
            if let (Some(from), Some(to)) = (froms.get(i), tos.get(i)) {
                // Empty sources would poison the automaton: daachorse
                // ignores all other patterns when the set contains an
                // empty string (matching empty at every boundary), so
                // conversion would silently stop. Drop them here.
                if !from.is_empty() {
                    out.push((from.to_owned(), to.to_owned()));
                }
            }
        }
    }
    out
}

/// Get the builtin conversion table for a target Chinese variant.
///
/// Accessing the raw table is only useful when building a custom converter.
/// Otherwise, there is [`get_builtin_converter`](crate::get_builtin_converter).
#[inline(always)]
pub fn get_builtin_table(target: Variant) -> Table<'static> {
    use Variant::*;

    match target {
        Zh => ZH_TABLE,
        #[cfg(any(feature = "mediawiki-hant", feature = "opencc-hant"))]
        ZhHant => ZH_HANT_TABLE,
        #[cfg(any(feature = "mediawiki-hans", feature = "opencc-hans"))]
        ZhHans => ZH_HANS_TABLE,
        #[cfg(any(feature = "mediawiki-tw", feature = "opencc-tw"))]
        ZhTW => ZH_HANT_TW_TABLE,
        #[cfg(any(feature = "mediawiki-hk", feature = "opencc-hk"))]
        ZhHK => ZH_HANT_HK_TABLE,
        #[cfg(any(feature = "mediawiki-hk", feature = "opencc-hk"))]
        ZhMO => ZH_HANT_MO_TABLE,
        #[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
        ZhCN => ZH_HANS_CN_TABLE,
        #[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
        ZhMY => ZH_HANS_MY_TABLE,
        #[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
        ZhSG => ZH_HANS_SG_TABLE,
        #[allow(unreachable_patterns)]
        _ => panic!("No table for {} enabled.", target),
    }
}

/// Get the builtin word store for a target Chinese variant.
///
/// The store backs every builtin converter on the variant's script side.
/// For the load benchmark; otherwise there is
/// [`get_builtin_converter`](crate::get_builtin_converter).
#[doc(hidden)]
#[inline(always)]
pub fn get_builtin_store(target: Variant) -> &'static VarZeroSlice<str, Index32> {
    use Variant::*;

    match target {
        Zh => VarZeroSlice::<str, Index32>::new_empty(),
        #[cfg(any(feature = "mediawiki-hans", feature = "opencc-hans"))]
        ZhHans => hans_all_store(),
        #[cfg(any(feature = "mediawiki-tw", feature = "opencc-tw"))]
        ZhTW => hant_all_store(),
        #[cfg(any(feature = "mediawiki-hk", feature = "opencc-hk"))]
        ZhHK => hant_all_store(),
        #[cfg(any(feature = "mediawiki-hk", feature = "opencc-hk"))]
        ZhMO => hant_all_store(),
        #[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
        ZhCN => hans_all_store(),
        #[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
        ZhMY => hans_all_store(),
        #[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
        ZhSG => hans_all_store(),
        #[cfg(any(feature = "mediawiki-hant", feature = "opencc-hant"))]
        ZhHant => hant_all_store(),
        #[allow(unreachable_patterns)]
        _ => panic!("No store for {} enabled.", target),
    }
}

#[doc(hidden)]
#[inline(always)]
pub fn get_builtin_serialized_daac(target: Variant) -> &'static [u8] {
    use Variant::*;

    match target {
        Zh => unreachable!(),
        #[cfg(any(feature = "mediawiki-hant", feature = "opencc-hant"))]
        ZhHant => ZH_HANT_DAAC,
        #[cfg(any(feature = "mediawiki-hans", feature = "opencc-hans"))]
        ZhHans => ZH_HANS_DAAC,
        #[cfg(any(feature = "mediawiki-tw", feature = "opencc-tw"))]
        ZhTW => ZH_HANT_TW_DAAC,
        #[cfg(any(feature = "mediawiki-hk", feature = "opencc-hk"))]
        ZhHK => ZH_HANT_HK_DAAC,
        #[cfg(any(feature = "mediawiki-hk", feature = "opencc-hk"))]
        ZhMO => ZH_HANT_MO_DAAC,
        #[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
        ZhCN => ZH_HANS_CN_DAAC,
        #[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
        ZhMY => ZH_HANS_MY_DAAC,
        #[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
        ZhSG => ZH_HANS_SG_DAAC,
        #[allow(unreachable_patterns)]
        _ => panic!("No daac for {} enabled.", target),
    }
}

// https://github.com/wikimedia/mediawiki/blob/6eda8891a0595e72e350998b6bada19d102a42d9/includes/language/converters/ZhConverter.php#L144
