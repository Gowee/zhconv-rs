//! Converters lazily built from built-in [`tables`](crate::tables).
//!
//! These converters are lazily loaded from serialized automata built at build-time, and cached for
//! later use.

use daachorse::CharwiseDoubleArrayAhoCorasick;
use std::sync::LazyLock;

#[cfg(feature = "compress")]
use crate::utils::zstd_decompress;
use crate::{tables::*, Variant, ZhConverter, ZhConverterBuilder};

// FIX: Doc

// Be careful with the order of tables.
// The converter build process relies on HashMap::extend, during which latter rules overwrite the
// early ones. And we expect more specific rules take precedence (e.g. zh-TW > zh-Hant).
// Ref: https://github.com/wikimedia/mediawiki/blob/6eda8891a0595e72e350998b6bada19d102a42d9/includes/language/converters/ZhConverter.php#L157

/// Placeholding converter (`zh`/原文). Nothing will be converted with this.
pub static ZH_BLANK_CONVERTER: LazyLock<ZhConverter> =
    LazyLock::new(|| ZhConverterBuilder::new().target(Variant::Zh).build());
/// Converter to `zh-Hant` (繁體中文), lazily built from [`ZH_HANT_TABLE`](crate::ZH_HANT_TABLE).
pub static ZH_TO_HANT_CONVERTER: LazyLock<ZhConverter> =
    LazyLock::new(|| deserialize_converter(Variant::ZhHant, ZH_HANT_DAAC, [ZH_HANT_TABLE]));
/// Converter to `zh-Hans` (简体中文), lazily built from [`ZH_HANS_TABLE`](crate::ZH_HANS_TABLE).
pub static ZH_TO_HANS_CONVERTER: LazyLock<ZhConverter> =
    LazyLock::new(|| deserialize_converter(Variant::ZhHans, ZH_HANS_DAAC, [ZH_HANS_TABLE]));
/// Converter to `zh-Hant-TW` (臺灣正體), lazily built from [`ZH_HANT_TABLE`](crate::ZH_HANT_TABLE)
/// and [`ZH_TW_TABLE`](crate::ZH_TW_TABLE).
pub static ZH_TO_TW_CONVERTER: LazyLock<ZhConverter> = LazyLock::new(|| {
    deserialize_converter(Variant::ZhTW, ZH_HANT_TW_DAAC, [ZH_HANT_TABLE, ZH_TW_TABLE])
});
/// Coonverter to `zh-Hant-HK` (香港繁體), lazily built from [`ZH_HANT_TABLE`](crate::ZH_HANT_TABLE)
/// and [`ZH_HK_TABLE`](crate::ZH_HK_TABLE).
pub static ZH_TO_HK_CONVERTER: LazyLock<ZhConverter> = LazyLock::new(|| {
    deserialize_converter(Variant::ZhHK, ZH_HANT_HK_DAAC, [ZH_HANT_TABLE, ZH_HK_TABLE])
});
/// Converter to `zh-Hant-MO` (澳門繁體), lazily built from [`ZH_HANT_TABLE`](crate::ZH_HANT_TABLE)
/// and [`ZH_MO_TABLE`](crate::ZH_MO_TABLE).
pub static ZH_TO_MO_CONVERTER: LazyLock<ZhConverter> = LazyLock::new(|| {
    deserialize_converter(Variant::ZhMO, ZH_HANT_MO_DAAC, [ZH_HANT_TABLE, ZH_MO_TABLE])
});
/// Converter to `zh-Hans-CN` (大陆简体), lazily built from [`ZH_HANS_TABLE`](crate::ZH_HANS_TABLE)
/// and [`ZH_CN_TABLE`](crate::ZH_CN_TABLE).
pub static ZH_TO_CN_CONVERTER: LazyLock<ZhConverter> = LazyLock::new(|| {
    deserialize_converter(Variant::ZhCN, ZH_HANS_CN_DAAC, [ZH_HANS_TABLE, ZH_CN_TABLE])
});
/// Converter to `zh-Hans-SG` (新加坡简体), lazily built from [`ZH_HANS_TABLE`](crate::ZH_HANS_TABLE)
/// and [`ZH_SG_TABLE`](crate::ZH_SG_TABLE).
pub static ZH_TO_SG_CONVERTER: LazyLock<ZhConverter> = LazyLock::new(|| {
    deserialize_converter(Variant::ZhSG, ZH_HANS_SG_DAAC, [ZH_HANS_TABLE, ZH_SG_TABLE])
});
/// Converter to `zh-Hans-MY` (大马简体), lazily built from [`ZH_HANS_TABLE`](crate::ZH_HANS_TABLE)
/// and [`ZH_MY_TABLE`](crate::ZH_MY_TABLE).
pub static ZH_TO_MY_CONVERTER: LazyLock<ZhConverter> = LazyLock::new(|| {
    deserialize_converter(Variant::ZhMY, ZH_HANS_MY_DAAC, [ZH_HANS_TABLE, ZH_MY_TABLE])
});

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
#[allow(clippy::needless_borrow)]
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
