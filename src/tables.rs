//! Built-in conversion tables extracted from [zhConversion.php](https://phabricator.wikimedia.org/source/mediawiki/browse/master/includes/languages/data/ZhConversion.php)
//! maintained by MediaWiki and the Chinese Wikipedia community.

use itertools;
use lazy_static::lazy_static;

use crate::converter::{ZhConverter, ZhConverterBuilder};
use crate::Variant;

/// Simplified Chinese to Traditional Chinese conversion table, including no region-specific phrases
pub const ZH_HANT_TABLE: (&str, &str) = (
    include_str!(concat!(env!("OUT_DIR"), "/zh2Hant.from.conv")),
    include_str!(concat!(env!("OUT_DIR"), "/zh2Hant.to.conv")),
);
/// Traditional Chinese to Simplified Chinese conversion table, including no region-specific phrases
pub const ZH_HANS_TABLE: (&str, &str) = (
    include_str!(concat!(env!("OUT_DIR"), "/zh2Hans.from.conv")),
    include_str!(concat!(env!("OUT_DIR"), "/zh2Hans.to.conv")),
);
/// Taiwan-specific phrases conversion table
pub const ZH_TW_TABLE: (&str, &str) = (
    include_str!(concat!(env!("OUT_DIR"), "/zh2TW.from.conv")),
    include_str!(concat!(env!("OUT_DIR"), "/zh2TW.to.conv")),
);
/// Hong Kong-specific phrases conversion table
pub const ZH_HK_TABLE: (&str, &str) = (
    include_str!(concat!(env!("OUT_DIR"), "/zh2HK.from.conv")),
    include_str!(concat!(env!("OUT_DIR"), "/zh2HK.to.conv")),
);
/// Macau-specific phrases conversion table
pub const ZH_MO_TABLE: (&str, &str) = ZH_HK_TABLE;
/// Mainland China-specific phrases conversion table
pub const ZH_CN_TABLE: (&str, &str) = (
    include_str!(concat!(env!("OUT_DIR"), "/zh2CN.from.conv")),
    include_str!(concat!(env!("OUT_DIR"), "/zh2CN.to.conv")),
);
/// Mainland Singapore-specific phrases conversion table
pub const ZH_SG_TABLE: (&str, &str) = ZH_CN_TABLE;
/// Mainland Singapore-specific phrases conversion table
pub const ZH_MY_TABLE: (&str, &str) = ZH_SG_TABLE;

// Ref: https://github.com/wikimedia/mediawiki/blob/6eda8891a0595e72e350998b6bada19d102a42d9/includes/language/converters/ZhConverter.php#L157
lazy_static! {
    /// For `ZH_TO_TW_CONVERTER`, merged from `ZH_HANT_TABLE` and `ZH_TW_TABLE`
    pub static ref ZH_HANT_TW_TABLE: (&'static str, &'static str) =
        merge_tables_leaked(ZH_TW_TABLE, ZH_HANT_TABLE);
    /// For `ZH_TO_HK_CONVERTER`, merged from `ZH_HANT_TABLE` and `ZH_HK_TABLE`
    pub static ref ZH_HANT_HK_TABLE: (&'static str, &'static str) =
        merge_tables_leaked(ZH_HK_TABLE, ZH_HANT_TABLE);
    /// For `ZH_TO_MO_CONVERTER`, merged from `ZH_HANT_TABLE` and `ZH_MO_TABLE`
    pub static ref ZH_HANT_MO_TABLE: (&'static str, &'static str) =
        merge_tables_leaked(ZH_MO_TABLE, ZH_HANT_TABLE);
    /// For `ZH_TO_CN_CONVERTER`, merged from `ZH_HANS_TABLE` and `ZH_CN_TABLE`
    pub static ref ZH_HANS_CN_TABLE: (&'static str, &'static str) =
        merge_tables_leaked(ZH_CN_TABLE, ZH_HANS_TABLE);
    /// For `ZH_TO_SG_CONVERTER`, merged from `ZH_HANS_TABLE` and `ZH_SG_TABLE`
    pub static ref ZH_HANS_SG_TABLE: (&'static str, &'static str) =
        merge_tables_leaked(ZH_SG_TABLE, ZH_HANS_TABLE);
    /// For `ZH_TO_MY_CONVERTER`, merged from `ZH_HANS_TABLE` and `ZH_MY_TABLE`
    pub static ref ZH_HANS_MY_TABLE: (&'static str, &'static str) =
        merge_tables_leaked(ZH_MY_TABLE, ZH_HANS_TABLE);
}

// TODO: How to make these lazy consts more idiomatic?

/// Merge two conversion table and leak the merged string.
fn merge_tables_leaked(conv1: (&str, &str), conv2: (&str, &str)) -> (&'static str, &'static str) {
    let (froms, tos) = merge_tables(conv1, conv2);
    (
        Box::leak(froms.into_boxed_str()),
        Box::leak(tos.into_boxed_str()),
    )
}

/// Merge two conversion table.
pub fn merge_tables(conv1: (&str, &str), conv2: (&str, &str)) -> (String, String) {
    let mut froms = String::with_capacity(conv1.0.len() + conv2.0.len());
    let mut tos = String::with_capacity(conv1.1.len() + conv2.1.len());
    // merge_by detains the first occurrence
    let mut it = itertools::Itertools::merge_by(
        itertools::zip(conv1.0.trim().split('|'), conv1.1.trim().split('|')),
        itertools::zip(conv2.0.trim().split('|'), conv2.1.trim().split('|')),
        |pair1, pair2| pair1.0.len() >= pair2.0.len(),
    )
    .peekable();
    while let Some((from, to)) = it.next() {
        froms.push_str(from);
        tos.push_str(to);
        if it.peek().is_some() {
            froms.push('|');
            tos.push('|');
        }
    }
    (froms, tos)
}

/// Helper function to build a `ZhConverter` from a conversion table.
///
/// It is just a simple wrapper around [`ZhConverterBuilder`](crate::ZhConverterBuilder).
pub fn build_converter(variant: Variant, table: (&str, &str)) -> ZhConverter {
    ZhConverterBuilder::new()
        .target(variant)
        .table(table)
        .dfa(true)
        .build()
}

// https://github.com/wikimedia/mediawiki/blob/6eda8891a0595e72e350998b6bada19d102a42d9/includes/language/converters/ZhConverter.php#L144
