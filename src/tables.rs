//! Built-in conversion tables extracted from [zhConversion.php](https://phabricator.wikimedia.org/source/mediawiki/browse/master/includes/languages/data/ZhConversion.php)
//! (maintained by MediaWiki and Chinese Wikipedia) and [OpenCC](https://github.com/BYVoid/OpenCC/tree/master/data/dictionary).
//!
//! # Note
//! Region specific conversion tables exclude basic rulesets such as `zh-Hant` or `zh-Hans`. They
//! should not be used on their own. For example, to convert text to `zh-TW` (i.e. `zh-Hant-TW`),
//! both [`ZH_HANT_TABLE`] and [`ZH_TW_TABLE`] should specified together, in order, when building
//! the converter.

// use itertools;
// use lazy_static::lazy_static;
use std::iter;

use crate::converter::{ZhConverter, ZhConverterBuilder};
use crate::Variant;

pub type Table<'s> = (&'s str, &'s str);

pub(crate) const EMPTY_TABLES: [Table; 0] = [];
/// Simplified Chinese to Traditional Chinese conversion table, including no region-specific phrases
pub const ZH_HANT_TABLE: Table<'static> = (
    include_str!(concat!(env!("OUT_DIR"), "/zh2Hant.from.conv")),
    include_str!(concat!(env!("OUT_DIR"), "/zh2Hant.to.conv")),
);
pub(crate) const ZH_HANT_TABLES: [Table; 1] = [ZH_HANT_TABLE];
/// Traditional Chinese to Simplified Chinese conversion table, including no region-specific phrases
pub const ZH_HANS_TABLE: Table<'static> = (
    include_str!(concat!(env!("OUT_DIR"), "/zh2Hans.from.conv")),
    include_str!(concat!(env!("OUT_DIR"), "/zh2Hans.to.conv")),
);
pub(crate) const ZH_HANS_TABLES: [Table; 1] = [ZH_HANS_TABLE];
/// Taiwan-specific phrases conversion table, in addition to zh-Hant
pub const ZH_TW_TABLE: Table<'static> = (
    include_str!(concat!(env!("OUT_DIR"), "/zh2TW.from.conv")),
    include_str!(concat!(env!("OUT_DIR"), "/zh2TW.to.conv")),
);
pub(crate) const ZH_HANT_TW_TABLES: [Table; 2] = [ZH_HANT_TABLE, ZH_TW_TABLE];
/// Hong Kong-specific phrases conversion table, in addition to zh-Hant
pub const ZH_HK_TABLE: Table<'static> = (
    include_str!(concat!(env!("OUT_DIR"), "/zh2HK.from.conv")),
    include_str!(concat!(env!("OUT_DIR"), "/zh2HK.to.conv")),
);
pub(crate) const ZH_HANT_HK_TABLES: [Table; 2] = [ZH_HANT_TABLE, ZH_HK_TABLE];
/// Macau-specific phrases conversion table, in addition to zh-Hant
pub const ZH_MO_TABLE: Table<'static> = ZH_HK_TABLE;
pub(crate) const ZH_HANT_MO_TABLES: [Table; 2] = [ZH_HANT_TABLE, ZH_MO_TABLE];
/// Mainland China-specific phrases conversion table, in addition to zh-Hans
pub const ZH_CN_TABLE: Table<'static> = (
    include_str!(concat!(env!("OUT_DIR"), "/zh2CN.from.conv")),
    include_str!(concat!(env!("OUT_DIR"), "/zh2CN.to.conv")),
);
pub(crate) const ZH_HANS_CN_TABLES: [Table; 2] = [ZH_HANS_TABLE, ZH_CN_TABLE];
/// Mainland Singapore-specific phrases conversion table, in addition to zh-Hans
pub const ZH_SG_TABLE: Table<'static> = ZH_CN_TABLE;
pub(crate) const ZH_HANS_SG_TABLES: [Table; 2] = [ZH_HANS_TABLE, ZH_SG_TABLE];
/// Mainland Singapore-specific phrases conversion table, in addition to zh-Hans
pub const ZH_MY_TABLE: Table<'static> = ZH_SG_TABLE;
pub(crate) const ZH_HANS_MY_TABLES: [Table; 2] = [ZH_HANS_TABLE, ZH_MY_TABLE];

// pub const ZH_HANT_TW_TABLES: [Table]= [ZH_HANT_TABLE, ZH_TW_TABLE];
// pub const ZH_HANT_HK_TABLES: [Table]= [ZH_HANT_TABLE, ZH_HK_TABLE];
// pub const ZH_HANT_MO_TABLES:

// struct

