[![CI status](https://github.com/Gowee/zhconv-rs/actions/workflows/main.yml/badge.svg)](https://github.com/Gowee/zhconv-rs/actions)
[![docs.rs](https://docs.rs/zhconv/badge.svg)](https://docs.rs/zhconv)
[![Crates.io](https://img.shields.io/crates/v/zhconv.svg)](https://crates.io/crates/zhconv)
[![PyPI version](https://img.shields.io/pypi/v/zhconv-rs)](https://pypi.org/project/zhconv-rs/)
[![NPM version](https://badge.fury.io/js/zhconv.svg)](https://www.npmjs.com/package/zhconv)
# zhconv-rs 中文简繁及地區詞轉換
zhconv-rs converts Chinese text among traditional/simplified scripts or regional variants (e.g. `zh-TW <-> zh-CN <-> zh-HK <-> zh-Hans <-> zh-Hant`), built on the top of rulesets from MediaWiki/Wikipedia and OpenCC.

The implementation is powered by the [Aho-Corasick](https://github.com/BurntSushi/aho-corasick) automaton, ensuring linear time complexity with respect to the length of input text and conversion rules (`O(n+m)`), processing dozens of MiBs text per second.

🔗 **Web App: https://zhconv.pages.dev** (powered by WASM)

⚙️ **Cli**: `cargo install zhconv-cli` or check [releases](https://github.com/Gowee/zhconv-rs/releases).

🦀 **Rust Crate**: `cargo add zhconv` (see doc comments and [cli/](https://github.com/Gowee/zhconv-rs/tree/main/cli) for examples)

🐍 **Python Package via PyO3**: `pip install zhconv-rs` (WASM with wheels)

<details open>
 <summary>Python snippet</summary>

```python
# > pip install zhconv_rs
# Convert with builtin rulesets:
from zhconv_rs import zhconv
assert zhconv("天干物燥 小心火烛", "zh-tw") == "天乾物燥 小心火燭"
assert zhconv("霧失樓臺，月迷津渡", "zh-hans") == "雾失楼台，月迷津渡"
assert zhconv("《-{zh-hans:三个火枪手;zh-hant:三劍客;zh-tw:三劍客}-》是亞歷山大·仲馬的作品。", "zh-cn", mediawiki=True) == "《三个火枪手》是亚历山大·仲马的作品。"
assert zhconv("-{H|zh-cn:雾都孤儿;zh-tw:孤雛淚;zh-hk:苦海孤雛;zh-sg:雾都孤儿;zh-mo:苦海孤雛;}-《雾都孤儿》是查尔斯·狄更斯的作品。", "zh-tw", True) == "《孤雛淚》是查爾斯·狄更斯的作品。"

# Convert with custom rules:
from zhconv_rs import make_converter
assert make_converter(None, [("天", "地"), ("水", "火")])("甘肅天水") == "甘肅地火"

import io
convert = make_converter("zh-hans", io.StringIO("䖏 处\n罨畫 掩画")) # or path to rule file
assert convert("秀州西去湖州近 幾䖏樓臺罨畫間") == "秀州西去湖州近 几处楼台掩画间"
```
</details>

**JS (Webpack)**: `npm install zhconv` or `yarn add zhconv` (WASM, [instructions](https://rustwasm.github.io/wasm-pack/book/tutorials/npm-browser-packages/using-your-library.html))

**JS in browser**: https://cdn.jsdelivr.net/npm/zhconv-web@latest (WASM)

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

## Supported variants

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

*Note:*  `zh-TW` and `zh-HK` are based on `zh-Hant`. `zh-CN` are based on `zh-Hans`. Currently, `zh-MO` shares the same rulesets with `zh-HK` unless additional rules are manually configured; `zh-MY` and `zh-SG` shares the same rulesets with `zh-CN` unless additional rules are manually configured. 
</details>

<!--
## Comparisions with other tools
- OpenCC: Dict::MatchPrefix (iterating from maxlen to minlen character by character to match) [https://github.dev/BYVoid/OpenCC/blob/21995f5ea058441423aaff3ee89b0a5d4747674c/src/Dict.cpp#L25](MatchPrefix), [segments converter](https://github.dev/BYVoid/OpenCC/blob/21995f5ea058441423aaff3ee89b0a5d4747674c/src/Conversion.cpp#L27) [segmentizer](https://github.dev/BYVoid/OpenCC/blob/21995f5ea058441423aaff3ee89b0a5d4747674c/src/MaxMatchSegmentation.cpp#L34)
- zhConversion.php: strtr (iterating from maxlen to minlen for every known key length to match) [https://github.dev/php/php-src/blob/217fd932fa57d746ea4786b01d49321199a2f3d5/ext/standard/string.c#L2974]
- zhconv-rs regex-based automaton
-->

## Performance
`cargo bench` on `Intel(R) Xeon(R) CPU @ 2.80GHz` (GitPod), without parsing inline conversion rules:
```
load zh2Hant            time:   [45.442 ms 45.946 ms 46.459 ms]
load zh2Hans            time:   [8.1378 ms 8.3787 ms 8.6414 ms]
load zh2TW              time:   [60.209 ms 61.261 ms 62.407 ms]
load zh2HK              time:   [89.457 ms 90.847 ms 92.297 ms]
load zh2MO              time:   [96.670 ms 98.063 ms 99.586 ms]
load zh2CN              time:   [27.850 ms 28.520 ms 29.240 ms]
load zh2SG              time:   [28.175 ms 28.963 ms 29.796 ms]
load zh2MY              time:   [27.142 ms 27.635 ms 28.143 ms]
zh2TW data54k           time:   [546.10 us 553.14 us 561.24 us]
zh2CN data54k           time:   [504.34 us 511.22 us 518.59 us]
zh2Hant data689k        time:   [3.4375 ms 3.5182 ms 3.6013 ms]
zh2TW data689k          time:   [3.6062 ms 3.6784 ms 3.7545 ms]
zh2Hant data3185k       time:   [62.457 ms 64.257 ms 66.099 ms]
zh2TW data3185k         time:   [60.217 ms 61.348 ms 62.556 ms]
zh2TW data55m           time:   [1.0773 s 1.0872 s 1.0976 s]
``` 

The benchmark was performed on a previous version that had only Mediawiki rulesets. In the newer version, with OpenCC rulesets activated by default, the performance may degrade ~2x.

## Differences with other converters
* `ZhConver{sion,ter}.php` of MediaWiki: zhconv-rs just takes conversion tables listed in [`ZhConversion.php`](https://github.com/wikimedia/mediawiki/blob/master/includes/languages/data/ZhConversion.php#L14). MediaWiki relies on the inefficient PHP built-in function [`strtr`](https://github.com/php/php-src/blob/217fd932fa57d746ea4786b01d49321199a2f3d5/ext/standard/string.c#L2974). Under the basic mode, zhconv-rs guarantees linear time complexity (`T = O(n+m)` instead of `O(nm)`) and single-pass scanning of input text. Optionally, zhconv-rs supports the same conversion rule syntax with MediaWiki.
* OpenCC: The [conversion rulesets](https://github.com/BYVoid/OpenCC/tree/master/data/dictionary) of OpenCC is independent of MediaWiki. The core [conversion implementation](https://github.dev/BYVoid/OpenCC/blob/21995f5ea058441423aaff3ee89b0a5d4747674c/src/Conversion.cpp#L27) of OpenCC is kinda similar to the aforementioned `strtr`. However, OpenCC supports pre-segmentation and maintains multiple rulesets which are applied successively. By contrast, the Aho-Corasick-powered zhconv-rs merges rulesets from MediaWiki and OpenCC in compile time and converts text in single-pass linear time, resulting in much more efficiency. Though, conversion results may differ in some cases.

## Limitations
The converter utilizes an aho-corasick automaton with the leftmost-longest matching strategy. This strategy gives priority to the leftmost-matched words or phrases. For instance, if a ruleset includes both `干 -> 幹` and `天干物燥 -> 天乾物燥`, the converter would prioritize `天乾物燥` because `天干物燥` gets matched earlier compared to `干` at a later position. The strategy is generally effective but may occasionally lead to unexpected results.

Futuremore, since an automaton is infeasible to update after being built, the converter must (re)build it from scratch for every ruleset. The automata for built-in rulesets (i.e. conversion tables) are built on demand and cached by default. However, if a short input text contains global conversion rules (in MediaWiki syntax like -{H|zh-hans:鹿|zh-hant:马}-), this process incurs a significant overhead, potentially being less efficient than a naive implementation.

## Credits
All rulesets that power the converter come from the [MediaWiki](https://github.com/wikimedia/mediawiki) project and [OpenCC](https://github.com/BYVoid/OpenCC).

The project takes the following projects/pages as references:
- https://github.com/gumblex/zhconv : Python implementation of `zhConver{ter,sion}.php`.
- https://github.com/BYVoid/OpenCC/ : Widely adopted Chinese converter.
- https://zh.wikipedia.org/wiki/Wikipedia:字詞轉換處理
- https://zh.wikipedia.org/wiki/Help:高级字词转换语法
- https://github.com/wikimedia/mediawiki/blob/master/includes/language/LanguageConverter.php
<!--- https://www.hankcs.com/nlp/simplified-traditional-chinese-conversion.html-->

## TODO
- [x] Support [Module:CGroup](https://zh.wikipedia.org/wiki/Module:CGroup)
- [ ] Propogate error properly with Anyhow and thiserror
- [x] Python lib
- [x] More exmaples in README
- [x] Add rulesets from OpenCC
