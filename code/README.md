# code/ —— mini-codex 随书实现（Rust workspace）

> 状态：**part1（第 3–6 章）已落地**。随书代码按 Part 落位为真实源文件，
> 每个里程碑都可从对应 git tag 检出并 `cargo test --workspace` 通过（测试不联网）。

## 里程碑与 tag

| tag | 对应正文 | 该态有什么 |
| --- | --- | --- |
| `part0` | 第 1–2 章 | ch01 单包模型客户端（提交历史）→ ch02 改造成 workspace：`mcx-cli / mcx-core / mcx-protocol / mcx-tools / mcx-sandbox` 五 crate 骨架，依赖方向由 Cargo 强制 |
| `part1` | 第 3–6 章 | Op/Event 双通道；Session 三层循环（submission / turn / tool loop）；SSE 流式（任意切分、CRLF、[DONE]）；Item/Thread/Turn + JSONL Rollout（前向兼容）；Tool trait + Registry + read_file 桩 + FakeTool 测试 |

每个 tag 门禁：`cargo fmt --check`、`cargo clippy -D warnings`、`cargo test --workspace`
（当前 15 个用例：ScriptedLlm 双轮、tool loop 结果回填、加工具不改引擎循环、SSE 任意切分 /
CRLF / 逐字节、Rollout 往返与坏行/Unknown 宽容）。

## 约束（写代码前必读）

- 依赖统一在 `code/Cargo.toml` 的 `[workspace.dependencies]` 锁定（勘误中漂移过的
  ratatui / landlock / rusqlite 等引入时必须补 `cargo deny check`）。
- 沙箱相关尊重第 11 章「三个操作系统一个抽象」的边界，`#[cfg(target_os)]` 只在平台适配层。
- 测试不得要求真实 LLM key：`ScriptedLlm` + 假工具（见前言承诺）。
- 安全边界：任何「放行执行」的默认配置偏保守（默认 deny/批准），与第 10–12 章一致。

## 继续填实（part2+）

1. 从对应章正文把代码块按 crate 归属**人工归位**；书为「现场推演」会省略模块外壳，
   归位时补 `mod` / `use` / `main` 接线，与书的推进顺序一一对应。
2. 每完成一个 Part 的可编译验收态，运行 `scripts/checkpoint-tag.sh partN "…"` 打 tag。
3. 第 20 章回归集放 `tests/regression/`，由每周评测流水线消费。
