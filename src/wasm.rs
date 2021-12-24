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
pub fn get_build_timestamp() -> String {
    env!("VERGEN_BUILD_TIMESTAMP").into()
}
#[wasm_bindgen]
pub fn get_commit() -> String {
    env!("VERGEN_GIT_SHA").into()
}
#[wasm_bindgen]
pub fn get_mediawiki_commit() -> String {
    env!("MEDIAWIKI_COMMIT_HASH").into()
}

#[wasm_bindgen]
pub fn zhconv(text: &str, target: &str, mediawiki: bool, cgroup: &str) -> String {
    console_error_panic_hook::set_once();

    let target = Variant::from_str(target).expect("Unsupported target variant");
    match (mediawiki, !cgroup.is_empty()) {
        (false, false) => crate::zhconv(text, target),
        (true, false) => crate::zhconv_mw(text, target),
        (false, true) => ZhConverterBuilder::new()
            .target(target)
            .table(get_builtin_table(target))
            .conv_lines(cgroup)
            .dfa(false)
            .build()
            .convert(text),
        (true, true) => ZhConverterBuilder::new()
            .target(target)
            .table(get_builtin_table(target))
            .conv_lines(cgroup)
            .rules_from_page(text)
            .dfa(false)
            .build()
            .convert_allowing_inline_rules(text),
    }
}
