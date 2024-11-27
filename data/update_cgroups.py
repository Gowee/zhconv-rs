#!/usr/bin/env python3
import os
import re
import logging
import json
from itertools import count, chain
from mwclient import Site

CGROUPS_DIR = os.path.join(os.path.dirname(__file__), "cgroups")

logger = logging.getLogger(__name__)

REGEX_MODULE_NAME = re.compile(r"""name\s*=\s*(['"])(?P<name>.+?)(\1)""")
REGEX_MODULE_DESCRIPTION = re.compile(r"""description\s*=\s*(['"])(?P<desc>.*?)(\1)""")
# REGEX_RULE = re.compile(
#     r"""original\s*=\s*(["'])(?P<original1>.*?)\1\s*,\s*rule\s*=\s*(['"])(?P<content1>[^']+)\3|Item\(\s*(['"])(?P<original2>.*?)\5\s*,\s*([^"])(?P<content2>.+?)\7\s*\)"""
# )
REGEX_TEMPLATE_HEADER = re.compile(
    r"""{{\s*CGroupH\s*\|\s*name\s*=\s*(?P<name>[^|]+)\s*\|\s*desc\s*=\s*(?P<desc>.*?)\s*}}"""
)
REGEX_LANG = re.compile(r"\{\{lang\|[a-zA-z]{2}\|([^}]+)}}")
# For Module:CGroup
# REGEX_RULE1 = re.compile(
#     r"""original\s*=\s*(["'])(?P<original>.*?)\1\s*,\s*rule\s*=\s*(['"])(?P<conv>.+?)\3"""
# )
# REGEX_RULE2 = re.compile(
#     r"""rule\s*=\s*(["'])(?P<conv>.*?)\1\s*,\s*original\s*=\s*(['"])(?P<original>.+?)\3"""
# )
# REGEX_RULE3 = re.compile(
#     r"""(?P<original>)rule\s*=\s*(["'])(?P<conv>.*?)\2"""  # no original, e.g. [[Module:CGroup/OnePiece]]
# )
# REGEX_RULE4 = re.compile(
#     r"""Item\(\s*((['"])(?P<original>.*?)\2|nil)\s*,\s*(['"])(?P<conv>.+?)\4\s*\)"""
# )
# For [[Category:公共轉換組模板]]; e.g. [[Template:CGroup/People]], [[Template:CGroup/文學]]
REGEX_RULE5 = re.compile(
    r"""{{\s*(CI(tem(Hidden)?)?|CNoteA)\s*\|(\s*desc\s*=\s*[^|]*\s*\|)?\s*original\s*=\s*(?P<original>[^|]*?)\s*(\|\s*desc\s*=\s*[^|]*\s*)?\|\s*(1=)?\s*(?P<conv>[^}]+)(\|\s*desc\s*=\s*[^|]*\s*)?(\|\s*)?}}"""
)
REGEX_RULE6 = re.compile(
    r"""{{\s*(CI(tem(Hidden)?)?|CNoteA)\s*(\|\s*desc\s*=\s*[^|]*\s*)?\|\s*(1=)?\s*(?P<conv>[^|]+)\s*(\|\s*desc\s*=\s*[^|]*\s*)?(\|\s*original\s*=\s*(?P<original>.*?))?\s*(\|\s*desc\s*=\s*[^|]*\s*)?(\|\s*)?}}"""
)
REGEX_RULE7 = re.compile(
    r"""{{\s*CItemLan\s*\|\s*([12]=)?\s*(?P<conv>[^|]+)\s*(\|\s*([12]=)?\s*(?P<original>.*?))?\s*(\|\s*)?}}"""
)
REGEX_RULE8 = re.compile(
    r"""{{\s*CItemLan\/R\s*\|\s*([12]=)?\s*(?P<original>[^|]+)\|\s*([12]=)?\s*(?P<conv>.*?)\s*(\|\s*)?}}"""
)
# e.g. ※此字在您的系统上可能无法显示，因而变成空白、方块或问号。
REGEX_SPECIAL_CHAR_NOTICE = re.compile(r"""<span[^>]*>([^>]+)<\/span>""")

# REGEX_CGROUP_LIST = re.compile(
#     r"""{{\s*CGroup\/Item\|([^|]*)\|([^|]*)\|([^|]*)}}"""
# )  # only picks template, module ends with |module=yes}}


