use std::collections::{BTreeMap, HashMap};
use std::iter::IntoIterator;
use std::str::FromStr;

use itertools::{self, Itertools};
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
    regex: Regex,
    mapping: HashMap<String, String>, // Or str?
                                      // cgroups: Vec<Cgroup>
                                      // mediawiki: bool
}

impl ZhConverter {
    pub fn new(regex: Regex, mapping: HashMap<String, String>) -> ZhConverter {
        ZhConverter {
            variant: Variant::Zh,
            regex,
            mapping,
        }
    }

    // pub fn set_cgroups()

    #[inline]
    pub fn from_pairs(mut pairs: Vec<(String, String)>) -> ZhConverter {
        pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        let size_hint = (pairs.len() * (3 + 1)).saturating_sub(1); // TODO: correct?; 3: 3bytes / CJK characters in usual; 1 for |
        Self::from_pairs_sorted(
            Variant::Zh, /* TODO: */
            pairs.into_iter().map(|(from, to)| (from, to)),
            size_hint,
        )
    }

    pub fn from_pairs_sorted(
        target: Variant,
        pairs: impl Iterator<Item = (impl AsRef<str>, impl AsRef<str>)>,
        size_hint: usize,
    ) -> ZhConverter {
        // TODO: panic if unsorted
        // FIX: mediawiki
        let mut pat = String::with_capacity(size_hint);
        let mut mapping = HashMap::new();
        let mut it = pairs.into_iter().peekable();
        while let Some((from, to)) = it.next() {
            let (from, to) = (from.as_ref(), to.as_ref());
            pat.push_str(from);
            if it.peek().is_some() {
                pat.push('|');
            }
            mapping.insert(from.to_owned(), to.to_owned());
        }
        ZhConverter {
            variant: target,
            regex: Regex::new(&pat).unwrap(),
            mapping,
        }
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
        for (s, e) in self.regex.find_iter(text).map(|m| (m.start(), m.end())) {
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
        // let mut starts = vec![];
        let mut pieces = vec![];
        // loop {
        while let Some(m1) = p1.find_at(text, pos) {
            // convert anything before the (possible) toplevel -{
            // TODO: pass &mut String in
            converted.push_str(&self.convert(&text[pos..m1.start()]));
            if m1.as_str() != "-{" {
                // not start tag, just something to exclude
                converted.push_str(&text[m1.start()..m1.end()]); // TODO: adapt to nohtml
                pos = m1.end();
                continue;
            }
            // found toplevel -{
            pos = m1.start() + 2;
            // starts.push(m1.start());
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
                    // starts.push(m2.start());
                    pieces.last_mut().unwrap().push_str(&text[pos..m2.start()]);
                    pieces.push(String::new()); // e.g. -{ zh: AAA -{
                    pos = m2.end();
                } else {
                    // end tag
                    // let start = starts.pop().unwrap();
                    let mut piece = pieces.pop().unwrap();
                    piece.push_str(&text[pos..m2.start()]);
                    let upper = if let Some(upper) = pieces.last_mut() {
                        upper
                    } else {
                        &mut converted
                    };
                    if let Ok(rule) = ConvRule::from_str(&piece) {
                        // just take it output; mutations to global rules are ignored
                        rule.write_output(upper, self.variant).unwrap();
                    } else {
                        // rule is invalid
                        // TODO: what should we do actually?
                        // for now, we just do nothing to it
                        upper.push_str(&piece);
                    }
                    pos = m2.end();
                    if pieces.is_empty() {
                        // return to toplevel
                        break;
                    }
                    // starts.last().unwrap()
                }
            }
            while let Some(piece) = pieces.pop() {
                // let piece = pieces.pop().unwrap();
                converted.push_str("-{");
                converted.push_str(&piece);
            }
            // TODO: produce convert(&text[pos..])
        }
        if pos < text.len() {
            // no more conv rules, just convert and append
            converted.push_str(&self.convert(&text[pos..]));
        }
        // }
        // unimplemented!();
        converted
    }

    // pub fn convert

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
pub struct ZhConverterBuilder<'t, 'c> {
    target: Variant,
    tables: Vec<(&'t str, &'t str)>,
    texts: Vec<&'c str>,
    adds: BTreeMap<&'c str, &'c str>,
    removes: BTreeMap<&'c str, &'c str>,
}

