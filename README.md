[![CI status](https://github.com/Gowee/zhconv-rs/actions/workflows/main.yml/badge.svg)](https://github.com/Gowee/zhconv-rs/actions)
[![docs.rs](https://docs.rs/zhconv/badge.svg)](https://docs.rs/zhconv)
[![Crates.io](https://img.shields.io/crates/v/zhconv.svg)](https://crates.io/crates/zhconv)
[![PyPI version](https://img.shields.io/pypi/v/zhconv-rs)](https://pypi.org/project/zhconv-rs/)
[![NPM version](https://badge.fury.io/js/zhconv.svg)](https://www.npmjs.com/package/zhconv)

# zhconv-rs — 中文简繁及地區詞轉換

zhconv-rs converts Chinese between Traditional, Simplified and regional variants, using rulesets sourced from [MediaWiki](https://github.com/wikimedia/mediawiki)/Wikipedia and [OpenCC](https://github.com/BYVoid/OpenCC), which are merged, flattened and prebuilt into [Aho‑Corasick](https://en.wikipedia.org/wiki/Aho–Corasick_algorithm) automata for single-pass, linear-time conversions.

🔗 **Web app (wasm):** <https://zhconv.pages.dev>

⚙️ **Cli**: `cargo install zhconv` or download from [releases](https://github.com/Gowee/zhconv-rs/releases)

🦀 **Rust crate**: `cargo add zhconv` (see [docs](https://docs.rs/zhconv/latest/zhconv/) for details)

```rust
use zhconv::{zhconv, Variant};
assert_eq!(zhconv("雾失楼台，月迷津渡", Variant::ZhTW), "霧失樓臺，月迷津渡");
assert_eq!(zhconv("驛寄梅花，魚傳尺素", "zh-Hans".parse().unwrap()), "驿寄梅花，鱼传尺素");
```

🐍 **Python package**: `pip install zhconv-rs` or `pip install zhconv-rs-opencc` (for additional OpenCC dictionaries)

```python
from zhconv_rs import zhconv
assert zhconv("天干物燥 小心火烛", "zh-tw") == "天乾物燥 小心火燭"
```

<details>
 <summary>More usage</summary>

```python
assert zhconv("《-{zh-hans:三个火枪手;zh-hant:三劍客;zh-tw:三劍客}-》是亞歷山大·仲馬的作品。", "zh-cn", mediawiki=True) == "《三个火枪手》是亚历山大·仲马的作品。"
assert zhconv("-{H|zh-cn:雾都孤儿;zh-tw:孤雛淚;zh-hk:苦海孤雛;zh-sg:雾都孤儿;zh-mo:苦海孤雛;}-《雾都孤儿》是查尔斯·狄更斯的作品。", "zh-tw", True) == "《孤雛淚》是查爾斯·狄更斯的作品。"

# Customize conversion tables:
from zhconv_rs import make_converter
assert make_converter(None, [("天", "地"), ("水", "火")])("甘肅天水") == "甘肅地火"

import io
convert = make_converter("zh-hans", io.StringIO("䖏 处\n罨畫 掩画")) # or path to rule file
assert convert("秀州西去湖州近 幾䖏樓臺罨畫間") == "秀州西去湖州近 几处楼台掩画间"
```

</details>

<a href="https://deploy.workers.cloudflare.com/?url=https://github.com/gowee/zhconv-rs">
    <img src="https://deploy.workers.cloudflare.com/button" align="right" alt="Deploy to Cloudflare Workers">
</a>

🧩 **API demo**: <https://zhconv.bamboo.workers.dev>

**Node.js package**: `npm install zhconv` or `yarn add zhconv`

**JS in browser**: <https://cdn.jsdelivr.net/npm/zhconv-web@latest>

<details>
 <summary>HTML snippet</summary>

```html
<script type="module">
    // Use ES module import syntax to import functionality from the module
    // that we have compiled.
    //
    // Note that the `default` import is an initialization function which
    // will "boot" the module and make it ready to use. Currently browsers
    // don't support natively imported WebAssembly as an ES module, but
    // eventually the manual initialization won't be required!
    import init, { zhconv } from 'https://cdn.jsdelivr.net/npm/zhconv-web@latest/zhconv.js'; // specify a version tag if in prod

    async function run() {
        await init();

        alert(zhconv(prompt("Text to convert to zh-hans:"), "zh-hans"));
    }

    run();
</script>
```


</details>

## Variants and conversion tables

Unlike OpenCC, whose dictionaries are bidirectional (e.g., `s2t`, `tw2s`), zhconv-rs follows MediaWiki’s approach and provides one conversion table per target variant:

<details>
 <summary>zh-Hant, zh-Hans, zh-TW, zh-HK, zh-MO, zh-CN, zh-SG, zh-MY</summary>

| Target                                 | Tag       | Script  | Description                                   |
| -------------------------------------- | --------- | ------- | --------------------------------------------- |
| **S**implified **C**hinese / 简体中文  | `zh-Hans` | SC / 简 | W/O substituing region-specific phrases.      |
| **T**raditional **C**hinese / 繁體中文 | `zh-Hant` | TC / 繁 | W/O substituing region-specific phrases.      |
| Chinese (Taiwan) / 臺灣正體            | `zh-TW`   | TC / 繁 | With Taiwan-specific phrases adapted.         |
| Chinese (Hong Kong) / 香港繁體         | `zh-HK`   | TC / 繁 | With Hong Kong-specific phrases adapted.      |
| Chinese (Macau) / 澳门繁體             | `zh-MO`   | TC / 繁 | Same as `zh-HK` for now.                      |
| Chinese (Mainland China) / 大陆简体    | `zh-CN`   | SC / 简 | With mainland China-specific phrases adapted. |
| Chinese (Singapore) / 新加坡简体       | `zh-SG`   | SC / 简 | Same as `zh-CN` for now.                      |
| Chinese (Malaysia) / 大马简体          | `zh-MY`   | SC / 简 | Same as `zh-CN` for now.                      |

*Note:*  `zh-TW` and `zh-HK` are derived from `zh-Hant`. `zh-CN` is derived from `zh-Hans`. Currently, `zh-MO` shares the same dictionary as `zh-HK`, and `zh-MY`/`zh-SG` share the same dictionary as `zh-CN`, unless additional rules are provided.
</details>

Chained dictionary groups from OpenCC are flattened and merged with the MediaWiki conversion table for each target variant, then compiled into an Aho-Corasick automaton at compile-time. After internal compression, the bundled conversion tables and automata occupy ~0.6 MiB (with MediWiki enabled only) or ~2.7 MiB (with both MediaWiki and OpenCC enabled).

## Performance

Even with all rulesets enabled, zhconv-rs remains faster than most alternatives. Check with `cargo bench compare --features bench,mediawiki,opencc`:

![Comparison with other crates, targetting zh-Hans](violin-to-hans.svg)
![Comparison with other crates, targetting zh-TW](violin-to-tw.svg)

Conversion runs in a single pass in `O(n+m)` linear time by default, where `n` is the length of the input text and `m` is the maximum length of source word in conversion tables, regardless of which rulesets are enabled. When converting wikitext containing MediaWiki conversion rules, the time complexity may degrade to `O(n*m)` in the worst case, if the corresponding function or flag is explicitly chosen.

On a typical modern PC, prebuilt converters load in a few milliseconds with default features (~2–5 ms). Enabling the optional opencc feature increases load time (typically 20–25 ms per target). Throughput generally ranges from 100–200 MB/s.

`cargo bench base --features bench` on `AMD EPYC 7B13` (GitPod) by v0.3:

<details>
<summary>Using conversion tables sourced from MediaWiki by default</summary>

```
load/zh2Hant            time:   [4.6368 ms 4.6862 ms 4.7595 ms]
load/zh2Hans            time:   [2.2670 ms 2.2891 ms 2.3138 ms]
load/zh2TW              time:   [4.7115 ms 4.7543 ms 4.8001 ms]
load/zh2HK              time:   [5.4438 ms 5.5474 ms 5.6573 ms]
load/zh2MO              time:   [4.9503 ms 4.9673 ms 4.9850 ms]
load/zh2CN              time:   [3.0809 ms 3.1046 ms 3.1323 ms]
load/zh2SG              time:   [3.0543 ms 3.0637 ms 3.0737 ms]
load/zh2MY              time:   [3.0514 ms 3.0640 ms 3.0787 ms]
zh2CN wikitext basic    time:   [385.95 µs 388.53 µs 391.39 µs]
zh2TW wikitext basic    time:   [393.70 µs 395.16 µs 396.89 µs]
zh2TW wikitext extended time:   [1.5105 ms 1.5186 ms 1.5271 ms]
zh2CN 天乾物燥          time:   [46.970 ns 47.312 ns 47.721 ns]
zh2TW data54k           time:   [200.72 µs 201.54 µs 202.41 µs]
zh2CN data54k           time:   [231.55 µs 232.86 µs 234.30 µs]
zh2Hant data689k        time:   [2.0330 ms 2.0513 ms 2.0745 ms]
zh2TW data689k          time:   [1.9710 ms 1.9790 ms 1.9881 ms]
zh2Hant data3185k       time:   [15.199 ms 15.260 ms 15.332 ms]
zh2TW data3185k         time:   [15.346 ms 15.464 ms 15.629 ms]
zh2TW data55m           time:   [329.54 ms 330.53 ms 331.58 ms]
is_hans data55k         time:   [404.73 µs 407.11 µs 409.59 µs]
infer_variant data55k   time:   [1.0468 ms 1.0515 ms 1.0570 ms]
is_hans data3185k       time:   [22.442 ms 22.589 ms 22.757 ms]
infer_variant data3185k time:   [60.205 ms 60.412 ms 60.627 ms]
```

</details>

<details>
<summary>Using conversion tables derived from OpenCC additionally (`--features opencc`)</summary>

```
load/zh2Hant            time:   [22.074 ms 22.338 ms 22.624 ms]
load/zh2Hans            time:   [2.7913 ms 2.8126 ms 2.8355 ms]
load/zh2TW              time:   [23.068 ms 23.286 ms 23.520 ms]
load/zh2HK              time:   [23.358 ms 23.630 ms 23.929 ms]
load/zh2MO              time:   [23.363 ms 23.627 ms 23.913 ms]
load/zh2CN              time:   [3.6778 ms 3.7222 ms 3.7722 ms]
load/zh2SG              time:   [3.6522 ms 3.6848 ms 3.7202 ms]
load/zh2MY              time:   [3.6642 ms 3.7079 ms 3.7545 ms]
zh2CN wikitext basic    time:   [396.17 µs 402.51 µs 409.36 µs]
zh2TW wikitext basic    time:   [442.16 µs 447.53 µs 453.27 µs]
zh2TW wikitext extended time:   [1.5795 ms 1.6007 ms 1.6233 ms]
zh2CN 天乾物燥          time:   [47.884 ns 48.878 ns 49.953 ns]
zh2TW data54k           time:   [255.25 µs 259.01 µs 262.92 µs]
zh2CN data54k           time:   [233.74 µs 236.99 µs 240.67 µs]
zh2Hant data689k        time:   [3.9696 ms 4.0005 ms 4.0327 ms]
zh2TW data689k          time:   [3.4593 ms 3.4896 ms 3.5203 ms]
zh2Hant data3185k       time:   [27.710 ms 27.955 ms 28.206 ms]
zh2TW data3185k         time:   [30.298 ms 30.858 ms 31.428 ms]
zh2TW data55m           time:   [500.95 ms 515.80 ms 531.34 ms]
is_hans data55k         time:   [461.22 µs 470.99 µs 481.20 µs]
infer_variant data55k   time:   [1.1669 ms 1.1759 ms 1.1852 ms]
is_hans data3185k       time:   [26.609 ms 26.964 ms 27.385 ms]
infer_variant data3185k time:   [74.878 ms 76.262 ms 77.818 ms]
```

</details>

## Limitations

### Accuracy

Rule-based converters cannot capture every possible linguistic nuance. Like most others, the implementation employs a leftmost-longest matching strategy (a.k.a forward maximum matching), prioritizing to the earliest and longest matches in the text. For example, if a ruleset contains both `干 → 幹`, `天干 → 天干`, and `天干物燥 → 天乾物燥`, the converter will prefer the longer match `天乾物燥`, since it appears earlier and spans more characters. This generally works well but may cause occasional mis-conversions.

### Wikitext support

The implementation supports most MediaWiki conversion syntax, while not fully compliant with the original MediaWiki implementation.

Since rebuilding automata dynamically is impractical, rules (e.g., `-{H|zh-hans:鹿|zh-hant:马}-` in MediaWiki syntax) in text are extracted in a first pass, a temporary automaton is constructed, and the text is converted in a second pass. The time complexity may degrade to `O(n*m)` in the worst case, where `n` is the input text length and `m` is the maximum length of source words in dictionaries, which is equivalent to a brute-force approach.

## License

The library itself is licensed under MIT OR Apache-2.0, at the licensee’s option. **BUT** it may bundle:

- Conversion tables from MediaWiki (the default, gated by the feature `mediawiki`) which are licensed under GPL-2.0-or-later.
- Dictionaries from OpenCC (gated by the feature `opencc`)  licensed under Apache-2.0.

For MIT compatibility, disable the default `mediawiki` feature and enable `opencc` (with optional `compress`, recommended for reducing binary size) to use prebuilt converters and tables.

## Credits

Rulesets: [MediaWiki](https://github.com/wikimedia/mediawiki) and [OpenCC](https://github.com/BYVoid/OpenCC).

Fast double-array Aho-Corasick automata implementation in Rust: [daachorse](https://github.com/daac-tools/daachorse)

References & related implementations:

- <https://github.com/gumblex/zhconv> : Python implementation of `zhConver{ter,sion}.php`.
- <https://github.com/BYVoid/OpenCC/> : Widely adopted Chinese converter.
- <https://zh.wikipedia.org/wiki/Wikipedia:字詞轉換處理>
- <https://zh.wikipedia.org/wiki/Help:高级字词转换语法>
- <https://github.com/wikimedia/mediawiki/blob/master/includes/language/LanguageConverter.php>
