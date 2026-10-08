use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::iter::IntoIterator;

use std::str::FromStr;

use daachorse::{CharwiseDoubleArrayAhoCorasick, CharwiseDoubleArrayAhoCorasickBuilder, MatchKind};
use zerovec::vecs::{Index32, VarZeroVecOwned};
use zerovec::{VarZeroSlice, VarZeroVec};

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

// CJK compat table codegen'd by build.rs as a sorted static array. Direct
// binary search suffices: every entry is 1-char -> 1-char, so no daachorse
// automaton is built for it (and none of it enters the conversion chain).
#[cfg(feature = "cjk-compat")]
include!(concat!(env!("OUT_DIR"), "/cjk_norm.rs"));

/// Normalize [CJK compatibility ideographs](https://github.com/BYVoid/OpenCC/blob/master/data/dictionary/CJK_Compatibility_Ideographs.txt)
/// to their standard forms.
///
/// Mirrors OpenCC's `normalization` chain, which runs before
/// segmentation/conversion in every upstream config.
///
/// This is a pre-pass, not part of conversion itself: [`ZhConverter`] never
/// calls it (single-pass, no hidden work). It runs in the [`crate::zhconv()`] and
/// [`crate::zhconv_mw()`] helpers — and anywhere else entry-point code chooses —
/// so custom-converter users compose it explicitly when wanted.
///
/// Returns borrowed input when no compatibility character is present (zero
/// allocation); otherwise an owned normalized string.
///
/// # Example
/// ```
/// # #[cfg(feature = "cjk-compat")]
/// # {
/// use zhconv::normalize_cjk_compat;
/// assert_eq!(normalize_cjk_compat("函數").into_owned(), "函數");
/// assert_eq!(normalize_cjk_compat("plain ascii"), "plain ascii");
/// # }
/// ```
/// Why byte search works here (byte census of real prose): ordinary Chinese
/// characters live outside the hunted ranges — CJK Unified is `E4–E9`-led,
/// CJK-native punct is `E3`-led, ASCII/markup is 1 byte — so none of them
/// can ever produce a candidate. The only `EF` in ordinary prose is fullwidth
/// punctuation (`，（）：；－！？．`, U+FF01–FF1B, second byte `BC`),
/// rows below the compat rows (`A4–AB`); `F0` means supplementary plane
/// (emoji/CJK-ext, ~1 char per 55KB here), likewise verified byte-exact.
/// Continuations (the bulk of bytes; 61% here) can never match by
/// construction. Measured data54k (55,192 bytes): 946 `EF` + 1 `F0` + 0 true
/// compat chars — i.e. the scan does vector work over everything and scalar
/// work at ~1.7% of positions, with zero decodes and zero allocs.
///
/// Shape: SIMD byte scan (`memchr2`) for exact candidate leads, then a
/// memchr-loop that copies each untouched stretch as a byte slice, decodes +
/// looks up only at candidates, and allocates lazily — zero allocation
/// unless a key truly maps (anything else passes through; borrowed when
/// nothing mapped yet).
///
/// Cost model (measured `-O3` micro-harness vs full `zh2TW` convert):
/// clean 54KB ~17µs (4.4%), clean 3.2MB ~1.4ms (3.3%); compat-sprinkled
/// input (a compat char every ~50 chars) 55KB ~34µs — ~9x faster than a
/// brute-force per-char scan (decode + look up every char). Byte detection
/// is exact at range level
/// (`EF A4..=AB` => U+F900..=U+FAFF; `F0` verified byte-exact); lookup may
/// still miss on unmapped range codepoints and falls back to borrowed.
///
/// Known limitation (accepted): inputs consisting mostly of rejected `EF`
/// hits — e.g. a synthetic wall of fullwidth punctuation, every ~3rd byte
/// a hit, all rejected — scan slower than plain char decoding (measured
/// ~2.5-3x on 103KB across runs), because each memchr restart costs on the
/// order of ~10ns (single-box, drifts with load/CPU) against a few ns/char
/// for decoding. Breakeven sits around ~15-25% hit density (estimate);
/// ordinary prose at ~4% stays far below it, and compat-sprinkled input
/// (~34µs above) is unaffected. Remedy if such walls ever occur in practice:
/// detect-once-then-char-loop.
#[cfg(feature = "cjk-compat")]
pub fn normalize_cjk_compat(text: &str) -> std::borrow::Cow<'_, str> {
    use std::borrow::Cow;
    let b = text.as_bytes();
    // Single cursor: always sits on a char boundary. It starts at 0, matches
    // can only occur on lead bytes, and it advances by whole chars — length
    // known from the lead byte alone (`EF` => 3, `F0` => 4), no decoding.
    // Slices between stops (`text[pos..i]`) are therefore always
    // boundary-safe.
    let mut pos = 0usize;
    let mut out: Option<String> = None;
    // 0xEF/0xF0 occur only as lead bytes in valid UTF-8; followers of a lead
    // always exist, so indexing below is safe. Length is known from the lead
    // alone, so `pos` advances by whole chars and continuation bytes are
    // never scanned as candidates.
    while let Some(rel) = memchr::memchr2(b'\xEF', b'\xF0', &b[pos..]) {
        let i = pos + rel;
        // b[i] is EF or F0 (memchr guarantee): length known, compat tested.
        let (len, is_compat) = match b[i] {
            0xEF => (3, matches!(b[i + 1], 0xA4..=0xAB)),
            _ => (
                4,
                b[i + 1] == 0xAF
                    && matches!(b[i + 2], 0xA0..=0xA8)
                    && (b[i + 2] != 0xA8 || matches!(b[i + 3], 0x80..=0x9D)),
            ),
        };
        if is_compat {
            // Candidate is in-range; map it only if it is a real key.
            let ch = text[i..].chars().next().unwrap();
            debug_assert_eq!(len, ch.len_utf8());
            if let Ok(j) = CJK_NORM_PAIRS.binary_search_by_key(&(ch as u32), |&(k, _)| k) {
                // First hit copies everything before it, not just `text[pos..i]`:
                // earlier chars (e.g. the leading `，。` in `，。數x`) were
                // scanned but never written, since `out` didn't exist yet.
                let o = out.get_or_insert_with(|| {
                    // Actually, we cannot guarantee there is no re-allocation
                    // even with_capacity since the dict sometimes maps to longer
                    // output, e.g. 𤋮 U+FA6C -> 𤋮 U+242EE
                    // TODO: evaluate whether to use capped headroom formula here?
                    let mut s = String::with_capacity(text.len());
                    s.push_str(&text[..pos]);
                    s
                });
                o.push_str(&text[pos..i]);
                o.push(char::from_u32(CJK_NORM_PAIRS[j].1).unwrap());
                pos = i + len;
                continue;
            }
        }
        // Pass-through (rejected punct/emoji, or unmapped range char): copy it
        // now if output is live, else leave it for the next bulk copy.
        // (Advancing `pos` without emitting here would silently drop input
        // once `out` exists — the single-cursor invariant is "out holds the
        // normalized form of text[..pos] whenever out is live".)
        if let Some(o) = out.as_mut() {
            o.push_str(&text[pos..i + len]);
        }
        pos = i + len;
    }
    match out {
        Some(mut o) => {
            o.push_str(&text[pos..]);
            Cow::Owned(o)
        }
        None => Cow::Borrowed(text),
    }
}

