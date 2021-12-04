use itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

use super::ZhConverter;

pub const ZH_HANT_CONV: (&'static str, &'static str) = (
    include_str!(concat!(env!("OUT_DIR"), "/zh2Hant.from.conv")),
    include_str!(concat!(env!("OUT_DIR"), "/zh2Hant.to.conv")),
);
pub const ZH_HANS_CONV: (&'static str, &'static str) = (
    include_str!(concat!(env!("OUT_DIR"), "/zh2Hans.from.conv")),
    include_str!(concat!(env!("OUT_DIR"), "/zh2Hans.to.conv")),
);
pub const ZH_TW_CONV: (&'static str, &'static str) = (
    include_str!(concat!(env!("OUT_DIR"), "/zh2TW.from.conv")),
    include_str!(concat!(env!("OUT_DIR"), "/zh2TW.to.conv")),
);
pub const ZH_HK_CONV: (&'static str, &'static str) = (
    include_str!(concat!(env!("OUT_DIR"), "/zh2HK.from.conv")),
    include_str!(concat!(env!("OUT_DIR"), "/zh2HK.to.conv")),
);
pub const ZH_CN_CONV: (&'static str, &'static str) = (
    include_str!(concat!(env!("OUT_DIR"), "/zh2CN.from.conv")),
    include_str!(concat!(env!("OUT_DIR"), "/zh2CN.to.conv")),
);

// pub const ZH_HANT_TO: &str = include_str!(concat!(env!("OUT_DIR"), "/zh2Hant.to.conv"));

// pub const ZH_HANS_FROM: &str = include_str!(concat!(env!("OUT_DIR"), "/zh2Hans.from.conv"));
// pub const ZH_HANS_TO: &str = include_str!(concat!(env!("OUT_DIR"), "/zh2Hans.to.conv"));

// pub const ZH_TW_FROM: &str = include_str!(concat!(env!("OUT_DIR"), "/zh2TW.from.conv"));
// pub const ZH_TW_TO: &str = include_str!(concat!(env!("OUT_DIR"), "/zh2TW.to.conv"));

// pub const ZH_HK_FROM: &str = include_str!(concat!(env!("OUT_DIR"), "/zh2HK.from.conv"));
// pub const ZH_HK_TO: &str = include_str!(concat!(env!("OUT_DIR"), "/zh2HK.to.conv"));

// pub const ZH_CN_FROM: &str = include_str!(concat!(env!("OUT_DIR"), "/zh2CN.from.conv"));
// pub const ZH_CN_TO: &str = include_str!(concat!(env!("OUT_DIR"), "/zh2CN.to.conv"));

pub fn make_converter((froms, tos): (&str, &str)) -> ZhConverter {
    // dbg!(froms, tos);
    let p = Regex::new(froms).unwrap();
    let m: HashMap<String, String> = itertools::zip(froms.trim().split("|"), tos.trim().split("|"))
        .map(|(a, b)| (a.to_owned(), b.to_owned()))
        .collect();
    // dbg!(&p,&m);
    ZhConverter {
        regex: p,
        mapping: m,
    }
}

// https://github.com/wikimedia/mediawiki/blob/6eda8891a0595e72e350998b6bada19d102a42d9/includes/language/converters/ZhConverter.php#L144
