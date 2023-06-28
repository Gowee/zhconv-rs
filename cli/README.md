# zhconv-cli 中文简繁及地區詞轉換
zhconv-cli converts Chinese text among several scripts or regional variants (e.g. `zh-TW <-> zh-CN <-> zh-HK <-> zh-Hans <-> zh-Hant`), based on conversion rulesets from Mediawiki/Wikipedia and OpenCC.

For more information, check [zhconv-rs](https://github.com/Gowee/zhconv-rs).

```
USAGE:
    zhconv [FLAGS] [OPTIONS] <VARIANT> [--] [FILE]...

FLAGS:
        --wikitext      Treat the input text as wikitext and process inline conversion rules in MediaWiki syntax
        --dump-table    Dump the built-in conversion table, along with additional rules specified if any
    -h, --help          Prints help information
    -V, --version       Prints version information

OPTIONS:
        --rule <rules>...               Additional conversion rules in MediaWiki syntax (excluding -{, }-)
        --rules_file <rule-files>...    File(s) consisting of additional conversion rules in MediaWiki syntax (excluding
                                        -{, }-) seperated by LF

ARGS:
    <VARIANT>    Target variant to convert to (zh, zh-Hant, zh-Hans, zh-TW, zh-HK, zh-MO, zh-CN, zh-SG, zh-MY)
    <FILE>...    File(s) to convert in-place (omit for stdio)
```