/// A ZhConverter, built by [`ZhConverterBuilder`].
///
/// `target_words` is borrowed for builtin converters (zero-copy over the
/// bundled store) and owned for custom-built ones.
pub struct ZhConverter<'a> {
    variant: Variant,
    automaton: Option<CharwiseDoubleArrayAhoCorasick<u32>>,
    target_words: VarZeroVec<'a, str, Index32>,
}

impl<'a> ZhConverter<'a> {
    /// Create a new converter from an automaton and a mapping.
    ///
    /// # Safety
    /// The `automaton` and `target_words` must form a coordinated, compatible pair
    /// (e.g. exported by [`into_inner`](Self::into_inner) or produced by [`ZhConverterBuilder`]).
    /// Automaton match values MUST encode valid packed `(offset << 10) | len` descriptors
    /// referencing valid UTF-8 slices within `target_words.as_bytes()`.
    ///
    /// It is provided for convenience and not expected to be called directly.
    /// [`ZhConverterBuilder`] would take care of these details.
    #[doc(hidden)]
    pub unsafe fn new(
        automaton: CharwiseDoubleArrayAhoCorasick<u32>,
        target_words: VarZeroVec<'a, str, Index32>,
    ) -> ZhConverter<'a> {
        ZhConverter {
            variant: Variant::Zh,
            automaton: Some(automaton),
            target_words,
        }
    }

    /// Create a new converter from an automaton and a mapping, as well as specifying a target
    /// variant to be used by [`convert_as_wikitext_basic`](Self::convert_as_wikitext_basic) and
    /// [`convert_as_wikitext_extended`](Self::convert_as_wikitext_extended) and related functions.
    ///
    /// # Safety
    /// Same safety invariant as [`new`](Self::new).
    ///
    /// It is provided for convenience and not expected to be called directly.
    /// [`ZhConverterBuilder`] would take care of these details.
    #[doc(hidden)]
    pub unsafe fn with_target_variant(
        automaton: CharwiseDoubleArrayAhoCorasick<u32>,
        target_words: VarZeroVec<'a, str, Index32>,
        variant: Variant,
    ) -> ZhConverter<'a> {
        ZhConverter {
            variant,
            automaton: Some(automaton),
            target_words,
        }
    }

    /// Break a converter back into its automaton, target_words, and variant.
    ///
    /// Inverse of [`new`](Self::new) / [`with_target_variant`](Self::with_target_variant).
    /// `None` automaton means a blank converter (empty mapping).
    #[doc(hidden)]
    pub fn into_inner(
        self,
    ) -> (
        Option<CharwiseDoubleArrayAhoCorasick<u32>>,
        VarZeroVec<'a, str, Index32>,
        Variant,
    ) {
        let Self {
            variant,
            automaton,
            target_words,
        } = self;
        (automaton, target_words, variant)
    }

    /// Create a new converter of a sequence of `(from, to)` pairs.
    ///
    /// It uses [`ZhConverterBuilder`] internally.
    ///
    /// # Panics
    /// Panics if any target word exceeds 1023 bytes in length, or if total serialized
    /// target words exceed 4MB (exceeding the packed representation limit).
    #[inline(always)]
    pub fn from_pairs(
        pairs: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> ZhConverter<'static> {
        ZhConverterBuilder::new().conv_pairs(pairs).build()
    }

    /// Create a new converter of a sequence of `(from, to)` pairs.
    ///
    /// It takes a target variant to be used by [`convert_as_wikitext_basic`](Self::convert_as_wikitext_basic)
    /// and [`convert_as_wikitext_extended`](Self::convert_as_wikitext_extended) and related
    /// functions, in addition to [`from_pairs`](Self::from_pairs).
    ///
    /// It uses [`ZhConverterBuilder`] internally.
    ///
    /// # Panics
    /// Panics if any target word exceeds 1023 bytes in length, or if total serialized
    /// target words exceed 4MB (exceeding the packed representation limit).
    #[inline(always)]
    pub fn from_pairs_with_target_variant(
        variant: Variant,
        pairs: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
    ) -> ZhConverter<'static> {
        ZhConverterBuilder::new()
            .target(variant)
            .conv_pairs(pairs)
            .build()
    }

    /// Heuristic extra capacity to preallocate for converted text.
    ///
    /// # Statistical Background & Length Distribution
    /// - **Equal Length Dominance**: Across all built-in dictionary pairs (48,000+ rules),
    ///   **91% ~ 93% of rules have identical byte lengths** ($\Delta = 0$). For example,
    ///   BMP CJK ideographs map 3 UTF-8 bytes to 3 UTF-8 bytes (e.g. `学` -> `學`), and 2-character
    ///   compounds map 6B -> 6B.
    /// - **Natural Expansion/Shrinkage Cancellation**: For the minority of rules where lengths differ,
    ///   expansions (e.g. `内存` 6B -> `記憶體` 9B, +3B) and shrinkages (e.g. `计算机` 9B -> `電腦` 6B, -3B;
    ///   `功能變數名稱` 18B -> `域名` 6B, -12B; Ext-B 4B -> 3B, -1B) naturally offset each other in
    ///   natural language. Across the entirety of all 11,849 rules in ZhTW, the dictionary as a whole
    ///   actually shrinks by -972 bytes; ZhCN net expansion across all 7,361 rules is only +529 bytes.
    /// - **Sub-linear Growth in Practice**: Empirical audits on corpora from 16 bytes to 3.26 MB
    ///   (including `data3185k.txt`, `honglou.txt`, `sanguo.txt`) reveal that net positive expansion
    ///   plateaus between 200B ~ 400B (< 0.12‰ on multi-megabyte texts). Growth is sub-linear and
    ///   asymptotically bounded rather than $O(N)$ linear.
    ///
    /// # Formula Rationale: `(text_len >> 6).min(512) + 32`
    /// - `text_len >> 6`: ~1.5% proportional headroom for short-to-medium texts.
    /// - `.min(512)`: Caps the headroom at 512 bytes, preventing massive memory bloat on large inputs
    ///   (e.g. avoiding 100KB over-allocation on 3MB files or multi-megabyte waste on 100MB inputs).
    /// - `+ 32`: Safety cushion for local phrase expansion spikes and 16 trailing slack bytes
    ///   for the SIMD (`u128`) blind store.
    #[inline(always)]
    pub(crate) fn empirical_headroom(text_len: usize) -> usize {
        (text_len >> 6).min(512) + 32
    }

    #[inline(always)]
    pub(crate) fn unpack_target_val(val: u32) -> (usize, usize) {
        ((val >> 10) as usize, (val & 0x3FF) as usize)
    }

    /// Resolve target replacement word directly from packed DAAC value `(offset << 10) | len`.
    ///
    /// Slices directly into contiguous pre-encoded UTF-8 dictionary bytes without
    /// secondary indexing or tagged branching.
    #[inline(always)]
    fn get_target_word(&self, val: u32) -> &str {
        let (offset, len) = Self::unpack_target_val(val);
        let bytes = self.target_words.as_bytes();
        let slice = bytes
            .get(offset..offset + len)
            .expect("target word slice out of bounds");
        // SAFETY: `self.target_words` was constructed from valid UTF-8 strings
        // via `VarZeroVec<str, Index32>`. The packed `(offset, len)` slices out
        // an exact target word boundary, guaranteeing that `slice` is valid UTF-8.
        unsafe { std::str::from_utf8_unchecked(slice) }
    }

    /// Convert text.
    ///
    /// Note: unlike the [`crate::zhconv()`] helper, this performs no CJK
    /// compatibility normalization beforehand. Call [`normalize_cjk_compat()`]
    /// first when the input may contain compatibility ideographs.
    #[inline(always)]
    pub fn convert(&self, text: &str) -> String {
        if text.is_empty() {
            return String::new();
        }
        let mut output = String::with_capacity(text.len() + Self::empirical_headroom(text.len()));
        self.convert_to(text, &mut output);
        output
    }

    /// Same as `convert`, except that it takes a `&mut String` as dest instead of returning a `String`.
    ///
    /// Note: unlike the [`crate::zhconv()`] helper, this performs no CJK
    /// compatibility normalization beforehand. Call [`normalize_cjk_compat()`]
    /// first when the input may contain compatibility ideographs.
    pub fn convert_to(&self, text: &str, output: &mut String) {
        if text.is_empty() {
            return;
        }

        let automaton = match self.automaton.as_ref() {
            Some(automaton) => automaton,
            None => {
                output.push_str(text);
                return;
            }
        };

        // Only reserve heuristic headroom if the caller provided no remaining capacity
        // (e.g. fresh `String::new()` or fully filled String being appended to).
        // If caller already pre-allocated remaining capacity, respect caller's buffer.
        let remaining = output.capacity().saturating_sub(output.len());
        if remaining == 0 {
            let anticipated_extension = text.len() + Self::empirical_headroom(text.len());
            output.reserve(anticipated_extension);
        }

        // SAFETY: We only append valid UTF-8 sequences (gap slices from valid UTF-8
        // `text`, and target replacement words from valid UTF-8 `target_words`).
        // String length is updated via `set_len` only after all bytes are copied.
        let buf = unsafe { output.as_mut_vec() };
        let text_bytes = text.as_bytes();
        let text_len = text_bytes.len();
        let target_bytes = self.target_words.as_bytes();
        let target_len = target_bytes.len();

        let mut last = 0;
        for m in automaton.leftmost_find_iter(text) {
            let s = m.start();
            let e = m.end();
            let val = m.value();
            // Automaton values pack `(offset << 10) | len` directly into the underlying
            // `target_words` VZV byte slice (high 22 bits offset up to 4MB, low 10 bits len up to 1023B),
            // completely bypassing runtime `Index32` secondary table lookups.
            // (Design note: See build.rs `write_daac_file` documentation for rationale on
            // uniform `(offset << 10) | len` packing over tagged short-word inlining).
            let (offset, len) = Self::unpack_target_val(val);
            let gap = s - last;

            let cur_len = buf.len();
            let chunk_slack = gap + len + 16;
            buf.reserve(chunk_slack);

            debug_assert!(
                buf.capacity() >= cur_len + gap + len + 16,
                "buffer capacity must accommodate gap, replacement, and 16B blind write padding"
            );
            debug_assert!(
                last + gap <= text_bytes.len(),
                "gap slice must stay within input text bounds"
            );
            debug_assert!(
                offset + len <= target_bytes.len(),
                "target word slice must stay within target table bounds"
            );

            // SAFETY: `cur_len <= buf.capacity()` (guaranteed by Vec invariants and reserve above).
            let dst = unsafe { buf.as_mut_ptr().add(cur_len) };

            // 25.9% of matches in real corpora are contiguous (gap == 0);
            // skipping copy_nonoverlapping saves tens of thousands of redundant 0-byte memcpy calls.
            if gap > 0 {
                // SAFETY:
                // 1. Source: `last + gap <= text_bytes.len()`, staying within input `text_bytes`.
                // 2. Destination: `cur_len + gap <= buf.capacity()`, guaranteed by `chunk_slack` reservation.
                // 3. Non-overlapping: `text` and `output` are separate memory buffers.
                unsafe {
                    std::ptr::copy_nonoverlapping(text_bytes.as_ptr().add(last), dst, gap);
                }
            }
            // SAFETY: `cur_len + gap <= buf.capacity()`, within allocated capacity.
            let replace_dst = unsafe { dst.add(gap) };

            // Word length distribution: 97% ~ 99% of target words are <= 16 bytes (1 to 5 Chinese chars).
            // When len <= 16 and within table bounds (offset + 16 <= target_len), emit a single
            // unaligned 128-bit SIMD move (movups on x86-64, ldp/stp on AArch64, v128 on WASM)
            // without loop or memcpy call overhead.
            // Any extra bytes written past `len` are safe slack and will be overwritten or truncated by set_len.
            if len <= 16 && offset + 16 <= target_len {
                debug_assert!(
                    cur_len + gap + 16 <= buf.capacity(),
                    "16B blind write must stay within output capacity"
                );
                // SAFETY:
                // 1. Source: `offset + 16 <= target_len` guarantees reading 16 valid bytes in `target_bytes`.
                // 2. Destination: `cur_len + gap + 16 <= buf.capacity()` guarantees writing 16 valid bytes at `replace_dst`.
                // 3. Alignment: `read_unaligned` and `write_unaligned` support arbitrary unaligned byte pointers.
                unsafe {
                    let src = target_bytes.as_ptr().add(offset);
                    let w = (src as *const u128).read_unaligned();
                    (replace_dst as *mut u128).write_unaligned(w);
                }
                debug_assert_eq!(
                    // SAFETY: `replace_dst` contains at least `len` initialized bytes written above.
                    unsafe { std::slice::from_raw_parts(replace_dst, len) },
                    &target_bytes[offset..offset + len],
                    "16B SIMD copy must produce byte-identical prefix to source word"
                );
            } else {
                assert!(
                    offset + len <= target_len,
                    "target word slice out of bounds"
                );
                // SAFETY:
                // 1. Source: `offset + len <= target_len` verified by the assertion above.
                // 2. Destination: `cur_len + gap + len <= buf.capacity()`, guaranteed by `chunk_slack` reservation.
                // 3. Non-overlapping: `target_bytes` (dictionary) and `output` are separate memory allocations.
                unsafe {
                    let src = target_bytes.as_ptr().add(offset);
                    std::ptr::copy_nonoverlapping(src, replace_dst, len);
                }
            }

            // SAFETY:
            // 1. `cur_len + gap + len <= buf.capacity()` ensured by upfront `reserve(chunk_slack)`.
            // 2. All bytes from `0..cur_len + gap + len` are fully initialized:
            //    - `0..cur_len`: prior valid UTF-8 output.
            //    - `cur_len..cur_len + gap`: copied from valid UTF-8 `text`.
            //    - `cur_len + gap..cur_len + gap + len`: copied from valid UTF-8 dictionary `target_bytes`.
            unsafe {
                buf.set_len(cur_len + gap + len);
            }
            last = e;
        }

        let tail = text_len - last;
        if tail > 0 {
            let cur_len = buf.len();
            if buf.capacity() - cur_len < tail {
                buf.reserve(tail);
            }
            debug_assert!(
                cur_len + tail <= buf.capacity(),
                "tail copy must stay within output buffer capacity"
            );
            debug_assert!(
                last + tail == text_len,
                "tail copy spans remaining text exactly"
            );
            // SAFETY:
            // 1. Source: `last + tail == text_len`, copying the exact remaining slice of valid UTF-8 `text`.
            // 2. Destination: `cur_len + tail <= buf.capacity()` ensured by the reservation above.
            // 3. Non-overlapping: `text` and `output` are distinct memory allocations.
            // 4. Length update: all `cur_len + tail` bytes are initialized with valid UTF-8 sequences.
            unsafe {
                std::ptr::copy_nonoverlapping(
                    text_bytes.as_ptr().add(last),
                    buf.as_mut_ptr().add(cur_len),
                    tail,
                );
                buf.set_len(cur_len + tail);
            }
        }

        debug_assert!(
            std::str::from_utf8(buf).is_ok(),
            "output buffer must remain valid UTF-8"
        );
    }

    /// Convert text, along with a secondary converter.
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
        if text.is_empty() {
            return String::new();
        }
        let mut output = String::with_capacity(text.len() + Self::empirical_headroom(text.len()));
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

    /// Convert text, along with a secondary conversion table (typically temporary).
    ///
    /// The worst-case time complexity of the implementation is `O(n*m)` where `n` and `m` are the
    /// length of the text and the maximum lengths of sources words in conversion rulesets.
    /// (i.e. brute-force).
    // TODO: optimize secondary converter pipeline if desired
    fn convert_to_with(
        &self,
        text: &str,
        output: &mut String,
        shadowing_automaton: Option<&CharwiseDoubleArrayAhoCorasick<u32>>,
        shadowing_target_words: &VarZeroSlice<str, Index32>,
        shadowed_source_words: &HashSet<String>,
    ) {
        if text.is_empty() {
            return;
        }

        let automaton = match self.automaton.as_ref() {
            Some(automaton) => automaton,
            None => {
                output.push_str(text);
                return;
            }
        };

        let remaining = output.capacity().saturating_sub(output.len());
        if remaining == 0 {
            let anticipated_extension = text.len() + Self::empirical_headroom(text.len());
            output.reserve(anticipated_extension);
        }

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
                        self.get_target_word(m.value()),
                    )
                });
            }
            if right_match.is_none() || right_match.unwrap().0 < last {
                right_match = shadowing_automaton.and_then(|shadowing_automaton| {
                    shadowing_automaton
                        .leftmost_find_iter(&text[last..])
                        .next()
                        .map(|m| {
                            let (offset, len) = Self::unpack_target_val(m.value());
                            let bytes = shadowing_target_words.as_bytes();
                            let slice = bytes
                                .get(offset..offset + len)
                                .expect("shadowing target word slice out of table bounds");
                            // SAFETY: `shadowing_target_words` contains valid UTF-8 strings
                            // in a `VarZeroSlice<str, Index32>`. The packed `(offset, len)` slices out
                            // an exact word boundary, guaranteeing that `slice` is valid UTF-8.
                            let target_word = unsafe { std::str::from_utf8_unchecked(slice) };
                            (last + m.start(), last + m.end(), target_word)
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
                        //
                        // NOTE: In case there are two rules like `{abcd -> foo, abc -> bar}`,
                        // even if the former is disabled, the latter won't take effect, since `a`
                        // will be skipped and the next search start at `bc`.
                        // It is inevitable if we do not re-build a new automaton every time.
                        let first_char_len = text[a.0..].chars().next().unwrap().len_utf8();
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
    ///
    /// # Panics
    /// Panics if dynamic wikitext rules contain a target word exceeding 1023 bytes in length,
    /// or if total serialized dynamic target words exceed 4MB (inheriting limits from
    /// [`ZhConverterBuilder::build`]).
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
        self.convert_to_as_wikitext(text, output, &mut None, false, false, None)
    }

    /// Same as [`convert_to_as_wikitext_extended`](Self::convert_to_as_wikitext_extended), except
    /// that it takes a `&mut String` as dest instead of returning a `String`.
    ///
    /// # Panics
    /// Panics if dynamic wikitext rules contain a target word exceeding 1023 bytes in length,
    /// or if total serialized dynamic target words exceed 4MB (inheriting limits from
    /// [`ZhConverterBuilder::build`]).
    #[inline(always)]
    pub fn convert_to_as_wikitext_extended(&self, text: &str, output: &mut String) {
        self.convert_to_as_wikitext(text, output, &mut None, true, true, None)
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
    ///
    /// # Panics
    /// Panics if dynamic wikitext rules contain a target word exceeding 1023 bytes in length,
    /// or if total serialized dynamic target words exceed 4MB (inheriting limits from
    /// [`ZhConverterBuilder::build`]). Also panics if `secondary_converter_builder` contains
    /// preloaded conversion tables.
    #[inline(always)]
    pub fn convert_as_wikitext(
        &self,
        text: &str,
        secondary_converter_builder: &mut Option<ZhConverterBuilder>,
        skip_html_code_blocks: bool,
        apply_global_rules: bool,
        preprocess: Option<for<'hook> fn(&'hook str) -> std::borrow::Cow<'hook, str>>,
    ) -> String {
        let mut output = String::with_capacity(text.len());
        self.convert_to_as_wikitext(
            text,
            &mut output,
            secondary_converter_builder,
            skip_html_code_blocks,
            apply_global_rules,
            preprocess,
        );
        output
    }

    /// Same as [`convert_as_wikitext`](Self::convert_as_wikitext), except
    /// that it takes a `&mut String` as dest instead of returning a `String`.
    ///
    /// `preprocess` runs on every prose span before conversion — never inside
    /// `-{…}-` rule blocks. `None` means raw conversion. `Some` takes a plain
    /// function pointer (`fn`s and non-capturing closures qualify; e.g.
    /// `normalize_cjk_compat` with `cjk-compat`) — closures with captures
    /// compose by pre-applying whole input instead.
    ///
    /// # Panics
    /// Panics if dynamic wikitext rules contain a target word exceeding 1023 bytes in length,
    /// or if total serialized dynamic target words exceed 4MB (inheriting limits from
    /// [`ZhConverterBuilder::build`]). Also panics if `secondary_converter_builder` contains
    /// preloaded conversion tables.
    ///
    /// # Example
    /// ```
    /// use zhconv::ZhConverter;
    /// let converter = ZhConverter::from_pairs([("函數", "函式")]);
    /// # #[cfg(feature = "cjk-compat")]
    /// # {
    /// # use zhconv::normalize_cjk_compat;
    /// # let mut out = String::new();
    /// # converter.convert_to_as_wikitext("函數", &mut out, &mut None, false, false, Some(normalize_cjk_compat));
    /// # assert_eq!(out, "函式");
    /// # }
    /// let mut out = String::new();
    /// converter.convert_to_as_wikitext("函數", &mut out, &mut None, false, false, None);
    /// assert_eq!(out, "函數");
    /// ```
    pub fn convert_to_as_wikitext(
        &self,
        text: &str,
        output: &mut String,
        secondary_converter_builder: &mut Option<ZhConverterBuilder>,
        skip_html_code_blocks: bool,
        apply_global_rules: bool,
        preprocess: Option<for<'hook> fn(&'hook str) -> std::borrow::Cow<'hook, str>>,
    ) {
        if text.is_empty() {
            return;
        }

        // Ref: https://github.com/wikimedia/mediawiki/blob/7bf779524ab1fd8e1d74f79ea4840564d48eea4d/includes/language/LanguageConverter.php#L855
        //  and https://github.com/wikimedia/mediawiki/blob/7bf779524ab1fd8e1d74f79ea4840564d48eea4d/includes/language/LanguageConverter.php#L910
        //  and https://github.com/wikimedia/mediawiki/blob/7bf779524ab1fd8e1d74f79ea4840564d48eea4d/includes/language/LanguageConverter.php#L532

        let preprocess = preprocess.as_ref();
        #[allow(clippy::type_complexity)]
        let mut convert_to: Box<dyn Fn(&str, &mut String)> =
            Box::new(move |text: &str, output: &mut String| {
                let text = match preprocess {
                    Some(f) => &f(text),
                    None => text,
                };
                self.convert_to(text, output)
            });
        if secondary_converter_builder.is_some() || apply_global_rules {
            // build a secondary automaton from global rules specified in wikitext
            let mut builder = secondary_converter_builder.take().unwrap_or_default();
            if !builder.tables.is_empty() {
                panic!("The secondary converter builder should not load conversion tables");
            }
            // let mut shadowing_pairs = HashMap::new();
            let global_rules_in_page = PageRules::from_str(text).expect("infallible");
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
                    let text = match preprocess {
                        Some(f) => &f(text),
                        None => text,
                    };
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
                    // start tag
                    pieces.last_mut().unwrap().push_str(&text[pos..m2.start()]);
                    pos = m2.end();

                    // if there are two many open start tags, ignore the new nested rule
                    //
                    // MediaWiki would show: language-converter-depth-warning/已超出語言轉換器深度限制
                    if pieces.len() >= NESTED_RULE_MAX_DEPTH {
                        continue;
                    }

                    pieces.push(String::new()); // e.g. -{ zh: AAA -{
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
                    pos = m2.end();
                    if pieces.is_empty() {
                        // return to toplevel
                        break;
                    }
                }
            }
            for piece in pieces.iter() {
                output.push_str("-{");
                // replicating the behaviour of MediaWiki LC here, even though the it is pretty
                // weird: `-{简-{简}` is converted to `-{簡-{簡}` with `zh-hant` for example
                convert_to(piece, output);
            }
            pieces.clear();
        }
        if pos < text.len() {
            // no more conv rules, just convert and append
            convert_to(&text[pos..], output);
        }
    }

    /// Search the text
    #[doc(hidden)]
    pub fn search<'s, 'i: 's>(
        &'i self,
        text: &'s str,
    ) -> impl Iterator<Item = (usize, usize, &'i str)> + 's {
        self.automaton
            .as_ref()
            .map(|automaton| {
                automaton
                    .leftmost_find_iter(text)
                    .map(|m| (m.start(), m.end(), self.get_target_word(m.value())))
            })
            .into_iter()
            .flatten()
    }

    /// Count the sum of lengths of source words to be replaced by the converter, in chars
    ///
    /// Source words identical to target words are omitted.
    #[doc(hidden)]
    pub fn count_replaced(&self, text: &str) -> usize {
        self.search(text)
            .map(|(s, e, to)| {
                if &text[s..e] == to {
                    0
                } else {
                    text[s..e].chars().count()
                }
            })
            .sum()
    }
}

