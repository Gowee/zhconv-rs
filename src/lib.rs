//! This crate provides a ZhConverter that converts Chinese variants among each other. The
//! implementation is based on the [Aho-Corasick](https://docs.rs/aho-corasick/latest) automaton
//! with the leftmost-longest matching strategy and linear time complexity with respest to the
//! length of input text and conversion rules. It ships with a bunch of conversion tables,
//! extracted from [zhConversion.php](https://phabricator.wikimedia.org/source/mediawiki/browse/master/includes/languages/data/ZhConversion.php)
//! which is maintained and used by MediaWiki and Chinese Wikipedia.
//!
//! The converter is never meant to be 100% accurate. In Chinese Wikipedia, it is pretty common
//! for editors to apply additional [CGroups](https://zh.wikipedia.org/wiki/Module:CGroup) and
//! [manual conversion rules](https://zh.wikipedia.org/wiki/Help:%E9%AB%98%E7%BA%A7%E5%AD%97%E8%AF%8D%E8%BD%AC%E6%8D%A2%E8%AF%AD%E6%B3%95)
//! on the page base. For completeness, the converter also optionally supports the conversion rule
//! syntax used in MediaWiki in the form `-{FOO BAR}-` and loading external rules defined line by
//! line, which are typically extracted and pre-processed from a [CGroup](https://zh.wikipedia.org/wiki/Category:%E5%85%AC%E5%85%B1%E8%BD%AC%E6%8D%A2%E7%BB%84%E6%A8%A1%E5%9D%97)
//! for a specific topic.
//!
//! # Usage
//! This crate TODO: will be [on crates.io](https://crates.io/crates/regex).
//! ```toml
//! [dependencies]
//! zhconv = "0.2"
//! ```
//!
//! # Example
//!
//! Basic conversion:
//! ```
//! use zhconv::{zhconv, Variant};
//! assert_eq!(zhconv("天干物燥 小心火烛", Variant::ZhHant), "天乾物燥 小心火燭");
//! assert_eq!(zhconv("鼠曲草", Variant::ZhHant), "鼠麴草");
//! assert_eq!(zhconv("阿拉伯联合酋长国", Variant::ZhHant), "阿拉伯聯合酋長國");
//! assert_eq!(zhconv("阿拉伯联合酋长国", Variant::ZhTW), "阿拉伯聯合大公國");
//! ```
//!
//! With MediaWiki conversion rule:
//! ```
//! use zhconv::{zhconv_mw, Variant};
//! assert_eq!(zhconv_mw("天-{干}-物燥 小心火烛", "zh-Hant".parse::<Variant>().unwrap()), "天干物燥 小心火燭");
//! assert_eq!(zhconv_mw("-{zh-tw:鼠麴草;zh-cn:香茅}-是菊科草本植物。", Variant::ZhCN), "香茅是菊科草本植物。");
//! assert_eq!(zhconv_mw("菊科草本植物包括-{zh-tw:鼠麴草;zh-cn:香茅;}-等。", Variant::ZhTW), "菊科草本植物包括鼠麴草等。");
//! assert_eq!(zhconv_mw("-{H|zh:馬;zh-cn:鹿;}-馬克思主義", Variant::ZhCN), "鹿克思主义"); // global rule
//! ```

use std::str::FromStr;

use lazy_static::lazy_static;

mod converter;
pub mod pagerules;
pub mod rule;
pub mod tables;
mod utils;
pub mod variant;

use self::utils::for_wasm;

for_wasm! {
    mod wasm;
}

pub use self::converter::{ZhConverter, ZhConverterBuilder};
pub use self::variant::Variant;

