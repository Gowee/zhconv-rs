use std::convert::AsRef;
// use std::ffi::VaListImpl;
use std::collections::HashMap;
use std::fmt::{self, Display};
use std::str::FromStr;

use once_cell::sync::Lazy;
use regex::{Match, Matches, Regex};
use std::iter::Map;

use crate::variant::{Variant, VariantMap};

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
                    .map(|c| c.get_text_by_target(target))
                    .unwrap_or("")
            ), // unwrap?
            Some(Output::VariantName(variant)) => write!(dest, "{}", variant.get_name()), // TODO: correct format?
            // TODO: but mediawiki does not expect Unid when displaying description
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
                // output rule description
                'D' => {
                    output = Some(Output::Description);
                }
                // output nothing
                'H' => {
                    action = Some(Action::Add);
                    output = None;
                }
                'S' => {
                    // output as normal (by default)
                }
                'A' => {
                    // A implies +S
                    action = Some(Action::Add)
                }
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

// /// The inner of a [`ConvRule`] without flags and actions
// #[derive(Debug, Clone)]
// pub enum Conv {
//     //     /// Not a conversion, just return the inner rule as is, e.g. `-{简体字繁體字}-`
//     //     /// For flag `N`, this stores the right-side param in the raw format.
//     //     Verbatim(String),
//     /// Bi-directional mapping, e.g. `-{zh-hans:计算机; zh-hant:電腦;}-`.
//     Bid(VariantMap),
//     /// Uni-directional mapping, e.g. `-{H|巨集=>zh-cn:宏;}- `.
//     Unid(String, VariantMap),
// }

/// The inner of a [`ConvRule`] without flags and actions
#[derive(Debug, Clone)]
pub struct Conv {
    pub bid: VariantMap<String>,
    pub unid: VariantMap<Vec<(String, String)>>,
}

impl Conv {
    // #[inline(always)]
    // pub fn as_verbatim(&self) -> Option<&str> {
    //     match self {
    //         Conv::Verbatim(inner) => Some(inner),
    //         _ => None,
    //     }
    // }

    // #[inline(always)]
    // pub fn get_bid(&self) -> Option<&VariantMap> {
    //     self.0
    // }

    // #[inline(always)]
    // pub fn is_bid(&self) -> bool {
    //     self.as_bid().is_some()
    // }

    // #[inline(always)]
    // pub fn into_bid(self) -> Option<VariantMap> {
    //     match self {
    //         Conv::Bid(map) => Some(map),
    //         _ => None,
    //     }
    // }

    #[inline]
    /// The the text to display for the target variant
    pub fn get_text_by_target(&self, target: Variant) -> &str {
        self.bid.get_text_with_fallback(target).unwrap() // FIX:
                                                         // match self {
                                                         //     // &Conv::Verbatim(ref inner) => inner.as_ref(),
                                                         //     Conv::Bid(map) => map.get_text_with_fallback(target).unwrap(), // FIX:
                                                         //     Conv::Unid(_from, _map) => {
                                                         //         todo!() // Unid should not come with output
                                                         //     }
                                                         // }
    }

    #[inline]
    pub fn get_convs_by_target(&self, target: Variant) -> Vec<(&str, &str)> {
        // TODO: iterator
        let mut pairs = self.bid.get_convs_by_target(target);
        pairs.extend(
            self.unid
                .get_convs_by_target(target)
                .iter()
                .map(|(f, t)| (f.as_ref(), t.as_ref())),
        );
        pairs
        // match self {
        //     // &Conv::Verbatim(ref inner) => vec![(inner, inner)],
        //     Conv::Bid(map) => map.get_convs_by_target(target),
        //     Conv::Unid(from, map) => {
        //         if let Some(to) = map.get_text(target) {
        //             // TODO: fallback here?
        //             vec![(from, to)]
        //         } else {
        //             vec![]
        //         }
        //     }
        // }
    }
}

impl Display for Conv {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        // match self {
        //     // &Conv::Verbatim(ref inner) => fmt.write_str(inner),
        //     Conv::Bid(map) => map.fmt(fmt),
        //     Conv::Unid(from, ref map) => write!(fmt, "{} ⇒ {}", from, map),
        // }
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
        if s.is_empty() {
            // TODO: return?
        }

        let s = s.trim();
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
                    let (first, last) = s.split_at(i);
                    (Some(first), &last[2..])
                },
            );
            let (variant, to) = right.split_at(s.find(':').ok_or(())?);
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
            // match &s[i]
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

// A `([Action], [Conv])` pair with some helper methods
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

pub fn extract_rules<'s>(
    text: &'s str,
) -> Map<Matches<'static, 's>, impl FnMut(Match<'s>) -> Result<ConvRule, RuleError>> {
    // note: the regex works a little differently from the parser in converter
    (*REGEX_RULE).find_iter(text).map(|m| {
        let rule = m.as_str();
        dbg!(rule);
        dbg!(ConvRule::from_str(&rule[2..rule.len() - 2]))
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
