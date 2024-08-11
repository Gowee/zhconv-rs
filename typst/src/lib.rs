use std::str::{self, FromStr};

use zhconv::{zhconv as zhconv_plain, zhconv_mw, Variant};
use wasm_minimal_protocol::*;

initiate_protocol!();

#[wasm_func]
pub fn zhconv(text: &[u8], target: &[u8], wikitext_flag: &[u8]) -> Result<Vec<u8>, String> {
    let text = str::from_utf8(text).map_err(|_e| String::from("Invalid text"))?;
    let target = str::from_utf8(target)
        .map_err(|_e| String::from("Invalid target variant"))
        .and_then(|target| {
            Variant::from_str(target)
                .map_err(|_e| format!("Unsupported target variant: {}", target))
        })?;
    let wikitext = wikitext_flag[0] != 0;
    if wikitext {
        Ok(zhconv_mw(text, target).into())
    } else {
        Ok(zhconv_plain(text, target).into())
    }
}

#[wasm_func]
pub fn is_hans(text: &[u8]) -> Result<Vec<u8>, String> {
    let text = str::from_utf8(text).map_err(|_e| String::from("Invalid text"))?;
    Ok(vec![zhconv::is_hans(text) as u8])
}