def cgroup_modules(site):
    # return
    # yield None
    for entry in site.search("Module:CGroup/"):
        title = entry["title"]
        if title in {"Module:CGroup/core", "Module:CGroup/preload"}:
            continue  # not targets
        if not title.startswith("Module:CGroup/"):
            continue  # e.g. Module:CGroupViewer
        if "/" in title.removeprefix("Module:CGroup/"):
            logger.info(f"Skip {title} (sub page)")
            continue
        if entry["size"] < 66 or "return require(" in entry["snippet"]:
            logger.info(f"Skip {title} (too small or redirecting)")
            continue
        yield title


def cgroup_templates(site):
    # page = site.pages['Template:CGroup/list']
    # seen = set()
    # for entry in REGEX_CGROUP_LIST.find_iter(page.text()):
    #     for i in range(1, 3 + 1):
    #         title = f'Template:CGroup/{entry.group(i)}'
    #         if title in seen: continue
    #         seen.add(title)
    # page = site.pages[title]
    # if page.exists:
    #     if len(text := page.text()) > 66 and not ("#重定向" in text[:100] or "#REDIECT" in text[:100]):
    #         yield page
    for entry in site.search("Template:CGroup"):
        title = entry["title"]
        if title in {
            "Template:CGroup/doc",
            "Template:CGroup/list",
            "Template:CGroup/preload",
            "Template:CGroup/sandbox",
            "Template:CGroup/CHead",
            "Template:CGroup/editintro",
            "Template:CGroup/New Style",
            # "Template:CGroup/Science" # including several other templates
        }:
            continue  # not targets
        if not title.startswith("Template:CGroup/"):
            continue
        if "/" in title.removeprefix("Template:CGroup/"):
            logger.info(f"Skip {title} (sub page)")
            continue
        if (
            entry["size"] < 66
            or "#重定向" in entry["snippet"]
            or "#REDIRECT" in entry["snippet"]
        ):
            logger.info(f"Skip {title} (too small or redirecting)")
            continue
        yield title


# def parse_lua_table(s):
#     def r(**kwargs):
#         return kwargs

#     s = s.strip()
#     if s.startswith("{") and s.endswith("}"):
#         s = s[1:-1]
#         try:
#             return eval(f"r({s})", {"__builtins__": {}, "r": r})
#         except (SyntaxError, NameError):
#             pass
#     return None


def parse_module_line(s):
    def parse_lua_args(s):
        def r(*args, **kwargs):
            return (args, kwargs)  # *args, kwargs

        try:
            return eval(f"r({s})", {"__builtins__": {}, "r": r, "nil": None})
        except (SyntaxError, NameError):
            return None

    s = s.strip().rstrip(",")
    if not s.startswith("--"):  # Lua comments
        if s.startswith("{") and s.endswith("}"):
            s = s[1:-1]
            if args := parse_lua_args(s):
                args = args[1]  # kwargs
                if "original" in args or "rule" in args:  # O.W. it might not be a rule
                    return (args.get("original", ""), args.get("rule", ""))
        elif s.startswith("Item(") and s.endswith(")"):
            s = s[5:-1]
            if (args := parse_lua_args(s)) and len(args[0]) >= 2:
                return args[0]  # positional args
    return None


def parse_template_line(s):
    return (
        m := (
            REGEX_RULE5.search(s)
            or REGEX_RULE6.search(s)
            or REGEX_RULE7.search(s)
            or REGEX_RULE8.search(s)
        )
    ) and (m.group("original"), m.group("conv"))