lazy_static! {
    #[allow(non_upper_case_globals)]
    /// Placeholding converter (`zh`/原文). Nothing will be converted with this.
    pub static ref ZH_BLANK_CONVERTER: ZhConverter = tables::build_converter(Variant::Zh, ("", ""));
    /// Zh2Hant converter (`zh-Hant`/繁體中文), lazily built from [`ZH_HANT_TABLE`](crate::tables::ZH_HANT_TABLE).
    pub static ref ZH_TO_HANT_CONVERTER: ZhConverter = tables::build_converter(Variant::ZhHant, tables::ZH_HANT_TABLE);
    /// Zh2Hans converter (`zh-Hans`/简体中文), lazily built from [`ZH_HANS_TABLE`](crate::tables::ZH_HANS_TABLE).
    pub static ref ZH_TO_HANS_CONVERTER: ZhConverter = tables::build_converter(Variant::ZhHans, tables::ZH_HANS_TABLE);
    /// Zh2TW converter (`zh-Hant-TW`/臺灣正體), lazily built from [`ZH_HANT_TW_TABLE`](crate::tables::ZH_HANT_TW_TABLE).
    pub static ref ZH_TO_TW_CONVERTER: ZhConverter = tables::build_converter(Variant::ZhTW, *tables::ZH_HANT_TW_TABLE);
    /// Zh2HK converter (`zh-Hant-HK`/香港繁體), lazily built from [`ZH_HANT_HK_TABLE`](crate::tables::ZH_HANT_HK_TABLE).
    pub static ref ZH_TO_HK_CONVERTER: ZhConverter = tables::build_converter(Variant::ZhHK, *tables::ZH_HANT_HK_TABLE);
    /// Zh2MO converter (`zh-Hant-MO`/澳門繁體), lazily built from [`ZH_HANT_MO_TABLE`](crate::tables::ZH_HANT_MO_TABLE).
    pub static ref ZH_TO_MO_CONVERTER: ZhConverter = tables::build_converter(Variant::ZhMO, *tables::ZH_HANT_MO_TABLE);
    /// Zh2CN converter (`zh-Hans-CN`/大陆简体), lazily built from [`ZH_HANS_CN_TABLE`](crate::tables::ZH_HANS_CN_TABLE).
    pub static ref ZH_TO_CN_CONVERTER: ZhConverter = tables::build_converter(Variant::ZhCN, *tables::ZH_HANS_CN_TABLE);
    /// Zh2SG converter (`zh-Hans-SG`/新加坡简体), lazily built from [`ZH_HANS_SG_TABLE`](crate::tables::ZH_HANS_SG_TABLE).
    pub static ref ZH_TO_SG_CONVERTER: ZhConverter = tables::build_converter(Variant::ZhSG, *tables::ZH_HANS_SG_TABLE);
    /// Zh2MY converter (`zh-Hans-MY`/大马简体), lazily built from [`ZH_HANS_MY_TABLE`](crate::tables::ZH_HANS_MY_TABLE).
    pub static ref ZH_TO_MY_CONVERTER: ZhConverter = tables::build_converter(Variant::ZhMY, *tables::ZH_HANS_MY_TABLE);
}

/// Helper function for general conversion.
///
/// For fine-grained control and custom conversion rules, these is [`ZhConverter`].
#[inline(always)]
pub fn zhconv(text: &str, target: Variant) -> String {
    get_builtin_converter(target).convert(text)
}

/// Helper function for general conversion, supporting conversion rules in MediaWiki syntax.
///
/// # Note
/// Different from the implementation of MediaWiki, this crate use a automaton which makes it
/// impossible to mutate global rules during converting. So the function always searches the text
/// for global rules such as `-{H|FOO BAR}-` in the first pass. If such rules exists, it build a
/// new converter from the scratch with built-in conversion tables, which **takes extra time**.
/// Otherwise, it just picks a built-in converter. Then it converts the text with the chosen
/// converter during when non-global rules are parsed and applied.
///
/// For fine-grained control and custom conversion rules, these is [`ZhConverter`].
pub fn zhconv_mw(text: &str, target: Variant) -> String {
    use crate::pagerules::PageRules;
    let page_rules = PageRules::from_str(text).expect("Page rules parsing in infallible for now");
    if page_rules.as_conv_actions().is_empty() {
        // if there is no global rules specified inline, just pick the built-in converter
        return get_builtin_converter(target).convert_allowing_inline_rules(text);
    }
    // O.W. we have to build a new converter
    let base = get_builtin_table(target);
    ZhConverterBuilder::new()
        .target(target)
        .table(base)
        .page_rules(&page_rules)
        // .conv_lines("zh-cn:人工智能; zh-hk:人工智能; zh-tw:人工智慧;\nzh:訪問; zh-cn:访问; zh-tw:存取;\nzh-cn:访问控制表;zh-tw:存取控制串列\nzh-cn:接入点;\n")
        .build()
        .convert_allowing_inline_rules(text)
}

/// Get the builtin converter for a target Chinese variant.
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

/// Get the builtin conversion table for a target Chinese variant.
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
