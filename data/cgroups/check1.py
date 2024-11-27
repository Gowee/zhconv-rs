#!/usr/bin/env python3
import os
from glob import glob
import json

CGROUPS_DIR = os.path.join(os.path.dirname(__file__), "./")
OUTPUT_PATH = os.path.join(os.path.dirname(__file__), "../../web/public/cgroups.json")


def combine_names(name, desc):
    # TODO: handle conv rule
    if name in desc:
        combined = desc
    elif desc in name:
        combined = name
    else:
        combined = f"{name} / {desc}"
    return combined


def check_conv(conv):
    # packed = "\n".join(rule['conv'] for rule in rules)
    # packed = ""
    # for rule in rules:
    #     # rule['original'] is unused for now
    #     packed += f"-{{H|{rule['conv']}}}-"
    if "=>" in conv:
        ff = None
        for single in filter(None, map(lambda s: s.strip(), conv.split(""))):
            if "=>" not in single:
                print("E1", conv, f"no => in {single}")
                break
            f, t = single.split("=>")
            f = f.strip()
            t = t.strip()
            if ff is None:
                ff = f
            elif f != ff:
                print("E2", conv, f"{f} != {ff}")
                break
    # return packed


def main():
    cgroups = {}
    for f in glob(os.path.join(CGROUPS_DIR, "*.json")):
        with open(f, "r") as f:
            cgroup = json.loads(f.read())
            # name = combine_names(cgroup["name"], cgroup['description'])
            for rule in cgroup["rules"]:
                check_conv(rule["conv"])
            # cgroups[name] = rules
    with open(OUTPUT_PATH, "w") as f:
        f.write(json.dumps(cgroups, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
