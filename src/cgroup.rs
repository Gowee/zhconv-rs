use crate::{
    rule::{Action, Conv, ConvAction},
    variant::{Variant, VariantMap},
};

/// A group of rules, corresponding to a [Module:CGroup](https://zh.wikipedia.org/wiki/Module:CGroup)
pub struct CGroup {
    title: Option<VariantMap>,
    conv_actions: Vec<ConvAction>,
}

impl CGroup {
    pub fn get_title(&self, target: Variant) -> Option<&str> {
        // MEDIAWIKI: fallback applies to title conversion
        self.title
            .as_ref()
            .and_then(|map| map.get_text_with_fallback(target))
    }

    pub fn as_conv_actions(&self) -> &[ConvAction] {
        &self.conv_actions
    }
}
