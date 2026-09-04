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

## 附录 D　延伸阅读与源码索引

> 附录 A 告诉你学什么语法，附录 B 告诉你怎么写 spec，附录 C 告诉你为什么这么选型。
> 这个附录回答剩下两个问题：**去哪儿读源码**，以及**接下来读什么**。

---

### 配套资源：先把仓库拿到手

先分清两条源码线，后面不会混：**D.1 索引的是参照实现 codex-rs**（OpenAI，公开仓库）；
而你在读的这本书，它自己的配套实现是同一个仓库里的 `mini-codex`。本小节给的是后者。

- **在线阅读（mdBook）**：https://0end1.github.io/codex-in-rust/ —— 随每次 `main` 推送自动构建部署。
- **本书仓库（书稿 + 实现同仓）**：https://github.com/0end1/codex-in-rust
  —— `book/src/` 是书稿，`code/crates/` 是随书实现（产品名 `mini-codex`，含 `mcx-protocol`、`mcx-core`、
  `mcx-tools`、`mcx-sandbox`、`mcx-cli` 五个 crate）。
- **里程碑发布**：https://github.com/0end1/codex-in-rust/releases —— tag `v0.1.0` 起，每个里程碑带发布说明。

想按书里的章节节奏跟实现，**用 tag 而不是 HEAD**——前言与 README 承诺了六个 checkpoint
（README 的「tag ↔ 章节」表是权威版本，下面这张是它的镜像）：

```bash
git clone https://github.com/0end1/codex-in-rust.git
git tag                    # 列出已发布的 tag：part0、part1 已上线，part2…part5 随实现补打
git checkout part1         # 从第 7 章的起点接着实现（tags 随实现陆续发布）
```

| tag | 覆盖章节 | 想读「这一段的最终代码」就 checkout 到 |
|---|---|---|
| `part0` | 第 1–2 章 | 起步（workspace 决策） |
| `part1` | 第 3–6 章 | 能跑 tool loop |
| `part2` | 第 7–9 章 | 能真改文件、真执行命令 |
| `part3` | 第 10–12 章 | 安全篇完成 |
| `part4` | 第 13–16 章 | 会话可停、可续、可回滚 |
| `part5` | 第 17–22 章 | 完整版（自举） |

> tag 随代码落地逐步打出（`scripts/checkpoint-tag.sh`）。截至 `v0.1.0`，`part0`、`part1` 已发布，
> 其余随实现归位补打——以 Release 页 Tags 为准。
> 每章结束对应仓库里一个 commit——想读某一章当时的状态，`git checkout` 到那章的 commit 比看 HEAD 更准。

---

### D.1 codex-rs 源码索引：按章节对照

这是本附录最有用的一张表。拿着它去读 codex-rs，每一章都有对应的真实代码。

> **重要提醒**：codex-rs 是一个活跃开发的项目，目录结构会变。**如果下面的路径对不上，用第 2.2 节的几个命令自己找**：
> ```bash
> rg "submission_loop" codex-rs/        # 找主循环
> rg "enum EventMsg" codex-rs/          # 找事件定义
> cargo tree -p codex-core --depth 2    # 看依赖图
> ```
> **路径会漂移，但名字不会。** `submission_loop`、`EventMsg`、`SandboxPolicy`、`RolloutRecorder` 这些符号名是稳定的锚点。