def fetch_cgroups(site):
    # ok_cnt = 0
    emptys = []
    seen = set()
    fails = []
    for nth, title in zip(
        count(1), chain(cgroup_modules(site), cgroup_templates(site))
    ):
        logger.info(f"Processing no.{nth} {title}")
        try:
            # name = title.split("/")[-1]
            # if name in seen:  # modules that appear first take higher precedence
            #     logger.info(f"Skip {title} (name already seen)")
            #     continue
            # # seen.add(name)
            page = site.pages[title]
            text = REGEX_LANG.sub(r"\1", page.text())
            if (
                title.startswith("Module:")
                and (mname := REGEX_MODULE_NAME.search(text))
                and (mdesc := REGEX_MODULE_DESCRIPTION.search(text))
            ):
                name = str(mname.group("name"))
                description = str(mdesc.group("desc"))
            elif mheader := REGEX_TEMPLATE_HEADER.search(text):
                name = mheader.group("name")
                description = mheader.group("desc")
            else:
                fails.append(title)
                logger.warning("Failed to parse " + title)
                continue
            if name in seen:  # modules that appear first take higher precedence
                logger.info(f"Skip {title} (name already seen)")
                continue
            seen.add(name)
            rules = []
            if title.startswith("Module:"):
                match_rule = parse_module_line
            else:  # Template:
                match_rule = parse_template_line

            for mrule in filter(
                None,
                map(
                    match_rule,
                    text.split("\n"),
                ),
            ):
                conv = REGEX_SPECIAL_CHAR_NOTICE.sub(
                    r"\1",
                    mrule[1].replace("{{=}}", "="),  # e.g. `=>` in {{CItem}}
                )
                rules.append(
                    {
                        "original": mrule[0],
                        "conv": conv,
                    }
                )

            cgroup = {
                "name": name,
                "description": description,
                "path": title,
                "rules": rules,
            }
            yield cgroup
            if len(rules) == 0:
                logger.warning(f"0 rules found in {title}")
                emptys.append(title)
            # ok_cnt += 1
        except KeyError:
            fails.append(title)
            logger.warning(f"Skip {title} (missing metadata)")
        except Exception:
            fails.append(title)
            logger.exception(f"Error when processing {title}")
    logger.info(
        f"{nth} group(s) successfully fetched with {len(emptys)} empty, {len(fails)} failed."
    )
    if emptys:
        logger.info(f"empty: " + ", ".join(emptys))
    if fails:
        logger.info(f"fail: " + ", ".join(fails))


# def fetch_module(site):
#     ok_cnt = 0
#     empty_cnt = 0
#     for nth, entry in zip(count(1), site.search("Module:CGroup/")):
#         try:
#             title = entry["title"]
#             if "/" in title.removeprefix("Module:CGroup/"):
#                 logger.debug(f"Skip {title} (sub page)")
#                 continue
#             if (
#                 entry["size"] < 66
#                 or "return require(" in entry["snippet"].lstrip()[:100]
#             ):
#                 logger.debug(f"Skip {title} (too small or redirecting)")
#                 continue
#             logger.info(f"Processing no.{nth} {title}")
#             page = site.pages[title]
#             text = page.text()
#             if (mname := REGEX_NAME.search(text)) and (
#                 mdesc := REGEX_DESCRIPTION.search(text)
#             ):
#                 name = str(mname.group("name"))
#                 description = str(mdesc.group("desc"))
#             else:
#                 logger.warning("Failed to parse " + title)
#                 continue
#             rules = []
#             for mrule in filter(
#                 None,
#                 map(
#                     lambda line: REGEX_RULE1.search(line)
#                     or REGEX_RULE2.search(line)
#                     or REGEX_RULE3.search(line)
#                     or REGEX_RULE4.search(line),
#                     filter(
#                         lambda line: not line.lstrip().startswith("--"),
#                         text.split("\n"),
#                     ),
#                 ),
#             ):
#                 rules.append(
#                     {
#                         "original": mrule.group("original"),
#                         "content": mrule.group("conv"),
#                     }
#                 )

#             cgroup = {
#                 "name": name,
#                 "description": description,
#                 "path": title,
#                 "rules": rules,
#             }
#             yield cgroup
#             if len(rules) == 0:
#                 logger.warning(f"0 rules found in {title}")
#                 empty_cnt += 1
#             ok_cnt += 1
#         except KeyError:
#             logger.warning(
#                 f"Skip {entry.get('title') or entry.get('pageid') or entry} (missing metadata)"
#             )
#         except Exception:
#             logger.exception(
#                 f"Error when processing {entry.get('title') or entry.get('pageid') or entry}"
#             )
#     logger.info(f"{ok_cnt}/{nth} modules successfully fetched with {empty_cnt} empty")


def main():
    logging.basicConfig(level=os.environ.get("LOGLEVEL") or logging.INFO)
    logger.info("Up and running...")
    if not os.path.exists("./cgroups"):
        os.mkdir(CGROUPS_DIR)
    site = Site("zh.wikipedia.org")
    # cgroups = []
    for cgroup in fetch_cgroups(site):
        name = (cgroup["path"] or "").split("/")[-1] or cgroup["name"]
        with open(os.path.join(CGROUPS_DIR, f"{name}.json"), "w") as f:
            f.write(json.dumps(cgroup, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
