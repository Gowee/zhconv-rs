use std::str::{self, FromStr};

#[cfg(target_arch = "wasm32")]
use wasm_minimal_protocol::*;
use zhconv::{zhconv as zhconv_plain, zhconv_mw, Variant};

#[cfg(target_arch = "wasm32")]
initiate_protocol!();

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn zhconv(text: &[u8], target: &[u8], wikitext_flag: &[u8]) -> Result<Vec<u8>, String> {
    let text = str::from_utf8(text).map_err(|_e| String::from("Invalid text"))?;
    let target = str::from_utf8(target)
        .map_err(|_e| String::from("Invalid target variant"))
        .and_then(|target| {
            Variant::from_str(target)
                .map_err(|_e| format!("Unsupported target variant: {}", target))
        })?;
    let wikitext = match wikitext_flag {
        [0] => false,
        [1] => true,
        _ => {
            return Err(String::from(
                "Invalid wikitext flag: expected one byte, 0 or 1",
            ))
        }
    };
    if wikitext {
        Ok(zhconv_mw(text, target).into())
    } else {
        Ok(zhconv_plain(text, target).into())
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_func)]
pub fn is_hans(text: &[u8]) -> Result<Vec<u8>, String> {
    let text = str::from_utf8(text).map_err(|_e| String::from("Invalid text"))?;
    Ok(vec![zhconv::is_hans(text) as u8])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_wire_arguments() {
        assert_eq!(
            zhconv(&[0xff], b"zh-hans", &[0]),
            Err("Invalid text".into())
        );
        assert_eq!(
            zhconv(b"text", &[0xff], &[0]),
            Err("Invalid target variant".into())
        );
        assert!(zhconv(b"text", b"not-a-variant", &[0])
            .unwrap_err()
            .contains("Unsupported target"));
        for flag in [&[][..], &[2], &[0, 1]] {
            assert!(zhconv(b"text", b"zh-hans", flag)
                .unwrap_err()
                .contains("Invalid wikitext flag"));
        }
        assert_eq!(is_hans(&[0xff]), Err("Invalid text".into()));
    }

    #[test]
    fn preserves_utf8_and_argument_boundaries() {
        let text = "汉字\0{field}\n🦀𠀀";
        assert_eq!(
            zhconv(text.as_bytes(), b"ZH-HANT", &[0]).unwrap(),
            "漢字\0{field}\n🦀𠀀".as_bytes()
        );
        assert_eq!(zhconv(b"", b"zh-hans", &[0]).unwrap(), b"");
    }

    #[test]
    fn keeps_wikitext_processing_explicit() {
        let text = "-{zh-hans:甲;zh-hant:乙;}-";
        assert_eq!(
            zhconv(text.as_bytes(), b"zh-hant", &[1]).unwrap(),
            "乙".as_bytes()
        );
        assert_eq!(
            zhconv(text.as_bytes(), b"zh-hant", &[0]).unwrap(),
            text.as_bytes()
        );
    }
}