| 章 | 主题 | codex-rs 中去看什么 | 关键符号（搜索用） |
|---|---|---|---|
| 1 | Harness 四动词 | 仓库根 `AGENTS.md`、`README.md` | — |
| 2 | 项目结构 | 根 `Cargo.toml` 的 `members`、`ls codex-rs/` | `workspace.members` |
| 3 | Op / Event | `protocol/src/protocol.rs`、`core/src/codex.rs` | `enum Op`、`enum EventMsg`、`submission_loop` |
| 4 | 流式 SSE | `core/src/client.rs`、`core/src/stream_events_utils` | `ResponseEvent`、`process_sse` |
| 5 | Item 与持久化格式 | `protocol/src/models.rs`、`core/src/rollout.rs` | `enum ResponseItem`、`RolloutRecorder` |
| 6 | 工具系统 | `core/src/tools/`、`core/src/openai_tools.rs` | `ToolRegistry`、`trait ToolHandler` |
| 7 | shell 与进程 | `core/src/exec.rs` | `ExecCommandBegin`、`process_exec_tool_call` |
| 8 | apply_patch | `apply-patch/` 整个 crate | `ApplyPatchFileChange`、`parse_patch` |
| 9 | 读文件 / 列目录 | `core/src/tools/` 下的文件类工具 | `read_file`、`list_dir` |
| 10 | 审批策略 | `protocol/src/protocol.rs`、`core/src/exec.rs` | `ReviewDecision`、`ApprovalPolicy`、`ask_for_approval` |
| 11 | 沙箱 | `linux-sandbox/`、`sandboxing/`、`windows-sandbox-rs/`、`process-hardening/` | `SandboxPolicy`、`spawn_command_under_linux_sandbox`、`seatbelt` |
| 12 | execpolicy / hooks | `execpolicy/`、`core/src/execpolicy.rs` | `ExecPolicy`、`parse_policy`、`hooks` |
| 13 | 配置分层 | `core/src/config.rs`、`core/src/config_types.rs` | `ConfigToml`、`load_config_as_toml_with_cli_overrides` |
| 14 | AGENTS.md | `core/src/project_doc.rs`、仓库根 `AGENTS.md` | `get_user_instructions`、`AGENTS.md` |
| 15 | 上下文压缩 | `core/src/compact.rs`、`core/src/context_manager/` | `should_compact`、`summarize` |
| 16 | 会话持久化 | `core/src/rollout.rs`、`core/src/state_db.rs` | `rollout-*.jsonl`、`resume`、`fork` |
| 17 | app-server | `app-server/`、`app-server-protocol/` | `thread/start`、`turn/start`（JSON-RPC 方法名） |
| 18 | MCP | `core/src/mcp_connection_manager.rs`、`core/src/mcp_tool_call.rs`、`mcp-server/` | `McpConnectionManager`、`tools/list` |
| 19 | TUI | `tui/` | `App`、`ChatWidget`、`history_cell` |
| 20 | 评测 | **无对应**（本书自建实践） | — |
| 21 | 子代理 | 该能力仍在演进，以主分支为准 | — |
| 22 | 自举 | 仓库根 `AGENTS.md`、CI 配置 | `.github/workflows/` |

#### 三个最值得先读的文件

如果你只有两小时，按这个顺序：

1. **`protocol/src/protocol.rs`** —— `Op` 和 `EventMsg` 两个 enum。看完你就理解了这个系统怎么动。
2. **`core/src/codex.rs`** —— `submission_loop`。三层循环的心跳在这里。
3. **`core/src/exec.rs`** —— 一条命令从"模型说要执行"到"沙箱里跑完再回传"的完整链路。**这一条链路走通，你就理解了 80% 的模块协作方式。**

---

### D.2 按主题索引：本书用到的第三方生态

| 主题 | crate / 工具 | 本书章 | 备注 |
|---|---|---|---|
| 异步运行时 | `tokio` | 3, 17, 21 | 官方站 tokio.rs，指南写得极好 |
| 异步工具箱 | `tokio-util` | 3, 7 | `CancellationToken` 在这里，不在 tokio 本体 |
| 流组合子 | `futures-util` | 4, 18 | `StreamExt` |
| HTTP 客户端 | `reqwest` | 1, 4 | 注意 feature 开关（见勘误 P0-1） |
| 序列化 | `serde` / `serde_json` | 5, 13 | tagged enum 是本书持久化格式的基石 |
| 错误处理 | `thiserror` / `anyhow` | 3, 5 | 库用 thiserror，二进制用 anyhow |
| 异步 trait | `async-trait` | 3, 6, 18 | 每次调用一次堆分配，可忽略 |
| 终端 UI | `ratatui` / `crossterm` | 19 | **版本间 API 变动大，注意锁版本** |
| 文件遍历 | `ignore` | 9 | 与 ripgrep 同源 |
| 嵌入式数据库 | `rusqlite` | 16 | 建议开 `bundled` 避免系统依赖 |
| 配置解析 | `toml` | 13 | — |
| 结构化日志 | `tracing` / `tracing-subscriber` | 20 | span 是核心概念 |
| 正则 | `regex` | 12 | — |
| 启动前钩子 | `ctor` | 11 | 注意与 `linkme` 冲突 |
| Linux 文件沙箱 | `landlock` | 11 | 内核 5.13+，ABI 版本与内核强相关 |
| Linux 沙箱（主流） | `bubblewrap`（bwrap，外部命令） | 11 | Codex v0.115 起的主路径 |
| seccomp 过滤 | `seccompiler` | 11 | 拦 `socket()` 的 `AF_INET`，豁免 `AF_UNIX` |
| 系统调用封装 | `nix` | 11, 7 | **0.26 → 0.29 有 breaking change** |

