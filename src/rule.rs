//! Structs and functions for processing conversion rule, as is defined in [ConverterRule.php](https://doc.wikimedia.org/mediawiki-core/master/php/ConverterRule_8php.html).
//!
//! **Note**: This module is not stable yet and just exposed for convenience. It might have
//! breaking changes at any time in violation of semver.

use std::collections::HashMap;
use std::convert::AsRef;
use std::fmt::{self, Display};
use std::iter::{self, Map};
use std::str::FromStr;

use once_cell::sync::Lazy;
use regex::{Match, Matches, Regex};

use crate::variant::{Variant, VariantMap};

/// A single rule used for language conversion, usually extracted from wikitext in the syntax `-{ }-`.
///
/// Ref: [ConverterRule.php](https://doc.wikimedia.org/mediawiki-core/master/php/ConverterRule_8php.html)
/// and [Help:高级字词转换语法](https://zh.wikipedia.org/wiki/Help:%E9%AB%98%E7%BA%A7%E5%AD%97%E8%AF%8D%E8%BD%AC%E6%8D%A2%E8%AF%AD%E6%B3%95)
/// (not fully compliant)
#[derive(Debug, Clone)]
pub struct ConvRule {
    pub(crate) action: Option<Action>,
    pub(crate) output: Option<Output>,
    pub(crate) conv: Option<Conv>,
    pub(crate) set_title: bool,
}

#[derive(Debug, Clone)]
pub struct ConvRuleWithVariant<'r> {
    rule: &'r ConvRule,
    variant: Variant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output {
    Normal,
    VariantName(Variant),
    Description,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Action {
    Add,
    Remove,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum RuleError {
    InvalidFlag(char),
    InvalidConv,
    InvalidConvForTitle,
    InvalidVariant,
}

impl ConvRule {
    pub fn targeted(&self, target: Variant) -> ConvRuleWithVariant {
        ConvRuleWithVariant {
            rule: self,
            variant: target,
        }
    }

    pub fn into_conv_action(self) -> Option<ConvAction> {
        if let (Some(action), Some(conv)) = (self.action, self.conv) {
            Some(ConvAction(action, conv))
        } else {
            None
        }
    }

    /// Same as `from_str`, except that any unrecognized rule is treated as [`Conv::Asis`]
    pub fn from_str_infallible(s: &str) -> ConvRule {
        s.parse().unwrap_or_else(|_| ConvRule {
            action: None,
            output: Some(Output::Normal),
            conv: Some(Conv::Asis(s.to_owned())),
            set_title: true,
        })
    }
}

impl FromStr for ConvRule {
    type Err = RuleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (flags, body) = s.find('|').map_or_else(
            || ("", s),
            |i| {
                let (first, last) = s.split_at(i);
                (first, &last[1..])
            },
        );
        let mut set_title = false;
        let mut action = None;
        let mut output = Some(Output::Normal);
        // Ref: https://github.com/wikimedia/mediawiki/blob/7bf779524ab1fd8e1d74f79ea4840564d48eea4d/includes/language/LanguageConverter.php#L158
        //  and https://github.com/wikimedia/mediawiki/blob/ec6fd491074a6ace5ccc7bc05b01c30512a5723d/includes/language/ConverterRule.php#L408
        //  (not fully compliant, especially for multi-flags cases)
        for flag in flags.chars() {
            match flag {
                // FIX: 'A'
                '+' => action = Some(Action::Add),
                '-' => action = Some(Action::Remove),
                // no conv, just display the inner as-is
                'R' => {
                    return Ok(ConvRule {
                        action: None,
                        output: Some(Output::Normal),
                        conv: Some(Conv::Asis(body.to_owned())),
                        set_title: false,
                    });
                }
                // output the variant name of the context
                'N' => {
                    output = Some(Output::VariantName(
                        Variant::from_str(body).map_err(|_| RuleError::InvalidVariant)?,
                    ));
                }
                // Display the rule **D**escription
                'D' => {
                    output = Some(Output::Description);
                }
                // add a global rule without displaying anything (**H**idden)
                'H' => {
                    action = Some(Action::Add);
                    output = None;
                }
                // display as normal (by default)
                'S' => {}
                // add a global rule; A implies +S
                'A' => action = Some(Action::Add),
                // convert the title of some an article
                'T' => {
                    set_title = true;
                    output = None
                }
                unknown => return Err(RuleError::InvalidFlag(unknown)),
            }
        }
        let conv = if let Some(Output::VariantName(_)) = output {
            None
        } else {
            Some(Conv::from_str(body).map_err(|_| RuleError::InvalidConv)?)
        };
        if set_title
            && conv
                .as_ref()
                .and_then(|c| c.get_bid())
                .map(|b| !b.is_empty())
                != Some(true)
        {
            return Err(RuleError::InvalidConvForTitle);
        }
        Ok(Self {
            action,
            output,
            conv,
            set_title,
        })
    }
}

impl<'r> Display for ConvRuleWithVariant<'r> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rule = self.rule;
        match &rule.output {
            None => Ok(()),
            Some(Output::Normal) => write!(
                f,
                "{}",
                rule.conv
                    .as_ref()
                    .and_then(|c| c.get_text_by_target(self.variant))
                    .unwrap_or("") // mediawiki would show: 在手动语言转换规则中检测到错误
            ),
            Some(Output::VariantName(variant)) => write!(f, "{}", variant.get_name()),
            Some(Output::Description) => {
                if let Some(conv) = rule.conv.as_ref() {
                    write!(f, "{}", conv)
                } else {
                    Ok(())
                }
            }
        }
    }
}

