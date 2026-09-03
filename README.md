# 用 Rust 造一个 Codex

> AI Agent 系统设计与 AI 软件工程 —— 用 Rust 从零实现一个 `mini-codex`，
> 第 22 章用它来开发它自己。

![CI](https://img.shields.io/github/actions/workflow/status/<owner>/codex-in-rust/ci.yml?label=ci)
[![在线阅读](https://img.shields.io/badge/%E5%9C%A8%E7%BA%BF%E9%98%85%E8%AF%BB-mdBook-blue)](https://<owner>.github.io/codex-in-rust/)
![License](https://img.shields.io/badge/%E6%96%87%E5%AD%97-CC%20BY--NC--ND%204.0-lightgrey)
![License](https://img.shields.io/badge/%E4%BB%A3%E7%A0%81-Apache--2.0-green)

---

## 这是什么 / 不是什么

**是什么**：一本随书代码开源的书。22 章，五个 Part，从“最小 Agent 循环”一路推到
“用 mini-codex 开发 mini-codex”（自举）。每一章结束，对应仓库里的一个 commit；
每个 Part 结束，打一个 `partN` tag —— 想从第 N 章开始？`git checkout partN`。

**名字**：本仓库是 `codex-in-rust`（对应书名《用 Rust 造一个 Codex》）；书中从零实现的产品、
CLI 与测试都叫 **`mini-codex`** —— 仓库名是书，产品名是代码。

**不是什么**：不是可以直接上生产的 Codex 竞品。它是教学参考实现——
它存在的目的是让你读得懂、拆得开、改得动，然后自己造一个。

> 状态：**仓库处于“书稿 + 工程骨架”阶段**。
> - 书稿已按章拆入 `book/src/`（mdBook 源），一致性检查工具已就位并接入 CI。
> - `code/` 的 Rust workspace 结构已定，正文最终实现按 `code/README.md` 的清单抽取归位中。
> - 按书前言承诺，`part0…part5` 六个 tag 将随实现逐步打出（`scripts/checkpoint-tag.sh`）。

## 快速开始（在线读 / 本地跟学）

```bash
# 在线阅读（mdBook，部署后生效）
open https://<owner>.github.io/codex-in-rust/

# 本地读
mdbook build book/ && open book/book/index.html

# 本地校验书稿代码块（改动 book 前请跑）
tools/verify/run_all.sh
```

想从某一部分开始实现（而不是从头读）：

```bash
git clone https://github.com/<owner>/codex-in-rust.git
git checkout part3          # 第 10 章起点：安全篇（tags 随实现陆续发布）
cd code && cargo test       # 书里绝大多数测试不联网，几毫秒跑完
```

## 仓库结构

```
codex-in-rust/
├─ book/                 # 书（mdBook 源：前言 / 5 Part / 22 章 / 附录）
├─ code/                 # 随书实现（Rust workspace，= part5 最终态，抽取归位中）
├─ tools/verify/         # 书稿 ↔ 代码一致性检查（逐块 rustc + 符号核验）
├─ scripts/              # checkpoint-tag 等工程脚本
└─ .github/              # CI 四流水线 + issue/PR 模板
```

## tag ↔ 章节 ↔ 能力

| tag | 覆盖章节 | 打完时应具备的能力 |
|---|---|---|
| `part0` | 第 1–2 章 | 起步（Cargo / workspace 决策） |
| `part1` | 第 3–6 章 | 能跑 tool loop |
| `part2` | 第 7–9 章 | 能真改文件、真执行命令 |
| `part3` | 第 10–12 章 | 安全篇完成 |
| `part4` | 第 13–16 章 | 会话可停、可续、可回滚 |
| `part5` | 第 17–22 章 | 完整版（自举） |

## 怎么读这本书

前言给了三种读法；推荐组合是 **先快速扫一遍**（1–2 天），再 **照章实现**（每 Part 打
一个 checkpoint），最后 **复盘 Design Rationale**。卡住时用 `cargo test` 定位——
书的验收测试就是“章末验收”的机器可判定版。

## 贡献

勘误（typo / 技术错误 / 代码不符）是我们最高价值的贡献，模板见
[`ISSUE_TEMPLATE/1_errata.md`](.github/ISSUE_TEMPLATE/1_errata.md)。
**读者挑战没有官方答案** —— 去
[Discussions](https://github.com/<owner>/codex-in-rust/discussions) 贴解法，互相评审，
维护者不揭底。

细节：见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 许可

- 书稿（`book/`、README）：[CC BY-NC-ND 4.0](LICENSE-TEXT)
- 代码与工具（`code/`、`tools/`、`.github/`、`scripts/`）：[Apache-2.0](LICENSE-CODE)

发布前待办：作者署名 / GitHub `<owner>` 占位替换 / `rust-toolchain.toml` 版本最终化。
