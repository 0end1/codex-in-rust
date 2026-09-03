#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""从修订稿抽取若干个“仅依赖 std + serde/thiserror”的错误/策略枚举，
拼成一个最小 crate 做真实 cargo 编译验证（离线，验证 derive/字段/命名语义）。
用法: python3 verify_tools/build_probe.py
"""
import re, os, shutil

MD = "用Rust造一个Codex_全书_修订稿.md"
text = open(MD, encoding="utf-8").read()
lines = text.split("\n")

def find_line(pred, start=0):
    for i in range(start, len(lines)):
        if pred(lines[i]):
            return i
    return -1

def grab_enum(name):
    """抽取 'pub enum NAME {' 所在行到平衡 } 的文本（连同前面最近的 derive 行）。"""
    idx = find_line(lambda l: re.match(r"^\s*pub enum %s\b" % name, l))
    assert idx is not None, name
    # 向上连带收集属性/注释行（derive、serde rename 等）
    start = idx
    while start > 0 and lines[start - 1].strip().startswith("#"):
        start -= 1
    depth = 0
    j = idx
    while j < len(lines):
        depth += lines[j].count("{") - lines[j].count("}")
        if depth == 0 and j > idx:
            break
        j += 1
    return "\n".join(lines[start:j + 1])

enums = ["ToolError", "PatchError", "HookError", "HookVerdict", "ApprovalPolicy", "LlmError"]
body = []
for e in enums:
    txt = grab_enum(e)
    body.append(txt)
    body.append("")

# LlmError 引用了 reqwest::Error —— 抽取后替换为占位错误以免依赖 reqwest（语义演示）
src_main = """// 自动生成：由《用Rust造一个Codex》修订稿抽取的错误/策略枚举的真实编译验证。
#![allow(dead_code, unused)]
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::HashMap;
use std::path::PathBuf;

// 占位：仅用于让 LlmError::Network(#[from] X) 无需外部 crate 也能编译
// （真实 reqwest::Error 实现了 Debug + Display + std::error::Error）
mod reqwest_stub {
    #[derive(Debug)]
    pub struct Error;
    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) }
    }
    impl std::error::Error for Error {}
}

""" + "\n".join(body) + """
// LlmError 的 Network 字段在正文引用 reqwest::Error；本 probe 用同名 stub 顶替以离线编译
fn main() {}
"""
# 修正 LlmError 里对 reqwest::Error 的引用（只出现在 Network 变体）
src_main = src_main.replace("Network(#[from] reqwest::Error),",
                            "Network(#[from] reqwest_stub::Error),")

probe_dir = os.path.join("verify_tools", "probe_crate")
os.makedirs(os.path.join(probe_dir, "src"), exist_ok=True)
with open(os.path.join(probe_dir, "Cargo.toml"), "w") as fh:
    fh.write('''[package]
name = "probe"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
thiserror = "2"
''')
with open(os.path.join(probe_dir, "src", "main.rs"), "w") as fh:
    fh.write(src_main)
print("probe written; enums:", enums)
