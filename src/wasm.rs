use std::str::FromStr;

use console_error_panic_hook;
use wasm_bindgen::prelude::*;

use crate::{get_builtin_table, Variant, ZhConverterBuilder};

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
