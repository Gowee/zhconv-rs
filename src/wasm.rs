use std::str::FromStr;
use std::panic;

use console_error_panic_hook;
// use lru::LruCache;
use once_cell::unsync::Lazy;
use wasm_bindgen::prelude::*;

use crate::{get_builtin_table, Variant, ZhConverterBuilder};

// const BUILDER_CACHE_SIZE: usize = 3;
// NOTE: out current design of ZhConverter make it meaningless to cache since automaton is built
//       from page inline rules
// pub fn build_converter(target: Variant, cgroup: &str) ->  {
//     let cache = Lazy::new(|| LruCache::new(BUILDER_CACHE_SIZE));
//     if let Some(cached) = cache.get(&(target, cgroup)) {
//         cached
//     }
//     let cache = Lazy::new(|| LruCache::new(BUILDER_CACHE_SIZE));
//     if !cache.contains(&(target, cgroup)) {
//         cache.put((target, cgroup), ZhConverterBuilder::new()
//                 .target(target)
//                 .table(get_builtin_table(target))
//                 .page_rules(&rules)
//                 .dfa(false)
//                 .build()
//                 .convert(text))
//     }
// }

#[wasm_bindgen]
pub fn zhconv(text: &str, target: &str, mediawiki: bool, cgroup: &str) -> String {
    console_error_panic_hook::set_once();

    let target = Variant::from_str(target).expect("Unsupported target variant");
    // let mediawiki = mediawiki.unwrap_or(false);
    // let cgroup = cgroup.unwrap_or("");
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

// #[wasm_bindgen]
// pub fn zhconv_mw(text: &str, target: &str) -> String {
//     let target = Variant::from_str(target).expect("Unsupported target variant");
//     crate::zhconv_mw(text, target)
// }
