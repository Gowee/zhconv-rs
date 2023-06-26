//! Converters lazily built from built-in [`tables`](crate::tables).
//!
//! These converters are lazily built on demand with [`dfa`](crate::ZhConverterBuilder::dfa)
//! activated for better conversion performance, and cached for later use.

use daachorse::CharwiseDoubleArrayAhoCorasick;
use lazy_static::lazy_static; // TODO: once_cell

use crate::{tables::*, utils::zstd_decompress, Variant, ZhConverter, ZhConverterBuilder};

// FIX: Doc

// More specific rules should take precedence when merging (e.g. zh-TW > zh-Hant).
// Since the converter build process relies on HashMap::extend, latter rules overwrite the early ones.
lazy_static! {
    #[allow(non_upper_case_globals)]
    /// Placeholding converter (`zh`/原文). Nothing will be converted with this.
    pub static ref ZH_BLANK_CONVERTER: ZhConverter = ZhConverterBuilder::new().target(Variant::Zh).build();
    /// Zh2Hant converter (`zh-Hant`/繁體中文), lazily built from [`ZH_HANT_TABLE`](crate::ZH_HANT_TABLE).
    pub static ref ZH_TO_HANT_CONVERTER: ZhConverter = deserialize_converter(Variant::ZhHant, ZH_HANT_DAAC, [ZH_HANT_TABLE]);
    /// Zh2Hans converter (`zh-Hans`/简体中文), lazily built from [`ZH_HANS_TABLE`](crate::ZH_HANS_TABLE).
    pub static ref ZH_TO_HANS_CONVERTER: ZhConverter = deserialize_converter(Variant::ZhHans, ZH_HANS_DAAC, [ZH_HANS_TABLE]);
    /// Zh2TW converter (`zh-Hant-TW`/臺灣正體), lazily built from [`ZH_HANT_TABLE`](crate::ZH_HANT_TABLE)
    /// and [`ZH_TW_TABLE`](crate::ZH_TW_TABLE).
    pub static ref ZH_TO_TW_CONVERTER: ZhConverter = deserialize_converter(Variant::ZhTW, ZH_HANT_TW_DAAC, [ZH_HANT_TABLE, ZH_TW_TABLE]);
    /// Zh2HK converter (`zh-Hant-HK`/香港繁體), lazily built from [`ZH_HANT_TABLE`](crate::ZH_HANT_TABLE)
    /// and [`ZH_HK_TABLE`](crate::ZH_HK_TABLE).
    pub static ref ZH_TO_HK_CONVERTER: ZhConverter = deserialize_converter(Variant::ZhHK, ZH_HANT_HK_DAAC, [ZH_HANT_TABLE, ZH_HK_TABLE]);
    /// Zh2MO converter (`zh-Hant-MO`/澳門繁體), lazily built from [`ZH_HANT_TABLE`](crate::ZH_HANT_TABLE)
    /// and [`ZH_MO_TABLE`](crate::ZH_MO_TABLE).
    pub static ref ZH_TO_MO_CONVERTER: ZhConverter = deserialize_converter(Variant::ZhMO, ZH_HANT_MO_DAAC, [ZH_HANT_TABLE, ZH_MO_TABLE]);
    /// Zh2CN converter (`zh-Hans-CN`/大陆简体), lazily built from [`ZH_HANS_TABLE`](crate::ZH_HANS_TABLE)
    /// and [`ZH_CN_TABLE`](crate::ZH_CN_TABLE).
    pub static ref ZH_TO_CN_CONVERTER: ZhConverter = deserialize_converter(Variant::ZhCN, ZH_HANS_CN_DAAC, [ZH_HANS_TABLE, ZH_CN_TABLE]);
    /// Zh2SG converter (`zh-Hans-SG`/新加坡简体), lazily built from [`ZH_HANS_TABLE`](crate::ZH_HANS_TABLE)
    /// and [`ZH_SG_TABLE`](crate::ZH_SG_TABLE).
    pub static ref ZH_TO_SG_CONVERTER: ZhConverter = deserialize_converter(Variant::ZhSG, ZH_HANS_SG_DAAC, [ZH_HANS_TABLE, ZH_SG_TABLE]);
    /// Zh2MY converter (`zh-Hans-MY`/大马简体), lazily built from [`ZH_HANS_TABLE`](crate::ZH_HANS_TABLE)
    /// and [`ZH_MY_TABLE`](crate::ZH_MY_TABLE).
    pub static ref ZH_TO_MY_CONVERTER: ZhConverter = deserialize_converter(Variant::ZhMY, ZH_HANS_MY_DAAC, [ZH_HANS_TABLE, ZH_MY_TABLE]);
}

/// Get the builtin converter for a target Chinese variant.
#[inline(always)]
pub fn get_builtin_converter(target: Variant) -> &'static ZhConverter {
    use Variant::*;
    match target {
        Zh => &ZH_BLANK_CONVERTER,
        ZhHant => &ZH_TO_HANT_CONVERTER,
        ZhHans => &ZH_TO_HANS_CONVERTER,
        ZhTW => &ZH_TO_TW_CONVERTER,
        ZhHK => &ZH_TO_HK_CONVERTER,
        ZhMO => &ZH_TO_MO_CONVERTER,
        ZhCN => &ZH_TO_CN_CONVERTER,
        ZhMY => &ZH_TO_MY_CONVERTER,
        ZhSG => &ZH_TO_SG_CONVERTER,
    }
}

#[doc(hidden)]
pub fn deserialize_converter(
    variant: Variant,
    daac: &[u8],
    tables: impl IntoIterator<Item = Table<'static>>,
) -> ZhConverter {
    #[cfg(feature = "compress")]
    let daac = zstd_decompress(daac);

    ZhConverter::with_target_variant(
        unsafe { CharwiseDoubleArrayAhoCorasick::deserialize_unchecked(&daac).0 },
        tables
            .into_iter()
            .flat_map(|t| expand_table(t).map(|(_f, t)| t))
            .collect(), // TODO: avoid String
        variant,
    )
}