impl<'t, 'c> ZhConverterBuilder<'t, 'c> {
    pub fn new(target: Variant) -> Self {
        Self {
            target,
            ..Default::default()
        }
    }

    // Add a conversion table
    pub fn table(mut self, table: (&'t str, &'t str)) -> Self {
        self.tables.push(table);
        self
    }

    //  [CGroup](https://zh.wikipedia.org/wiki/Module:CGroup) (a.k.a 公共轉換組)

    /// Add a set of rules extracted from a page
    ///
    /// A helper wrapper around `conv_actions`. These rules take the higher precedence over those
    /// specified via `table`.
    #[inline(always)]
    pub fn page_rules(self, page_rules: &'c PageRules) -> Self {
        self.conv_actions(page_rules.as_conv_actions())
    }

    /// Add a set of rules
    ///
    /// These rules take the higher precedence over those specified via `table`.
    pub fn conv_actions(mut self, conv_actions: impl IntoIterator<Item = &'c ConvAction>) -> Self {
        for conv_action in conv_actions {
            dbg!(&conv_action);
            let map = if conv_action.adds() {
                &mut self.adds
            } else {
                &mut self.removes
            };
            let conv = conv_action.as_conv();
            for (from, to) in conv.get_convs_by_target(self.target).into_iter() {
                map.insert(from, to);
            }
            dbg!(&map);
        }
        self
    }

    /// Add a single conversion pair
    ///
    /// It takes the precedence over those specified via `table`. It shares the same precedence level with those specified via `cgroup`.
    pub fn add_conv(mut self, from: &'c str, to: &'c str) -> Self {
        self.adds.insert(from, to);
        self
    }

    /// Remove a single conversion pair
    ///
    /// Any rule with the same `from`, whether specified via `add_conv`, `cgroup` or `table`, is removed.
    pub fn remove_conv(mut self, from: &'c str, to: &'c str) -> Self {
        self.removes.insert(from, to);
        self
    }

    /// Add a text of conv lines
    ///
    /// e.g.
    /// ```
    /// zh-cn:天堂执法者; zh-hk:夏威夷探案; zh-tw:檀島警騎2.0;
    /// zh-cn:史蒂芬·'史蒂夫'·麦格瑞特; zh-tw:史提夫·麥加雷; zh-hk:麥星帆;
    /// zh-cn:丹尼尔·'丹尼/丹诺'·威廉姆斯; zh-tw:丹尼·威廉斯; zh-hk:韋丹尼;
    ///     /// ```
    pub fn conv_lines(mut self, lines: &'c str) -> Self {
        self.texts.push(lines);
        self
    }

    pub fn build(self) -> ZhConverter {
        let Self {
            target,
            tables,
            texts,
            adds,
            removes,
        } = self;
        dbg!(&adds, &removes);

        let mut convs: Vec<(String, String)> = Vec::new();
        for text in texts {
            for line in text.lines() {
                convs.extend(
                    line.parse::<Conv>()
                        .expect("Valid conversion")
                        .get_convs_by_target(target)
                        .into_iter()
                        .map(|(f, t)| (f.to_owned(), t.to_owned())),
                );
            }
        }

        let size_hint = (tables
            .iter()
            .map(|&table| table.0.len() + 1)
            .sum::<usize>()
            .saturating_sub(1)
            + if adds.is_empty() {
                0
            } else {
                (1 + adds.len() * 4).saturating_sub(1)
            })
        .saturating_sub((removes.len() * 4).saturating_sub(1));
        let cmp_fn = |&pair1: &(&str, &str), &pair2: &(&str, &str)| pair1.0.len() >= pair2.0.len();
        let it = itertools::kmerge_by(
            tables
                .into_iter() // earlier tables have greater precedence
                .map(|(froms, tos)| itertools::zip(froms.trim().split('|'), tos.trim().split('|'))),
            cmp_fn,
        )
        .merge_by(adds.into_iter().map(|(from, to)| (from, to)), cmp_fn)
        .dedup_by(|pair1, pair2| pair1.0 == pair2.0)
        .filter(|(from, _to)| !removes.contains_key(from));
        // TODO: GROUP > tables
        ZhConverter::from_pairs_sorted(target, it, size_hint)
    }
}
