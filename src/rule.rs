use std::convert::AsRef;
use std::fmt::{self, Display};
use std::str::FromStr;

use once_cell::sync::Lazy;
use regex::{Match, Matches, Regex};
use std::iter::Map;

use crate::{
    utils::split_once,
    variant::{Variant, VariantMap},
};

/// A single rule used for language conversion, usually extracted from wikitext in the syntax `-{ }-`, as documented in [ConverterRule.php](https://doc.wikimedia.org/mediawiki-core/master/php/ConverterRule_8php.html)

// pub enum ConvFlag {
//     /// Set up a rule without displaying anything (**H**idden), when used with [Action::Add].
//     H,
//     /// Display the **D**escription of the rule.
//     D,
//     /// Display the rule content as-is without parsing anything inside (**R**aw).
//     R,
//     ///
//     S,
//     /// Convert **T**itle of some an article
//     T,
// }

#[derive(Debug, Clone)]
pub struct ConvRule {
    action: Option<Action>,
    output: Option<Output>,
    conv: Conv,
    set_title: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Output {
    Normal,
    VariantName,
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
}

impl ConvRule {
    pub fn write_output(&self, mut dest: impl fmt::Write, target: Variant) -> fmt::Result {
        match self.output {
            None => Ok(()),
            Some(Output::Normal) => write!(dest, "{}", self.conv.get_text_by_target(target)),
            Some(Output::VariantName) => write!(dest, "{}", target), // TODO: correct format?
            // TODO: but mediawiki does not expect Unid when displaying description
            Some(Output::Description) => write!(dest, "{}", self.conv),
        }
    }
}

impl FromStr for ConvRule {
    type Err = RuleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (flags, conv) = s.find('|').map_or_else(
            || ("", s),
            |i| {
                let (first, last) = s.split_at(i);
                (first, &last[1..])
            },
        );
        if flags.is_empty() {
            return Ok(ConvRule {
                action: None,
                output: Some(Output::Normal),
                conv: Conv::from_str(conv).map_err(|_| RuleError::InvalidConv)?,
                set_title: false,
            });
            // return Ok(ConvRule {set_title: false, action: None, output: Output::Verbatim});
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
                // no conv, just output the inner as-is
                'R' => {
                    return Ok(ConvRule {
                        action: None,
                        output: Some(Output::Normal),
                        conv: Conv::from_str(conv).map_err(|_| RuleError::InvalidConv)?,
                        set_title: false,
                    });
                }
                // output the variant name of the context
                'N' => {
                    output = Some(Output::VariantName);
                }
                // output rule description
                'D' => {
                    output = Some(Output::Description);
                }
                // output nothing
                'H' => {
                    output = None;
                }
                // output as normal (by default)
                'S' => {}
                'T' => {
                    set_title = true;
                }
                unknown => return Err(RuleError::InvalidFlag(unknown)),
            }
        }
        let conv = Conv::from_str(conv).map_err(|_| RuleError::InvalidConv)?;
        Ok(Self {
            action,
            output,
            conv,
            set_title,
        })
    }
}

/// The inner of a [`ConvRule`] without flags and actions
#[derive(Debug, Clone)]
pub enum Conv {
    /// Not a conversion, just return the inner rule as is, e.g. `-{简体字繁體字}-`
    Verbatim(String),
    /// Bi-directional mapping, e.g. `-{zh-hans:计算机; zh-hant:電腦;}-`.
    Bid(VariantMap),
    /// Uni-directional mapping, e.g. `-{H|巨集=>zh-cn:宏;}- `.
    Unid(String, VariantMap),
}

impl Conv {
    pub fn get_text_by_target(&self, target: Variant) -> &str {
        match self {
            &Conv::Verbatim(ref inner) => inner.as_ref(),
            &Conv::Bid(ref map) => map.get_text_with_fallback(target).unwrap(), // FIX:
            &Conv::Unid(ref from, ref map) => {
                todo!() // Unid should come with output
            }
        }
    }

    pub fn get_convs_by_target(&self, target: Variant) -> Vec<(&str, &str)> {
        match self {
            &Conv::Verbatim(ref inner) => vec![(inner, inner)],
            &Conv::Bid(ref map) => map.get_convs_by_target(target),
            &Conv::Unid(ref from, ref map) => {
                if let Some(to) = map.get_text(target) {
                    vec![(from, to)]
                } else {
                    vec![]
                }
            }
        }
    }
}

impl Display for Conv {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Conv::Verbatim(ref inner) => fmt.write_str(inner),
            &Conv::Bid(ref map) => map.fmt(fmt),
            &Conv::Unid(ref from, ref map) => write!(fmt, "{} ⇒ {}", from, map),
        }
    }
}

impl FromStr for Conv {
    type Err = ();

    fn from_str(s: &str) -> Result<Conv, Self::Err> {
        if s.is_empty() {
            // return
        }
        let (left, right) = s.find("=>").map_or_else(
            || (None, s),
            |i| {
                let (first, last) = s.split_at(i);
                (Some(first), &last[1..])
            },
        );

        match (
            left,
            right
                .parse::<VariantMap>()
                .ok()
                .and_then(|m| if m.is_empty() { None } else { Some(m) }),
        ) {
            (Some(from), Some(map)) => Ok(Conv::Unid(from.to_owned(), map)), // this allow -{FOO => }-
            (None, Some(map)) => Ok(Conv::Bid(map)),
            (None, None) => Ok(Conv::Verbatim(s.to_owned())), // FIX: ?
            (Some(_), None) => {
                Err(())
                // TODO: treat as valid? e.g. -{FOO => BAR}-
                // Conv::Unid(from.to_owned(), VariantMap::from())
            }
        }
    }
}

