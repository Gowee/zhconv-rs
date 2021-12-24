[![Crates.io](https://img.shields.io/crates/v/zhconv.svg)](https://crates.io/crates/zhconv)
[![CI status](https://github.com/Gowee/zhconv-rs/actions/workflows/main.yml/badge.svg)](https://github.com/Gowee/zhconv-rs/actions)
# zhconv-rs 中文简繁及地區詞轉換
zhconv-rs in a Rust lib, cli tool, and also a web app to convert Chinese text among several script or region variants (e.g. `zh-TW <-> zh-CN <-> zh-HK <-> zh-Hans <-> zh-Hant`). 

It is built on the top of [zhConversion.php](https://github.com/wikimedia/mediawiki/blob/master/includes/languages/data/ZhConversion.php#L14) conversion tables from Mediawiki, which is the one also used on Chinese Wikipedia.

**Web App: https://zhconv.pages.dev/**

## Supported variants

| Target                                 | Tag       | Script  | Description                                 |
| -------------------------------------- | --------- | ------- | ------------------------------------------- |
| **S**implified **C**hinese / 简体中文  | `zh-Hans` | SC / 简 | W/O substituing region-specific words.  |
| **T**raditional **C**hinese / 繁體中文 | `zh-Hant` | TC / 繁 | Ditto.                                      |
| Chinese (Taiwan) / 臺灣正體            | `zh-TW`   | TC / 繁 | With Taiwan-specific words adapted.         |
| Chinese (Hong Kong) / 香港繁體         | `zh-HK`   | TC / 繁 | With Hong Kong-specific words adapted.      |
| Chinese (Macau) / 澳门繁體             | `zh-MO`   | TC / 繁 | With Taiwan-specific words adapted.         |
| Chinese (Mainland China) / 大陆简体    | `zh-CN`   | SC / 简 | With mainland China-specific words adapted. |
| Chinese (Singapore) / 新加坡简体       | `zh-SG`   | SC / 简 | With Singapore-specific words adapted.      |
| Chinese (Malaysia) / 大马简体          | `zh-MY`   | SC / 简 | With Malaysia-specific words adapted.       |

*Note:*  `zh-TW`, `zh-HK` and `zh-MO` are based on `zh-Hant`. `zh-CN`, `zh-MY` and `zh-SG` are based on `zh-Hans`. Currently, `zh-MO` is based on `zh-HK` with few addional Macau-specific words; `zh-MY` and `zh-SG` are both based on `zh-CN` with with few addional Singapore/Malaysia-specific words. 
<!--
## Comparisions with other tools
- OpenCC: Dict::MatchPrefix (iterating from maxlen to minlen character by character to match) [https://github.dev/BYVoid/OpenCC/blob/21995f5ea058441423aaff3ee89b0a5d4747674c/src/Dict.cpp#L25](MatchPrefix), [segments converter](https://github.dev/BYVoid/OpenCC/blob/21995f5ea058441423aaff3ee89b0a5d4747674c/src/Conversion.cpp#L27) [segmentizer](https://github.dev/BYVoid/OpenCC/blob/21995f5ea058441423aaff3ee89b0a5d4747674c/src/MaxMatchSegmentation.cpp#L34)
- zhConversion.php: strtr (iterating from maxlen to minlen for every known key length to match) [https://github.dev/php/php-src/blob/217fd932fa57d746ea4786b01d49321199a2f3d5/ext/standard/string.c#L2974]
- zhconv-rs regex-based automaton
-->

## Credits
The converter takes as the reference implementation.
## TODO
- [x] Support [Module:CGroup](https://zh.wikipedia.org/wiki/Module:CGroup)
