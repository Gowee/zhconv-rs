//! Structs for handling variants and mapping of variants.
//!
//! **Note**: This module is not stable yet and just exposed for convenience. It might have
//! breaking changes at any time in violation of semver.

use std::collections::HashMap;
use std::convert::From;
use std::default::Default;
use std::fmt::{self, Display};
use std::str::FromStr;

use strum::{Display, EnumString, IntoStaticStr, VariantNames};

use crate::utils::get_with_fallback;

/// Chinese variants (a.k.a 中文變體), parsed from language tags, as listed in [Help:高级字词转换语法#组合转换标签](https://zh.wikipedia.org/wiki/Help:高级字词转换语法#组合转换标签).
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, Display, EnumString, VariantNames, IntoStaticStr,
)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive)]
#[derive(Default)]
pub enum Variant {
    #[default]
    /// Chinese (dummy variant)
    Zh,
    /// Script: Traditional Chinese
    ZhHant,
    /// Script: Simplified Chinese
    ZhHans,
    /// Short for `zh-Hant-TW`, Script: Traditional Chinese, Region: Taiwan
    ZhTW,
    /// Short for`zh-Hant-HK`, Script: Traditional Chinese, Region: Hong Kong
    ZhHK,
    /// Short for`zh-Hant-MO`, Script: Traditional Chinese, Region: Macau
    ZhMO,
    /// Short for`zh-Hans-MY`, Script: Simplified Chinese, Region: Malaysia
    ZhMY,
    /// Short for`zh-Hans-SG`, Script: Simplified Chinese, Region: Singapore
    ZhSG,
    /// Short for`zh-Hans-CN`, Script: Simplified Chinese, Region: China (mainland)
    ZhCN,
    // Unknown(String)
}

impl Variant {
    #[inline(always)]
    pub fn get_name(self) -> &'static str {
        // actually, the name should also follow variant context, but just use these for simplicity
        use Variant::*;
        match self {
            Zh => "原文", // 中文
            ZhHant => "繁体",
            ZhHans => "简体",
            ZhTW => "臺灣",
            ZhHK => "香港",
            ZhMO => "澳門",
            ZhMY => "大马",
            ZhSG => "新加坡",
            ZhCN => "大陆", // a.k.a mainland China
        }
    }
}

/// Map variants to text, e.g. `zh-hans:计算机; zh-hant:電腦;`
#[derive(Debug, Clone)]
pub struct VariantMap<T>(pub HashMap<Variant, T>);

impl VariantMap<String> {
    /// Get the text for the target variant, if any
    #[inline(always)]
    pub fn get_text(&self, target: Variant) -> Option<&str> {
        self.0.get(&target).map(String::as_str)
    }

    /// Get the text for the target variant with automatic fallback.
    ///
    /// It will panic if the inner map is empty itself.
    pub fn get_text_with_fallback(&self, target: Variant) -> Option<&str> {
        // Ref: https://github.com/wikimedia/mediawiki/blob/6eda8891a0595e72e350998b6bada19d102a42d9/includes/language/converters/ZhConverter.php#L65
        use Variant::*;
        match_fallback!(
            self.0,
            target,
            Zh -> [ZhHans, ZhHant, ZhCN, ZhTW, ZhHK, ZhSG, ZhMO, ZhMY],
            ZhHans -> [ ZhCN, ZhSG, ZhMY ],
            ZhHant -> [ ZhTW, ZhHK, ZhMO ],
            ZhCN -> [ ZhHans, ZhSG, ZhMY ],
            ZhSG -> [ ZhHans, ZhCN, ZhMY ],
            ZhMY -> [ ZhHans, ZhSG, ZhCN ],
            ZhTW -> [ ZhHant, ZhHK, ZhMO ],
            ZhHK -> [ ZhHant, ZhMO, ZhTW ],
            ZhMO -> [ ZhHant, ZhHK, ZhTW ],
        )
        // TODO: falling back to zh finally?
        // even though the rules defined in ZhConverter.php fallbakcs to Zh,
        // tests shows that it display a error when no other more concrete variants available
    }