/// A builder that helps build a [`ZhConverter`](ZhConverter).
///
/// # Limits
/// Custom conversion rules are packed into a compact representation supporting target words
/// up to 1023 bytes in length and a total serialized target word store of up to 4MB. Exceeding
/// these limits will cause [`build`](Self::build) to panic.
///
/// # Example
/// Build a Zh2CN converter with some additional rules.
/// ```
/// # #[cfg(any(feature = "mediawiki", feature = "opencc"))]
/// # {
/// use zhconv::{zhconv, ZhConverterBuilder, Variant, get_builtin_table};
/// // extracted from https://zh.wikipedia.org/wiki/Template:CGroup/Template:CGroup/文學.
/// let rules = r"zh-hans:三个火枪手;zh-hant:三劍客;zh-tw:三劍客;
///                    zh-cn:雾都孤儿;zh-tw:孤雛淚;zh-hk:苦海孤雛;zh-sg:雾都孤儿;zh-mo:苦海孤雛;";
/// let converter = ZhConverterBuilder::new()
///                     .target(Variant::ZhCN)
///                     .table(get_builtin_table(Variant::ZhCN))
///                     .conv_lines(rules.lines())
///                     .build();
/// let original = "《三劍客》是亞歷山大·仲馬的作品。《孤雛淚》是查爾斯·狄更斯的作品。";
/// assert_eq!(converter.convert(original), "《三个火枪手》是亚历山大·仲马的作品。《雾都孤儿》是查尔斯·狄更斯的作品。");
/// assert_eq!(zhconv(original, Variant::ZhCN), "《三剑客》是亚历山大·仲马的作品。《孤雏泪》是查尔斯·狄更斯的作品。")
/// # }
#[derive(Debug, Clone, Default)]
pub struct ZhConverterBuilder<'t> {
    target: Variant,
    /// The base conversion table
    tables: Vec<Table<'t>>,
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

    /// Add a conversion table, which is typically those returned by
    /// [`get_builtin_table`](crate::get_builtin_table).
    pub fn table(mut self, table: Table<'t>) -> Self {
        self.tables.push(table);
        self
    }

    /// Add conversion tables (e.g. a slice of `ZH_*_TABLE` constants).
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

    /// Add [`ConvAction`]s, which are typically parsed from rules in the MediaWiki syntax.
    ///
    /// These rules take the higher precedence over those specified via `table`.
    /// For general usage, check [`conv_pairs`](#method.conv_pairs) which takes
    /// `from -> to` pairs.
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

    /// Add [`Conv`]s, which are typically parsed from rules in MediaWiki syntax.
    ///
    /// For general usage, check [`conv_pairs`](#method.conv_pairs) which takes
    /// `from -> to` pairs.
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

    /// Mark [`Conv`]s as removed.
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
    /// It takes the precedence over those specified via `table`, while shares the same precedence
    /// level with those specified via `convs` or `conv_lines`.
    ///
    /// # Panics
    /// While this method does not panic directly, calling [`build`](Self::build) will panic
    /// if any target word exceeds 1023 bytes in length or if total target words exceed 4MB.
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
    /// Any rule with the same `from`, whether specified via `conv_pairs`, `conv_lines` or `table`,
    /// is removed.
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
    /// Any rule with the same `from`, whether specified via `conv_pairs`, `conv_lines` or `table`,
    /// is removed.
    pub fn unconv_pair(mut self, from: impl AsRef<str>, to: impl AsRef<str>) -> Self {
        self.removes
            .insert(from.as_ref().to_owned(), to.as_ref().to_owned());
        self
    }

    /// Add rules in the MediaWiki conversion syntax line by line.
    ///
    /// e.g.
    /// ```text
    /// zh-cn:天堂执法者; zh-hk:夏威夷探案; zh-tw:檀島警騎2.0;
    /// zh-cn:史蒂芬·'史蒂夫'·麦格瑞特; zh-tw:史提夫·麥加雷; zh-hk:麥星帆;
    /// zh-cn:丹尼尔·'丹尼/丹诺'·威廉姆斯; zh-tw:丹尼·威廉斯; zh-hk:韋丹尼;
    /// ```  
    ///
    /// # Panics
    /// Panics if a rule contains an empty source pattern. Additionally, calling [`build`](Self::build)
    /// will panic if any target word exceeds 1023 bytes in length or if total target words exceed 4MB.
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
    /// It internally aggregates previously specified tables, rules and pairs, from which an
    /// automaton and a mapping are built.
    /// Custom-built converters always own their target_words; only builtin
    /// converters borrow the bundled store.
    ///
    /// # Panics
    /// Panics if any target word exceeds 1023 bytes in length, or if total serialized
    /// target words exceed 4MB (exceeding the packed representation limit).
    // TODO: If daachorse adds iter_patvals in future to export patterns and values,
    // tables could be exported directly from flat payload without VarZeroVec.
    pub fn build(&self) -> ZhConverter<'static> {
        let mapping = self.build_mapping();
        if mapping.is_empty() {
            return ZhConverter {
                variant: self.target,
                automaton: None,
                target_words: VarZeroVec::from(VarZeroVecOwned::<str, Index32>::new()),
            };
        }
        let mut target_words = Vec::with_capacity(mapping.len());
        let mut sources = Vec::with_capacity(mapping.len());
        for (f, t) in mapping {
            sources.push(f);
            target_words.push(t);
        }
        let target_vzv = VarZeroVecOwned::<str, Index32>::try_from_elements(&target_words)
            .expect("pack target words");
        let target_words = VarZeroVec::from(target_vzv);
        let vzv_slice = target_words.as_slice();
        let vzv_bytes = target_words.as_bytes();
        // TODO: support oversized custom dictionaries (>1023B words or >4MB table)
        // via indexed fallback or wider values if ever desired in practice?
        assert!(
            vzv_bytes.len() <= 0x3FFFFF,
            "runtime converter target words size {} exceeds 4MB limit",
            vzv_bytes.len()
        );

        let mut patvals = Vec::with_capacity(sources.len());
        for (i, f) in sources.into_iter().enumerate() {
            let s = vzv_slice.get(i).unwrap();
            let offset = s.as_ptr() as usize - vzv_bytes.as_ptr() as usize;
            let len = s.len();
            assert!(
                len <= 0x3FF,
                "runtime target word {:?} length {} exceeds 1023 bytes",
                s,
                len
            );
            assert!(
                offset <= 0x3FFFFF,
                "runtime target word {:?} offset {} exceeds 4MB",
                s,
                offset
            );
            let packed = ((offset as u32) << 10) | (len as u32);
            patvals.push((f, packed));
        }

        let automaton = CharwiseDoubleArrayAhoCorasickBuilder::new()
            .match_kind(MatchKind::LeftmostLongest)
            // Disable prefilter: conversion tables have high text coverage, so prefiltering cannot skip ahead and only adds overhead.
            .use_prefilter(false)
            .build_with_values(patvals)
            .expect("Rules feed to DAAC already filtered");
        ZhConverter {
            variant: self.target,
            automaton: Some(automaton),
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
        // Size from view ranges (exact pair count), not byte lengths.
        let mut mapping = HashMap::with_capacity(
            (tables
                .iter()
                .map(|t| {
                    t.ranges
                        .iter()
                        .map(|&(s, e)| e.saturating_sub(s))
                        .sum::<usize>()
                })
                .sum::<usize>()
                + adds.len())
            .saturating_sub(removes.len()),
        );
        mapping.extend(
            tables
                .iter()
                .flat_map(|&table| expand_table(table))
                // Empty sources would poison the automaton (daachorse
                // ignores all other patterns when the set contains an
                // empty string, silently stopping conversion);
                // `expand_table` already drops them, belt and braces.
                .filter(|(from, _to)| !from.is_empty())
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

#[cfg(all(test, feature = "cjk-compat"))]
mod normalize_tests {
    use super::normalize_cjk_compat;

    fn norm(text: &str) -> String {
        normalize_cjk_compat(text).into_owned()
    }

    #[test]
    fn passthrough_basics() {
        assert_eq!(norm(""), "");
        assert_eq!(norm("plain ascii 123"), "plain ascii 123");
        // Fullwidth punct + emoji are rejected at byte level, untouched.
        assert_eq!(norm("Hello，世界🎉"), "Hello，世界🎉");
        // BMP + supplementary mapped keys.
        assert_eq!(norm("函數"), "函數");
        // U+2F800 -> U+4E3D (later conversion turns 丽人 into 麗人).
        assert_eq!(norm("丽人"), "丽人");
    }

    #[test]
    fn no_drop_after_live_output() {
        // Rejected hits (punct) after a mapped hit must survive: copied out
        // at the next hit, or at the end if none follows.
        assert_eq!(norm("數，。x"), "數，。x");
        assert_eq!(norm("a數b，c"), "a數b，c");
        // Unmapped in-range char (U+FA0E has no CJK entry) after a hit.
        assert_eq!(norm("數﨎x"), "數﨎x");
        // Hit at string end (nothing follows it).
        assert_eq!(norm("x數"), "x數");
    }

    /// Differential test: the implementation must agree byte-for-byte with an
    /// obviously-correct per-char reference (independent map-or-keep per
    /// character — too slow and always-allocating for production, perfect as
    /// an oracle) on fixed edge cases plus 500 seeded-random hostile inputs.
    ///
    /// The fixed inputs pin known shapes (empty, single mapped / unmapped /
    /// punct / emoji char, all-compat, junk-before-first-hit like `，。數x`);
    /// the random ones cover shapes nobody hand-picked. The extra `Borrowed`
    /// assert pins the zero-alloc fast path: clean input must never allocate.
    ///
    /// Scope: the matching/copying algorithm only (drops, duplications,
    /// boundary slicing). The table contents are shared with the
    /// implementation (a wrong dict entry passes both sides); dict parsing
    /// itself is covered by the data-crate tests and build-time asserts.
    #[test]
    fn matches_naive_reference() {
        use std::borrow::Cow;
        use std::collections::HashMap;
        let reference: HashMap<char, char> = super::CJK_NORM_PAIRS
            .iter()
            .map(|&(k, v)| (char::from_u32(k).unwrap(), char::from_u32(v).unwrap()))
            .collect();
        let alphabet = [
            'a', 'Z', ' ', '\n', '中', '文', '，', '。', '！', '「', '🎉', '🍜', '面', '數', '車',
            '丽', '﨎', '﨏', // in-range but unmapped
        ];
        // Stand-in for a test RNG so the test needs no extra dev-dependency:
        // one linear congruential generator (the Numerical Recipes
        // constants 1664525/1013904223; `wrapping_*` = arithmetic mod 2^32,
        // since plain `*`/`+` would panic on overflow in debug builds).
        // Fixed seed ⇒ same inputs every run: reproducible, never flaky.
        // Statistical quality is irrelevant here; any deterministic spread
        // over the alphabet exercises the code paths.
        let mut rng: u32 = 0x12345678;
        let mut next_u32 = || {
            rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
            rng
        };
        let mut fixed: Vec<String> = vec![
            String::new(),
            "數".to_string(),
            "﨎".to_string(),
            "，".to_string(),
            "🎉".to_string(),
            "數車丽".to_string(),
            "數，。﨎🎉x".to_string(),
            "，。數x".to_string(), // junk before first hit
        ];
        for _ in 0..500 {
            let len = (next_u32() % 60) as usize;
            let mut s = String::new();
            for _ in 0..len {
                s.push(alphabet[(next_u32() % alphabet.len() as u32) as usize]);
            }
            fixed.push(s);
        }
        for s in &fixed {
            let expected: String = s
                .chars()
                .map(|c| reference.get(&c).copied().unwrap_or(c))
                .collect();
            assert_eq!(
                normalize_cjk_compat(s).into_owned(),
                expected,
                "mismatch on {s:?}"
            );
            // Zero-alloc property: no mapped key present implies borrowed.
            if !s.chars().any(|c| reference.contains_key(&c)) {
                assert!(
                    matches!(normalize_cjk_compat(s), Cow::Borrowed(_)),
                    "should borrow on {s:?}"
                );
            }
        }
    }
}

#[cfg(test)]
mod converter_tests {
    use super::ZhConverter;
    use std::borrow::Cow;

    #[test]
    fn converter_performs_no_normalization() {
        // `ZhConverter` is a pure single-pass primitive: compatibility
        // ideographs pass through untouched unless the caller runs
        // `normalize_cjk_compat()` (as the `zhconv()` helper does) first.
        let c = ZhConverter::from_pairs([("a", "b")]);
        assert_eq!(c.convert("a數"), "b數");
    }

    /// `zhconv_mw` normalizes prose per span via the hook, but `-{…}-` rule
    /// content stays byte-exact (table backend only affects the prose part).
    #[cfg(all(
        feature = "cjk-compat",
        any(feature = "mediawiki-tw", feature = "opencc-tw")
    ))]
    #[test]
    fn zhconv_mw_keeps_rule_blocks_raw() {
        let out = crate::zhconv_mw("函數-{zh-tw:滑數;zh-cn:鼠标}-", crate::Variant::ZhTW);
        assert!(out.ends_with("滑數"), "rule block stays raw: {out:?}");
        assert_eq!(
            out.matches('數').count(),
            1,
            "prose normalized, rule untouched: {out:?}"
        );
    }

    fn shouty(s: &str) -> Cow<'_, str> {
        Cow::Owned(s.replace('b', "a"))
    }

    #[test]
    #[allow(clippy::type_complexity)]
    fn wikitext_preprocess_hook() {
        // Hook runs on prose spans. Plain `fn`s and non-capturing closures
        // both coerce to the pointer; bare `None` means raw conversion.
        let closed: for<'a> fn(&'a str) -> Cow<'a, str> = |s| Cow::Owned(s.replace('b', "a"));
        let c = ZhConverter::from_pairs([("a", "A")]);
        let hooks: [Option<for<'a> fn(&'a str) -> Cow<'a, str>>; 2] = [Some(shouty), Some(closed)];
        for hook in hooks {
            let mut out = String::new();
            c.convert_to_as_wikitext("b", &mut out, &mut None, false, false, hook);
            assert_eq!(out, "A");
        }
        let mut out = String::new();
        c.convert_to_as_wikitext("b", &mut out, &mut None, false, false, None);
        assert_eq!(out, "b");
    }

    #[test]
    fn test_allocation_policy_take_str() {
        let c = ZhConverter::from_pairs([("abc", "def")]);
        let input = "abc xyz";
        let out = c.convert(input);
        assert_eq!(out, "def xyz");
        let expected_cap = input.len() + ZhConverter::empirical_headroom(input.len());
        assert!(out.capacity() >= expected_cap);
    }

    #[test]
    fn test_allocation_policy_take_mut_string_preserves_capacity() {
        let c = ZhConverter::from_pairs([("abc", "def")]);
        let input = "abc xyz";
        // Preallocate buffer that accommodates the text and loop chunk_slack (16B SIMD write bound)
        let mut out = String::with_capacity(32);
        let initial_cap = out.capacity();
        assert!(initial_cap >= 32);
        c.convert_to(input, &mut out);
        assert_eq!(out, "def xyz");
        // Capacity must not have reallocated/expanded upfront because remaining was > 0
        assert_eq!(out.capacity(), initial_cap);
    }

    #[test]
    fn test_allocation_policy_take_mut_string_zero_remaining_reserves_headroom() {
        let c = ZhConverter::from_pairs([("abc", "def")]);
        let input = "abc xyz";
        let mut out = String::new();
        assert_eq!(out.capacity(), 0);
        c.convert_to(input, &mut out);
        assert_eq!(out, "def xyz");
        let expected_cap = input.len() + ZhConverter::empirical_headroom(input.len());
        assert!(out.capacity() >= expected_cap);
    }

    #[test]
    fn test_wikitext_allocation_policy_uses_input_len() {
        let c = ZhConverter::from_pairs([("abc", "def")]);
        let input = "abc xyz";
        let out_basic = c.convert_as_wikitext_basic(input);
        assert_eq!(out_basic, "def xyz");
        assert!(out_basic.capacity() >= input.len());
        // Plain convert reserves empirical_headroom (adds >= 32B), while wikitext preallocates input.len().
        let plain_out = c.convert(input);
        assert!(plain_out.capacity() > out_basic.capacity());

        let out_ext = c.convert_as_wikitext_extended(input);
        assert_eq!(out_ext, "def xyz");
        assert!(out_ext.capacity() >= input.len());
    }

    #[test]
    #[should_panic(expected = "exceeds 1023 bytes")]
    fn test_builder_oversized_word_panics() {
        let oversized = "a".repeat(1024);
        ZhConverter::from_pairs([("key", oversized.as_str())]);
    }

    #[test]
    fn test_long_target_word_fallback() {
        // Words > 16 bytes trigger the non-SIMD fallback copy branch
        let long_word = "a".repeat(32);
        let c = ZhConverter::from_pairs([("target", long_word.as_str())]);
        let out = c.convert("hello target world");
        assert_eq!(out, format!("hello {} world", "a".repeat(32)));
    }

    #[test]
    fn test_tiny_dictionary_target_bytes_under_16() {
        // Test that a converter whose entire target_bytes storage is small (< 16B)
        // converts correctly without out-of-bounds reads or underflow.
        let c = ZhConverter::from_pairs([("a", "b")]);
        let out = c.convert("a");
        assert_eq!(out, "b");
        let out2 = c.convert("aaa");
        assert_eq!(out2, "bbb");
        let out3 = c.convert("xayaz");
        assert_eq!(out3, "xbybz");
    }

    #[test]
    fn test_empty_string_conversion_zero_allocation() {
        let c = ZhConverter::from_pairs([("a", "b")]);
        let out = c.convert("");
        assert_eq!(out, "");
        assert_eq!(out.capacity(), 0);

        let out_basic = c.convert_as_wikitext_basic("");
        assert_eq!(out_basic, "");
        assert_eq!(out_basic.capacity(), 0);

        let out_ext = c.convert_as_wikitext_extended("");
        assert_eq!(out_ext, "");
        assert_eq!(out_ext.capacity(), 0);

        let out_gen = c.convert_as_wikitext("", &mut None, false, false, None);
        assert_eq!(out_gen, "");
        assert_eq!(out_gen.capacity(), 0);

        let mut out_to = String::new();
        c.convert_to_as_wikitext("", &mut out_to, &mut None, false, false, None);
        assert_eq!(out_to, "");
        assert_eq!(out_to.capacity(), 0);
    }
}
