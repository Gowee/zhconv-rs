use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::iter::IntoIterator;

use std::str::FromStr;

use daachorse::{CharwiseDoubleArrayAhoCorasick, CharwiseDoubleArrayAhoCorasickBuilder, MatchKind};

use crate::tables::Table;
use crate::{
    pagerules::PageRules,
    rule::{Conv, ConvAction, ConvRule},
    tables::expand_table,
    utils::regex,
    variant::Variant,
};

// Ref: https://github.com/wikimedia/mediawiki/blob/7bf779524ab1fd8e1d74f79ea4840564d48eea4d/includes/language/LanguageConverter.php#L76
const NESTED_RULE_MAX_DEPTH: usize = 10;

/// A ZhConverter, built by [`ZhConverterBuilder`].
pub struct ZhConverter {
    variant: Variant,
    automaton: Option<CharwiseDoubleArrayAhoCorasick<u32>>,
    target_words: Vec<String>,
}

impl ZhConverter {
    /// Create a new converter from a automaton and a mapping.
    ///
    /// It is provided for convenience and not expected to be called directly.
    /// [`ZhConverterBuilder`] would take care of these
    /// details.
    pub fn new(
        automaton: CharwiseDoubleArrayAhoCorasick<u32>,
        target_words: Vec<String>,
    ) -> ZhConverter {
        ZhConverter {
            variant: Variant::Zh,
            automaton: Some(automaton),
            target_words,
        }
    }

    /// Create a new converter from a automaton and a mapping, as well as specifying a target
    /// variant to be used by [`convert_as_wikitext_basic`](Self::convert_as_wikitext_basic) and
    /// [`convert_as_wikitext_extended`](Self::convert_as_wikitext_extended) and related functions.
    ///
    /// It is provided for convenience and not expected to be called directly.
    /// [`ZhConverterBuilder`] would take care of these details.
    pub fn with_target_variant(
        automaton: CharwiseDoubleArrayAhoCorasick<u32>,
        target_words: Vec<String>,
        variant: Variant,
    ) -> ZhConverter {
        ZhConverter {
            variant,
            automaton: Some(automaton),
            target_words,
        }
    }

