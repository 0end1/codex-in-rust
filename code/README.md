# code/ —— mini-codex 随书实现（Rust workspace）

> 状态：**工程骨架待填实**。本目录当前只定义了结构与约束，尚未包含任何 crate 源码。

## 目标形态

仓库 `main` 即第 22 章“自举”完成后的完整实现。正文每章介绍一个新 crate / 新模块时，
代码按章落位；正文引用的“第 N 章最终态”必须能从对应 git tag 检出并 `cargo test` 通过。
crate 边界以正文实际演进为准，最终形态预计约含（核对各章 workspace 块后确定）：

```
code/
├─ Cargo.toml            # [workspace]，依赖在根统一锁定
├─ rustfmt.toml
├─ crates/
│  ├─ mcx-cli/           # 第 1 章起步，演化为 CLI + 19 章 TUI 宿主
│  ├─ mcx-core/          # Op/Event 引擎、上下文、压缩、会话（核心演进最多）
│  ├─ mcx-protocol/      # SSE / Item / 协议演进
│  ├─ mcx-tools/         # 工具系统：shell、apply_patch、read 等
│  ├─ mcx-sandbox/       # 三 OS 沙箱抽象（bubblewrap/seatbelt/windows 令牌）
│  ├─ mcx-telemetry/     # 可观测性与评测
│  └─ …                  # 正文随章新增者（如 execpolicy 所在 crate、app-server、MCP 工具）
└─ tests/                # 第 20 章回归评测集镜像
```

## 填实步骤（发布前必做，一次性）

1. 从 `book/src/chapters/ch22.md` 及对应终态章节，把每章正文出现的代码块按上面 crate
   归属**人工归位**到真实源文件——书里为了“现场推演”会省略模块外壳，归位时需补
   `mod`/`use`/`main` 等接线，这一过程与书的推进顺序一一对应。
2. 每完成一个 Part 的可编译验收态，运行 `scripts/checkpoint-tag.sh partN "…"` 打 tag；
   这同时就在重建“逐章 git 历史”（回填脚本见 `scripts/`）。
3. 每个 tag 都要求：`cargo fmt --check`、`cargo clippy -D warnings`、
   `cargo test --workspace`（测试不联网：`ScriptedLlm` + 假工具，见前言承诺）。
4. 第 20 章回归集放 `tests/regression/`，由每周评测流水线消费。

## 约束（写代码前必读）

- 依赖统一在 `code/Cargo.toml` 的 `[workspace.dependencies]` 精确锁定版本
  （尤其 ratatui、landlock、rusqlite 等在勘误中出过漂移问题的依赖），新增依赖需跑
  `cargo deny check`。
- 沙箱相关代码尊重书第 11 章“三个操作系统一个抽象”的模块边界，`#[cfg(target_os)]`
  只允许在平台适配层出现。
- 测试不得要求真实 LLM key；真实模型只走 secrets 门控的冒烟。
- 安全边界：任何“放行执行”的默认配置都必须偏保守（默认 deny/批准），与第 10–12 章一致。
