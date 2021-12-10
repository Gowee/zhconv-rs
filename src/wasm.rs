use std::str::FromStr;

use wasm_bindgen::prelude::*;

use crate::{Variant};

#[wasm_bindgen]
pub fn zhconv(text: &str, target: &str) -> String {
    let target = Variant::from_str(target).expect("Unsupported target variant");
    crate::zhconv(text, target)
}

#[wasm_bindgen]
pub fn zhconv_mw(text: &str, target: &str) -> String {
    let target = Variant::from_str(target).expect("Unsupported target variant");
    crate::zhconv_mw(text, target)
}
