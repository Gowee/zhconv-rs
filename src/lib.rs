//! zhconv-rs converts Chinese between Traditional, Simplified and regional variants, using
//! rulesets sourced from [zhConversion.php](https://github.com/wikimedia/mediawiki/blob/master/includes/Languages/Data/ZhConversion.php)
//! by MediaWiki and Chinese Wikipedia and [OpenCC](https://github.com/BYVoid/OpenCC/tree/master/data),
//! which are merged, flattened and then precompiled into [Aho-Corasick](https://en.wikipedia.org/wiki/Aho–Corasick_algorithm)
//! automata by [daachorse](https://github.com/daac-tools/daachorse) for single-pass, linear-time
//! conversions.
//!
//! The non-default feature `opencc` enables additional OpenCC dictionaries. Unlike other
//! implementations, dictionaries cannot be chosen (enabled or disabled partly) at runtime since
//! they are merged and precompiled into separate automata for each target variant.  
//!
//! As with MediaWiki and OpenCC, the accuracy is generally acceptable while limited.
//! The converter optionally supports additional conversion rules in MediaWiki syntax (refer to [conversion groups](https://zh.wikipedia.org/wiki/Module:CGroup)
//! and [manual conversion rules](https://zh.wikipedia.org/wiki/Help:高级字词转换语法) on Chinese
//! Wikipedia), external rules defined line by line, and custom conversions defined by `(FROM, TO)`
//! pairs. Prebuilding converter with custom rules or dictionaries is not yet supported.
//!
//! ## Usage
//! The crate is [on crates.io](https://crates.io/crates/zhconv).
//! ```toml
//! [dependencies]
//! zhconv = { version = "?", features = ["opencc"] } # enable additional OpenCC dictionaries
//! ```
//!
//! ## Example
//!
//! Basic conversion:
//! ```
//! use zhconv::{zhconv, Variant};
//! assert_eq!(zhconv("天干物燥 小心火烛", "zh-Hant".parse().unwrap()), "天乾物燥 小心火燭");
//! assert_eq!(zhconv("鼠曲草", Variant::ZhHant), "鼠麴草");
//! assert_eq!(zhconv("阿拉伯联合酋长国", Variant::ZhHant), "阿拉伯聯合酋長國");
//! assert_eq!(zhconv("阿拉伯联合酋长国", Variant::ZhTW), "阿拉伯聯合大公國");
//! ```
//!
//! With MediaWiki conversion syntax:
//! ```
//! use zhconv::{zhconv_mw, Variant};
//! assert_eq!(zhconv_mw("天-{干}-物燥 小心火烛", "zh-Hant".parse::<Variant>().unwrap()), "天干物燥 小心火燭");
//! assert_eq!(zhconv_mw("-{zh-tw:鼠麴草;zh-cn:香茅}-是菊科草本植物。", Variant::ZhCN), "香茅是菊科草本植物。");
//! assert_eq!(zhconv_mw("菊科草本植物包括-{zh-tw:鼠麴草;zh-cn:香茅;}-等。", Variant::ZhTW), "菊科草本植物包括鼠麴草等。");
//! ```
//! Set global rules inline (note that such rules always apply globally regardless of their
//! location, unlike in MediaWiki where they affect only the text that follows):
//! ```
//! use zhconv::{zhconv_mw, Variant};
//! assert_eq!(zhconv_mw("-{H|zh:馬;zh-cn:鹿;}-馬克思主義", Variant::ZhCN), "鹿克思主义"); // add
//! assert_eq!(zhconv_mw("&二極體\n-{-|zh-hans:二极管; zh-hant:二極體}-\n", Variant::ZhCN), "&二极体\n\n"); // remove
//! ```
//!
//! To load or add additional conversion rules such as CGroups or `(FROM, TO)` pairs,
//! see [`ZhConverterBuilder`].
//!
//! Other useful function:
//! ```
//! use zhconv::{is_hans, is_hans_confidence, infer_variant, infer_variant_confidence};
//! assert!(!is_hans("秋冬濁而春夏清，晞於朝而生於夕"));
//! assert!(is_hans_confidence("滴瀝明花苑，葳蕤泫竹叢") < 0.5);
//! println!("{}", infer_variant("錦字緘愁過薊水，寒衣將淚到遼城"));
//! println!("{:?}", infer_variant_confidence("zhconv-rs 中文简繁及地區詞轉換"));
//! ```

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
pub use self::converters::get_builtin_converter;
use self::converters::*;
pub use self::tables::get_builtin_tables;
pub use self::variant::Variant;

