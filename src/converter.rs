use std::collections::{BTreeMap, HashMap};

use lazy_static::lazy_static;
use regex::Regex;

use crate::{cgroup::CGroup, variant::Variant};

pub struct ZhConverter {
    regex: Regex,
    mapping: HashMap<String, String>, // Or str?
                                      // cgroups: Vec<Cgroup>
}

impl ZhConverter {
    pub fn new(regex: Regex, mapping: HashMap<String, String>) -> ZhConverter {
        ZhConverter { regex, mapping }
    }

    // pub fn set_cgroups()

    pub fn from_pairs(mut pairs: Vec<(String, String)>) -> ZhConverter {
        pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
        let mut pat = String::with_capacity(pairs.len() * 2 + 1);
        let mut mapping = HashMap::new();
        let mut it = pairs.into_iter().peekable();
        while let Some((from, to)) = it.next() {
            pat.push_str(&from);
            if it.peek().is_some() {
                pat.push_str("|");
            }
            mapping.insert(from, to);
        }
        ZhConverter {
            regex: Regex::new(&pat).unwrap(),
            mapping,
        }
    }

    pub fn convert(&self, text: &str) -> String {
        // Ref: https://github.dev/rust-lang/regex/blob/5197f21287344d2994f9cf06758a3ea30f5a26c3/src/re_trait.rs#L192
        let mut converted = String::with_capacity(text.len());
        let mut last = 0;
        let mut cnt = HashMap::<usize, usize>::new();
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
    inline_conv: bool
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

    /// Add a [CGroup](https://zh.wikipedia.org/wiki/Module:CGroup) (a.k.a 公共轉換組)
    ///
    /// Rules in `CGroup` take the precedence over those specified via `table`.
    pub fn cgroup(mut self, cgroup: &'c CGroup) -> Self {
        for conv_action in cgroup.as_conv_actions().iter() {
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
        
        unimplemented!()
    }
}