/// The inner of a [`ConvRule`] without flags and actions
///
/// A single `Conv` may consist of multiple uni-directional and/or bi-diretional mappings in any
/// order. e.g.,
/// uni-directional mapping: `巨集=>zh-cn:宏;`,
/// bi-directional mapping: `zh-hans:计算机; zh-hant:電腦;`,
/// mixed: `zh-hk:橘;zh-tw:芭樂;蘋果=>zh-cn:梨;`.
///
/// Or it can be an as-is text which prevents such text being converted by other rules. Typically,
/// it helps avoid over-conversion applied to surnames. Be noted that it might not be effective in
/// rare cases due to the leftmost-longest matching strategy.
#[derive(Debug, Clone)]
pub enum Conv {
    Asis(String),
    Map(ConvMap),
}

#[derive(Debug, Clone)]
pub struct ConvMap {
    pub bid: VariantMap<String>,
    pub unid: VariantMap<Vec<(String, String)>>,
}

impl Conv {
    #[inline]
    /// The text to display for the target variant
    pub fn get_text_by_target(&self, target: Variant) -> Option<&str> {
        use Conv::*;
        match self {
            Asis(s) => Some(s),
            Map(m) => m.bid.get_text_with_fallback(target),
        }
    }

    pub fn get_conv_pairs(&self, target: Variant) -> impl Iterator<Item = (&str, &str)> {
        let mut mit = self
            .as_map()
            .map(|m| {
                m.bid.get_conv_pairs(target).chain(
                    m.unid
                        .get_conv_pairs(target)
                        .iter()
                        .map(|(f, t)| (f.as_ref(), t.as_ref())),
                )
            })
            .into_iter()
            .flatten()
            .filter(|(f, _t)| !f.is_empty()); // filter out emtpy froms that troubles AC
        let mut maybe_asis = self.as_asis().map(|s| Some((s, s)));

        iter::from_fn(move || {
            if let Some(asis_yielding) = maybe_asis.as_mut() {
                asis_yielding.take()
            } else {
                mit.next()
            }
        })
    }

    pub fn as_asis(&self) -> Option<&str> {
        use Conv::*;
        match self {
            Asis(s) => Some(s.as_ref()),
            _ => None,
        }
    }

