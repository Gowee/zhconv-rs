# Files

`zhConversion.php` is fetched and used by `build.rs` to generate built-in conversion tables for the lib.

`cgroup_extractor.py` pulls and formats CGroups from Chinese Wikipedia into `cgroups/`.

`cgroups/merge_for_web.py` combines `cgroups/*.json` into a monolithic `cgroups.json` to be placed in `:/web/public/cgroups.json`, which is used by the web app.

The lib and the cli are not referencing CGroups for now. 
