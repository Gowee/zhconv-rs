use std::str::FromStr;

use itertools::Itertools;

use console_error_panic_hook;
use wasm_bindgen::prelude::*;

use crate::{get_builtin_converter, Variant, ZhConverterBuilder};

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
#[wasm_bindgen]
pub fn get_opencc_commit() -> Option<String> {
    option_env!("OPENCC_COMMIT_HASH").map(|s| s.into())
}

/// Convert a text to a target Chinese variant.
///
/// Supported target variants: zh, zh-Hant, zh-Hans, zh-TW, zh-HK, zh-MO, zh-CN, zh-SG, zh-MY.
/// If `wikitext` is `True`, inline conversion rules such as `-{foo...bar}-` are parsed.
/// `rules` should be line-seperated in MediaWiki syntax without -{ or }- tags, like
/// `zh-hans:鹿; zh-hant:馬`.
#[wasm_bindgen]
pub fn zhconv(text: &str, target: &str, wikitext: Option<bool>, rules: Option<String>) -> String {
    console_error_panic_hook::set_once();

    let wikitext = wikitext.unwrap_or(false);
    let target = Variant::from_str(target).expect("Unsupported target variant");
    let converter = get_builtin_converter(target);
    let mut builder = rules.map(|rs| ZhConverterBuilder::new().conv_lines(rs.lines()));
    if wikitext {
        converter.convert_as_wikitext(text, &mut builder, true, true)
    } else {
        match builder {
            Some(builder) => converter.convert_with_secondary_converter(text, &builder.build()),
            None => converter.convert(text),
        }
    }
}

#[wasm_bindgen]
pub fn is_hans(text: &str) -> bool {
    console_error_panic_hook::set_once();

    crate::is_hans(text)
}

#[wasm_bindgen]
pub fn is_hans_confidence(text: &str) -> f32 {
    console_error_panic_hook::set_once();

    crate::is_hans_confidence(text)
}

#[wasm_bindgen]
pub fn infer_variant(text: &str) -> String {
    console_error_panic_hook::set_once();

    crate::infer_variant(text).to_string()
}

#[wasm_bindgen]
pub fn infer_variant_confidence(text: &str) -> String {
    console_error_panic_hook::set_once();

    crate::infer_variant_confidence(text)
        .into_iter()
        .map(|(v, c)| format!("{};q={:.3}", v, c))
        .join(", ")
}
