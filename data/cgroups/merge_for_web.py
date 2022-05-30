#!/usr/bin/env python3
import os
from glob import glob
import json
import re
from datetime import datetime

CGROUPS_DIR = os.path.join(os.path.dirname(__file__), "./")
OUTPUT_PATH = os.path.join(os.path.dirname(__file__), "../../web/public/cgroups.json")

REGEX_LINK = re.compile(r"\[\[(.+?)(\|.+?)?\]\]")

def combine_names(name, desc):
    # TODO: handle conv rule
    name = name.strip()
    desc = desc.strip()
    if name in desc:
        combined = desc
    elif desc in name:
        combined = name
    else:
        combined = f"{name} / {desc}"
    combined = REGEX_LINK.sub(r"\1", combined)
    return combined


def pack_rules(rules):
    packed = "\n".join(rule['conv'] for rule in rules)
    # packed = "" 
    # for rule in rules:
    #     # rule['original'] is unused for now
    #     packed += f"-{{H|{rule['conv']}}}-"
    return packed

def now():
    return datetime.now().timestamp()


def main():
    cgroups = {}
    for f in glob(os.path.join(CGROUPS_DIR, "*.json")):
        with open(f, "r") as f:
            cgroup = json.loads(f.read())
            name = combine_names(cgroup["name"], cgroup['description'])
            rules = pack_rules(cgroup["rules"])
            cgroups[name] = rules
    with open(OUTPUT_PATH, "w") as f:
        f.write(json.dumps({'timestamp': now(), 'data': cgroups}, ensure_ascii=False, indent=2))
    # Remember to apply `zhconv --mediawiki Zh` to the final json for rules in titles/descriptions.


if __name__ == "__main__":
    main()