// A `([Action], [Conv])` with some helper methods
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

pub fn extract_rules<'s>(
    text: &'s str,
) -> Map<Matches<'static, 's>, impl FnMut(Match<'s>) -> Result<ConvRule, RuleError>> {
    (*REGEX_RULE).find_iter(text).map(|m| {
        let rule = m.as_str();
        ConvRule::from_str(&rule[2..rule.len() - 4])
    })
}

// pub trait CGroup: Iterator<Item=ConvRule> {
//     fn aggregate() -> (String, Vec<>
// }

// /// An iterator that yields [`ConvRule`], returned by [`extract_rules`]
// pub struct Rules<'s> {
//     text: &'s str,

// }

// impl<'s> Iterator for Rules<'s> {
//     fn next(&mut self) -> Option<Result<ConvRule, ConvError>> {

//     }
// }

// pub enum ConvRule {
//     Verbatim(String),
//     NonVerbatim(NonVerbatimRule),
// }

// pub struct NonVerbatimRule {
//     set_title: bool,
//     action: Option<Action>,
//     output: Option<Output>,
//     conv: Conv,
// }

// pub enum Output {
//     Normal,
//     VariantName,
//     Description,
// }

// pub enum Action {
//     Add,
//     Remove,
// }

// pub enum RuleError {
//     InvalidFlag(char),
//     InvalidConv,
// }

// impl FromStr for NonVerbatimRule {
//     type Err = RuleError;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let (flags, conv) = s.find('|').map_or_else(
//             || ("", s),
//             |i| {
//                 let (first, last) = s.split_at(i);
//                 (first, &last[1..])
//             },
//         );
//         if flags.is_empty() {
//             return Ok(NonVerbatimRule {
//                 set_title: false,
//                 action: None,
//                 output: Some(Output::Normal),
//                 conv: Conv::from_str(conv).map_err(|_| RuleError::InvalidConv)?,
//             });
//             // return Ok(ConvRule {set_title: false, action: None, output: Output::Verbatim});
//         }
//         let mut action = None;
//         for flag in flags.chars() {
//             match flag {
//                 '+' => {
//                     action = Some(Action::Add)
//                 }
//                 '-' => {
//                     action = Some(Action::Remove)
//                 },
//                 'R'
//                 // 'R' =>

//             }
//         }

//         let conv = Conv::from_str(conv).map_err(|_| RuleError::InvalidConv)?;
//     }
// }

// /// The inner of a [`ConvRule`] without flags and actions
// pub enum Conv {
//     // /// Not a conversion, just return the inner rule as is, e.g. `-{简体字繁體字}-`
//     // Verbatim(String),
//     /// Bi-directional mapping, e.g. `-{zh-hans:计算机; zh-hant:電腦;}-`.
//     Bid(VariantMap),
//     /// Uni-directional mapping, e.g. `-{H|巨集=>zh-cn:宏;}- `.
//     Unid(String, VariantMap),
// }

// impl FromStr for Conv {
//     type Err = ();

//     fn from_str(s: &str) -> Result<Conv, Self::Err> {
//         if s.is_empty() {
//             // return
//         }
//         let (left, right) = s.find("=>").map_or_else(
//             || (None, s),
//             |i| {
//                 let (first, last) = s.split_at(i);
//                 (Some(first), &last[1..])
//             },
//         );

//         match (
//             left,
//             right
//                 .parse::<VariantMap>()
//                 .ok()
//                 .and_then(|m| if m.is_empty() { None } else { Some(m) }),
//         ) {
//             (Some(from), Some(map)) => Ok(Conv::Unid(from.to_owned(), map)), // this allow -{FOO => }-
//             (None, Some(map)) => Ok(Conv::Bid(map)),
//             (_, _) => Err(()),
//             // (Some(_), None) => {
//             //     Err(())
//             //     // TODO: treat as valid? e.g. -{FOO => BAR}-
//             //     // Conv::Unid(from.to_owned(), VariantMap::from())
//             // }
//         }
//     }
// }

// static REGEX_RULE: Lazy<Regex> = Lazy::new(|| Regex::new(r"-\{.+?\}-").unwrap());

// pub fn extract_rules<'s>(
//     text: &'s str,
// ) -> Map<Matches<'static, 's>, impl FnMut(Match<'s>) -> Result<ConvRule, RuleError>> {
//     (*REGEX_RULE).find_iter(text).map(|m| {
//         let rule = m.as_str();
//         ConvRule::from_str(&rule[2..rule.len() - 4])
//     })
// }

// // /// An iterator that yields [`ConvRule`], returned by [`extract_rules`]
// // pub struct Rules<'s> {
// //     text: &'s str,

// // }

// // impl<'s> Iterator for Rules<'s> {
// //     fn next(&mut self) -> Option<Result<ConvRule, ConvError>> {

// //     }
// // }