    pub fn as_map(&self) -> Option<&ConvMap> {
        use Conv::*;
        match self {
            Map(m) => Some(m),
            _ => None,
        }
    }

    /// Get the inner bi-directional map, if any.
    ///
    /// Typically intended to be used by [`PageRules`](crate::pagerules::PageRules) to extract the
    /// map for a page title (e.g. `-{T|zh:黑;zh-cn:白}-`).
    pub fn get_bid(&self) -> Option<&VariantMap<String>> {
        self.as_map().map(|m| &m.bid)
    }

    /// Same as `from_str`, except that any unrecognized conv is treated as [`Conv::Asis`]
    pub fn from_str_infallible(s: &str) -> Conv {
        s.parse().unwrap_or_else(|_| Conv::Asis(s.to_owned()))
    }
}

impl Display for Conv {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        use Conv::*;
        match self {
            Asis(s) => write!(fmt, "{}", s),
            Map(m) => {
                write!(fmt, "{}{}", m.bid, m.unid)
            }
        }
    }
}

impl FromStr for Conv {
    type Err = ();

    fn from_str(s: &str) -> Result<Conv, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            // TODO: return?
        }
        let mut bid = HashMap::new();
        let mut unid = HashMap::new();
        // TODO: implement a clean iterator instead
        let mut parse_single = |s: &str| -> Result<(), Self::Err> {
            if s.trim().is_empty() {
                return Ok(());
            };
            let (left, right) = s.find("=>").map_or_else(
                || (None, s),
                |i| {
                    assert!(0 < i && i < s.len());
                    let (first, last) = s.split_at(i);
                    (Some(first), &last[2..])
                },
            );
            let (variant, to) = right.split_at(right.find(':').ok_or(())?);
            let to = &to[1..]; // strip ":"
            let variant = variant.trim().parse::<Variant>().map_err(|_| ())?;
            if let Some(from) = left {
                if from.is_empty() {
                    return Err(()); // e.g. {EMPTY}=>zh:foobar
                }
                unid.entry(variant)
                    .or_insert_with(Vec::new)
                    .push((from.to_owned(), to.to_owned()));
            } else {
                bid.insert(variant, to.to_owned());
            }
            Ok(())
        };
        let mut i = 0;
        let mut ampersand = None; // handle entity escape
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
        }
        if i != s.as_bytes().len() && parse_single(&s[i..]).is_err() {
            if bid.is_empty() && unid.is_empty() {
                return Ok(Conv::Asis(s.to_owned()));
            } else {
                return Err(());
                // Or we just discard this part?
            }
        }
        Ok(Conv::Map(ConvMap {
            bid: bid.into(),
            unid: unid.into(),
        }))
    }
}

/// A `([Action], [Conv])` pair with some helper methods
#[derive(Debug, Clone)]
pub struct ConvAction(Action, Conv);

impl ConvAction {
    pub fn is_add(&self) -> bool {
        self.0 == Action::Add
    }

    pub fn is_remove(&self) -> bool {
        self.0 == Action::Remove
    }

    pub fn as_conv(&self) -> &Conv {
        &self.1
    }
}

impl AsRef<Conv> for ConvAction {
    fn as_ref(&self) -> &Conv {
        &self.1
    }
}

static REGEX_RULE: Lazy<Regex> = Lazy::new(|| Regex::new(r"-\{.+?\}-").unwrap());

/// Extract a set rules from a text.
pub fn extract_rules<'s>(
    text: &'s str,
) -> Map<Matches<'static, 's>, impl FnMut(Match<'s>) -> Result<ConvRule, RuleError>> {
    // note: the regex works a little differently from the parser in converter
    (*REGEX_RULE).find_iter(text).map(|m| {
        let rule = m.as_str();
        ConvRule::from_str(&rule[2..rule.len() - 2])
    })
}