---

### D.3 平台安全机制的权威资料

第 11 章讲得快，这三块值得单独深入：

**Linux**
- **Landlock** —— 官网 landlock.io，内核文档 `Documentation/userspace-api/landlock.rst`。核心概念就三个：ruleset、rule、restrict-self。
- **seccomp-BPF** —— 内核文档 `Documentation/userspace-api/seccomp_filter.rst`。理解 BPF 程序的返回动作（`SECCOMP_RET_ALLOW` / `ERRNO` / `KILL`）是关键。
- **bubblewrap** —— 项目在 `containers/bubblewrap`。重点读它的 `--ro-bind` / `--bind` / `--argv0` 语义，以及路径特异性规则。
- **user namespace** —— `user_namespaces(7)` man page。理解为什么 bwrap 能免 root 工作。

**macOS**
- **Seatbelt / SBPL** —— 官方文档稀少，实践上最好读 `sandbox_init(3)` man page 加系统自带的 profile 样例（`/usr/share/sandbox/`）。
- 调试手段：`sandbox-exec -p <profile>` 加 `-D` 注入参数变量（本书 11 章用过）。

**Windows**
- **受限令牌** —— `CreateRestrictedToken`，MSDN 文档。
- **Job Object** —— 用于进程树管控（对应第 7 章的进程树清理）。

> **一句提醒**：这三套机制的**能力边界差别很大**。Windows 那套的网络阻断不是内核级的，忽略环境变量的工具可以绕过。做跨平台 agent 时，安全策略要按最弱的那环来设计对外承诺。

---

### D.4 Agent 架构与 AI 软件工程：延伸阅读

#### 一手来源（最值得读）

- **OpenAI Codex 仓库** —— 本书的参照实现。重点不是抄代码，是看它的 **AGENTS.md 和 CI 配置**：那才是"Harness 长什么样"的真实答案。
- **OpenAI 关于 Codex 与 harness 的工程博客** —— 书中引用的"5 个月 100 万行零手写"出自这里。
- **Model Context Protocol 规范** —— modelcontextprotocol.io。第 18 章的实现只是皮毛，协议本身值得通读一遍。
- **Terminal Bench** —— 书中 LangChain 那个 52.8% → 66.5% 的评测基准。**注意**：引用该数据时务必核对手来源与版本，不同 agent 配置差异巨大。

#### 主题延伸

| 想深入的方向 | 该读什么 |
|---|---|
| **上下文工程** | 本书第 9、14、15 章是入门；进阶去看各家 agent 的 prompt 组装代码（Codex 的 `core/src/prompting` 或同类） |
| **Agent 评测** | 本书第 20 章给了最小可用方案；更系统的做法去看 SWE-bench、Terminal Bench 的评测设计 |
| **多 Agent 协作** | 第 21 章偏保守（因为大多数场景不值得）；反面观点去看各家 multi-agent 产品的工程博客 |
| **形式化验证 agent** | 这是前沿方向，关注"可判定规约"和"机械验证"两条线的交叉研究 |

#### 关于"AI 软件工程"这个主题本身

本书的 22 条原理是作者自己的归纳，不是业界共识。**如果你想看不同的归纳角度**，建议关注两类来源：

1. **做 agent 产品的团队的工程博客**（他们踩的是真实的坑）
2. **用 agent 重写自家系统的团队的复盘**（他们讲的是真实的收益）

**避开**只有观点没有数据的内容，以及只有 demo 没有失败案例的内容。

---

### D.5 中英术语对照

读书和读源码时用得上。

