# 附录

## 附录 A　Rust 知识点路线图（新手查表）

| 章节 | 语言特性 | 标准库 / 生态 crate |
|---|---|---|
| 1–2 | 所有权入门、`Result`/`?`、module 与 crate 可见性 | `reqwest`、Cargo workspace |
| 3–4 | `async/await`、enum + match、`Option` 组合子 | `tokio`、`futures::Stream` |
| 5–6 | trait、trait object、`Box<dyn Trait>`、derive | `serde`、`thiserror`、`async-trait` |
| 7–9 | 生命周期、迭代器、`#[cfg]` 条件编译 | `tokio::process`、`ignore` |
| 10–11 | 策略模式、错误处理分层、`#[cfg]` 条件编译、FFI 与 `unsafe` 隔离 | `landlock`、`seccompiler`、`nix` |
| 12–13 | 环境自检与自动降级、正则、子进程执行外部钩子 | `regex`、`toml` |
| 14–16 | `Option` 三态、不可变数据、`Arc` | `toml`、`rusqlite`、`notify` |
| 17–19 | `select!`、取消令牌、并发共享 | `tokio`、`ratatui`、`crossterm` |
| 20–22 | `JoinSet`、自定义测试 harness、发布构建 | `tracing`、`insta`、`criterion` |

## 附录 B　Spec 写作模板（全书通用）

写一个 spec 让它成为契约而非感想，需要四样东西：

1. **可判定的验收条件**：“p95 < 200ms @ 50 QPS”，而不是“要快”。
2. **带脏数据的范例**：空列表、重复提交、时区边界、用户在金额框里粘贴 emoji。这些范例就是测试用例的草稿。
3. **不变量**：重构后仍必须成立的性质（余额不为负、每次状态转移都要记日志、调用支付必须带幂等键）。
4. **非目标**：明确列出“这次改动不许碰什么”。**agent 的失败模式是做得太多，不是做得太少**。

## 附录 C　每章的 Design Rationale 速查

| 决策 | 备选方案 | 为什么选它 |
|---|---|---|
| Op/Event 队列对 | 直接循环调用 | 界面与引擎速度不匹配，晚解耦必重写 |
| apply_patch DSL | unified diff | 模型会复述、不会数行号 |
| 审批 ⊥ 沙箱 | 单一档位 | “要不要问我”与“最坏坏到哪”是两件事 |
| 默认开沙箱 | 可选开启 | 靠人记得开启的机制等于没有 |
| `.git`/`.codex` 单独只读 | 只锁可写根 | 否则 agent 能改 git hooks 自我提权 |
| 分层 AGENTS.md | 单一大文件 | 上下文预算要跟任务范围成比例 |
| JSONL append-only | SQLite 主存 | 可 diff、可回放、崩溃友好 |
| 压缩切点在 user turn | 任意位置切 | 避免造出有问无答的残缺历史 |
| 细粒度 Item | 粗粒度消息 | 渲染、审计、评测、压缩全靠它 |
| 独立 helper 二进制做沙箱 | 库内实现 | 限制必须在 exec 前施加且不可被解除 |
| 依赖方向机械强制 | 文档约定 | agent 不看文档，但逃不过编译错误 |

---

## 写作与配套建议

- **代码仓库**：随书公开 mini-codex 仓库，每个 Part 一个 git tag，读者可 checkout 到任意章节起点。
- **每章配套**：`examples/` 下可独立运行的小样例（新手单独练 Rust 特性用）。
- **读者挑战**：每章末尾放 2–3 个“不写答案”的动手题，答案在下一章开头自然揭晓。
- **避坑专栏**：每章一个“我踩过的坑”，比如 `claude-*` 模型名必须在 Ollama 兜底之前解析、seccomp 必须豁免 AF_UNIX、exec 路径默认不持久化 Extended 事件导致审计丢内容。
