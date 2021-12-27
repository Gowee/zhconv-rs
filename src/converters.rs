//! Converters lazily built from built-in [`tables`](crate::tables).
//!
//! These converters are lazily built on demand with [`dfa`](crate::ZhConverterBuilder::dfa)
//! activated for better conversion performance, and cached for later use.

use lazy_static::lazy_static; // TODO: once_cell

use crate::{tables::*, Variant, ZhConverter};

lazy_static! {
    #[allow(non_upper_case_globals)]
    /// Placeholding converter (`zh`/原文). Nothing will be converted with this.
    pub static ref ZH_BLANK_CONVERTER: ZhConverter = build_converter(Variant::Zh, ("", ""));
    /// Zh2Hant converter (`zh-Hant`/繁體中文), lazily built from [`ZH_HANT_TABLE`](crate::ZH_HANT_TABLE).
    pub static ref ZH_TO_HANT_CONVERTER: ZhConverter = build_converter(Variant::ZhHant, ZH_HANT_TABLE);
    /// Zh2Hans converter (`zh-Hans`/简体中文), lazily built from [`ZH_HANS_TABLE`](crate::ZH_HANS_TABLE).
    pub static ref ZH_TO_HANS_CONVERTER: ZhConverter = build_converter(Variant::ZhHans, ZH_HANS_TABLE);
    /// Zh2TW converter (`zh-Hant-TW`/臺灣正體), lazily built from [`ZH_HANT_TW_TABLE`](crate::ZH_HANT_TW_TABLE).
    pub static ref ZH_TO_TW_CONVERTER: ZhConverter = build_converter(Variant::ZhTW, *ZH_HANT_TW_TABLE);
    /// Zh2HK converter (`zh-Hant-HK`/香港繁體), lazily built from [`ZH_HANT_HK_TABLE`](crate::ZH_HANT_HK_TABLE).
    pub static ref ZH_TO_HK_CONVERTER: ZhConverter = build_converter(Variant::ZhHK, *ZH_HANT_HK_TABLE);
    /// Zh2MO converter (`zh-Hant-MO`/澳門繁體), lazily built from [`ZH_HANT_MO_TABLE`](crate::ZH_HANT_MO_TABLE).
    pub static ref ZH_TO_MO_CONVERTER: ZhConverter = build_converter(Variant::ZhMO, *ZH_HANT_MO_TABLE);
    /// Zh2CN converter (`zh-Hans-CN`/大陆简体), lazily built from [`ZH_HANS_CN_TABLE`](crate::ZH_HANS_CN_TABLE).
    pub static ref ZH_TO_CN_CONVERTER: ZhConverter = build_converter(Variant::ZhCN, *ZH_HANS_CN_TABLE);
    /// Zh2SG converter (`zh-Hans-SG`/新加坡简体), lazily built from [`ZH_HANS_SG_TABLE`](crate::ZH_HANS_SG_TABLE).
    pub static ref ZH_TO_SG_CONVERTER: ZhConverter = build_converter(Variant::ZhSG, *ZH_HANS_SG_TABLE);
    /// Zh2MY converter (`zh-Hans-MY`/大马简体), lazily built from [`ZH_HANS_MY_TABLE`](crate::ZH_HANS_MY_TABLE).
    pub static ref ZH_TO_MY_CONVERTER: ZhConverter = build_converter(Variant::ZhMY, *ZH_HANS_MY_TABLE);
}

/// Get the builtin converter for a target Chinese variant.
#[inline(always)]
pub fn get_builtin_converter(target: Variant) -> &'static ZhConverter {
    use Variant::*;
    match target {
        Zh => &*ZH_BLANK_CONVERTER,
        ZhHant => &*ZH_TO_HANT_CONVERTER,
        ZhHans => &*ZH_TO_HANS_CONVERTER,
        ZhTW => &*ZH_TO_TW_CONVERTER,
        ZhHK => &*ZH_TO_HK_CONVERTER,
        ZhMO => &*ZH_TO_MO_CONVERTER,
        ZhCN => &*ZH_TO_CN_CONVERTER,
        ZhMY => &*ZH_TO_MY_CONVERTER,
        ZhSG => &*ZH_TO_SG_CONVERTER,
    }
}

/// Get the builtin conversion table for a target Chinese variant.
///
/// Accessing a table is only necessary when building a custom converter.
/// Otherwise, there is [`get_builtin_converter`].
#[inline(always)]
pub fn get_builtin_table(target: Variant) -> (&'static str, &'static str) {
    use Variant::*;
    match target {
        Zh => ("", ""),
        ZhHant => ZH_HANT_TABLE,
        ZhHans => ZH_HANS_TABLE,
        ZhTW => *ZH_HANT_TW_TABLE,
        ZhHK => *ZH_HANT_HK_TABLE,
        ZhMO => *ZH_HANT_MO_TABLE,
        ZhCN => *ZH_HANS_CN_TABLE,
        ZhMY => *ZH_HANS_MY_TABLE,
        ZhSG => *ZH_HANS_SG_TABLE,
    }
}
