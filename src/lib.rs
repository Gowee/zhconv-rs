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
//! on a specific topic.
//!
//! # Usage
//! This crate is [on crates.io](https://crates.io/crates/zhconv).
//! ```toml
//! [dependencies]
//! zhconv = "0.1"
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
//! With MediaWiki conversion rules:
//! ```
//! use zhconv::{zhconv_mw, Variant};
//! assert_eq!(zhconv_mw("天-{干}-物燥 小心火烛", "zh-Hant".parse::<Variant>().unwrap()), "天干物燥 小心火燭");
//! assert_eq!(zhconv_mw("-{zh-tw:鼠麴草;zh-cn:香茅}-是菊科草本植物。", Variant::ZhCN), "香茅是菊科草本植物。");
//! assert_eq!(zhconv_mw("菊科草本植物包括-{zh-tw:鼠麴草;zh-cn:香茅;}-等。", Variant::ZhTW), "菊科草本植物包括鼠麴草等。");
//! assert_eq!(zhconv_mw("-{H|zh:馬;zh-cn:鹿;}-馬克思主義", Variant::ZhCN), "鹿克思主义"); // global rule
//! ```
//!
//! To load or add additional conversion rules such as CGroup, see [`ZhConverterBuilder`].
//!

use std::str::FromStr;

mod converter;
mod utils;

pub mod converters;
pub mod tables;

pub mod pagerules;
pub mod rule;
pub mod variant;

use self::utils::for_wasm;

for_wasm! {
    mod wasm;
}

pub use self::converter::{ZhConverter, ZhConverterBuilder};
pub use self::converters::{get_builtin_converter, get_builtin_table};
pub use self::variant::Variant;

/// Helper function for general conversion.
///
/// For fine-grained control and custom conversion rules, these is [`ZhConverter`].
#[inline(always)]
pub fn zhconv(text: &str, target: Variant) -> String {
    get_builtin_converter(target).convert(text)
}

/// Helper function for general conversion, activating inline conversion rules in MediaWiki syntax.
///
/// For general cases, [`zhconv`](#method.zhconv) should work well. Both of them share the same
/// built-in conversions tables.
///
/// # Note
/// Different from the implementation of MediaWiki, this crate use a automaton which makes it
/// infeasible to mutate global rules during converting. So the function always searches the text
/// for global rules such as `-{H|FOO BAR}-` in the first pass. If such rules exists, it build a
/// new converter from the scratch with built-in conversion tables, which **takes extra time**.
/// Otherwise, it just picks a built-in converter. Then it converts the text with the chosen
/// converter during when non-global rules are parsed and applied.
///
/// For fine-grained control and custom conversion rules, these is [`ZhConverter`].
pub fn zhconv_mw(text: &str, target: Variant) -> String {
    use crate::pagerules::PageRules;
    let page_rules = PageRules::from_str(text).expect("Page rules parsing is infallible for now");
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
        .build()
        .convert_allowing_inline_rules(text)
}