/// Helper function for general conversion using built-in converters.
///
/// Built-in converters are pre-built, lazily loaded and cached for later use. For fine-grained
/// control and custom conversion rules, check [`ZhConverter`] and [`ZhConverterBuilder`].
#[inline(always)]
pub fn zhconv(text: &str, target: Variant) -> String {
    get_builtin_converter(target).convert(text)
}

/// Helper function for general conversion, activating wikitext support.
///
/// It function share the same built-in conversion converters as [`zhconv`](#method.zhconv), but
/// additionally supports conversion rules in MediaWiki syntax.
///
/// # Note
/// The implementation scans the input text at first to extract possible global rules like
/// `-{H|FOO BAR}-`.
/// If there are no global rules, the overall time complexity is `O(n + n)`.
/// Otherwise, the overall time complexity may degrade to `O(n + n * m)` in the worst case, where
/// `n` is input text length and `m` is the maximum lengths of source words in conversion rulesets.
/// In case global rules support are not expected, it is better to use
/// `get_builtin_converter(target).convert_as_wikitext_basic(text)` instead, which incurs no extra
/// overhead.
///
// /// Different from the implementation of MediaWiki, this crate use a automaton which makes it
// /// infeasible to mutate global rules during converting. So the function always searches the text
// /// for global rules such as `-{H|FOO BAR}-` in the first pass. If such rules exists, it build a
// /// new converter from the scratch with built-in conversion tables, which **takes extra time**.
// /// Otherwise, it just picks a built-in converter. Then it converts the text with the chosen
// /// converter during when non-global rules are parsed and applied.
///
/// For fine-grained control and custom conversion rules, check [`ZhConverterBuilder`].
///
/// Although it is designed to replicate the behavior of the MediaWiki implementation, it is not
/// fully compliant.
pub fn zhconv_mw(text: &str, target: Variant) -> String {
    get_builtin_converter(target).convert_as_wikitext_extended(text)
}

/// Determine whether the given text looks like Simplified Chinese over Traditional Chinese.
///
/// Equivalent to `is_hans_confidence(text) > 0.5`.
pub fn is_hans(text: &str) -> bool {
    is_hans_confidence(text) > 0.5
}

/// Determine whether the given text looks like Simplified Chinese over Traditional Chinese.
///
/// The return value is a real number in the range `[0, 1]` (inclusive) that indicates
/// confidence level. A value close to 1 indicate high confidence. A value close to 0
/// indicates low confidence. `0.5` indicates undeterminable (half-half).
/// If there is no enough input, `NaN` is returned.
pub fn is_hans_confidence(text: &str) -> f32 {
    let non_hant_score = ZH_TO_HANT_CONVERTER.count_replaced(text) as f32;
    let non_hans_score = ZH_TO_HANS_CONVERTER.count_replaced(text) as f32;
    // let mut ratio = if non_hans_score == 0 {
    //     f32::MAX
    // } else {
    //     non_hant_score as f32 / non_hans_score as f32
    // } - 1.0;
    // if ratio < 0.0 {
    //     ratio = -(1.0 / (ratio + 1.0) - 1.0);
    // }
    // 1f32 / (1f32 + E.powf(-ratio))
    non_hant_score / (non_hans_score + non_hant_score)
}

/// Determine the Chinese variant of the input text.
///
/// # Limitations
/// Since the built-in conversion tables does not have actual rules specific to `zh-SG` / `zh-MO` /
/// `zh-MY`, they would never be returned.
///
/// The accuracy has not been assessed. Avoid relying on this for serious purposes.
pub fn infer_variant(text: &str) -> Variant {
    // let non_cn_score = ZH_TO_CN_CONVERTER.count_replaced(text);
    // let non_tw_score = ZH_TO_TW_CONVERTER.count_replaced(text);
    // let non_hk_score = ZH_TO_HK_CONVERTER.count_replaced(text);

    // // authored by ChatGPT
    // if non_cn_score <= non_tw_score && non_cn_score <= non_hk_score {
    //     Variant::ZhCN
    // } else if non_tw_score <= non_cn_score && non_tw_score <= non_hk_score {
    //     Variant::ZhTW
    // } else {
    //     Variant::ZhHK
    // }
    infer_variant_confidence(text)[0].0
}

