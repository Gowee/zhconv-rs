//! Built-in conversion tables extracted from [zhConversion.php](https://phabricator.wikimedia.org/source/mediawiki/browse/master/includes/languages/data/ZhConversion.php)
//! (maintained by MediaWiki and Chinese Wikipedia) and [OpenCC](https://github.com/BYVoid/OpenCC/tree/master/data/dictionary).
//!
// // ! # Note
// // ! Region specific conversion tables exclude basic rulesets such as `zh-Hant` or `zh-Hans`. They
// // ! should not be used on their own. For example, to convert text to `zh-TW` (i.e. `zh-Hant-TW`),
// // ! both [`ZH_HANT_TABLE`] and [`ZH_TW_TABLE`] should specified together, in order, when building
// // ! the converter.
use std::iter;

use crate::converter::{ZhConverter, ZhConverterBuilder};
use crate::Variant;

pub type Table<'s> = (&'s str, &'s str);
// pub(crate) const EMPTY_DAAC: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/empty.daac"));

/// Empty table
pub const ZH_TABLE: Table<'static> = ("", "");
pub(crate) const ZH_TABLES: [Table; 0] = [];
/// Simplified Chinese to Traditional Chinese conversion table, including no region-specific phrases
pub const ZH_HANT_TABLE: Table<'static> = (
    include_str!(concat!(env!("OUT_DIR"), "/ZH_TO_HANT.from.conv")),
    include_str!(concat!(env!("OUT_DIR"), "/ZH_TO_HANT.to.conv")),
);
pub(crate) const ZH_HANT_TABLES: [Table; 1] = [ZH_HANT_TABLE];
#[doc(hidden)]
pub(crate) const ZH_HANT_DAAC: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/ZH_TO_HANT.daac"));
/// Traditional Chinese to Simplified Chinese conversion table, including no region-specific phrases
pub const ZH_HANS_TABLE: Table<'static> = (
    include_str!(concat!(env!("OUT_DIR"), "/ZH_TO_HANS.from.conv")),
    include_str!(concat!(env!("OUT_DIR"), "/ZH_TO_HANS.to.conv")),
);
pub(crate) const ZH_HANS_TABLES: [Table; 1] = [ZH_HANS_TABLE];
#[doc(hidden)]
pub(crate) const ZH_HANS_DAAC: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/ZH_TO_HANS.daac"));
/// Taiwan-specific phrases conversion table, excluding `ZH_HANT_TABLE`
pub const ZH_TW_TABLE: Table<'static> = (
    include_str!(concat!(env!("OUT_DIR"), "/ZH_TO_TW.from.conv")),
    include_str!(concat!(env!("OUT_DIR"), "/ZH_TO_TW.to.conv")),
);
pub(crate) const ZH_HANT_TW_TABLES: [Table; 2] = [ZH_HANT_TABLE, ZH_TW_TABLE];
#[doc(hidden)]
pub(crate) const ZH_HANT_TW_DAAC: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/ZH_TO_HANT_TW.daac"));
/// Hong Kong-specific phrases conversion table, excluding `ZH_HANT_TABLE`
pub const ZH_HK_TABLE: Table<'static> = (
    include_str!(concat!(env!("OUT_DIR"), "/ZH_TO_HK.from.conv")),
    include_str!(concat!(env!("OUT_DIR"), "/ZH_TO_HK.to.conv")),
);
pub(crate) const ZH_HANT_HK_TABLES: [Table; 2] = [ZH_HANT_TABLE, ZH_HK_TABLE];
#[doc(hidden)]
pub(crate) const ZH_HANT_HK_DAAC: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/ZH_TO_HANT_HK.daac"));
/// Macao-specific phrases conversion table, excluding `ZH_HANT_TABLE`
pub const ZH_MO_TABLE: Table<'static> = ZH_HK_TABLE;
pub(crate) const ZH_HANT_MO_TABLES: [Table; 2] = [ZH_HANT_TABLE, ZH_MO_TABLE];
#[doc(hidden)]
pub(crate) const ZH_HANT_MO_DAAC: &[u8] = ZH_HANT_HK_DAAC;
/// Mainland China-specific phrases conversion table, excluding `ZH_HANS_TABLE`
pub const ZH_CN_TABLE: Table<'static> = (
    include_str!(concat!(env!("OUT_DIR"), "/ZH_TO_CN.from.conv")),
    include_str!(concat!(env!("OUT_DIR"), "/ZH_TO_CN.to.conv")),
);
pub(crate) const ZH_HANS_CN_TABLES: [Table; 2] = [ZH_HANS_TABLE, ZH_CN_TABLE];
#[doc(hidden)]
pub(crate) const ZH_HANS_CN_DAAC: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/ZH_TO_HANS_CN.daac"));
/// Singapore-specific phrases conversion table, excluding `ZH_HANS_TABLE`
pub const ZH_SG_TABLE: Table<'static> = ZH_CN_TABLE;
pub(crate) const ZH_HANS_SG_TABLES: [Table; 2] = [ZH_HANS_TABLE, ZH_SG_TABLE];
#[doc(hidden)]
pub(crate) const ZH_HANS_SG_DAAC: &[u8] = ZH_HANS_CN_DAAC;
/// Malaysia-specific phrases conversion table, excluding `ZH_HANS_TABLE`
pub const ZH_MY_TABLE: Table<'static> = ZH_SG_TABLE;
pub(crate) const ZH_HANS_MY_TABLES: [Table; 2] = [ZH_HANS_TABLE, ZH_MY_TABLE];
#[doc(hidden)]
pub(crate) const ZH_HANS_MY_DAAC: &[u8] = ZH_HANS_SG_DAAC;