| 中文 | 英文 | 首次出现 |
|---|---|---|
| 缰绳 / 承载层 | harness | 第 1 章 |
| 下行指令 | Op（operation） | 第 3 章 |
| 上行事件 | Event / EventMsg | 第 3 章 |
| 会话主循环 | submission loop | 第 3 章 |
| 轮次 | turn | 第 3 章 |
| 消息项 | Item / ResponseItem | 第 5 章 |
| 内部标签枚举 | internally tagged enum | 第 5 章 |
| 前向兼容 | forward compatibility | 第 5 章 |
| 工具注册表 | tool registry | 第 6 章 |
| 审批策略 | approval policy | 第 10 章 |
| 沙箱策略 | sandbox policy | 第 11 章 |
| 升级 / 越权重试 | escalation | 第 10 章 |
| 进程加固 | process hardening | 第 11 章 |
| 上下文压缩 | compaction | 第 15 章 |
| 提示缓存 | prompt cache | 第 15 章 |
| 回放日志 | rollout | 第 16 章 |
| 队列对协议 | queue-pair protocol | 第 17 章 |
| 子代理 | subagent / spawned agent | 第 21 章 |
| 自举 | bootstrapping | 第 22 章 |
| 熵管理 / 垃圾回收 | entropy management / GC | 第 22 章 |

---

### D.6 接下来做什么

读完 22 章、跑通 mini-codex 之后，按投入产出比排序：

**1. 把第 20 章的评测集真实跑起来（最高优先级）**

本书唯一一个"不跑就无法完成"的部分。20 个任务，机械验收，进 CI。**没有基线，你根本不知道自己改的 harness 是好是坏。**

**2. 挑一个你自己的项目，把第 13–14 章用上去**

给它写一份 AGENTS.md，配好分层配置，然后让 mini-codex 去干一件真实的活。**这一步会暴露出书里没讲的无数细节**——那正是你该学的东西。

**3. 补齐第 11 章在你平台上的沙箱实现**

书里给了三平台的抽象和 Linux 的完整路径。如果你在 macOS 或 Windows 上主力开发，**亲手实现那一档是理解沙箱最好的方式**。

**4. 读一遍 codex-rs 的 git 历史**

第 2.2 节提过这个技巧，这里再说一次：

```bash
git log --oneline --reverse -- codex-rs/core/src/exec.rs | head -40
```

**按时间正序看一个大文件的演进**，比读最终代码学到的多得多。你会看到他们为什么改、改错了什么、又怎么改回来的。

**5. 贡献回去**

如果你在勘误清单之外发现了新问题，或者写了更好的实现——本书仓库（https://github.com/0end1/codex-in-rust，勘误模板见仓库 `.github/ISSUE_TEMPLATE/`）欢迎 PR。**这本书的最后一章讲的是自举和熵管理，而开源协作本质上是同一件事的社会化版本。**

---

### D.7 一个诚实的建议

这本书会过时。

具体来说：**第 11 章的版本号、第 18 章的 MCP 细节、附录 D.2 的 crate 版本**——这些一两年内就会变。

不会过时的是那些原理：为什么要解耦界面和引擎、为什么沙箱要默认开启、为什么压缩切点要在 turn 边界、为什么架构违规得是编译错误。**这些是设计判断，不是技术细节。**

所以当你三年后回来看这本书，如果代码跑不起来了，直接翻到第 22 章末尾那张 22 条原理的表。**那张表是这本书真正想留下的东西。**

而那时如果你能对着它们说出"这条现在不成立了，因为……"——那这本书就成功了。

---

## 写作与配套建议

- **代码仓库**：随书公开 codex-in-rust 仓库，每个 Part 一个 git tag，读者可 checkout 到任意章节起点。
- **每章配套**：`examples/` 下可独立运行的小样例（新手单独练 Rust 特性用）。
- **读者挑战**：每章末尾放 2–3 个“不写答案”的动手题，答案在下一章开头自然揭晓。
- **避坑专栏**：每章一个“我踩过的坑”，比如 `claude-*` 模型名必须在 Ollama 兜底之前解析、seccomp 必须豁免 AF_UNIX、exec 路径默认不持久化 Extended 事件导致审计丢内容。
