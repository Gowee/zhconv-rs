[![CI status](https://github.com/Gowee/zhconv-rs/actions/workflows/main.yml/badge.svg)](https://github.com/Gowee/zhconv-rs/actions)
[![docs.rs](https://docs.rs/zhconv/badge.svg)](https://docs.rs/zhconv)
[![Crates.io](https://img.shields.io/crates/v/zhconv.svg)](https://crates.io/crates/zhconv)
[![PyPI version](https://img.shields.io/pypi/v/zhconv-rs)](https://pypi.org/project/zhconv-rs/)
[![NPM version](https://badge.fury.io/js/zhconv.svg)](https://www.npmjs.com/package/zhconv)
# zhconv-rs 中文简繁及地區詞轉換
zhconv-rs converts Chinese text among several scripts or regional variants (e.g. `zh-TW <-> zh-CN <-> zh-HK <-> zh-Hans <-> zh-Hant`), built on the top of [zhConversion.php](https://github.com/wikimedia/mediawiki/blob/master/includes/languages/data/ZhConversion.php#L14) conversion tables from Mediawiki, which is the one also used on Chinese Wikipedia.

**Web App: https://zhconv.pages.dev/** (powered by WASM)

**Cli**: `cargo install zhconv-cli` or check [releases](https://github.com/Gowee/zhconv-rs/releases)(TODO).

**Rust Crate**:
```toml
[dependencies]
zhconv = "0.1" # see doc comments and cli/ for examples
```

**Python Package via PyO3**:
```sh
pip install zhconv-rs
# >>> from zhconv_rs import zhconv
# >>> assert zhconv("霧失樓臺，月迷津渡", "zh-hans") == "雾失楼台，月迷津渡"
```

**JS (Webpack)**: `npm install zhconv` or `yarn add zhconv`

**JS in browser**: `https://cdn.jsdelivr.net/npm/zhconv-web@latest/zhconv.js`

<details>
 <summary>HTML snippet code</summary>

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

*Note:*  `zh-TW` and `zh-HK` are based on `zh-Hant`. `zh-CN` are based on `zh-Hans`. Currently, `zh-MO` shares the same conversion table with `zh-HK` unless additonal rules / CGroups are applied; `zh-MY` and `zh-SG` shares the same conversion table with `zh-CN` unless additional rules / CGroups are applied. 
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

<!--
## Differences between other tools
* `ZhConver{sion,ter}.php` of MediaWiki: zhconv-rs are just based on conversion tables listed in `ZhConversion.php`. MediaWiki relies on the inefficient PHP built-in function [`strtr`](https://github.com/php/php-src/blob/217fd932fa57d746ea4786b01d49321199a2f3d5/ext/standard/string.c#L2974). Under the basic mode, zhconv-rs guarantees linear time complexity with single-pass scanning of input text. Optionally, zhconv-rs supports the same conversion rule syntax with MediaWiki.
* OpenCC: OpenCC has self-maintained conversion tables that are different from MediaWiki. The [converter implementation](https://github.dev/BYVoid/OpenCC/blob/21995f5ea058441423aaff3ee89b0a5d4747674c/src/Conversion.cpp#L27) of OpenCC is kinda similar to the aforementioned `strtr`. zhconv-rs uses the [Aho-Corasick](https://docs.rs/aho-corasick/) algorithm, which would be much faster in general.

All of these implementation shares the same leftmost-longest matching strategy. So conversion results should generally be the same given the same conversion tables.
-->

## Limitations
The converter is implemented upon a aho-corasick automaton with the leftmost-longest matching strategy. That is, leftest matched words or phrases always take a higher priority. For example, if both `干 -> 幹` and `天干物燥 -> 天乾物燥` are specified in a ruleset, `天乾物燥` would be picked since `天干物燥` would be matched earlier at the initial position compared to `干` at a latter position. The strategy works well most of the time. But it might also result in some unexpected cases, rarely.

Besides, since an automaton is infeasible to update after being built, the converter will have to (re)build it from scratch for every ruleset. All automata for built-in rulesets (i.e. conversion tables) are built on demand and cached by default. But, typically, such overhead would be significant if there are global conversion rules (in MediaWiki syntax like `-{H|zh-hans:鹿|zh-hant:马}-`) in a short text (even less efficient than a naïve implementation).

## Credits
All data that powers the converter, including conversion tables and CGroups, comes from the MediaWiki project.

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
- [ ] More exmaples in README