    /// Create a new converter of a sequence of `(from, to)` pairs.
    ///
    /// It use [`ZhConverterBuilder`] internally.
    #[inline(always)]
    pub fn from_pairs(
        pairs: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> ZhConverter {
        ZhConverterBuilder::new().conv_pairs(pairs).build()
    }

    /// Create a new converter of a sequence of `(from, to)` pairs.
    ///
    /// It takes a target variant to be used by [`convert_as_wikitext_basic`](Self::convert_as_wikitext_basic)
    /// and [`convert_as_wikitext_extended`](Self::convert_as_wikitext_extended) and related
    /// functions, in addition to [`from_pairs`](Self::from_pairs).
    ///
    /// It use [`ZhConverterBuilder`] internally.
    #[inline(always)]
    pub fn from_pairs_with_target_variant(
        variant: Variant,
        pairs: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> ZhConverter {
        ZhConverterBuilder::new()
            .target(variant)
            .conv_pairs(pairs)
            .build()
    }

    /// Convert a text.
    #[inline(always)]
    pub fn convert(&self, text: &str) -> String {
        let mut output = String::with_capacity(text.len());
        self.convert_to(text, &mut output);
        output
    }

    /// Same as `convert`, except that it takes a `&mut String` as dest instead of returning a `String`.
    pub fn convert_to(&self, text: &str, output: &mut String) {
        let automaton = match self.automaton.as_ref() {
            Some(automaton) => automaton,
            None => {
                output.push_str(text);
                return;
            }
        };

        // Ref: https://github.dev/rust-lang/regex/blob/5197f21287344d2994f9cf06758a3ea30f5a26c3/src/re_trait.rs#L192
        let mut last = 0;
        // let mut cnt = HashMap::<usize, usize>::new();
        // leftmost-longest matching
        for (s, e, ti) in automaton
            .leftmost_find_iter(text)
            .map(|m| (m.start(), m.end(), m.value()))
        {
            if s > last {
                output.push_str(&text[last..s]);
            }
            // *cnt.entry(text[s..e].chars().count()).or_insert(0) += 1;
            output.push_str(&self.target_words[ti as usize]);
            last = e;
        }
        output.push_str(&text[last..]);
    }

    /// Convert a text, a long with a secondary converter.
    ///
    /// Conversion rules in the secondary converter shadow these existing ones in the original
    /// converter.
    /// For example, if the original converter contains a rule `香菜 -> 芫荽`, and the the secondary
    /// converter contains a rule `香菜 -> 鹽須`, the latter would take effect and `香菜` is converted
    /// to `鹽須`.
    ///
    /// The implementation match the text against the two converter alternatively, resulting in
    /// degraded performance. It would be better to build a new converter that combines the
    /// rulesets of both the two, especially when the secondary rulsets are non-trivial or the
    /// input text is large.
    ///
    /// The worst-case time complexity of the implementation is `O(n*m)` where `n` and `m` are the
    /// length of the text and the maximum lengths of sources words in conversion rulesets (i.e.
    /// brute-force).
    #[inline(always)]
    pub fn convert_with_secondary_converter(
        &self,
        text: &str,
        secondary_converter: &ZhConverter,
    ) -> String {
        let mut output = String::with_capacity(text.len());
        self.convert_to_with_secondary_converter(text, &mut output, secondary_converter);
        output
    }

    /// Same as [`convert_to_with_secondary_converter`](Self::convert_to_with_secondary_converter), except
    /// that it takes a `&mut String` as dest instead of returning a `String`.
    pub fn convert_to_with_secondary_converter(
        &self,
        text: &str,
        output: &mut String,
        secondary_converter: &ZhConverter,
    ) {
        let ZhConverter {
            automaton: shadowing_automaton,
            target_words: shadowing_target_words,
            ..
        } = secondary_converter;
        match shadowing_automaton {
            Some(shadowing_automaton) => self.convert_to_with(
                text,
                output,
                Some(shadowing_automaton),
                shadowing_target_words.as_slice(),
                &Default::default(),
            ),
            None => self.convert_to(text, output),
        }
    }

    /// Convert a text, a long with a secondary conversion table (typically temporary).
    ///
    /// The worst-case time complexity of the implementation is `O(n*m)` where `n` and `m` are the
    /// length of the text and the maximum lengths of sources words in conversion rulesets.
    /// (i.e. brute-force).
    fn convert_to_with(
        &self,
        text: &str,
        output: &mut String,
        shadowing_automaton: Option<&CharwiseDoubleArrayAhoCorasick<u32>>,
        shadowing_target_words: &[String],
        shadowed_source_words: &HashSet<String>,
    ) {
        let automaton = match self.automaton.as_ref() {
            Some(automaton) => automaton,
            None => {
                output.push_str(text);
                return;
            }
        };

        // let mut cnt = HashMap::<usize, usize>::new();
        let mut last = 0;
        let mut left_match: Option<(usize, usize, &str)> = None;
        let mut right_match: Option<(usize, usize, &str)> = None;

        while last < text.len() {
            // leftmost-longest matching
            if left_match.is_none() || left_match.unwrap().0 < last {
                let m = automaton.leftmost_find_iter(&text[last..]).next();
                left_match = m.map(|m| {
                    (
                        last + m.start(),
                        last + m.end(),
                        self.target_words[m.value() as usize].as_str(),
                    )
                });
            }
            if right_match.is_none() || right_match.unwrap().0 < last {
                right_match = shadowing_automaton.and_then(|shadowing_automaton| {
                    shadowing_automaton
                        .leftmost_find_iter(&text[last..])
                        .next()
                        .map(|m| {
                            (
                                last + m.start(),
                                last + m.end(),
                                shadowing_target_words[m.value() as usize].as_str(),
                            )
                        })
                });
            }

            let (s, e, target_word) = match (left_match, right_match) {
                (Some(a), Some(b)) if a.0 > b.0 || (a.0 == b.0 && a.1 <= b.1) => b, // shadowed: pick a word in shadowing automaton
                (None, Some(b)) => b,                                               // ditto
                (Some(a), _) => {
                    // not shadowed: pick a word in original automaton
                    if shadowed_source_words.contains(a.2) {
                        // source word is disabled: skip one char and re-search
                        let first_char_len = text.chars().next().unwrap().len_utf8();
                        (
                            last,
                            a.0 + first_char_len,
                            &text[last..a.0 + first_char_len],
                        )
                    } else {
                        a
                    }
                }
                (None, None) => {
                    // end
                    output.push_str(&text[last..]);
                    break;
                }
            };
            if s > last {
                output.push_str(&text[last..s]);
            }
            // *cnt.entry(text[s..e].chars().count()).or_insert(0) += 1;
            output.push_str(target_word);
            last = e;
        }
    }

    /// Convert the given text, parsing and applying adhoc Mediawiki conversion rules in it.
    ///
    /// Basic MediaWiki conversion rules like `-{FOOBAR}-` or `-{zh-hant:FOO;zh-hans:BAR}-` are
    /// supported.
    ///
    /// Unlike [`convert_to_as_wikitext_extended`](Self::convert_to_as_wikitext_extended), rules
    /// with additional flags like `{H|zh-hant:FOO;zh-hans:BAR}` that sets global rules are simply
    /// ignored. And, it does not try to skip HTML code blocks like `<code></code>` and
    /// `<script></script>`.
    #[inline(always)]
    pub fn convert_as_wikitext_basic(&self, text: &str) -> String {
        let mut output = String::with_capacity(text.len());
        self.convert_to_as_wikitext_basic(text, &mut output);
        output
    }

    /// Convert the given text, parsing and applying adhoc and global MediaWiki conversion rules in
    /// it.
    ///
    /// Unlike [`convert_to_as_wikitext_basic`](Self::convert_to_as_wikitext_basic), all flags
    /// documented in [Help:高级字词转换语法](https://zh.wikipedia.org/wiki/Help:高级字词转换语法)
    /// are supported. And it tries to skip HTML code blocks such as `<code></code>` and
    /// `<script></script>`.
    ///
    /// # Limitations
    ///
    /// The internal implementation are intendedly replicating the behavior of
    /// [LanguageConverter.php](https://github.com/wikimedia/mediawiki/blob/7bf779524ab1fd8e1d74f79ea4840564d48eea4d/includes/language/LanguageConverter.php#L855)
    /// in MediaWiki. But it is not fully compliant with MediaWiki and providing NO PROTECTION over
    /// XSS attacks.
    ///
    /// Compared to the plain `convert`, this is known to be MUCH SLOWER due to the inevitable
    /// nature of the implementation decision made by MediaWiki.
    #[inline(always)]
    pub fn convert_as_wikitext_extended(&self, text: &str) -> String {
        let mut output = String::with_capacity(text.len());
        self.convert_to_as_wikitext_extended(text, &mut output);
        output
    }

    /// Same as [`convert_to_as_wikitext_basic`](Self::convert_to_as_wikitext_basic), except that
    /// it takes a `&mut String` as dest
    /// instead of returning a `String`.
    #[inline(always)]
    pub fn convert_to_as_wikitext_basic(&self, text: &str, output: &mut String) {
        self.convert_to_as_wikitext(text, output, &mut None, false, false)
    }

    /// Same as [`convert_to_as_wikitext_extended`](Self::convert_to_as_wikitext_extended), except
    /// that it takes a `&mut String` as dest instead of returning a `String`.
    #[inline(always)]
    pub fn convert_to_as_wikitext_extended(&self, text: &str, output: &mut String) {
        self.convert_to_as_wikitext(text, output, &mut None, true, true)
    }

    /// The general implementation of MediaWiki syntax-aware conversion.
    ///
    /// Equivalent to [`convert_as_wikitext_basic`](Self::convert_as_wikitext_basic) if
    /// `addtional_conv_lines` is set empty and both `skip_html_code_blocks` and
    /// `apply_global_rules` are set to `false`.
    ///
    /// Equivalent to [`convert_as_wikitext_extended`](Self::convert_as_wikitext_extended),
    /// otherwise.
    ///
    /// `addtional_conv_lines` looks like:
    /// ```text
    /// zh-cn:天堂执法者; zh-hk:夏威夷探案; zh-tw:檀島警騎2.0;
    /// zh-cn:史蒂芬·'史蒂夫'·麦格瑞特; zh-tw:史提夫·麥加雷; zh-hk:麥星帆;
    /// zh-cn:丹尼尔·'丹尼/丹诺'·威廉姆斯; zh-tw:丹尼·威廉斯; zh-hk:韋丹尼;
    /// ```
    #[inline(always)]
    pub fn convert_as_wikitext(
        &self,
        text: &str,
        secondary_converter_builder: &mut Option<ZhConverterBuilder>,
        skip_html_code_blocks: bool,
        apply_global_rules: bool,
    ) -> String {
        let mut output = String::with_capacity(text.len());
        self.convert_to_as_wikitext(
            text,
            &mut output,
            secondary_converter_builder,
            skip_html_code_blocks,
            apply_global_rules,
        );
        output
    }

    /// Same as [`convert_as_wikitext`](Self::convert_as_wikitext), except
    /// that it takes a `&mut String` as dest instead of returning a `String`.
    pub fn convert_to_as_wikitext(
        &self,
        text: &str,
        output: &mut String,
        secondary_converter_builder: &mut Option<ZhConverterBuilder>,
        skip_html_code_blocks: bool,
        apply_global_rules: bool,
    ) {
        // Ref: https://github.com/wikimedia/mediawiki/blob/7bf779524ab1fd8e1d74f79ea4840564d48eea4d/includes/language/LanguageConverter.php#L855
        //  and https://github.com/wikimedia/mediawiki/blob/7bf779524ab1fd8e1d74f79ea4840564d48eea4d/includes/language/LanguageConverter.php#L910
        //  and https://github.com/wikimedia/mediawiki/blob/7bf779524ab1fd8e1d74f79ea4840564d48eea4d/includes/language/LanguageConverter.php#L532

        #[allow(clippy::type_complexity)]
        let mut convert_to: Box<dyn Fn(&str, &mut String)> =
            Box::new(|text: &str, output: &mut String| self.convert_to(text, output));
        if secondary_converter_builder.is_some() || apply_global_rules {
            // build a secondary automaton from global rules specified in wikitext
            let mut builder = secondary_converter_builder.take().unwrap_or_default();
            if !builder.tables.is_empty() {
                panic!("The secondary converter builder should not load conversion tables");
            }
            // let mut shadowing_pairs = HashMap::new();
            let global_rules_in_page = PageRules::from_str(text).expect("infaillible");
            for ca in global_rules_in_page.as_conv_actions() {
                match ca.is_add() {
                    true => builder = builder.conv_pairs(ca.as_conv().get_conv_pairs(self.variant)),
                    false => {
                        builder = builder.unconv_pairs(ca.as_conv().get_conv_pairs(self.variant))
                    }
                }
            }
            let ZhConverter {
                automaton: shadowing_automaton,
                target_words: shadowing_target_words,
                ..
            } = builder.build();
            let shadowed_source_words: HashSet<String> = builder.removes.keys().cloned().collect();
            *secondary_converter_builder = Some(builder);
            if shadowing_automaton.is_some() || !shadowed_source_words.is_empty() {
                convert_to = Box::new(move |text: &str, output: &mut String| {
                    self.convert_to_with(
                        text,
                        output,
                        shadowing_automaton.as_ref(),
                        shadowing_target_words.as_slice(),
                        &shadowed_source_words,
                    )
                })
            }
        };

        // TODO: is this O(n) instead of O(n^2)?
        // start of rule | noHtml | noStyle | no code | no pre
        let sor_or_html = regex!(
            r#"-\{|<script.*?>.*?</script>|<style.*?>.*?</style>|<code>.*?</code>|<pre.*?>.*?</pre>"#
        );
        // start of rule
        let sor = regex!(r#"-\{"#);
        let pat_outer = if skip_html_code_blocks {
            sor_or_html
        } else {
            sor
        };
        // TODO: we need to understand what the hell it is so that to adapt it to compatible syntax
        // 		$noHtml = '<(?:[^>=]*+(?>[^>=]*+=\s*+(?:"[^"]*"|\'[^\']*\'|[^\'">\s]*+))*+[^>=]*+>|.*+)(*SKIP)(*FAIL)';
        let pat_inner = regex!(r#"-\{|\}-"#);

        let mut pos = 0;
        let mut pieces = vec![];
        while let Some(m1) = pat_outer.find_at(text, pos) {
            // convert anything before (possible) the toplevel -{
            convert_to(&text[pos..m1.start()], output);
            if m1.as_str() != "-{" {
                // not start of rule, just <foobar></foobar> to exclude
                output.push_str(&text[m1.start()..m1.end()]); // kept as-is
                pos = m1.end();
                continue; // i.e. <SKIP><FAIL>
            }
            // found toplevel -{
            pos = m1.start() + 2;
            pieces.push(String::new());
            while let Some(m2) = pat_inner.find_at(text, pos) {
                // let mut piece = String::from(&text[pos..m2.start()]);
                if m2.as_str() == "-{" {
                    // if there are two many open start tag, ignore the new nested rule
                    if pieces.len() >= NESTED_RULE_MAX_DEPTH {
                        pos += 2;
                        continue;
                    }
                    // start tag
                    pieces.last_mut().unwrap().push_str(&text[pos..m2.start()]);
                    pieces.push(String::new()); // e.g. -{ zh: AAA -{
                    pos = m2.end();
                } else {
                    // end tag
                    let mut piece = pieces.pop().unwrap();
                    piece.push_str(&text[pos..m2.start()]);
                    // only take it output; mutations to global rules are ignored
                    let r = ConvRule::from_str_infallible(&piece);
                    if let Some(upper) = pieces.last_mut() {
                        write!(upper, "{}", r.targeted(self.variant)).unwrap();
                    } else {
                        write!(output, "{}", r.targeted(self.variant)).unwrap();
                    };
                    // if let Ok(rule) = dbg!(ConvRule::from_str(&piece)) {
                    //     rule.write_output(upper, self.variant).unwrap();
                    // } else {
                    //     // rule is invalid
                    //     // TODO: what should we do actually? for now, we just do nothing to it
                    //     upper.push_str(&piece);
                    // }
                    pos = m2.end();
                    if pieces.is_empty() {
                        // return to toplevel
                        break;
                    }
                }
            }
            while let Some(piece) = pieces.pop() {
                output.push_str("-{");
                output.push_str(&piece);
            }
            // TODO: produce convert(&text[pos..])
        }
        if pos < text.len() {
            // no more conv rules, just convert and append
            convert_to(&text[pos..], output);
        }
    }

    // TODO: inplace? we need to maintain a stack which could be at most O(n)
    //       and it requires access to underlying bytes for subtle mutations
    // pub fn convert_inplace(&self, text: &mut String) {
    //     let tbp = VecDeque::<&str>::new(); // to be pushed
    //     let mut wi = 0; // writing index
    //     let mut ri = 0; // reading index
    //     while let Some((s, e)) = self.regex.find_at(text, ri).map(|m| (m.start(), m.end())) {
    //         while !tbp.is_empty() && s - wi >= tbp[0].len() {
    //             let raw = unsafe { text.as_bytes_mut() };
    //             raw[wi..wi + tbp[0].len()].clone_from_slice(tbp[0].as_bytes());
    //             tbp.pop_front();
    //         }
    //     }
    // }

    /// Search the text
    #[doc(hidden)]
    pub fn search<'s, 'i: 's>(
        &'i self,
        text: &'s str,
    ) -> impl Iterator<Item = (usize, usize, &'i str)> + 's {
        self.automaton
            .as_ref()
            .map(|automaton| {
                automaton.leftmost_find_iter(text).map(|m| {
                    (
                        m.start(),
                        m.end(),
                        self.target_words[m.value() as usize].as_ref(),
                    )
                })
            })
            .into_iter()
            .flatten()
    }

    /// Count the sum of lengths of source words to be replaced by the converter, in chars
    #[doc(hidden)]
    pub fn count_replaced(&self, text: &str) -> usize {
        self.search(text)
            .map(|(s, e, _to)| text[s..e].chars().count())
            .sum()
    }
}

/// A builder that helps build a [`ZhConverter`](ZhConverter).
///
/// # Example
/// Build a Zh2CN converter with some additional rules.
/// ```
/// use zhconv::{zhconv, ZhConverterBuilder, Variant, get_builtin_tables};
/// // extracted from https://zh.wikipedia.org/wiki/Template:CGroup/Template:CGroup/文學.
/// let rules = r"zh-hans:三个火枪手;zh-hant:三劍客;zh-tw:三劍客;
///                    zh-cn:雾都孤儿;zh-tw:孤雛淚;zh-hk:苦海孤雛;zh-sg:雾都孤儿;zh-mo:苦海孤雛;";
/// let converter = ZhConverterBuilder::new()
///                     .target(Variant::ZhCN)
///                     .tables(get_builtin_tables(Variant::ZhCN))
///                     .conv_lines(rules.lines())
///                     .build();
/// let original = "《三劍客》是亞歷山大·仲馬的作品。《孤雛淚》是查爾斯·狄更斯的作品。";
/// assert_eq!(converter.convert(original), "《三个火枪手》是亚历山大·仲马的作品。《雾都孤儿》是查尔斯·狄更斯的作品。");
/// assert_eq!(zhconv(original, Variant::ZhCN), "《三剑客》是亚历山大·仲马的作品。《孤雏泪》是查尔斯·狄更斯的作品。")
#[derive(Debug, Clone, Default)]
pub struct ZhConverterBuilder<'t> {
    target: Variant,
    /// The base conversion table
    tables: Vec<(&'t str, &'t str)>,
    /// Rules to be added, from page rules or cgroups
    adds: HashMap<String, String>,
    /// Rules to be removed, from page rules or cgroups
    removes: HashMap<String, String>, // TODO: unnecessary owned type
}

impl<'t> ZhConverterBuilder<'t> {
    pub fn new() -> Self {
        Default::default()
    }

    /// Shorthand for `ZhConverterBuilder::new()::target(variant)`.
    #[inline(always)]
    pub fn targeted(variant: Variant) -> Self {
        Self::new().target(variant)
    }

    /// Set the target Chinese variant to convert to.
    ///
    /// The target variant is only useful to get proper conv pairs from
    /// [`ConvRule`](crate::rule::ConvRule)s. That is, if only tables are specified, the target
    /// variant would be useless.
    pub fn target(mut self, variant: Variant) -> Self {
        self.target = variant;
        self
    }

    /// Add a conversion table, which is typically those in [`tables`](crate::tables).
    pub fn table(mut self, table: Table<'t>) -> Self {
        self.tables.push(table);
        self
    }

    /// Add a set of conversion tables, which are typically returned by [`get_builtin_tables`](crate::get_builtin_tables).
    pub fn tables(mut self, tables: &[Table<'t>]) -> Self {
        self.tables.extend(tables.iter());
        self
    }

    // /// [CGroup](https://zh.wikipedia.org/wiki/Module:CGroup) (a.k.a 公共轉換組)
    // pub fn cgroup()

    /// Add a set of rules extracted from a page in wikitext.
    ///
    /// This is a helper wrapper around `page_rules`.
    #[inline(always)]
    pub fn rules_from_page(self, text: &str) -> Self {
        self.page_rules(
            &PageRules::from_str(text).expect("Page rules parsing is infallible for now"),
        )
    }

    /// Add a set of rules from `PageRules`.
    #[inline(always)]
    pub fn page_rules(self, page_rules: &PageRules) -> Self {
        self.conv_actions(page_rules.as_conv_actions())
    }

    /// Add a set of rules.
    ///
    /// These rules take the higher precedence over those specified via `table`.
    fn conv_actions<'i>(mut self, conv_actions: impl IntoIterator<Item = &'i ConvAction>) -> Self {
        for conv_action in conv_actions {
            let pairs = conv_action.as_conv().get_conv_pairs(self.target);
            if conv_action.is_add() {
                self.adds
                    .extend(pairs.map(|(f, t)| (f.to_owned(), t.to_owned())));
            } else {
                self.removes
                    .extend(pairs.map(|(f, t)| (f.to_owned(), t.to_owned())));
            }
        }
        self
    }

    /// Add [`Conv`]s.
    ///
    /// For general cases, check [`add_conv_pair`](#method.add_conv_pair) which takes a plain
    /// `from -> to` pair.
    pub fn convs(mut self, convs: impl IntoIterator<Item = impl AsRef<Conv>>) -> Self {
        for conv in convs.into_iter() {
            self.adds.extend(
                conv.as_ref()
                    .get_conv_pairs(self.target)
                    .map(|(f, t)| (f.to_owned(), t.to_owned())),
            )
        }
        self
    }

    /// Mark a conv as removed.
    pub fn unconvs(mut self, convs: impl IntoIterator<Item = impl AsRef<Conv>>) -> Self {
        for conv in convs.into_iter() {
            self.removes.extend(
                conv.as_ref()
                    .get_conv_pairs(self.target)
                    .map(|(f, t)| (f.to_owned(), t.to_owned())),
            )
        }
        self
    }

    /// Add `from -> to` conversion pairs.
    ///
    /// It takes the precedence over those specified via `table`. It shares the same precedence level with those specified via `cgroup`.
    pub fn conv_pairs(
        mut self,
        pairs: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        for (from, to) in pairs {
            let (from, to) = (from.into(), to.into());
            debug_assert!(!from.is_empty(), "Conv pair should have non-empty from.");
            if from.is_empty() {
                continue;
            }
            self.adds.insert(from, to);
        }
        self
    }

    /// Mark conversion pairs as removed.
    ///
    /// Any rule with the same `from`, whether specified via `add_conv_pair`, `conv_lines` or `table`, is removed.
    pub fn unconv_pairs(
        mut self,
        pairs: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> Self {
        for (from, to) in pairs {
            let (from, to) = (from.into(), to.into());
            debug_assert!(!from.is_empty(), "Conv pair should have non-empty from.");
            if from.is_empty() {
                continue;
            }
            self.removes.insert(from, to);
        }
        self
    }

    /// Mark a single conversion pair as removed.
    ///
    /// Any rule with the same `from`, whether specified via `add_conv_pair`, `conv_lines` or `table`, is removed.
    pub fn unconv_pair(mut self, from: impl AsRef<str>, to: impl AsRef<str>) -> Self {
        self.removes
            .insert(from.as_ref().to_owned(), to.as_ref().to_owned());
        self
    }

    /// Add a text of conv lines.
    ///
    /// e.g.
    /// ```text
    /// zh-cn:天堂执法者; zh-hk:夏威夷探案; zh-tw:檀島警騎2.0;
    /// zh-cn:史蒂芬·'史蒂夫'·麦格瑞特; zh-tw:史提夫·麥加雷; zh-hk:麥星帆;
    /// zh-cn:丹尼尔·'丹尼/丹诺'·威廉姆斯; zh-tw:丹尼·威廉斯; zh-hk:韋丹尼;
    /// ```  
    pub fn conv_lines(mut self, lines: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        for line in lines.into_iter() {
            let line = line.as_ref().trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(conv) = Conv::from_str(line) {
                self.adds
                    .extend(conv.get_conv_pairs(self.target).map(|(f, t)| {
                        if f.is_empty() {
                            panic!("Conv pair should have non-empty from.")
                        }
                        (f.to_owned(), t.to_owned())
                    }));
            }
        }
        self
    }

    /// Do the build.
    ///
    /// It internally aggregate previously specified tables, rules and pairs, from where an
    /// automaton and a mapping are built, which are then feed into the new converter.
    pub fn build(&self) -> ZhConverter {
        let mapping = self.build_mapping();
        let mut target_words = vec![];
        let automaton = if !mapping.is_empty() {
            target_words.reserve_exact(mapping.len());
            let sequence = mapping.into_iter();
            Some(
                CharwiseDoubleArrayAhoCorasickBuilder::new()
                    .match_kind(MatchKind::LeftmostLongest)
                    .build(sequence.map(|(f, t)| {
                        target_words.push(t);
                        f
                    }))
                    .expect("Rules feed to DAAC already filtered"),
            )
        } else {
            None
        };

        ZhConverter {
            variant: self.target,
            automaton,
            target_words,
        }
    }

    /// Aggregate previously specified tables, rules and pairs to build a mapping.
    ///
    /// It is used by [`build`](Self::build) internally.
    pub fn build_mapping(&self) -> HashMap<String, String> {
        let Self {
            tables,
            adds,
            removes,
            ..
        } = self;
        // TODO: do we need a HashMap at all?
        let mut mapping = HashMap::with_capacity(
            (tables.iter().map(|(fs, _ts)| fs.len()).sum::<usize>() + adds.len())
                .saturating_sub(removes.len()),
        );
        mapping.extend(
            tables
                .iter()
                .flat_map(|&table| expand_table(table))
                .filter(|(from, to)| !(from.is_empty() && to.is_empty())) // empty str would trouble AC
                .filter(|(from, _to)| !removes.contains_key(from)),
        );
        mapping.extend(
            adds.iter()
                .filter(|(from, _to)| !removes.contains_key(from.as_str()))
                .map(|(from, to)| (from.to_owned(), to.to_owned())),
        );
        mapping
    }
}
