use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use std::collections::HashMap;
use std::iter::IntoIterator;
use std::str::FromStr;

use itertools;
use once_cell::unsync::Lazy;
use regex::Regex;

use crate::{
    pagerules::PageRules,
    rule::{Conv, ConvAction, ConvRule},
    variant::Variant,
};

// Ref: https://github.com/wikimedia/mediawiki/blob/7bf779524ab1fd8e1d74f79ea4840564d48eea4d/includes/language/LanguageConverter.php#L76
const NESTED_RULE_MAX_DEPTH: usize = 10;

pub struct ZhConverter {
    variant: Variant,
    automaton: AhoCorasick,
    mapping: HashMap<String, String>,
}

impl ZhConverter {
    pub fn new(automaton: AhoCorasick, mapping: HashMap<String, String>) -> ZhConverter {
        ZhConverter {
            variant: Variant::Zh,
            automaton,
            mapping,
        }
    }

    #[inline(always)]
    pub fn from_pairs(pairs: &[(impl AsRef<str>, impl AsRef<str>)]) -> ZhConverter {
        let mut builder = ZhConverterBuilder::new();
        for (from, to) in pairs {
            builder = builder.add_conv_pair(from.to_owned(), to.to_owned());
        }
        builder.build()
    }

    /// Convert a text
    pub fn convert(&self, text: &str) -> String {
        let mut output = String::with_capacity(text.len());
        self.converted(text, &mut output);
        output
    }

    /// Same as [`convert`], except that it takes a `&mut String` as dest instead of returning a `String`
    pub fn converted(&self, text: &str, output: &mut String) {
        // Ref: https://github.dev/rust-lang/regex/blob/5197f21287344d2994f9cf06758a3ea30f5a26c3/src/re_trait.rs#L192
        let mut last = 0;
        let mut cnt = HashMap::<usize, usize>::new();
        // leftmost-longest matching
        for (s, e) in self.automaton.find_iter(text).map(|m| (m.start(), m.end())) {
            if s > last {
                output.push_str(&text[last..s]);
            }
            // if text[s..e].chars().count() > 2{
            *cnt.entry(text[s..e].chars().count()).or_insert(0) += 1;
            // }
            output.push_str(self.mapping.get(&text[s..e]).unwrap());
            last = e;
        }
        dbg!(cnt);
        output.push_str(&text[last..]);
    }

