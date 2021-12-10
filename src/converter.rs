use std::collections::{BTreeMap, HashMap};
use std::iter::Iterator;
use std::str::FromStr;

use itertools::{self, Itertools};
use once_cell::unsync::Lazy;
use regex::Regex;

use crate::{
    pagerules::PageRules,
    rule::{ConvAction, ConvRule},
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
        let size_hint = pairs.len() * 3 * 2 + 1; // TODO: correct?; 3: 3bytes / CJK characters in usual; 2: pair
        Self::from_pairs_sorted(pairs.into_iter().map(|(from, to)| (from, to)), size_hint)
    }

    pub fn from_pairs_sorted<'i>(
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
            pat.push_str(&from);
            if it.peek().is_some() {
                pat.push_str("|");
            }
            mapping.insert(from.to_owned(), to.to_owned());
        }
        ZhConverter {
            variant: Variant::Zh, // TODO:
            regex: Regex::new(&pat).unwrap(),
            mapping,
        }
    }

    pub fn convert(&self, text: &str) -> String {
        // Ref: https://github.dev/rust-lang/regex/blob/5197f21287344d2994f9cf06758a3ea30f5a26c3/src/re_trait.rs#L192
        let mut converted = String::with_capacity(text.len());
        let mut last = 0;
        let mut cnt = HashMap::<usize, usize>::new();
        // leftmost-longest matching
        for (s, e) in self.regex.find_iter(text).map(|m| (m.start(), m.end())) {
            if s > last {
                converted.push_str(&text[last..s]);
            }
            // if text[s..e].chars().count() > 2{
            *cnt.entry(text[s..e].chars().count()).or_insert(0) += 1;
            // }
            converted.push_str(&self.mapping.get(&text[s..e]).unwrap());
            last = e;
        }
        dbg!(cnt);
        converted.push_str(&text[last..]);
        converted
    }

    pub fn convert_allowing_inline_rules(&self, text: &str) -> String {
        // Ref: https://github.com/wikimedia/mediawiki/blob/7bf779524ab1fd8e1d74f79ea4840564d48eea4d/includes/language/LanguageConverter.php#L855
        //  and https://github.com/wikimedia/mediawiki/blob/7bf779524ab1fd8e1d74f79ea4840564d48eea4d/includes/language/LanguageConverter.php#L910
        let p1 = Lazy::new(|| Regex::new(r#"-\{"#).unwrap()); // TODO: exclude html
        let p2 = Lazy::new(|| Regex::new(r#"-\{|\}-"#).unwrap());
        let mut pos = 0;
        let mut converted = String::with_capacity(text.len());
        let mut starts = vec![];
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
            starts.push(m1.start());
            pieces.push(String::new());
            while let Some(m2) = p2.find_at(text, pos) {
                // let mut piece = String::from(&text[pos..m2.start()]);
                if m2.as_str() == "-{" {
                    // if there are two many open start tag, ignore the new nested rule
                    if starts.len() >= NESTED_RULE_MAX_DEPTH {
                        pos += 2;
                        continue;
                    }
                    // start tag
                    starts.push(m2.start());
                    pieces.last_mut().unwrap().push_str(&text[pos..m2.start()]);
                    pieces.push(String::new()); // e.g. -{ zh: AAA -{
                    pos = m2.end();
                } else {
                    // end tag
                    dbg!(&starts, &pieces);
                    let start = starts.pop().unwrap();
                    let mut piece = pieces.pop().unwrap();
                    dbg!(&piece);
                    piece.push_str(&text[pos..m2.start()]);
                    dbg!(&piece);
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
            while let Some(_start) = starts.pop() {
                let piece = pieces.pop().unwrap();
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

pub struct ZhConverterBuilder<'t, 'c> {
    target: Variant,
    tables: Vec<(&'t str, &'t str)>,
    adds: BTreeMap<&'c str, &'c str>,
    removes: BTreeMap<&'c str, &'c str>,
    inline_conv: bool,
}

impl<'t, 'c> ZhConverterBuilder<'t, 'c> {
    pub fn new(target: Variant) -> Self {
        Self {
            target,
            tables: Default::default(),
            adds: Default::default(),
            removes: Default::default(),
            inline_conv: Default::default(),
        }
    }

    /// Activate the support of inline conversion rules of Mediawiki
    pub fn activate_inline_conv(mut self) -> Self {
        self.inline_conv = true;
        self
    }

    // Add a conversion table
    pub fn table(mut self, table: (&'t str, &'t str)) -> Self {
        self.tables.push(table);
        self
    }

    //  [CGroup](https://zh.wikipedia.org/wiki/Module:CGroup) (a.k.a 公共轉換組)

    // Add a set of rules extracted from a page
    // / These rules take the same precedence over those specified via `table`.
    pub fn page_rules(mut self, page_rules: impl Iterator<Item = &'c ConvAction>) -> Self {
        for conv_action in page_rules {
            let map = if conv_action.adds() {
                &mut self.adds
            } else {
                &mut self.removes
            };
            let conv = conv_action.as_conv();
            for (from, to) in conv.get_convs_by_target(self.target).into_iter() {
                map.insert(from, to);
            }
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

    pub fn build(self) -> ZhConverter {
        let Self {
            target: _target,
            tables,
            adds,
            removes,
            inline_conv,
        } = self;
        let size_hint = tables.iter().map(|&table| table.0.len() + 1).sum::<usize>() - 1
            + if adds.is_empty() {
                0
            } else {
                1 + adds.len() * 4 - 1
            }
            - (removes.len() * 4 - 1);
        let cmp_fn = |&pair1: &(&str, &str), &pair2: &(&str, &str)| pair1.0.len() >= pair2.0.len();
        let it = itertools::kmerge_by(
            tables
                .into_iter() // earlier tables have greater precedence
                .map(|(froms, tos)| itertools::zip(froms.trim().split("|"), tos.trim().split("|"))),
            cmp_fn,
        )
        .merge_by(adds.into_iter().map(|(from, to)| (from, to)), cmp_fn)
        .dedup_by(|pair1, pair2| pair1.0 == pair2.0)
        .filter(|(from, _to)| !removes.contains_key(from));
        // TODO: GROUP > tables
        ZhConverter::from_pairs_sorted(it, size_hint)
        // for (from, to) in it {
        //     if self.removes.contains_key(&from) {
        //         continue;
        //     }
        // }

        // unimplemented!()
    }
}