// // Ref: https://github.com/wikimedia/mediawiki/blob/6eda8891a0595e72e350998b6bada19d102a42d9/includes/language/converters/ZhConverter.php#L157
// // More specific rules should take precedence when merging (e.g. zh-TW > zh-Hant).
// // Since the converter build process relies on HashMap::extend, latter rules overwrite the early ones.
// lazy_static! {
//     /// For `ZH_TO_TW_CONVERTER`, merged from `ZH_HANT_TABLE` and `ZH_TW_TABLE`
//     pub static ref ZH_HANT_TW_TABLE: (&'static str, &'static str) =
//         merge_tables_leaked(ZH_HANT_TABLE, ZH_TW_TABLE);
//     /// For `ZH_TO_HK_CONVERTER`, merged from `ZH_HANT_TABLE` and `ZH_HK_TABLE`
//     pub static ref ZH_HANT_HK_TABLE: (&'static str, &'static str) =
//         merge_tables_leaked(ZH_HANT_TABLE, ZH_HK_TABLE);
//     /// For `ZH_TO_MO_CONVERTER`, merged from `ZH_HANT_TABLE` and `ZH_MO_TABLE`
//     pub static ref ZH_HANT_MO_TABLE: (&'static str, &'static str) =
//         merge_tables_leaked(ZH_HANT_TABLE, ZH_MO_TABLE);
//     /// For `ZH_TO_CN_CONVERTER`, merged from `ZH_HANS_TABLE` and `ZH_CN_TABLE`
//     pub static ref ZH_HANS_CN_TABLE: (&'static str, &'static str) =
//         merge_tables_leaked(ZH_HANS_TABLE, ZH_CN_TABLE);
//     /// For `ZH_TO_SG_CONVERTER`, merged from `ZH_HANS_TABLE` and `ZH_SG_TABLE`
//     pub static ref ZH_HANS_SG_TABLE: (&'static str, &'static str) =
//         merge_tables_leaked(ZH_HANS_TABLE, ZH_SG_TABLE);
//     /// For `ZH_TO_MY_CONVERTER`, merged from `ZH_HANS_TABLE` and `ZH_MY_TABLE`
//     pub static ref ZH_HANS_MY_TABLE: (&'static str, &'static str) =
//         merge_tables_leaked(ZH_HANS_TABLE, ZH_MY_TABLE);
// }

// // TODO: How to make these lazy consts more idiomatic?

// /// Merge two conversion Table<'static> and leak the merged string.
// fn merge_tables_leaked(conv1: Table, conv2: Table) -> (&'static str, &'static str) {
//     let (froms, tos) = merge_tables(conv1, conv2);
//     (
//         Box::leak(froms.into_boxed_str()),
//         Box::leak(tos.into_boxed_str()),
//     )
// }

// /// Merge two conversion table.
// pub fn merge_tables(conv1: Table, conv2: Table) -> (String, String) {
//     let mut froms = String::with_capacity(conv1.0.len() + conv2.0.len());
//     let mut tos = String::with_capacity(conv1.1.len() + conv2.1.len());
//     // merge_by detains the first occurrence
//     let mut it = itertools::Itertools::merge_by(
//         std::iter::zip(conv1.0.trim().split('|'), conv1.1.trim().split('|')),
//         std::iter::zip(conv2.0.trim().split('|'), conv2.1.trim().split('|')),
//         |pair1, pair2| pair1.0.len() >= pair2.0.len(),
//     )
//     .peekable();
//     while let Some((from, to)) = it.next() {
//         froms.push_str(from);
//         tos.push_str(to);
//         if it.peek().is_some() {
//             froms.push('|');
//             tos.push('|');
//         }
//     }
//     (froms, tos)
// }

/// Expand a compressed built-in conversion table.
pub fn expand_table<'s>((froms, tos): Table<'s>) -> impl Iterator<Item = (String, String)> + 's {
    std::iter::zip(froms.trim().split('|'), tos.trim().split('|')).scan(
        String::from(""),
        move |last_from, (from, to)| {
            let from: String = expand_pair(from.chars(), last_from.chars()).collect();
            let to: String = expand_pair(to.chars(), from.chars()).collect();
            last_from.clear();
            last_from.push_str(&from);
            Some((from, to))
        },
    )
}

#[doc(hidden)]
pub fn expand_pair<'s>(
    mut s: impl Iterator<Item = char> + 's,
    mut base: impl Iterator<Item = char> + 's,
) -> impl Iterator<Item = char> + 's {
    let SYMBOL_START: char = '\x00';
    let SYMBOL_END: u32 = 32;

    let mut expanding = 0;
    iter::from_fn(move || {
        let b = base.next();
        if expanding == 0 {
            match s.next() {
                Some(a) if (a as u32) < SYMBOL_END => expanding = (a as u32 - SYMBOL_START as u32),
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
pub fn build_converter<'t>(variant: Variant, tables: &[Table<'t>]) -> ZhConverter {
    let mut builder = ZhConverterBuilder::new().target(variant);
    for &table in tables.iter() {
        builder = builder.table(table);
    }
    builder.dfa(true).build()
}

/// Get the builtin conversion Table<'static> for a target Chinese variant.
///
/// Accessing a Table<'static> is only necessary when building a custom converter.
/// Otherwise, there is [`get_builtin_converter`].
#[inline(always)]
pub fn get_builtin_tables(target: Variant) -> &'static [Table<'static>] {
    use Variant::*;

    match target {
        Zh => &EMPTY_TABLES,
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

// https://github.com/wikimedia/mediawiki/blob/6eda8891a0595e72e350998b6bada19d102a42d9/includes/language/converters/ZhConverter.php#L144