/// Expand a compressed built-in conversion table.
pub fn expand_table((froms, tos): Table<'_>) -> impl Iterator<Item = (String, String)> + '_ {
    std::iter::zip(froms.trim().split('|'), tos.trim().split('|')).scan(
        String::from(""),
        move |last_from, (from, to)| {
            let from: String = pair_expand(from.chars(), last_from.chars()).collect();
            let to: String = pair_expand(to.chars(), from.chars()).collect();
            last_from.clear();
            last_from.push_str(&from);
            Some((from, to))
        },
    )
}

const SURROGATE_START: u32 = 0x00;
const SURROGATE_END: u32 = 0x20;

#[doc(hidden)]
pub fn pair_expand<'s>(
    mut s: impl Iterator<Item = char> + 's,
    mut base: impl Iterator<Item = char> + 's,
) -> impl Iterator<Item = char> + 's {
    let mut expanding = 0;
    iter::from_fn(move || {
        let b = base.next();
        if expanding == 0 {
            match s.next() {
                Some(a) if (a as u32) < SURROGATE_END => expanding = a as u32 - SURROGATE_START,
                Some(a) => return Some(a),
                None => return None,
            }
        }
        expanding -= 1;
        Some(b.expect("compressed rulesets should be well-formed"))
    })
}

/// Helper function to build a `ZhConverter` from a conversion table.
///
/// It is just a simple wrapper around [`ZhConverterBuilder`](crate::ZhConverterBuilder).
#[doc(hidden)]
pub fn build_converter(variant: Variant, table: Table<'_>) -> ZhConverter {
    ZhConverterBuilder::new()
        .target(variant)
        .table(table)
        .build()
}

/// Get the builtin conversion tables for a target Chinese variant.
///
/// Accessing raw tables are only useful when building a custom converter.
/// Otherwise, there is [`get_builtin_converter`](crate::get_builtin_converter).
#[inline(always)]
pub fn get_builtin_tables(target: Variant) -> &'static [Table<'static>] {
    use Variant::*;

    match target {
        Zh => &ZH_TABLES,
        ZhHant => &ZH_HANT_TABLES,
        ZhHans => &ZH_HANS_TABLES,
        ZhTW => &ZH_HANT_TW_TABLES,
        ZhHK => &ZH_HANT_HK_TABLES,
        ZhMO => &ZH_HANT_MO_TABLES,
        ZhCN => &ZH_HANS_CN_TABLES,
        ZhMY => &ZH_HANS_MY_TABLES,
        ZhSG => &ZH_HANS_SG_TABLES,
    }
}

#[doc(hidden)]
#[inline(always)]
pub fn get_builtin_serialized_daac(target: Variant) -> &'static [u8] {
    use Variant::*;

    match target {
        Zh => unimplemented!(),
        ZhHant => ZH_HANT_DAAC,
        ZhHans => ZH_HANS_DAAC,
        ZhTW => ZH_HANT_TW_DAAC,
        ZhHK => ZH_HANT_HK_DAAC,
        ZhMO => ZH_HANT_MO_DAAC,
        ZhCN => ZH_HANS_CN_DAAC,
        ZhMY => ZH_HANS_MY_DAAC,
        ZhSG => ZH_HANS_SG_DAAC,
    }
}

// https://github.com/wikimedia/mediawiki/blob/6eda8891a0595e72e350998b6bada19d102a42d9/includes/language/converters/ZhConverter.php#L144
