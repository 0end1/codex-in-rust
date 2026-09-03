#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""把语法检查的 SUSPECT 块按“教学片段/真可疑”再分类。
逻辑：若块首行以 let / drop( / self. / .xxx / match / if / return / assert!
     / for / while / eprintln / tracing / handles. / session. / client. /
     Event:: / Item:: / data. / forward. / => / use ... 等非 item 起始，
     或块内容含明显省略标记（..Default::default()、{ .. } 顶层、// ... 等），
     视为 EXPECTED-FRAGMENT；否则按 REAL-SUSPECT 列出。
用法: python3 classify_suspects.py <md> [syntax_check.json 路径]
"""
import re, sys, json, os

MD = sys.argv[1]
default_json = os.path.join(os.path.dirname(os.path.abspath(MD)), "_verify_out", "syntax_check.json")
JSON = sys.argv[2] if len(sys.argv) > 2 else default_json
if not os.path.exists(JSON):
    sys.exit("找不到 %s —— 请先运行 syntax_check.py 生成产物" % JSON)
d = json.load(open(JSON))
lines = open(MD, encoding="utf-8").read().split("\n")

ITEM_START = re.compile(r"^(pub(\([^)]*\))?\s+)?(struct|enum|trait|type|fn|mod|impl|const|static|use|extern|async\s+fn|macro_rules!)")
def first_nonempty(b):
    for l in b:
        if l.strip():
            return l
    return ""

content_by_idx = {}
# 重新切块
fence_re = re.compile(r"^```([\w,.-]*)\s*$")
i = 0
ch = "前置"
ch_re = re.compile(r"^# 第\s*(\d+)\s*章")
idx = 0
while i < len(lines):
    m = fence_re.match(lines[i])
    if m and m.group(1) in ("rust", "rust,ignore"):
        s = i + 1
        i += 1
        buf = []
        while i < len(lines) and not fence_re.match(lines[i]):
            if ch_re.match(lines[i]):
                ch = int(ch_re.match(lines[i]).group(1))
            buf.append(lines[i]); i += 1
        idx += 1
        content_by_idx[idx] = buf
    else:
        if ch_re.match(lines[i]):
            ch = int(ch_re.match(lines[i]).group(1))
        i += 1

real, frag = [], []
for rec in d["detail"]["SUSPECT"]:
    b = content_by_idx[rec["idx"]]
    first = first_nonempty(b)
    stripped = first.lstrip()
    if ITEM_START.match(stripped) or stripped.startswith("#[") or stripped.startswith("#!"):
        real.append(rec)
    else:
        frag.append(rec)

print("EXPECTED-FRAGMENT:", len(frag), " REAL-SUSPECT:", len(real))
print("== REAL-SUSPECT ==")
for rec in sorted(real, key=lambda r: (str(r["ch"]), r["idx"])):
    b = content_by_idx[rec["idx"]]
    print("ch%-2s md[%s] idx%s :: %s" % (rec["ch"], rec["lines"], rec["idx"], first_nonempty(b).strip()[:100]))
    for e in rec["errors"][:2]:
        print("      ", e)
