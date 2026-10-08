//! Built-in converters loaded from prebuilt automata and word stores,
//! and cached for later use.

use daachorse::CharwiseDoubleArrayAhoCorasick;
use std::sync::LazyLock;
use zerovec::vecs::Index32;
use zerovec::{VarZeroSlice, VarZeroVec};

#[cfg(any(feature = "_mediawiki-base", feature = "_opencc-base"))]
use crate::tables::*;
#[cfg(feature = "compress")]
use crate::utils::zstd_decompress;
use crate::{Variant, ZhConverter, ZhConverterBuilder};

// Each builtin converter pairs one prebuilt automaton with its script
// side's shared word store. Regional precedence (e.g. zh-TW > zh-Hant)
// is resolved at build time: region extras drop base-identical pairs,
// and later rules win duplicate keys inside the automaton.
// Ref: https://github.com/wikimedia/mediawiki/blob/6eda8891a0595e72e350998b6bada19d102a42d9/includes/language/converters/ZhConverter.php#L157

/// Placeholding converter (`zh`/原文). Nothing will be converted with this.
pub static ZH_BLANK_CONVERTER: LazyLock<ZhConverter<'static>> =
    LazyLock::new(|| ZhConverterBuilder::new().target(Variant::Zh).build());
/// Converter to `zh-Hant` (繁體中文).
#[cfg(any(feature = "mediawiki-hant", feature = "opencc-hant"))]
pub static ZH_TO_HANT_CONVERTER: LazyLock<ZhConverter<'static>> = LazyLock::new(|| unsafe {
    deserialize_converter(Variant::ZhHant, ZH_HANT_DAAC, hant_all_store())
});
/// Converter to `zh-Hans` (简体中文).
#[cfg(any(feature = "mediawiki-hans", feature = "opencc-hans"))]
pub static ZH_TO_HANS_CONVERTER: LazyLock<ZhConverter<'static>> = LazyLock::new(|| unsafe {
    deserialize_converter(Variant::ZhHans, ZH_HANS_DAAC, hans_all_store())
});
/// Converter to `zh-Hant-TW` (臺灣正體).
#[cfg(any(feature = "mediawiki-tw", feature = "opencc-tw"))]
pub static ZH_TO_TW_CONVERTER: LazyLock<ZhConverter<'static>> = LazyLock::new(|| unsafe {
    deserialize_converter(Variant::ZhTW, ZH_HANT_TW_DAAC, hant_all_store())
});
/// Converter to `zh-Hant-HK` (香港繁體).
#[cfg(any(feature = "mediawiki-hk", feature = "opencc-hk"))]
pub static ZH_TO_HK_CONVERTER: LazyLock<ZhConverter<'static>> = LazyLock::new(|| unsafe {
    deserialize_converter(Variant::ZhHK, ZH_HANT_HK_DAAC, hant_all_store())
});
/// Converter to `zh-Hant-MO` (澳門繁體).
#[cfg(any(feature = "mediawiki-hk", feature = "opencc-hk"))]
pub static ZH_TO_MO_CONVERTER: LazyLock<ZhConverter<'static>> = LazyLock::new(|| unsafe {
    deserialize_converter(Variant::ZhMO, ZH_HANT_MO_DAAC, hant_all_store())
});
/// Converter to `zh-Hans-CN` (大陆简体).
#[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
pub static ZH_TO_CN_CONVERTER: LazyLock<ZhConverter<'static>> = LazyLock::new(|| unsafe {
    deserialize_converter(Variant::ZhCN, ZH_HANS_CN_DAAC, hans_all_store())
});
/// Converter to `zh-Hans-SG` (新加坡简体).
#[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
pub static ZH_TO_SG_CONVERTER: LazyLock<ZhConverter<'static>> = LazyLock::new(|| unsafe {
    deserialize_converter(Variant::ZhSG, ZH_HANS_SG_DAAC, hans_all_store())
});
/// Converter to `zh-Hans-MY` (大马简体).
#[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
pub static ZH_TO_MY_CONVERTER: LazyLock<ZhConverter<'static>> = LazyLock::new(|| unsafe {
    deserialize_converter(Variant::ZhMY, ZH_HANS_MY_DAAC, hans_all_store())
});

/// Get the builtin converter for a target Chinese variant.
#[inline(always)]
pub fn get_builtin_converter(target: Variant) -> &'static ZhConverter<'static> {
    use Variant::*;
    // using zh-cn for zh-{sg, my} and zh-hk for zh-mo, like in MediaWiki
    match target {
        Zh => &ZH_BLANK_CONVERTER,
        #[cfg(any(feature = "mediawiki-hant", feature = "opencc-hant"))]
        ZhHant => &ZH_TO_HANT_CONVERTER,
        #[cfg(any(feature = "mediawiki-hans", feature = "opencc-hans"))]
        ZhHans => &ZH_TO_HANS_CONVERTER,
        #[cfg(any(feature = "mediawiki-tw", feature = "opencc-tw"))]
        ZhTW => &ZH_TO_TW_CONVERTER,
        #[cfg(any(feature = "mediawiki-hk", feature = "opencc-hk"))]
        ZhHK => &ZH_TO_HK_CONVERTER,
        #[cfg(any(feature = "mediawiki-hk", feature = "opencc-hk"))]
        ZhMO => &ZH_TO_MO_CONVERTER,
        #[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
        ZhCN => &ZH_TO_CN_CONVERTER,
        #[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
        ZhMY => &ZH_TO_MY_CONVERTER,
        #[cfg(any(feature = "mediawiki-cn", feature = "opencc-cn"))]
        ZhSG => &ZH_TO_SG_CONVERTER,
        #[allow(unreachable_patterns)]
        _ => panic!("No converter targeting {} enabled.", target),
    }
}

/// Deserialize a converter from bundled automaton bytes and a static word store.
///
/// # Safety
/// `daac` and `store` must form a coordinated, compatible pair produced by `build.rs`
/// for the specified `variant`. Automaton match values MUST encode valid packed
/// `(offset << 10) | len` descriptors referencing valid UTF-8 slices within `store.as_bytes()`.
#[doc(hidden)]
#[allow(clippy::needless_borrow)]
pub unsafe fn deserialize_converter(
    variant: Variant,
    daac: &[u8],
    store: &'static VarZeroSlice<str, Index32>,
) -> ZhConverter<'static> {
    #[cfg(feature = "compress")]
    let daac = zstd_decompress(daac);

    // SAFETY: The caller guarantees that `daac` and `store` form a coordinated pair
    // whose packed (offset, len) values reference valid UTF-8 slices in `store`.
    unsafe {
        ZhConverter::with_target_variant(
            CharwiseDoubleArrayAhoCorasick::deserialize_unchecked(&daac).0,
            VarZeroVec::from(store),
            variant,
        )
    }
}