    /// Convert a text with inline conv rules parsed
    ///
    /// It only processes the display output of inline rules. Mutations to global rules specified
    /// via inline rules are just ignored.
    pub fn convert_allowing_inline_rules(&self, text: &str) -> String {
        // Ref: https://github.com/wikimedia/mediawiki/blob/7bf779524ab1fd8e1d74f79ea4840564d48eea4d/includes/language/LanguageConverter.php#L855
        //  and https://github.com/wikimedia/mediawiki/blob/7bf779524ab1fd8e1d74f79ea4840564d48eea4d/includes/language/LanguageConverter.php#L910
        let p1 = Lazy::new(|| Regex::new(r#"-\{"#).unwrap()); // TODO: exclude html
        let p2 = Lazy::new(|| Regex::new(r#"-\{|\}-"#).unwrap());
        let mut pos = 0;
        let mut converted = String::with_capacity(text.len());
        let mut pieces = vec![];
        while let Some(m1) = p1.find_at(text, pos) {
            // convert anything before the (possible) toplevel -{
            self.converted(&text[pos..m1.start()], &mut converted);
            if m1.as_str() != "-{" {
                // not start tag, just something to exclude
                converted.push_str(&text[m1.start()..m1.end()]); // TODO: adapt to nohtml
                pos = m1.end();
                continue;
            }
            // found toplevel -{
            pos = m1.start() + 2;
            pieces.push(String::new());
            while let Some(m2) = p2.find_at(text, pos) {
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
                    let upper = if let Some(upper) = pieces.last_mut() {
                        upper
                    } else {
                        &mut converted
                    };
                    if let Ok(rule) = ConvRule::from_str(&piece) {
                        // only take it output; mutations to global rules are ignored
                        rule.write_output(upper, self.variant).unwrap();
                    } else {
                        // rule is invalid
                        // TODO: what should we do actually? for now, we just do nothing to it
                        upper.push_str(&piece);
                    }
                    pos = m2.end();
                    if pieces.is_empty() {
                        // return to toplevel
                        break;
                    }
                }
            }
            while let Some(piece) = pieces.pop() {
                converted.push_str("-{");
                converted.push_str(&piece);
            }
            // TODO: produce convert(&text[pos..])
        }
        if pos < text.len() {
            // no more conv rules, just convert and append
            converted.push_str(&self.convert(&text[pos..]));
        }
        converted
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
}

#[derive(Debug, Clone, Default)]
pub struct ZhConverterBuilder<'t> {
    target: Variant,
    /// The base conversion table
    tables: Vec<(&'t str, &'t str)>,
    /// Rules to be added, from page rules or cgroups
    adds: HashMap<String, String>,
    /// Rules to be removed, from page rules or cgroups
    removes: HashMap<String, String>, // TODO: unnecessary owned type
    dfa: bool,
}

impl<'t> ZhConverterBuilder<'t> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn target(mut self, variant: Variant) -> Self {
        self.target = variant;
        self
    }

    // Add a conversion table
    pub fn table(mut self, table: (&'t str, &'t str)) -> Self {
        self.tables.push(table);
        self
    }

    //  [CGroup](https://zh.wikipedia.org/wiki/Module:CGroup) (a.k.a 公共轉換組)

    /// Add a set of rules extracted from a page
    ///
    /// This is a helper wrapper around `page_rules`.
    #[inline(always)]
    pub fn rules_from_page(self, text: &str) -> Self {
        self.page_rules(
            &PageRules::from_str(text).expect("Page rules parsing in infallible for now"),
        )
    }

    /// Add a set of rules from `PageRules`
    ///
    /// A helper wrapper around `conv_actions`. These rules take the higher precedence over those
    /// specified via `table`.
    #[inline(always)]
    pub fn page_rules(self, page_rules: &PageRules) -> Self {
        self.conv_actions(page_rules.as_conv_actions())
    }

    /// Add a set of rules
    ///
    /// These rules take the higher precedence over those specified via `table`.
    fn conv_actions<'i>(mut self, conv_actions: impl IntoIterator<Item = &'i ConvAction>) -> Self {
        for conv_action in conv_actions {
            let pairs = conv_action.as_conv().get_convs_by_target(self.target);
            if conv_action.adds() {
                self.adds
                    .extend(pairs.iter().map(|&(f, t)| (f.to_owned(), t.to_owned())));
            } else {
                self.removes
                    .extend(pairs.iter().map(|&(f, t)| (f.to_owned(), t.to_owned())));
            }
        }
        self
    }

    /// Add a single conversion pair
    ///
    /// It takes the precedence over those specified via `table`. It shares the same precedence level with those specified via `cgroup`.
    pub fn add_conv_pair(mut self, from: impl AsRef<str>, to: impl AsRef<str>) -> Self {
        self.adds
            .insert(from.as_ref().to_owned(), to.as_ref().to_owned());
        self
    }

    /// Remove a single conversion pair
    ///
    /// Any rule with the same `from`, whether specified via `add_conv`, `cgroup` or `table`, is removed.
    pub fn remove_conv_pair(mut self, from: impl AsRef<str>, to: impl AsRef<str>) -> Self {
        self.removes
            .insert(from.as_ref().to_owned(), to.as_ref().to_owned());
        self
    }

    /// Add a text of conv lines
    ///
    /// e.g.
    /// ```
    /// zh-cn:天堂执法者; zh-hk:夏威夷探案; zh-tw:檀島警騎2.0;
    /// zh-cn:史蒂芬·'史蒂夫'·麦格瑞特; zh-tw:史提夫·麥加雷; zh-hk:麥星帆;
    /// zh-cn:丹尼尔·'丹尼/丹诺'·威廉姆斯; zh-tw:丹尼·威廉斯; zh-hk:韋丹尼;
    /// ```
    pub fn conv_lines(mut self, lines: &str) -> Self {
        for line in lines.lines().map(str::trim).filter(|s| !s.is_empty()) {
            if let Ok(conv) = Conv::from_str(line.trim()) {
                self.adds.extend(
                    conv.get_convs_by_target(self.target)
                        .iter()
                        .map(|&(f, t)| (f.to_owned(), t.to_owned())),
                );
            }
        }
        self
    }

    /// Set whether to activate the DFA of Aho-Corasick.
    ///
    /// With DFA enabled, it takes rougly 5x time to build the converter while the conversion
    /// speed is < 2x faster.
    pub fn dfa(mut self, enabled: bool) -> Self {
        self.dfa = enabled;
        self
    }

    pub fn build(self) -> ZhConverter {
        let Self {
            target,
            tables,
            dfa,
            adds,
            removes,
        } = self;
        let mut mapping = HashMap::with_capacity(
            (tables.iter().map(|(fs, _ts)| fs.len()).sum::<usize>() + adds.len())
                .saturating_sub(removes.len()),
        );
        mapping.extend(
            tables
                .into_iter()
                .map(|(froms, tos)| itertools::zip(froms.trim().split('|'), tos.trim().split('|')))
                .flatten()
                .filter(|&(from, to)| !(from.is_empty() && to.is_empty())) // empty str will trouble AC
                .filter(|&(from, _to)| !removes.contains_key(from))
                .map(|(from, to)| (from.to_owned(), to.to_owned())),
        );
        mapping.extend(
            adds.into_iter()
                .filter(|(from, _to)| !removes.contains_key(from)), // .map(|(from, to)| (from.to_owned(), to.to_owned())),
        );
        let sequence = mapping.keys();
        let automaton = AhoCorasickBuilder::new()
            .match_kind(MatchKind::LeftmostLongest)
            .dfa(dfa)
            .build(sequence);
        ZhConverter {
            variant: target,
            mapping,
            automaton,
        }
    }
}
