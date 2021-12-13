use std::str::FromStr;

use lazy_static::lazy_static;

mod converter;
mod pagerules;
mod rule;
pub mod tables;
mod utils;
mod variant;

use self::utils::for_wasm;

for_wasm! {
    mod wasm;
}

pub use self::converter::*;
pub use self::pagerules::PageRules;
pub use self::rule::*;
pub use self::variant::Variant;
// include!(concat!(env!("OUT_DIR"), "/tables.rs"));

lazy_static! {
    #[allow(non_upper_case_globals)]
    pub static ref ZH_BLANK_CONVERTER: ZhConverter = tables::build_converter(("", ""));
    pub static ref ZH_TO_HANT_CONVERTER: ZhConverter = tables::build_converter(tables::ZH_HANT_TABLE);
    pub static ref ZH_TO_HANS_CONVERTER: ZhConverter = tables::build_converter(tables::ZH_HANS_TABLE);
    pub static ref ZH_TO_TW_CONVERTER: ZhConverter = tables::build_converter(*tables::ZH_HANT_TW_TABLE);
    pub static ref ZH_TO_HK_CONVERTER: ZhConverter = tables::build_converter(*tables::ZH_HANT_HK_TABLE);
    pub static ref ZH_TO_MO_CONVERTER: ZhConverter = tables::build_converter(*tables::ZH_HANT_MO_TABLE);
    pub static ref ZH_TO_CN_CONVERTER: ZhConverter = tables::build_converter(*tables::ZH_HANS_CN_TABLE);
    pub static ref ZH_TO_SG_CONVERTER: ZhConverter = tables::build_converter(*tables::ZH_HANS_SG_TABLE);
    pub static ref ZH_TO_MY_CONVERTER: ZhConverter = tables::build_converter(*tables::ZH_HANS_MY_TABLE);
}

/// Helper function for general conversion
///
/// For fine-grained control and custom conversion rules, these is [`ZhConverter`].
#[inline(always)]
pub fn zhconv(text: &str, target: Variant) -> String {
    get_builtin_converter(target).convert(text)
}

/// Helper function for general conversion, with Mediawiki conversion rules support
///
/// For fine-grained control and custom conversion rules, these is [`ZhConverter`].
#[inline(always)]
pub fn zhconv_mw(text: &str, target: Variant) -> String {
    let rules = PageRules::from_str(text).unwrap();
    let base = get_builtin_table(target);
    ZhConverterBuilder::new(target)
        .table(base)
        .page_rules(&rules)
        .build()
        .convert_allowing_inline_rules(text)
}

/// Get the builtin converter for a given target variant
#[inline(always)]
pub fn get_builtin_converter(target: Variant) -> &'static ZhConverter {
    use Variant::*;
    match target {
        Zh => &*ZH_BLANK_CONVERTER,
        ZhHant => &*ZH_TO_HANT_CONVERTER,
        ZhHans => &*ZH_TO_HANS_CONVERTER,
        ZhTW => &*ZH_TO_TW_CONVERTER,
        ZhHK => &*ZH_TO_HK_CONVERTER,
        ZhMO => &*ZH_TO_MO_CONVERTER,
        ZhCN => &*ZH_TO_CN_CONVERTER,
        ZhMY => &*ZH_TO_MY_CONVERTER,
        ZhSG => &*ZH_TO_SG_CONVERTER,
    }
}

/// Get the builtin conversion table for a given target variant
///
/// Accessing a table is only necessary when building a custom converter.
/// Otherwise, there is [`get_builtin_converter`].
#[inline(always)]
pub fn get_builtin_table(target: Variant) -> (&'static str, &'static str) {
    use tables::*;
    use Variant::*;
    match target {
        Zh => ("", ""),
        ZhHant => ZH_HANT_TABLE,
        ZhHans => ZH_HANS_TABLE,
        ZhTW => *ZH_HANT_TW_TABLE,
        ZhHK => *ZH_HANT_HK_TABLE,
        ZhMO => *ZH_HANT_MO_TABLE,
        ZhCN => *ZH_HANS_CN_TABLE,
        ZhMY => *ZH_HANS_MY_TABLE,
        ZhSG => *ZH_HANS_SG_TABLE,
    }
}
