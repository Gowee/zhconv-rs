# zhconv-cli 中文简繁及地區詞轉換
zhconv-cli converts Chinese text among several scripts or regional variants (e.g. `zh-TW <-> zh-CN <-> zh-HK <-> zh-Hans <-> zh-Hant`), built on the top of [zhConversion.php](https://github.com/wikimedia/mediawiki/blob/master/includes/languages/data/ZhConversion.php#L14) conversion tables from Mediawiki, which is the one also used on Chinese Wikipedia.

For more information, check [zhconv-rs](https://github.com/Gowee/zhconv-rs).

```
USAGE:
    zhconv [FLAGS] [OPTIONS] <VARIANT> [--] [FILE]...

FLAGS:
        --mediawiki    Processes inline MediaWiki conversion rules in the input
    -h, --help         Prints help information
    -V, --version      Prints version information

OPTIONS:
        --rule <rules>...               Additional conversion rules
        --rules_file <rule-files>...    File(s) consisting of additional conversion rules seperated by LF
        --dfa <dfa>                     Whether to build DFA for AC automaton
                                         With DFA enabled by default, it is slower to warm up while faster to convert.
                                         Omit to let the program to determine by input size

ARGS:
    <VARIANT>    Target variant to convert to (zh, zh-Hant, zh-Hans, zh-TW, zh-HK, zh-MO, zh-CN, zh-SG, zh-MY)
    <FILE>...    File(s) to convert in-place (omit for stdin/out)
```