/// Determine the Chinese variant of the input text with confidence.
///
/// # Returns
/// An array of `(variant, confidence_level)`, in descendent order of `confidence_level`, where
/// `confidence_level` is in the range `[0, 1]` (inclusive). `NaN` is returned if there is no
/// enough input.
///
/// # Limitations
/// The returned `confidence_level` of script variants (`ZhHant` and `ZhHans`) are always greater
/// than region variants (`ZhTW`, `ZhCN` and `ZhHK`) with the current implementation.
///
/// The accuracy has not been assessed. Avoid relying on this for serious purposes.
// /// Note that, unlike [`is_hans_confidence`](is_hans_confidence), a `confidence_level` greater
// /// than `0.5` might not imply high enough likelihood.
pub fn infer_variant_confidence(text: &str) -> [(Variant, f32); 5] {
    // let total = text.len() as f32;
    let non_cn_score = ZH_TO_CN_CONVERTER.count_replaced(text) as f32;
    let non_tw_score = ZH_TO_TW_CONVERTER.count_replaced(text) as f32;
    let non_hk_score = ZH_TO_HK_CONVERTER.count_replaced(text) as f32;
    let non_hant_score = ZH_TO_HANT_CONVERTER.count_replaced(text) as f32;
    let non_hans_score = ZH_TO_HANS_CONVERTER.count_replaced(text) as f32;

    let total_score = non_cn_score + non_tw_score + non_hk_score - non_hant_score;
    // let region_total = non_cn_score + non_tw_score + non_hk_score - non_hant_score;
    // let script_total = non_hant_score + non_hans_score;
    let hans = (
        Variant::ZhHans,
        1f32 - non_hans_score.min(total_score) / total_score,
    );
    let hant = (
        Variant::ZhHant,
        1f32 - non_hant_score.min(total_score) / total_score,
    );
    let tw = (
        Variant::ZhTW,
        1f32 - non_tw_score.min(total_score) / total_score,
    );
    let cn = (
        Variant::ZhCN,
        1f32 - non_cn_score.min(total_score) / total_score,
    );
    let hk = (
        Variant::ZhHK,
        1f32 - non_hk_score.min(total_score) / total_score,
    );
    // if hk and tw cannot be distinguished, we prefer hant
    // we always prefer hans over cn, since we cannot really distinguish cn from hans with the
    // current implementation
    let mut confidence_map = if tw.1 == hk.1 {
        [hans, hant, tw, cn, hk]
    } else {
        [tw, hk, hant, hans, cn]
    };
    // let mut confidence_map = [(Variant::ZhCN, 1f32 - non_cn_score / region_total),(Variant::ZhTW, 1f32 - non_tw_score / region_total),(Variant::ZhHK, 1f32 - non_hk_score / region_total),(Variant::ZhHans,1f32 - non_hans_score / script_total),(Variant::ZhHant, 1f32 - non_hant_score / script_total)];
    // let mut confidence_map = [(Variant::ZhCN, non_cn_score),(Variant::ZhTW, non_tw_score),(Variant::ZhHK, non_hk_score),(Variant::ZhHans,non_hans_score),(Variant::ZhHant, non_hant_score), (Variant::Zh, total)];

    // let mut confidence_map = [
    //     (Variant::ZhCN, 1f32 - non_cn_score / total),
    //     (Variant::ZhTW, 1f32 - non_tw_score / total),
    //     (Variant::ZhHK, 1f32 - non_hk_score / total),
    //     (Variant::ZhHans, 1f32 - non_hans_score / total),
    //     (Variant::ZhHant, 1f32 - non_hant_score / total),
    // ];
    confidence_map.sort_by(|a, b| b.1.total_cmp(&a.1));
    confidence_map
}

/// A helper trait that truncates a str around a specified index in constant time (`O(1)`),
/// intended to be used with `is_hans` and etc.
pub trait TruncatedAround {
    /// Truncate a str around the given index in constant time (`O(1)`).
    ///
    /// This method is intended to be used together with other functions like `is_hans` and
    /// `infer_variant`, especially when dealing with large input texts that need to be processed
    /// efficiently while tolerating less accuracy.
    /// Note that this trait does not guarantee whether the truncation index is rounded up or down.
    ///
    /// # Examples
    ///
    /// ```
    /// use zhconv::{TruncatedAround, is_hans};
    /// use std::fs;
    ///
    /// let s = "鵲飛空繞樹月輪殊未圓";
    /// assert_eq!(s.len(), 30);
    /// assert_eq!(s.truncated_around(15), "鵲飛空繞樹");
    /// assert_eq!(s.truncated_around(100), s);
    ///
    /// let ls = fs::read_to_string("benches/data3185k.txt").unwrap(); // long string
    /// let tls = ls.truncated_around(100 * 1024 + 123); // truncated to ~ 100KiB
    /// assert_eq!(is_hans(&ls), is_hans(&tls));
    /// ```
    fn truncated_around(&self, index: usize) -> &Self;
}

impl TruncatedAround for str {
    fn truncated_around(&self, index: usize) -> &Self {
        // Ref: std::str::ceil_char_boundary
        if index > self.len() {
            self
        } else {
            let upper_bound = Ord::min(index + 4, self.len());
            for end in index..upper_bound {
                if self.is_char_boundary(end) {
                    return &self[..end];
                }
            }
            unreachable!()
        }
    }
}
