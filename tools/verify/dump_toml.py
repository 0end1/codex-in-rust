#!/usr/bin/env python3
# -*- coding: utf-8 -*-
import re, sys
MD = "用Rust造一个Codex_全书_修订稿.md"
lines = open(MD, encoding="utf-8").read().split("\n")
f = re.compile(r"^```([\w,.-]*)\s*$")
i = 0
while i < len(lines):
    m = f.match(lines[i])
    if m and m.group(1) == "toml":
        print("### toml block at md line", i + 1)
        i += 1
        while i < len(lines) and not f.match(lines[i]):
            print(lines[i]); i += 1
        print()
    else:
        i += 1
