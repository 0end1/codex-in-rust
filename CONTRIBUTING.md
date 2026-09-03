# 贡献指南

欢迎任何形式贡献：勘误、代码修复、工具改进、结构建议、翻译讨论。

## 找活干

- [`good first issue`](https://github.com/<owner>/mini-codex/labels/good%20first%20issue)：小勘误，适合第一次来。
- [`勘误`](https://github.com/<owner>/mini-codex/labels/%E5%8B%98%E8%AF%AF) label 下的 issue。
- 报新 bug / 新勘误：用对应 issue 模板，别开空白 issue。

## 提勘误的黄金标准

位置精确（章/小节/行）+ 逐字原文 + 建议改法 + 影响程度。做不到“建议改法”没关系，
做到前三条就足够维护者十分钟内处理。

## 本地开发与校验

```bash
# 书稿改动：代码块一致性
tools/verify/run_all.sh            # 产物 *_verify_out/syntax_check.json

# 代码改动（code/ 落地后）
cd code && cargo fmt --check
cd code && cargo clippy --workspace --all-targets -- -D warnings
cd code && cargo test --workspace

# 书渲染
mdbook build book/ && open book/book/index.html
```

PR 模板里的自查清单不是摆设，逐条勾。CI 会重复跑一遍。

## 规矩（为什么是规矩，正文里有答案）

1. **正文与代码同 PR**：本仓库反对“代码改了书没跟”。正文引用的文件、符号，必须在同
   一 tag 的 `code/` 里真实存在。
2. **不改读者挑战的答案性质**：挑战没有官方答案。即使你知道，也请只在 Discussions
   讨论，不要往正文塞解法。
3. **不做与正文脱节的“更优架构”重构**：正文的演进顺序是唯一真相。想讨论架构，去
   Discussions；想实践，fork 一个你自己的分支。
4. **安全默认保守**：涉及执行权限、沙箱、审批逻辑的改动，默认 deny；放行必须要有正文
   第 10–12 章同款的机制与测试。
5. **版本锁定**：依赖一律进 `code/Cargo.toml` 的 `[workspace.dependencies]` 精确锁定；
   新增依赖需跑 `cargo deny check`。

## Review 流程

- 所有改动经 PR 合入 `main`；CI 全绿是底线。
- 勘误按影响程度给优先级；致命勘误（照做无法编译/误导关键结论）最优先处理。
- 维护者可能要求你把 PR 拆小——一行勘误别带十行重构。

## 鸣谢

合入的贡献者会进入各 Part 的致谢名单（随发布补充）。你的名字会留在 git 历史里，
这是开源最实在的署名。
