# Dataset

## Files

- `zhConversion.php` is fetched and used by `build.rs` to generate built-in conversion tables for the lib. ([source](https://github.com/wikimedia/mediawiki/blob/master/includes/languages/data/ZhConversion.php))

- `cgroup_extractor.py` pulls and formats CGroups (common conversion groups) from Chinese Wikipedia into `cgroups/`. ([source](https://zh.wikipedia.org/w/index.php?search=CGroup&title=Special:%E6%90%9C%E7%B4%A2&profile=advanced&fulltext=1&ns10=1&ns828=1))

- `cgroups/merge_for_web.py` combines `cgroups/*.json` into a monolithic `cgroups.json` to be placed in `:/web/public/cgroups.json`, which is included in the web app. The libs and the cli are not referencing CGroups for now. 

## Licenses
Dataset are neither maintained nor licensed by zhconv-rs. Instead, they are licensed in upstream projects. Check their sources for licenses.
