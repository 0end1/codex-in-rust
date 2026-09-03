#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""对《用Rust造一个Codex》修订稿中所有 ```rust / ```rust,ignore 代码块做逐块
rustc 语法解析检查，并做状态分类：
  PASS      : 无 error（含仅 warnings）——单独即可编译
  CONTEXT   : 全部 error 为 error[E...]（未解析符号/类型推断等，疑似依赖前后文）
  SUSPECT   : 出现无 E 码的 error（多为 parse / 结构性问题），需人工复核
用法: python3 verify_tools/syntax_check.py <md路径> [--rustc <路径>]
"""
import re
import sys
import os
import json
import subprocess
import tempfile

MD = sys.argv[1] if len(sys.argv) > 1 else None
if not MD or not os.path.exists(MD):
    sys.exit("usage: syntax_check.py <md>")

RUSTC = os.environ.get("RUSTC", "rustc")
EDITION = "2021"

def blocks(md_path):
    """yield (lang, content, start_line, end_line, nearest_ch)"""
    ch_re = re.compile(r"^# 第\s*(\d+)\s*章")
    fence = re.compile(r"^```([\w,.-]*)\s*$")
    lines = open(md_path, encoding="utf-8").read().split("\n")
    cur_ch = "前置"
    out = []
    i = 0
    while i < len(lines):
        m = fence.match(lines[i])
        if m:
            lang = m.group(1)
            start = i + 1
            i += 1
            buf = []
            while i < len(lines) and not fence.match(lines[i]):
                cm = ch_re.match(lines[i])
                if cm:
                    cur_ch = int(cm.group(1))
                buf.append(lines[i])
                i += 1
            end = i  # line of closing fence
            yield lang, "\n".join(buf), start, end, cur_ch
        else:
            cm = ch_re.match(lines[i])
            if cm:
                cur_ch = int(cm.group(1))
            i += 1

def classify_err(stderr):
    lines = stderr.split("\n")
    errors = [l for l in lines if l.startswith("error")]
    errors = [l for l in errors if "aborting due to" not in l]
    if not errors:
        return "PASS", [], lines
    coded = [l for l in errors if re.search(r"error\[E\d+\]", l)]
    uncoded = [l for l in errors if not re.search(r"error\[E\d+\]", l)]
    # 无 E 码的 error（排除 aborting 收尾）通常是 parse / lint / 结构错误
    if uncoded and not coded:
        return "SUSPECT", errors, lines
    if uncoded and coded:
        return "SUSPECT", errors, lines  # 混合也保守上报
    return "CONTEXT", errors, lines

def snippet(lines, n=3):
    return [l.strip()[:160] for l in lines[:n]]

def main():
    stats = {"PASS": 0, "CONTEXT": 0, "SUSPECT": 0}
    detail = {"PASS": [], "CONTEXT": [], "SUSPECT": []}
    samples = {"CONTEXT": {}, "SUSPECT": {}}
    tmpdir = tempfile.mkdtemp(prefix="codex_verify_")
    idx = 0
    for lang, content, s, e, ch in blocks(MD):
        if lang not in ("rust", "rust,ignore"):
            continue
        idx += 1
        f = os.path.join(tmpdir, "b%03d.rs" % idx)
        with open(f, "w", encoding="utf-8") as fh:
            fh.write(content)
        try:
            r = subprocess.run(
                [RUSTC, "--edition", EDITION, "--crate-type", "lib",
                 "--crate-name", "block%03d" % idx, f],
                capture_output=True, text=True, timeout=120)
            err = r.stderr or ""
        except Exception as ex:
            err = "EXC: %r" % ex
        status, errs, raw = classify_err(err)
        rec = {"idx": idx, "ch": ch, "lines": "%d-%d" % (s, e),
               "errors": snippet(raw)}
        stats[status] += 1
        detail[status].append(rec)
        if status == "CONTEXT":
            key = errs[0][:100] if errs else ""
            samples["CONTEXT"].setdefault(key, []).append(rec)
        if status == "SUSPECT":
            samples["SUSPECT"].setdefault(rec["ch"], []).append(rec)
    out = {
        "md": MD, "edition": EDITION, "rustc": RUSTC,
        "stats": stats, "detail": detail,
        "by_ch": {},
    }
    for st in ("PASS", "CONTEXT", "SUSPECT"):
        by_ch = {}
        for rec in detail[st]:
            by_ch.setdefault(rec["ch"], []).append(rec)
        out["by_ch"][st] = by_ch
    report = {
        "stats": stats,
        "suspect_first_errors": {
            str(k): v for k, v in list(samples["SUSPECT"].items())[:60]
        },
    }
    base = os.path.dirname(MD)
    out_dir = os.path.join(base, "_verify_out")
    os.makedirs(out_dir, exist_ok=True)
    with open(os.path.join(out_dir, "syntax_check.json"), "w",
              encoding="utf-8") as fh:
        json.dump(out, fh, ensure_ascii=False, indent=1)
    print(json.dumps(report, ensure_ascii=False, indent=1))
    print("detail json -> _verify_out/syntax_check.json")

if __name__ == "__main__":
    main()