    /// Get the pairs of conversion for a target variant
    pub fn get_conv_pairs(&self, target: Variant) -> impl Iterator<Item = (&str, &str)> {
        use Variant::*;
        // MEDIAWIKI: the code of the reference implementation is too obscure, try to replicate the
        //            the same behavior based on some tests
        let mut it = None;
        match target {
            // based on tests, the three are only used as fallbacks for regional scripts
            Zh | ZhHant | ZhHans => (),
            _ => {
                // It won't fallback to Zh finally. So Zh is only used as from?
                let to = match_fallback!(
                    self.0,
                    target,
                    // Zh -> [ZhHans, ZhHant, ZhCN, ZhTW, ZhHK, ZhSG, ZhMO, ZhMY],
                    // ZhHans -> [ ZhCN, ZhSG, ZhMY ],
                    // ZhHant -> [ ZhTW, ZhHK, ZhMO ],
                    ZhCN -> [ ZhHans, ZhSG, ZhMY ],
                    ZhSG -> [ ZhHans, ZhCN, ZhMY ],
                    ZhMY -> [ ZhHans, ZhSG, ZhCN ],
                    ZhTW -> [ ZhHant, ZhHK, ZhMO ],
                    ZhHK -> [ ZhHant, ZhMO, ZhTW ],
                    ZhMO -> [ ZhHant, ZhHK, ZhTW ],
                );

                if let Some(to) = to {
                    // for variant == target, from == to, it prevents the word from converting
                    it = Some(self.0.iter().filter_map(move |(_variant, from)| {
                        if from.is_empty() {
                            None
                        } else {
                            Some((from.as_ref(), to))
                        }
                    }));
                }
            }
        }

        it.into_iter().flatten()
    }
}

impl VariantMap<Vec<(String, String)>> {
    /// Get the pairs of conversion for a target variant
    pub fn get_conv_pairs(&self, target: Variant) -> &[(String, String)] {
        // MEDIAWIKI:
        // unlike inline bid conversion rules, global unid conversion rule has no fallback
        self.0.get(&target).map(|p| p.as_slice()).unwrap_or(&[])
    }
}

impl<T> VariantMap<T> {
    pub fn into_inner(self) -> HashMap<Variant, T> {
        self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty() // TODO: Deref
    }
}

impl FromStr for VariantMap<String> {
    type Err = (); // TODO: better error propagation

    fn from_str(s: &str) -> Result<VariantMap<String>, Self::Err> {
        let s = s.trim();
        let mut map = HashMap::new();
        // TODO: implement a clean iterator instead
        let mut parse_single = |s: &str| -> Result<(), Self::Err> {
            let (v, t) = s.split_at(s.find(':').ok_or(())?);
            let t = &t[1..]; // strip ":"
            map.insert(
                Variant::from_str(v.trim()).map_err(|_| ())?,
                t.trim().to_owned(),
            );
            Ok(())
        };
        let mut i = 0;
        let mut ampersand = None;
        // TODO: more robust parser?
        for (j, &c) in s.as_bytes().iter().enumerate() {
            match c {
                b'&' => {
                    ampersand = Some(j);
                    // if ampersand, the new & is the new start
                }
                b';' => {
                    if !(ampersand.is_some() && j - ampersand.unwrap() > 1) {
                        parse_single(&s[i..j])?;
                        i = j + 1;
                    }
                }
                _ => {
                    if ampersand.is_some() & !(b'#' == c || char::from(c).is_ascii_alphanumeric()) {
                        ampersand = None;
                    }
                }
            }
            // match &s[i]
        }
        if i != s.as_bytes().len() {
            parse_single(&s[i..])?;
        }
        Ok(VariantMap(map))
    }
}

impl Display for VariantMap<String> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (v, t) in self.0.iter() {
            // TODO: insertion order
            write!(f, "{}：{}；", v.get_name(), t)?;
        }
        Ok(())
    }
}

impl Display for VariantMap<Vec<(String, String)>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (variant, pairs) in self.0.iter() {
            for (from, to) in pairs.iter() {
                write!(f, "{}⇒{}: {}", from, variant, to)?;
            }
        }
        Ok(())
    }
}

impl<T> From<HashMap<Variant, T>> for VariantMap<T> {
    fn from(hm: HashMap<Variant, T>) -> Self {
        Self(hm)
    }
}

// Ref: https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=cdab97d0a7f71d9a13568c97ad3faf3a
macro_rules! match_fallback {
    ( $map:expr, $target:expr, $($t:tt)* ) => {
        match_fallback!(@build $map, $target, (), $($t)*)
    };
    (@build $map:expr, $target:expr, ($($arms:tt)*), $variant:ident -> [ $($fallbacks:tt)* ], $($others:tt)* ) => {
        match_fallback!(@build $map, $target, ($($arms)* $variant => get_with_fallback!($map, $variant, $($fallbacks)*),), $($others)*)
    };
    (@build $map:expr, $target:expr, ($($arms:tt)*) $(,)? ) => {
        match $target {
            $($arms)*
            #[allow(unreachable_patterns)]
            _ => None
        }.map(String::as_str)
    };
}
use match_fallback;
