use serde::de::{self, Deserialize, Deserializer, Unexpected};

// src: https://github.com/serde-rs/serde/issues/1344#issuecomment-410309140
pub fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match u8::deserialize(deserializer)? {
        0 => Ok(false),
        1 => Ok(true),
        other => Err(de::Error::invalid_value(
            Unexpected::Unsigned(other as u64),
            &"zero or one",
        )),
    }
}
