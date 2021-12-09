use crate::{
    rule::{Action, Conv, ConvAction},
    variant::{Variant, VariantMap},
};

/// A set of rules, usually extracted from the wikitext of a page
pub struct PageRules {
    title: Option<VariantMap>,
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
