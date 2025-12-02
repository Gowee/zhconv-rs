# Dataset

## Files

- `zhConversion.php` is sourced from [MediaWiki](https://github.com/wikimedia/mediawiki/blob/master/includes/languages/Data/ZhConversion.php) (licensed under GPLv2.0 or later) and `*.txt` are from OpenCC(https://github.com/BYVoid/OpenCC/tree/master/data) (licensed under Apache 2.0). They are used by `build.rs` to generate dictionaries and automata to be bundled.

- `update_basic.py` updates `zhConversion.php`, `*.txt` from upstream and relevant references in `build.rs` automatically.

- `update_cgroups.py` pulls and formats CGroups (common conversion groups) from Chinese Wikipedia into `cgroups/`. ([source](https://zh.wikipedia.org/w/index.php?search=CGroup&title=Special:%E6%90%9C%E7%B4%A2&profile=advanced&fulltext=1&ns10=1&ns828=1))

- `cgroups/merge_for_web.py` combines `cgroups/*.json` into a monolithic `cgroups.json` to be placed in `:/web/public/cgroups.json`, which is included in the web app. The libs and the cli are not referencing CGroups for now. 

## Licenses

Rulesets are neither maintained nor licensed by zhconv-rs. Check their sources for licenses.
