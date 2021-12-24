//! Structs and functions for processing conversion rule, as is defined in [ConverterRule.php](https://doc.wikimedia.org/mediawiki-core/master/php/ConverterRule_8php.html).
//!
//! **Note**: This module is exposed for convenience. It might have breaking changes at any time in
//!           violation of semver.

use std::collections::HashMap;
use std::convert::AsRef;
use std::fmt::{self, Display};
use std::str::FromStr;

use once_cell::sync::Lazy;
use regex::{Match, Matches, Regex};
use std::iter::Map;

use crate::variant::{Variant, VariantMap};

/// A single rule used for language conversion, usually extracted from wikitext in the syntax `-{ }-`.
///
/// Ref: [ConverterRule.php](https://doc.wikimedia.org/mediawiki-core/master/php/ConverterRule_8php.html)
#[derive(Debug, Clone)]
pub struct ConvRule {
    pub(crate) action: Option<Action>,
    pub(crate) output: Option<Output>,
    pub(crate) conv: Option<Conv>,
    pub(crate) set_title: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output {
    Normal,
    VariantName(Variant),
    Verbatim(String),
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
    pub fn write_output(&self, mut dest: impl fmt::Write, target: Variant) -> fmt::Result {
        match &self.output {
            None => Ok(()),
            Some(Output::Verbatim(inner)) => write!(dest, "{}", inner),
            Some(Output::Normal) => write!(
                dest,
                "{}",
                self.conv
                    .as_ref()
                    .and_then(|c| c.get_text_by_target(target))
                    .unwrap_or("")
            ),
            Some(Output::VariantName(variant)) => write!(dest, "{}", variant.get_name()),
            Some(Output::Description) => {
                if let Some(conv) = self.conv.as_ref() {
                    write!(dest, "{}", conv)
                } else {
                    Ok(())
                }
            }
        }
    }

    pub fn into_conv_action(self) -> Option<ConvAction> {
        if let (Some(action), Some(conv)) = (self.action, self.conv) {
            Some(ConvAction(action, conv))
        } else {
            None
        }
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
        if flags.is_empty() {
            return Ok(if let Ok(conv) = Conv::from_str(body) {
                // inline rule
                ConvRule {
                    action: None,
                    output: Some(Output::Normal),
                    conv: Some(conv),
                    set_title: false,
                }
            } else {
                // or verbatim
                ConvRule {
                    action: None,
                    output: Some(Output::Verbatim(body.to_owned())),
                    conv: None,
                    set_title: false,
                }
            });
        }
        let mut set_title = false;
        let mut action = None;
        let mut output = Some(Output::Normal);
        // Ref: https://github.com/wikimedia/mediawiki/blob/7bf779524ab1fd8e1d74f79ea4840564d48eea4d/includes/language/LanguageConverter.php#L158
        //  and https://github.com/wikimedia/mediawiki/blob/ec6fd491074a6ace5ccc7bc05b01c30512a5723d/includes/language/ConverterRule.php#L408
        // (not fully compliant, especially for multi-flags cases)
        for flag in flags.chars() {
            match flag {
                // FIX: 'A'
                '+' => action = Some(Action::Add),
                '-' => action = Some(Action::Remove),
                // no conv, just display the inner as-is
                'R' => {
                    return Ok(ConvRule {
                        action: None,
                        output: Some(Output::Verbatim(body.to_owned())),
                        conv: None,
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
        if set_title && (conv.is_none() || conv.as_ref().unwrap().bid.is_empty()) {
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

/// The inner of a [`ConvRule`] without flags and actions
///
/// Note: A single `Conv` can contain multiple uni-directional and/or bi-diretional mappings in
/// any order.
/// For example,
/// uni-directional mapping: `巨集=>zh-cn:宏;`,
/// bi-directional mapping: `zh-hans:计算机; zh-hant:電腦;`,
/// both: `zh-hk:橘;zh-tw:芭樂;蘋果=>zh-cn:梨;`
#[derive(Debug, Clone)]
pub struct Conv {
    pub bid: VariantMap<String>,
    pub unid: VariantMap<Vec<(String, String)>>,
}

impl Conv {
    #[inline]
    /// The the text to display for the target variant
    pub fn get_text_by_target(&self, target: Variant) -> Option<&str> {
        self.bid.get_text_with_fallback(target)
    }

    #[inline]
    pub fn get_conv_pairs(&self, target: Variant) -> Vec<(&str, &str)> {
        // TODO: iterator
        let mut pairs = self.bid.get_conv_pairs(target);
        pairs.extend(
            self.unid
                .get_conv_pairs(target)
                .iter()
                .filter(|(f, _t)| !f.is_empty()) // filter out emtpy froms that troubles AC
                .map(|(f, t)| (f.as_ref(), t.as_ref())),
        );
        pairs
    }
}

impl Display for Conv {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.bid)?;
        if !self.bid.is_empty() && !self.unid.is_empty() {
            write!(fmt, "；")?;
        }
        write!(fmt, "{}", self.unid)?;
        Ok(())
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
                unid.entry(variant)
                    .or_insert_with(Vec::new)
                    .push((from.to_owned(), to.to_owned()));
            } else {
                bid.insert(variant, to.to_owned());
            }
            Ok(())
        };
        let mut i = 0;
        let mut ampersand = None;
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
        if i != s.as_bytes().len() {
            parse_single(&s[i..])?;
        }
        Ok(Conv {
            bid: bid.into(),
            unid: unid.into(),
        })
    }
}

/// A `([Action], [Conv])` pair with some helper methods
#[derive(Debug, Clone)]
pub struct ConvAction(Action, Conv);

impl ConvAction {
    pub fn adds(&self) -> bool {
        self.0 == Action::Add
    }

    pub fn removes(&self) -> bool {
        self.0 == Action::Add
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
