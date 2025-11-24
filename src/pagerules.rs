//! Struct to extract global rules from wikitext.
//!
//! **Note**: This module is exposed for convenience. It might have breaking changes at any time in
//!           violation of semver.

use std::str::FromStr;

use crate::{
    rule::{extract_rules, ConvAction},
    variant::{Variant, VariantMap},
};

/// A set of rules, usually extracted from the wikitext of a page
#[derive(Debug, Clone)]
pub struct PageRules {
    title: Option<VariantMap<String>>,
    conv_actions: Vec<ConvAction>,
}

impl PageRules {
    pub fn get_title(&self, target: Variant) -> Option<&str> {
        // MEDIAWIKI: fallback applies to title conversion
        self.title
            .as_ref()
            .and_then(|map| map.get_text_with_fallback(target))
    }

    pub fn as_conv_actions(&self) -> &[ConvAction] {
        &self.conv_actions
    }

    // pub fn iter_adds(&self) -> impl Iterator<Item=&'> {

    // }
}

impl FromStr for PageRules {
    type Err = (); // TODO: better error propagation

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // pages are not structured data, so it is normal see a lot of incompliant usage of rule
        // we just ignore them to ensure this function never return Err
        let mut title = None;
        let mut conv_actions = vec![];
        // or should be propogate the error?
        for rule in extract_rules(s).filter_map(|r| r.ok()) {
            if rule.set_title {
                if let Some(map) = rule.conv.as_ref().and_then(|conv| conv.get_bid()).cloned() {
                    // actually, our parser ensure this is !is_empty
                    // just be more tolerant here
                    if !map.is_empty() {
                        title = Some(map); // unwrap?
                    }
                }
            }
            // it is absolutely normal that not all rules are global
            if let Some(ca) = rule.into_conv_action() {
                conv_actions.push(ca);
            }
        }
        Ok(PageRules {
            title,
            conv_actions,
        })
    }
}
