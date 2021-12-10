use std::str::FromStr;

use lazy_static::lazy_static;

mod converter;
pub mod convs;
mod pagerules;
mod rule;
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
// include!(concat!(env!("OUT_DIR"), "/convs.rs"));

lazy_static! {
    #[allow(non_upper_case_globals)]
    pub static ref ZhBlankConverter: ZhConverter = convs::build_converter(("", ""));
    pub static ref Zh2HantConverter: ZhConverter = convs::build_converter(convs::ZH_HANT_CONV);
    pub static ref Zh2HansConverter: ZhConverter = convs::build_converter(convs::ZH_HANS_CONV);
    pub static ref Zh2TWConverter: ZhConverter = convs::build_converter(*convs::ZH_HANT_TW_CONV);
    pub static ref Zh2HKConverter: ZhConverter = convs::build_converter(*convs::ZH_HANT_HK_CONV);
    pub static ref Zh2MOConverter: ZhConverter = convs::build_converter(*convs::ZH_HANT_MO_CONV);
    pub static ref Zh2CNConverter: ZhConverter = convs::build_converter(*convs::ZH_HANS_CN_CONV);
    pub static ref Zh2SGConverter: ZhConverter = convs::build_converter(*convs::ZH_HANS_SG_CONV);
    pub static ref Zh2MYConverter: ZhConverter = convs::build_converter(*convs::ZH_HANS_MY_CONV);
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
        Zh => &*ZhBlankConverter,
        ZhHant => &*Zh2HantConverter,
        ZhHans => &*Zh2HansConverter,
        ZhTW => &*Zh2TWConverter,
        ZhHK => &*Zh2HKConverter,
        ZhMO => &*Zh2MOConverter,
        ZhCN => &*Zh2CNConverter,
        ZhMY => &*Zh2MYConverter,
        ZhSG => &*Zh2SGConverter,
    }
}

/// Get the builtin conversion table for a given target variant
///
/// Accessing a table is only necessary when building a custom converter.
/// Otherwise, there is [`get_builtin_converter`].
#[inline(always)]
pub fn get_builtin_table(target: Variant) -> (&'static str, &'static str) {
    use convs::*;
    use Variant::*;
    match target {
        Zh => ("", ""),
        ZhHant => ZH_HANT_CONV,
        ZhHans => ZH_HANS_CONV,
        ZhTW => *ZH_HANT_TW_CONV,
        ZhHK => *ZH_HANT_HK_CONV,
        ZhMO => *ZH_HANT_MO_CONV,
        ZhCN => *ZH_HANS_CN_CONV,
        ZhMY => *ZH_HANS_MY_CONV,
        ZhSG => *ZH_HANS_SG_CONV,
    }
}
