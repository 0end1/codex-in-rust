#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""把《用Rust造一个Codex》单文件草稿拆分为 mdBook 源（前言 + 5 Part + 22 章 + 附录）。

用法:  python3 book/scripts/split-book.py
输入:  book/scratch/book-draft.md
输出:  book/src/ 下 index.md / preface.md / partN.md / chapters/chNN.md / appendix.md
规则:
  - 第 1~19 行封面区        -> index.md
  - 前言区(# 前言 .. # 目录 前) -> preface.md
  - "# 目录" 区整段丢弃（mdBook 自带目录）
  - "# 第 N 章"  -> chapters/chNN.md
  - "# 第X部分　..." -> partN.md (X=一二三四五)
  - "# 附录" 起至 EOF -> appendix.md
"""
from pathlib import Path

SRC = Path(__file__).resolve().parents[1] / "scratch" / "book-draft.md"
OUT = Path(__file__).resolve().parents[1] / "src"

CN_NUM = {"一": 1, "二": 2, "三": 3, "四": 4, "五": 5}


def start_new(marker):
    """判断行是否开启新文件，返回输出相对路径；None 表示不切。"""
    line = marker.strip()
    if line.startswith("# 第 ") and " 章" in line:
        # "# 第 10 章　标题"
        num = line[3 : line.index(" 章")].strip()
        if num.isdigit():
            return f"chapters/ch{int(num):02d}.md"
    for cn, n in CN_NUM.items():
        if line.startswith(f"# 第{cn}部分"):
            return f"part{n}.md"
    if line.startswith("# 附录"):
        return "appendix.md"
    if line.startswith("# 目录"):
        return "DISCARD"
    return None


def main() -> None:
    lines = SRC.read_text(encoding="utf-8").splitlines(keepends=True)
    (OUT / "chapters").mkdir(parents=True, exist_ok=True)

    # 封面区(1-19 行, 不含前言) 与 前言区(20 行 # 前言 .. 210 行)
    assert lines[0].startswith("# 用 Rust"), lines[0]
    assert lines[19].startswith("# 前言"), lines[19]
    assert lines[210].startswith("# 目录"), lines[210]
    (OUT / "index.md").write_text("".join(lines[0:19]), encoding="utf-8")
    (OUT / "preface.md").write_text("".join(lines[19:210]), encoding="utf-8")

    # 从 "# 目录"(index 210) 开始逐段切分
    cur_name: str | None = None
    cur: list[str] = []
    emitted = {}
    for line in lines[210:]:
        name = start_new(line)
        if name == "DISCARD":
            if cur_name:  # flush 上一个段落（只可能在 # 目录 之前）
                emitted[cur_name] = "".join(cur)
            cur_name, cur = None, []
            continue
        if name:
            if cur_name:
                emitted[cur_name] = "".join(cur)
            cur_name, cur = name, [line]
            continue
        if cur_name is not None:
            cur.append(line)
    if cur_name:
        emitted[cur_name] = "".join(cur)

    for name, text in emitted.items():
        (OUT / name).write_text(text, encoding="utf-8")

    chs = sorted(int(p.stem[2:]) for p in (OUT / "chapters").glob("ch*.md"))
    parts = sorted(int(p.stem[4:]) for p in OUT.glob("part*.md"))
    print(f"输出目录: {OUT}")
    print(f"章节: {len(chs)} 个 -> ch{chs[0]:02d}..ch{chs[-1]:02d}")
    print(f"Part 封面: {len(parts)} 个 -> part{parts}")
    print("含附录:", (OUT / "appendix.md").exists())
    total = sum(p.stat().st_size for p in OUT.rglob("*.md"))
    print(f"总字节: {total:,}")


if __name__ == "__main__":
    main()
