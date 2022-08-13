use std::str::FromStr;

use console_error_panic_hook;
use wasm_bindgen::prelude::*;

use crate::{get_builtin_table, Variant, ZhConverterBuilder};

// #[wasm_bindgen(typescript_custom_section)]
// const COMMIT_HASH: &str = concat!("COMMIT_HASH", env!("VERGEN_GIT_SHA"));
// #[wasm_bindgen(typescript_custom_section)]
// const BUILD_TIMESTAMP: &str = env!("VERGEN_BUILD_TIMESTAMP");
// #[wasm_bindgen(typescript_custom_section)]
// const MEDIAWIKI_COMMIT_HASH: &str = env!("MEDIAWIKI_COMMIT_HASH");

// #[allow(dead_code)]
// #[wasm_bindgen]
// pub struct BuildInfo {
//     timestamp: &'static str,
//     commit_hash: &'static str,
//     mediawiki_commit_hash: &'static str,
// }

#[wasm_bindgen]
pub fn get_build_timestamp() -> Option<String> {
    option_env!("VERGEN_BUILD_TIMESTAMP").map(|s| s.into())
}
#[wasm_bindgen]
pub fn get_commit() -> Option<String> {
    option_env!("VERGEN_GIT_SHA").map(|s| s.into())
}
#[wasm_bindgen]
pub fn get_mediawiki_commit() -> String {
    env!("MEDIAWIKI_COMMIT_HASH").into()
}

/// Convert a text to a target Chinese variant.
///
/// Supported target variants: zh, zh-Hant, zh-Hans, zh-TW, zh-HK, zh-MO, zh-CN, zh-SG, zh-MY.
/// If `mediawiki` is `True`, inline conversion rules such as `-{foo...bar}-` are parsed.
/// `rules` should be line-seperated in mediawiki syntax without -{ or }- tags, such as
/// `zh-hans:鹿; zh-hant:馬`.
#[wasm_bindgen]
pub fn zhconv(text: &str, target: &str, mediawiki: Option<bool>, rules: Option<String>) -> String {
    console_error_panic_hook::set_once();

    let mediawiki = mediawiki.unwrap_or(false);
    let rules = rules.unwrap_or(String::from(""));
    let target = Variant::from_str(target).expect("Unsupported target variant");
    match (mediawiki, !rules.is_empty()) {
        (false, false) => crate::zhconv(text, target),
        (true, false) => crate::zhconv_mw(text, target),
        (false, true) => ZhConverterBuilder::new()
            .target(target)
            .table(get_builtin_table(target))
            .conv_lines(&rules)
            .dfa(false)
            .build()
            .convert(text),
        (true, true) => ZhConverterBuilder::new()
            .target(target)
            .table(get_builtin_table(target))
            .conv_lines(&rules)
            .rules_from_page(text)
            .dfa(false)
            .build()
            .convert_allowing_inline_rules(text),
    }
}
