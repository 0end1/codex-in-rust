# 用 Rust 造一个 Codex
## AI Agent 系统设计与 AI 软件工程

---

**一句话定位**：跟着这本书，你会用 Rust 从零造出一个能在自己仓库里干活的 CLI Agent（下称 **mini-codex**），并且看懂 OpenAI Codex 这类系统每一个设计决策背后的理由。

**目标读者**：会一点编程、想顺带学 Rust 的人。你不需要写过 Rust，但你要接受“编译器很啰嗦、但它替你挡住了半夜的告警”。

**三条承诺**：

1. 每章都有**可运行的代码增量**，读完 22 章你手上有真东西，不是一堆笔记。
2. 每章都回答**“为什么这么设计”**，而不是“这样能跑”。
3. 每章都提炼一条**AI 软件工程原理**——这些原理在模型换成更强的下一代时依然成立。

**全书 22 章，五大部分。**

---

# 前言 · 读者指南

## 定稿文风：工程师现场推演体

本书用同一种声音写完全书，这种声音叫**工程师现场推演体**：我们不是站在讲台上宣布结论，而是站在一个真实任务前，一步步说出“我看到了什么、我在担心什么、我这样取舍、它坏在哪里”。它由五条纪律组成：

1. **先现象，后原理。** 每章从“会出事的版本”或“先看一眼现状”开始，再谈边界与取舍。如果一个设计决策不能用一句“如果不这么做，它会在什么场景、以什么症状坏掉”来解释，那它还没被想透。
2. **判断给边界，不给伪精确。** 经验判断写成“在……场景下，我们倾向……”，而不是“一定是”“永远是”。书里凡是给数字，只有两种身份：可复现实验的结果，或标明了来源与置信度的转述——两者都必须交代口径。
3. **术语即契约。** 每个关键概念第一次出现时给一句话定义（见下面术语表）；此后全书回指这一定义，不再换词。同一实体在正文、代码、注释中不出现三种叫法。
4. **重复是负债。** “前面已经讲过”的论点不再展开，只给回指；章与章之间需要呼应时用“伏笔—兑现”的显式钩子，不用悄悄重述。
5. **少用绝对词。** “唯一、永远、必然、一定、绝不”只留给逻辑上确实成立的地方；经验倾向用“通常、多数场景、我们没遇到例外”表达。

**一条贯穿始终的声明（Codex 定位）**：

> 本书把 OpenAI Codex 当作**解剖案例**来读，而不是当作**最终答案**来抄。我们会引用它公开的结构与做法来问“为什么这样设计”，但不会主张它是 agent 系统的唯一形态。你合上书时带走的，是一套能用来拆解任何 agent 系统的问法，不是一个“照着造就能赢”的蓝图。

### 术语统一表（全文锚定定义）

以下术语在全文（正文、标题、目录、附录）统一；代码标识符按代码本身，但正文引用代码实体时用锚定名：

| 术语 | 锚定用法 | 不再使用的变体 |
|---|---|---|
| **Harness**（专名，首字母大写） | 模型之外的一切工程实体：循环、工具、沙箱、审批、评测 | 大小写混用、与“engine”互换 |
| **Model** / 模型 | 散文默认写“模型”；“Agent = Model + Harness”等公式/专名场景保留大写 | 无 |
| **Agent / agent** | 小写 `agent` 指运行中的代理实体；标题/公式/原理名用大写专名 | 正文行文中混用大写 Agent |
| **Engine / 引擎** | 散文一律写“引擎”，代码实体一律写 `Session`；说“引擎”时不出现第三个同义名 | “核心引擎”“会话引擎”混用 |
| **LlmClient** | 模型的抽象接口；散文称“模型客户端”，首次出现即回指 `LlmClient` | 与“模型”“引擎”混称 |
| **Runtime** | 只在“运行时扩展（第 18 章 MCP）”语境使用；不替代 Session | 把 Session 叫 Runtime |
| **队列对 / Op / Event** | `Op`（下行）与 `Event`（上行）固定为“队列对” | “两条 channel 上塞/抛”这类方向动词混用 |
| **Event / Item** | Event=运行时信号（流、不落盘）；Item=会话记录（账本、全落盘） | 把 delta 当 item、把 Event 当记录 |
| **Session** | 引擎的具名实现（`Session::new` 等） | 无 |
| **thread / turn / item** | 第 5 章三层模型的三级命名，全文小写回指 | 无 |
| **真相来源（single source of truth）** | 全文只允许一个锚定定义：**事件流是 agent 系统的真相来源**（原理 #3）；审计/仓库等作为“另一份只读证据”单独命名，不再占用“真相来源”四字 | 审计记录叫“真相来源”、仓库叫“唯一真相来源”（见原理 #14 的处理说明） |

（配套约定：原理 #14 标题“仓库是唯一真相来源”是口号式标题，正文 14.6 会给出它与原理 #3 的分工声明。）

## 这本书是怎么来的

2026 年初，OpenAI 披露了一个内部实验：一个小团队用 Codex agent，在五个月内产出大约一百万行代码，零行手写。工程师的工作变成了三件事——**设计环境、声明意图、提供结构化反馈**。

同一时期，LangChain 把他们的编码 agent 在 Terminal Bench 2.0 上的成绩从 52.8% 提到 66.5%，**一个模型参数都没改**，只换了 harness。

这两个数据指向同一个结论：**模型正在快速商品化，真正难复制的是模型之外的一切。**

但市面上的 agent 教程，绝大多数在教你调 API 和写 prompt 模板。那些内容三个月就过期。这本书想留下的，是模型换三代之后依然成立的东西：沙箱怎么做、策略怎么建模、上下文怎么预算、会话怎么回放、架构怎么被机械强制。

所以这本书有一条贯穿始终的暗线——**22 条 AI 软件工程原理**。它们是这本书真正的骨架，代码只是让它们变得可触摸。

---

## 你应该怎么用这本书

### 三种读法

**读法一：跟着造（推荐）**

从第一章开始，每章敲一遍代码。22 章走完，你手上有 mini-codex——一个能在自己仓库里干活的 CLI Agent。

**读法二：只学设计**

跳过代码，只读每章的「Design Rationale」和「AI 软件工程原理」。这两部分加起来约占全书 30%，读一遍大约 4 小时。适合架构师和技术决策者。

**读法三：当 Codex 源码导读**

第 2 章给了「四步读源码法」（类型定义 → 主循环 → 追踪一个工具 → 横向展开）。拿着它去读 codex-rs 仓库，遇到不懂的模块再回来翻对应章节。

### 关于 Rust

**你不需要写过 Rust。** 但你要接受两件事：

1. 编译器会很啰嗦。它拒绝你代码的时候，通常是对的。
2. 前几章会比较慢——我们在第 1、3 章会停下来解释 `?` 运算符和 `Box<dyn Error>` 这类东西。**过了第 6 章就不再为语法停留了**，节奏会明显加快。

**如果你已经会 Rust**：直接跳过每章末尾的「Rust 修炼小结」，正文里的语言点讲解也都做成了可跳过的引用块。

### 代码仓库与 git tag

随书代码在 mini-codex 仓库，每个 Part 结束打一个 tag：

```
part0    第 1-2 章结束
part1    第 3-6 章结束（能跑 tool loop）
part2    第 7-9 章结束（能真改文件、真执行命令）
part3    第 10-12 章结束（安全篇完成）
part4    第 13-16 章结束（会话可停可续可回滚）
part5    第 17-22 章结束（完整版）
```

想从第 N 章开始，直接 `git checkout partN`。

**但如果某一章你卡住了**，最省时间的做法不是换起点，而是用 `cargo test` 定位——本书绝大多数核心逻辑都配了**不依赖网络的测试**（`ScriptedLlm` + 假工具），几毫秒就能跑完。

---

## 每一章长什么样

固定六段式，节奏稳定：

| 段落 | 回答什么 |
|---|---|
| **本章任务** | 这一章给 mini-codex 加哪块砖？ |
| **正文** | 具体怎么做（含可运行代码） |
| **避坑专栏** | 哪个坑一定会踩？症状是什么？怎么解？ |
| **Design Rationale** | 为什么是这样，而不是那样？ |
| **AI 软件工程原理** | 这条设计背后，agent 时代软件工程的什么规律？ |
| **Rust 修炼小结 / 章末验收 / 读者挑战** | 学了什么语言特性？怎么判定学会了？ |

**关于「章末验收」**：全部是可判定的检查项，比如“路径穿越 `../../etc/passwd` 被拒绝”，不是“理解了本章内容”。如果某条你答不上来，那一章就没读完。

**关于「读者挑战」**：**本书不提供答案。** 它们是设计过的开放问题，答案会在后面几章自然揭晓——但只有你自己想过，揭晓时才有价值。

**关于「避坑专栏」**：24 个，每个都是真实会踩的坑。它们的共同结构是：错误做法 → **症状**（你看到的现象）→ 解法 → 通用形式。**症状那一栏最有用**——很多坑的表现是“程序静静卡住，CPU 占用 0%”这种，不知道症状根本没法搜。

---

## 全书的伏笔链

这本书是**精心设计过前后呼应**的。前面埋的钩子，后面会逐一兑现。读的时候留意这些：

| 埋下 | 兑现 |
|---|---|
| 第 1 章　`SafetyRule` 结构体（20 行） | 第 12 章　长成 execpolicy 规则引擎 |
| 第 1 章　为什么选 rustls 静态链接 | 第 22 章　单文件 release 二进制 |
| 第 2 章　crate 边界是防熵的第一道墙 | 第 22 章　GC 式清理 agent |
| 第 2 章　Cargo 守不住故意违规 | 第 22 章　装上 cargo-deny + 架构测试 |
| 第 3 章　`ScriptedLlm` 假模型 | 第 20 章　放大成 20 个任务的回归评测集 |
| 第 3 章　“第 19 章你会重写” | 第 19.9 节　专门回扣兑现 |
| 第 3 章　审批可能死锁（伏笔） | 第 10 章　给出确定解 |
| 第 1 章避坑 #1　base_url 末尾斜杠 | 第 13 章　配置层统一规范化 |
| 第 5 章　崩溃留下的残缺行 | 第 16 章　原子 rename 彻底解决 |
| 第 6 章　为什么用 `Box<dyn Tool>` 而非 enum | 第 18 章　MCP 运行时塞进未知工具 |

**这也是为什么不建议跳章读。** 前五章看似简单，但它们埋了十个钩子。

---

## 22 条 AI 软件工程原理

全书的核心产出。贴在墙上，它们比任何一段代码都耐用：

| # | 原理 | 章 |
|---|---|---|
| 1 | 模型是商品，harness 是护城河 | 1 |
| 2 | 架构违规应该是编译错误，而不是 code review 的意见 | 2 |
| 3 | 事件流是 agent 系统的真相来源 | 3 |
| 4 | 面向不可靠输入编程 | 4 |
| 5 | 先设计能被机器判定的产物，再设计产生它的代码 | 5 |
| 6 | 工具设计的第一原则是让模型容易生成对，不是让人容易实现 | 6 |
| 7 | 每个副作用都要有边界 | 7 |
| 8 | 把不确定性从模型的弱项转移到强项 | 8 |
| 9 | 上下文工程 = 在正确的时间把正确的信息放进窗口 | 9 |
| 10 | 把价值判断外置成配置，把安全边界内置成机制 | 10 |
| 11 | 沙箱是默认开启的，不是可选增强 | 11 |
| 12 | 规则要能被询问，而不只是被执行 | 12 |
| 13 | 配置即意图的持久化 | 13 |
| 14 | 仓库是唯一真相来源 | 14 |
| 15 | 上下文是预算，不是缓存 | 15 |
| 16 | 可回放是评测和调试的前提 | 16 |
| 17 | 引擎与表面分离，能力就能被复用 | 17 |
| 18 | 扩展性要落在协议上，不要落在代码里 | 18 |
| 19 | 人类在环的位置要精心设计 | 19 |
| 20 | 没有评测的 harness 优化都是玄学 | 20 |
| 21 | 并行的是上下文，不是理解 | 21 |
| 22 | 技术债像高息贷款，小额高频还优于攒着一次还 | 22 |

如果只能带走三条：**#1、#11、#20**。

- **#1** 决定了你把时间花在哪
- **#11** 决定了你敢不敢真用它
- **#20** 决定了你怎么知道自己在变好

---

## 一个提醒

**这本书的代码未经实际编译验证。** 撰写环境没有 Rust 工具链和网络。

代码经过逐章人工检查（API 用法、类型一致性、crate 归属），但**首次 `cargo build` 大概率会撞上依赖版本解析问题**。

配套有一份 [`勘误与待验证清单.md`](./勘误与待验证清单.md)，按 P0–P4 分级列出了所有已知和可疑的问题。**建议开工前先扫一眼 P0**，能省两小时。

---

## 开始

翻到第一章。先跑通那个 40 行的程序，然后——**按书里说的，故意把它搞坏**。

那个实验是全书最重要的一课，而它只需要你花三分钟。

---

# 目录

- 第 1 章　Agent = Model + Harness
    - 1.1 先让它说句话
    - 1.2 现在，故意把它搞坏
    - 1.3 把约束写成代码
    - 1.4 Harness 到底是什么
    - 1.5 第一个反直觉：约束让 agent 更强，不是更弱
    - 1.6 Design Rationale
- 第 2 章　解剖 Codex：如何读一个 90+ crate 的开源项目
    - 2.1 先看看仓库长什么样
    - 2.2 为什么 agent 项目特别需要 workspace
    - 2.3 建立 mini-codex 的 workspace
    - 2.4 依赖方向：让编译器替你守架构
    - 2.5 如何读一个 90+ crate 的开源项目
    - 2.6 Design Rationale

**第一部分　骨架：让一个循环跑起来（第 3–6 章）**

- 第 3 章　最小 Agent 循环：Op 与 Event
    - 3.1 先写一个走不远的循环
    - 3.2 Op 与 Event：两条 channel
    - 3.3 submission_loop：会话的心跳
    - 3.4 turn loop：一次对话的完整处理
    - 3.5 接上 CLI：输入任务与渲染任务分离
    - 3.6 用假模型测试整个循环
    - 3.7 Design Rationale
- 第 4 章　流式响应：SSE 解析这一关必须过
    - 4.1 为什么要流式
    - 4.2 SSE 协议的四个坑
    - 4.3 增量解析器
    - 4.4 解析 Responses API 的增量事件
    - 4.5 接进 Session
    - 4.6 测试：在任意字节处切分
    - 4.7 Design Rationale
- 第 5 章　类型建模：Item、Event 与协议演进
    - 5.1 Event 是流，Item 是账本
    - 5.2 Thread / Turn / Item 三层模型
    - 5.3 用 serde 建模 tagged enum
    - 5.4 落盘：JSONL
    - 5.5 协议演进规则
    - 5.6 前向兼容测试
    - 5.7 Design Rationale

- 第 6 章　工具系统：Trait 与 Registry
    - 6.1 先把那个空位找出来
    - 6.2 工具抽象：`trait Tool`
    - 6.3 Registry：为什么是 `Box<dyn Tool>`，不是 enum
    - 6.4 工具描述即提示词
    - 6.5 填上 while：tool loop 的完整形态
    - 6.6 用假工具跑完整 turn
    - 6.7 Design Rationale

**第二部分　让它真能干活：三个核心工具（第 7–9 章）**

- 第 7 章　shell：进程、超时与进程树
    - 7.1 先看一个会出事的版本
    - 7.2 设计：把边界全部显式化
    - 7.3 核心执行：超时三段式 + 进程树清理
    - 7.4 杀死进程树，而不是单进程
    - 7.5 环境变量白名单
    - 7.6 接进 Session：让 CancellationToken 生效
    - 7.7 测试：不依赖网络，但要真跑进程
    - 7.8 Design Rationale
- 第 8 章　apply_patch：为什么不用 unified diff
    - 8.1 先承认一个反直觉
    - 8.2 用“上下文匹配”代替“行号寻址”
    - 8.3 解析器：状态机，不是正则表达式
    - 8.4 校验：路径穿越、规模、语义
    - 8.5 原子落地：先写临时文件，再 rename
    - 8.6 失败时给模型足够信息自我纠正
    - 8.7 与整文件重写的取舍
    - 8.8 测试：穷举 patch 的合法与非法
    - 8.9 Design Rationale
- 第 9 章　读文件、列目录与看图片
    - 9.1 先问一个被忽略的问题：模型怎么“知道”项目长什么样
    - 9.2 list_dir：尊重 .gitignore
    - 9.3 read_file：范围控制与二进制检测
    - 9.4 view_image：多模态输入的边界
    - 9.5 工具组合的经济学
    - 9.6 接进 Registry：工具装配点
    - 9.7 测试：导航行为可判定
    - 9.8 Design Rationale

**第三部分　安全：把“敢让它做什么”做成配置（第 10–12 章）**

- 第 10 章　审批策略：自主性是个可调旋钮
    - 10.1 先写一个会把系统卡死的审批器
    - 10.2 把审批拆成纯策略和可等待令牌
    - 10.3 四档策略的真实含义
    - 10.4 审批与沙箱正交：两个旋钮，不是一条开关
    - 10.5 升级流程：失败不是终点，而是一次受控越界
    - 10.6 兑现第 3 章伏笔：三条解法与本书选择
    - 10.7 把决策变成可测试、可审计的产物
    - 10.8 Design Rationale
- 第 11 章　沙箱：三个操作系统，一个抽象
    - 11.1 破直觉：它不是 Docker，也不是一条 `chroot` 调用
    - 11.2 题眼：限制为什么必须在 `exec` 之前、且不可撤销
    - 11.3 Linux：bubblewrap 为主，Landlock 退居 legacy
    - 11.4 macOS：动态生成 SBPL，而不是写死 profile
    - 11.5 Windows：受限令牌、ACL 与环境塑形
    - 11.6 进程加固：在 `main()` 之前消灭继承攻击面
    - 11.7 统一 runner 与四条诚实边界
    - 11.8 测试：真实进程，明确断言
    - 11.9 Design Rationale
- 第 12 章　execpolicy 与 hooks：声明式规则与生命周期拦截
    - 12.1 先写一个事后才发现问题的策略
    - 12.2 `execpolicy check`：规则必须能被离线询问
    - 12.3 规则文件、匹配与样例自测
    - 12.4 把规则接入工具循环
    - 12.5 PreToolUse / PostToolUse：可信生命周期拦截
    - 12.6 信任模型：项目级配置必须“受信任”才加载
    - 12.7 把整套链路连起来
    - 12.8 测试：离线评估与超时隔离
    - 12.9 Design Rationale

**第四部分　记忆：让长会话不崩（第 13–16 章）**

- 第 13 章　配置分层与 profiles
    - 13.1 一个会咬人的“简单配置”
    - 13.2 四层来源与合并顺序
    - 13.3 安全红线：项目配置只能装饰，不能夺权
    - 13.4 兑现第 1 章避坑 #1：base_url 的斜杠必须归一化
    - 13.5 profiles：把一套意图绑成一个名字
    - 13.6 可观测的合并结果
    - 13.7 完整测试：恶意项目 + profile 切换
    - 13.8 Design Rationale
- 第 14 章　AGENTS.md：把项目知识写进仓库
    - 14.1 一份“越大越好”的知识文件为什么注定失败
    - 14.2 三层结构与发现顺序
    - 14.3 32KB 是安全闸门，不是写作配额
    - 14.4 合并不是拼接：覆盖、追加与冲突
    - 14.5 路径安全：符号链接、越界与 canonicalize
    - 14.6 怎么写好 AGENTS.md：写不变量，不写通用常识
    - 14.7 测试：范围决定内容
    - 14.8 Design Rationale
- 第 15 章　上下文预算与压缩
    - 15.1 先数一数上下文都花在哪
    - 15.2 估算器：宁可稳定，不可精确
    - 15.3 Turn 是语义最小完整单元：反直觉的切点规则
    - 15.4 两种压缩与“append-only 交接”
    - 15.5 保留策略：Item 的挑选粒度
    - 15.6 完整的 Compactor：不破坏可回放性
    - 15.7 快照测试：压缩不能悄悄丢约束
    - 15.8 prompt cache 可观测性
    - 15.9 Design Rationale
- 第 16 章　会话持久化：resume、fork 与 rollback
    - 16.1 兑现第 5 章的残缺行伏笔
    - 16.2 为什么 JSONL 主存、SQLite 只做索引
    - 16.3 并发：锁文件 + WAL，而不是“SELECT FOR UPDATE”
    - 16.4 持久化分级：Limited 与 Extended
    - 16.5 从事件流重建：resume
    - 16.6 fork：分叉不是复制文件
    - 16.7 rollback：回到某轮，但真相不消失
    - 16.8 完整测试：崩溃、恢复、分叉、回滚
    - 16.9 导出完整工具调用时间线
    - 16.10 Design Rationale

**第五部分　长成真系统（第 17–22 章）**

- 第 17 章　队列对协议与 app-server
    - 17.1 一个能跑的 CLI，为什么还不能直接嵌入 IDE
    - 17.2 `Op`/`Event` 队列对的完整形态
    - 17.3 把引擎包成 JSON-RPC 2.0 服务
    - 17.4 stdio、WebSocket 与三级资源模型
    - 17.5 SDK 为什么“免费”：一个五行的 Python 驱动脚本
    - 17.6 取消、背压与优雅关闭：协议最容易烂的地方
    - 17.7 用假模型和 ScriptedLlm 测整条 RPC
    - 17.8 Design Rationale
- 第 18 章　MCP：让工具在运行时长出来
    - 18.1 一个会过时的工具枚举
    - 18.2 三种传输，同一个协议
    - 18.3 生命周期：initialize、能力协商、关闭
    - 18.4 工具发现：`tools/list` 与动态注册
    - 18.5 资源、资源模板与按需上下文
    - 18.6 工具太多时：索引、检索与惰性完整化
    - 18.7 信任与审批：外部 server 提供的工具凭什么执行
    - 18.8 健壮性：重连、列表变更与 server 崩溃隔离
    - 18.9 测试：假 transport + ScriptedLlm + 工具计数
    - 18.10 Design Rationale
- 第 19 章　TUI：把引擎接到人脸上
    - 19.1 回到第 3 章那张“四个问题”的表
    - 19.2 双层架构：渲染循环与事件循环分离
    - 19.3 `AppState`：UI 唯一可信状态
    - 19.4 事件循环：扇入、节流与审批桥
    - 19.5 组件：聊天流、diff、审批、状态栏
    - 19.6 流式性能：每个 token 都重绘全屏会炸
    - 19.7 键盘、取消与优雅退出
    - 19.8 测试：ScriptedLlm + 状态归约，不需要终端
    - 19.9 “现在你看到了”：第 3 章承诺的完整回扣
    - 19.10 Design Rationale

- 第 20 章　可观测性与回归评测
    - 20.1 没有基线的优化，只是换了一种失败
    - 20.2 用 `tracing` 把一次任务变成可查询的对象
    - 20.3 回归评测集：把 ScriptedLlm 放大 20 倍
    - 20.4 从事件流判定成功：原理 #5 的自测题
    - 20.5 评测必须进 CI：改一版 harness，分数不能退
    - 20.6 评测集怎么挑：覆盖失败模式，而非追求数量
    - 20.7 Design Rationale
- 第 21 章　子代理与多 Agent
    - 21.1 先承认一个反直觉事实
    - 21.2 一个会失控的“自由子代理”
    - 21.3 类型化派生：Spawn、工具策略与结果
    - 21.4 结果怎么回传：摘要不是可选项
    - 21.5 何时并行：JoinSet、取消传播与写冲突
    - 21.6 发现漂移：子代理失败不是终点，而是证据
    - 21.7 Design Rationale
- 第 22 章　自举：用 mini-codex 开发 mini-codex
    - 22.1 自举的第一原则：先锁住环境，再交钥匙
    - 22.2 依赖方向不是文档，而是第一道防火墙
    - 22.3 结构测试：把风格、可见性与状态规则写进 CI
    - 22.4 熵管理：GC 式的小额高频还债
    - 22.5 真实闭环：让它给自己加一个功能
    - 22.6 发布：把 rustls 的选择兑现为单文件二进制
    - 22.7 Design Rationale

---

# 第 1 章　Agent = Model + Harness

**本章任务**：跑通一个不到 40 行的程序，让模型说第一句话。然后——这是重点——**故意把它搞坏**，亲眼看着输出质量崩塌，再把约束写成代码补回去。

这一章不教你造 agent，它教你**建立判断力**。在你写第一行 agent 代码之前，你得分清楚：一个 agent 好不好用，到底是谁决定的。

---

## 1.1 先让它说句话

```bash
cargo new mini-codex
cd mini-codex
```

`Cargo.toml`：

```toml
[package]
name = "mini-codex"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

`src/main.rs`：

```rust
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENAI_API_KEY")?;

    let body = json!({
        "model": "gpt-4o-mini",
        "instructions": std::env::args().nth(1).unwrap_or_default(),
        "input": "帮我写一个函数，把两个日期之间相差的天数算出来。"
    });

    let response: Value = reqwest::Client::new()
        .post("https://api.openai.com/v1/responses")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    match extract_text(&response) {
        Some(text) => println!("{text}"),
        None => println!("没解析出文本：\n{}", serde_json::to_string_pretty(&response)?),
    }
    Ok(())
}

/// 从 Responses API 的返回里捞出助手的回复文本。
fn extract_text(response: &Value) -> Option<String> {
    let output = response.get("output")?.as_array()?;
    for item in output {
        if item.get("type")?.as_str()? != "message" {
            continue;
        }
        for part in item.get("content")?.as_array()? {
            if part.get("type")?.as_str()? == "output_text" {
                return part.get("text")?.as_str().map(str::to_string);
            }
        }
    }
    None
}
```

跑起来，注意第二个参数是我们留的“系统指令”入口：

```bash
export OPENAI_API_KEY="sk-..."
cargo run -- "你是一个 Rust 专家。用 chrono crate，只输出代码不要解释。"
```

你会拿到一段能用的代码。**这就是全世界最简单的“agent”——它甚至不能算 agent，它只是一个会说话的函数。**

> **Rust 速查 · 这段代码里的四个新手关卡**
>
> - **`#[tokio::main]`**：Rust 的 `main` 本来不能是异步的。这个属性宏帮你包一层，偷偷启动 tokio 运行时。async 代码不会自己跑，需要执行器推动——这就是那个开关。
> - **`?`（用在 `Result` 上）**：有值就拿出来继续，出错就立刻返回给调用者。等价于 `match` 的 Ok/Err 分支，但压成了一个字符。
> - **`Box<dyn Error>`**：“一个盒子，装着任何实现了 Error trait 的东西”。盒子是必需的——Rust 要在编译期知道类型大小，而“任何错误”大小不定，放堆上只留指针即可。这是 **trait object**。
> - **`?`（用在 `Option` 上）**：有值拿出来，是 `None` 就立刻返回 `None`。`extract_text` 里那条问号链靠它，**保证 JSON 结构和预期不符时也不会 panic**。
>
> 想深挖？见附录 A。正文里我们不再为语法停留。

> **为什么 `reqwest` 要关掉 default-features？**
> 默认的 OpenSSL 依赖目标机器的系统库。换成纯 Rust 的 `rustls`，你的二进制在别人机器上不会因为缺库而编译失败。**发布给别人用的命令行工具，静态链接是刚需**——这个思路在第 22 章会回来。

---

## 1.2 现在，故意把它搞坏

真正的学习从这一步开始。三个实验，**每次只改传给 `--` 后面那句话**，观察输出怎么变。

### 实验一：不给任何约束

```bash
cargo run -- ""
```

你会拿到一段代码，配一大段解释、几个“另外你也可以考虑”，可能用了 `time` 而不是 `chrono`，或者干脆手写了个日期计算。

**问题不在模型不会写。问题在你没说清楚要什么。**

### 实验二：加角色和输出格式

```diff
- cargo run -- ""
+ cargo run -- "你是一个 Rust 专家。只输出代码，不要解释。用 chrono crate。"
```

输出立刻收敛：纯代码，没废话，用了 chrono。

### 实验三：再加约束、边界和退路

```diff
- cargo run -- "你是一个 Rust 专家。只输出代码，不要解释。用 chrono crate。"
+ cargo run -- "你是一个 Rust 专家。用 chrono crate，只输出代码不要解释。
+ 要求：函数签名 fn days_between(a: &str, b: &str) -> Result<i64, String>；
+       输入是 YYYY-MM-DD 格式字符串；
+       解析失败返回 Err，不要 panic；
+       不要引入 chrono 之外的依赖；
+       如果无法保证等价改造，就只输出『无法安全改进』。"
```

这一次的输出几乎是可以直接贴进项目的。

### 三个实验说明了什么

**模型从头到尾没变，变的只有你塞进去的那段话。**

同一个 `gpt-4o-mini`，在实验一里给你一堆没法用的东西，在实验三里给你生产级代码。**差距不在模型，在你给它的约束、上下文和验证标准。**

这就是本书的第一块基石：

> **Agent = Model + Harness**

模型是马，有力气但不知道往哪走。Harness 是缰绳、马鞍和赛道——**模型之外的一切**。

---

## 1.3 把约束写成代码

实验三效果最好，但有个致命问题：**那句话散落在你的 shell 历史里。**

明天你忘了它，后天同事问“你当时怎么调的”，大后天 agent 又撒谎说“改好了”——你没有任何东西能机械地发现它撒谎。

所以现在做一件小事：**把“约束”从一段文本，变成一个 Rust 类型。**

```rust
/// 一条必须施加给模型的硬约束。
struct SafetyRule {
    /// 对人类和模型都可读的要求描述
    requirement: &'static str,
    /// 无法满足时，模型必须输出这句话，而不是硬着头皮做
    refusal: &'static str,
}

/// 把一组规则折叠进基础指令，产出最终的系统提示。
fn apply_rules(base: &str, rules: &[SafetyRule]) -> String {
    let mut prompt = String::from(base);
    for rule in rules {
        prompt.push_str(&format!(
            "\n- {}。若无法满足，只输出「{}」，不要编造。",
            rule.requirement, rule.refusal
        ));
    }
    prompt
}

fn main() {
    let rules = vec![
        SafetyRule {
            requirement: "用 chrono crate，不引入其他依赖",
            refusal: "无法安全改进",
        },
        SafetyRule {
            requirement: "解析失败返回 Err，不得 panic",
            refusal: "无法安全改进",
        },
        SafetyRule {
            requirement: "只输出代码，不输出解释",
            refusal: "无法安全改进",
        },
    ];

    println!("{}", apply_rules("你是一个 Rust 专家。", &rules));
}
```

只有二十几行，但它体现了全书最重要的一次观念转变：

| | 约束是一段文本 | 约束是一个类型 |
|---|---|---|
| 能不能复用 | 靠复制粘贴 | 调一次函数 |
| 能不能测试 | 不能 | 能断言拼出来的提示词 |
| 能不能审计 | 不知道用过哪些 | 规则列表就是审计记录 |
| 能不能强制 | 全靠模型自觉 | 可以接上后续检查 |

**第 12 章的 execpolicy，就是这个 `SafetyRule` 的工业化版本**——规则外置成声明文件、支持离线查询“这条命令会被怎么处理”。而第 20 章的回归评测，就是把“模型有没有违反规则”变成 CI 里的一条红线。

你现在写的这二十几行，是那条路的起点。

> **注意 `refusal` 这个字段。** 它让模型**有权利说做不到**。一个永远回答“好的我改了”的 agent，比一个会说“这个我没法安全做”的 agent 危险得多——前者把风险藏起来了，后者把风险交回给你。

---

## 1.4 Harness 到底是什么

一个可操作的定义。Harness 是四件事，四个动词：

| 动词 | 做什么 | 具体手段 |
|---|---|---|
| **Constrain** 约束 | 限制它能做什么 | 沙箱、权限、依赖方向、类型系统 |
| **Inform** 告知 | 让它知道该做什么 | 系统提示、AGENTS.md、工具描述、目录结构 |
| **Verify** 验证 | 检查它做对了没有 | 编译器、测试、lint、架构测试、CI |
| **Correct** 纠正 | 错了让它自己改 | 错误回传、重试、审批升级、清理任务 |

**这四件事，没有一件是关于模型的。** 你换更强的模型，harness 一样要做；你换更弱的模型，harness 做得好照样能救回来。

### 两个数据点

**一、OpenAI 的内部实验。** 2026 年 2 月披露：一个小团队用 Codex agent，在 **5 个月里产出大约 100 万行代码，零行手写**——包括应用逻辑、文档、CI 配置、可观测性埋点和工具链。工程师干三件事：**设计环境、声明意图、提供结构化反馈。**

**二、LangChain 的 harness 改造。** 他们的编码 agent 在 Terminal Bench 2.0 上从 52.8% 提到 66.5%，排名从三十开外进到前五，**一个模型参数都没改**。改动只有四项：完成前自检清单、启动时映射目录结构、循环检测、推理强度分档。

> **关于数据**：这组数字来自 LangChain 团队公开分享的评测结果，经二手来源转述。Terminal Bench 版本、agent 配置和评测环境都会影响结果，**请把它当作趋势信号，而不是可复现的精确基准**。本书引用它只为说明一件事：harness 的改动量级，可以和换模型相当。

**如果你的时间只够优化一件事，优化 harness。**

---

## 1.5 第一个反直觉：约束让 agent 更强，不是更弱

新手最容易犯的错，是觉得“给 agent 越多自由，它越聪明”。**正好相反。**

为什么？因为模型每生成一个 token 都在做选择。你的约束越少，它要走的路越多，走错的概率越大。**当 harness 把边界画清楚，它就不需要浪费 token 在死路上探索。**

想想你自己的体验：老板说“把这事搞定”，你会焦虑；老板说“用这个方案，周五前，预算不超过 X，做不完提前告诉我”，你反而能立刻开工。**模型也一样。**

你自己验证：

```bash
# 弱约束
cargo run -- "帮我改进这段代码"

# 强约束
cargo run -- "你是 Rust 性能专家。改进下面这段代码。
要求：不改变公开 API；不引入新依赖；用迭代器代替索引循环；
      只输出改动后的函数；如果无法保证等价，就说『无法安全改进』。"
```

第二种不仅质量更高，而且**它敢于说“做不到”**。

---

## AI 软件工程原理 #1

> **模型是商品，harness 是护城河。**

两个方向的含义，都值得认真对待。

**乐观的一面**：模型能力正在快速商品化。今天只有最强模型能做的事，明年中档模型就能做。这意味着**你在 harness 上的投入不会贬值**——它跟具体模型无关，换模型时照样有效，模型越强它发挥得越好。

**清醒的一面**：如果你的产品只是“包了一层 API”，替代品明天就会出现。真正难复制的是那套约束、验证、纠正的机制——它藏着你对这个具体领域的全部理解。

### 这条原理，就是传统软件工程与 AI 软件工程的分水岭

传统工程里，质量靠什么兜底？**Code review。** 人写代码，人评审，人负责。这套机制运转了几十年，前提是“写代码的人和被评审的人遵守同一套社会契约”。

到了 agent 时代，这个前提塌了：

| | 传统软件工程 | AI 软件工程 |
|---|---|---|
| 谁写代码 | 人 | 模型 |
| 质量兜底 | Code review | **机械验证** |
| 规范载体 | 文档、口头约定、老人带新人 | **代码结构、编译器、CI** |
| 违规后果 | 被 reviewer 打回 | **编译不过 / 测试挂掉** |

**核心转变**：你没法给一个模型做 code review——它不会脸红，不会记住，下次还犯。所以质量保障必须从“人的判断”前移到“系统的强制”。

你在 1.3 节写的那个 `SafetyRule`，就是这个转变的最小样本：**把“应该注意”变成“系统会检查”**。

### 落地：三问自查

从今天起，每当你想说“模型不够聪明”，先依次问：

1. **Inform 够不够？** 它是不是根本没看到关键信息？（第 9 章、第 14 章）
2. **Constrain 够不够？** 我是不是给了它太多自由？（第 10–12 章）
3. **Verify 有没有？** 它做错了，有没有机制立刻发现并让它自己改？（第 20 章）

三个都问完还是不行，那才是模型能力的问题。以经验看，**十个“模型不行”的抱怨里，八个是 harness 的问题**；剩下两个里，还有一个半能靠换 prompt 策略解决。

---

## 1.6 Design Rationale

**Q：为什么第一个例子要“故意搞坏”，而不是直接开始搭 agent？**

因为**判断力必须先于能力**。

如果你不知道“好”长什么样，那你在第 3 章搭出第一个循环时，会分不清“能跑”和“好用”的区别。后 21 章的每一个设计决策，本质上都在回答同一个问题：**怎么让 agent 的输出从实验一的水平，稳定提升到实验三的水平。**

先亲手制造一次质量崩塌，你才会真正相信那些约束是必要的——而不是觉得作者在啰嗦。

**Q：为什么用 Responses API 而不是 Chat Completions？**

因为 Codex 用的就是它。Responses API 原生支持推理项、工具调用和服务端压缩，这些在第 15 章会变成关键能力。**选 API 不是选哪个顺手，是选哪个能支撑你后面要做的东西。**

**Q：为什么 1.3 节要把约束写成类型，而不是给一段“最佳实践提示词”？**

因为提示词模板会过期，类型不会。更关键的是：**一旦约束是数据，你就能对它做检查、做测试、做审计。** 这是“提示词工程”和“AI 软件工程”的分界——前者产出一段话，后者产出一个系统。

---

## 避坑专栏 #1：base_url 末尾的斜杠

每个写 agent 的人都会踩一次：

```rust
// 错误：拼出来是 https://api.openai.com/v1//responses
let base = "https://api.openai.com/v1/";
let url = format!("{base}/responses");

// 正确
let base = "https://api.openai.com/v1";
let url = format!("{base}/responses");
```

某些服务端容忍双斜杠，某些不容忍；更烦的是**容忍的和拒绝的返回错误码还不一样**，你会怀疑人生半小时。

正确做法是在配置层统一规范化（第 13 章会做）：

```rust
let base = base.trim_end_matches('/').to_string();
```

> **通用形式**：凡是拼接 URL、路径、命令字符串的地方，边界字符（斜杠、空格、引号）必须在一处统一处理，而不是散落在 20 个调用点。

---

## Rust 修炼小结

| 本章遇到 | 是什么 | 后面在哪用到 |
|---|---|---|
| `Cargo.toml` + `features` | 依赖与可选功能开关 | 全书，编译时间是真实成本 |
| `#[tokio::main]` / `async` | 异步运行时入口 | 整个 agent 循环都是异步的 |
| `Result` / `Option` + `?` | 显式错误处理 | agent 每一步都可能失败 |
| `Box<dyn Error>` | trait object | 第 6 章会换成更专业的错误设计 |
| `struct` + `Vec<T>` + `format!` | 用类型表达规则 | 1.3 节，也是第 12 章的雏形 |

如果你从 Python/JS 转过来，最大落差在这里：Rust 逼你处理每一个“可能失败”。写的时候烦，但 agent 系统里到处是网络、文件、子进程、模型输出这四种不可信输入——**编译器替你挡住的 bug，在动态语言里会变成半夜的告警。**

---

## 章末验收

- [ ] `cargo run -- "你的指令"` 能拿到回复，且指令不同时输出明显不同
- [ ] 你能说出实验一和实验三之间**到底是什么变了**，并且答案里不包含“模型”这个词
- [ ] 你跑通了 1.3 节的 `SafetyRule`，并且能自己再加一条规则（比如“函数名必须用 snake_case”）
- [ ] 你能举出自己项目里属于 harness 的三样东西（提示词不算——想想 lint、CI、类型定义、目录结构）

---

## 读者挑战

本书不会直接给答案，但你在后面会自己想通。

1. 1.3 节里我们要求模型“无法保证等价就说做不到”。**如果它撒谎了呢？** 你该用什么机制发现？（提示：想想 Verify 这个动词）
2. 你现在有了一个“能跟模型说话”的程序。如果要让它**连续说十轮**，你会遇到什么新问题？
3. Harness 的四个动词里，你认为哪一个最容易被忽略？为什么？

---

## 下一章预告：天花板在模型之外的那套工程

上一章我们得到一个判断：**Harness 决定 Agent 的天花板。**

但 Harness 不是一个提示词文件，它是一个**软件系统**。它有模块边界、有依赖方向、有状态流转、有失败处理——它需要被正经地工程化。

下一章我们拆开 Codex，看看 OpenAI 是怎么用 Rust 把这套东西组织起来的。你会看到那四个动词（Constrain / Inform / Verify / Correct）在源码里各自落在哪个 crate 上，也会亲手搭起 mini-codex 的 workspace 骨架。

---

---


# 第 2 章　解剖 Codex：如何读一个 90+ crate 的开源项目

**本章任务**：看懂 codex-rs 的结构，搭起 mini-codex 的 workspace，并学会一套读大型 Rust 项目的方法。

上一章的四个动词——Constrain、Inform、Verify、Correct——听起来还挺抽象。这一章我们把它落到具体的 crate 上：

| 动词 | codex-rs 里由谁承担 |
|---|---|
| **Constrain** | `sandboxing`、`linux-sandbox`、`windows-sandbox-rs`、`execpolicy` |
| **Inform** | `core`（上下文装配）、AGENTS.md 加载 |
| **Verify** | 编译器、`execpolicy`、CI |
| **Correct** | `core` 的错误回传与重试、审批升级流程 |

**看清楚这张表，你就理解了为什么这本书要用 22 章来讲“模型之外的一切”。** 真正的工作量在这里。

---

## 2.1 先看看仓库长什么样

在读代码之前，先有个空间感。codex-rs 摘取核心部分后长这样：

```
codex-rs/
├── core/                     # 核心引擎：agent 循环、会话、工具分发
├── protocol/                 # 共享类型：Event、Op、配置、权限
│
├── cli/                      # 表面：命令行入口（分发到下面几个）
├── tui/                      # 表面：全屏终端 UI（Ratatui）
├── exec/                     # 表面：非交互式执行（CI/脚本）
├── app-server/               # 表面：JSON-RPC 服务器（给 IDE/桌面端用）
├── app-server-protocol/      # 表面：线上的协议类型
├── mcp-server/               # 表面：把 Codex 自己暴露成 MCP 服务器
│
├── sandboxing/               # 基础设施：统一沙箱接口与策略抽象
├── linux-sandbox/            # 基础设施：bwrap + Landlock + seccomp
├── macos-sandbox/            # 基础设施：Seatbelt 配置 + 应用沙箱
├── windows-sandbox-rs/       # 基础设施：受限令牌 + ACL
├── process-hardening/        # 基础设施：main() 之前的进程加固
├── execpolicy/               # 基础设施：执行策略的加载与求值
├── tools/                    # 基础设施：工具实现
│
├── Cargo.toml                # workspace 根
└── ...（其余 70+ 个 crate）
```

三个观察：

1. **`core` 只有一个，表面有五个。** 同一套引擎，被 TUI、CLI、CI、IDE、MCP 五种方式驱动。这不是过度设计——**这正是“引擎与表面分离”的收益**，第 17 章我们会亲手做到。
2. **基础设施比核心还多。** 沙箱拆了四个 crate（统一接口 + 三个平台实现），这就是第 11 章要讲的东西。
3. **`protocol` 在最上层被列出，但它在依赖图的最底下。** 它是所有 crate 的公共词汇表。

---

## 2.2 为什么 agent 项目特别需要 workspace

Cargo workspace 让多个 crate 共享一个 `Cargo.lock` 和 `target/`。但对 agent 项目，价值远不止省磁盘。

**agent 系统的复杂度来自“很多半独立的子系统”**：模型客户端、工具、沙箱、配置、持久化、UI、RPC……逻辑上互不隶属，物理上要在同一个二进制里。

全堆进一个 crate 用 `mod` 隔开会怎样？

- **模块之间没有强制边界。** 任何 `mod` 都能 `use` 任何兄弟模块的东西，只要加个 `pub`。评审拦得住人，拦不住 agent——**而你的 agent 不看评审意见**。
- **编译单元太大。** 改一行 TUI 代码，整个项目重编。
- **无法表达“这个子系统可以单独发布”**。Codex 的 `codex-linux-sandbox` 是独立可执行文件，它必须是个 crate。

用 workspace，每个 crate 是**有名字、有边界、可单独测试**的单元。

### Codex 的工程约定

Codex 团队在自己仓库的 AGENTS.md 里写了一条明确约定：**新增功能优先开新 crate，而不是往 `core` 里塞。**

一个 90+ crate 的项目能保持可维护，靠的就是这条纪律。理由我们在 2.6 节展开。

---

## 2.3 建立 mini-codex 的 workspace

删掉第 1 章建的 `src/`，改成这样：

```
mini-codex/
├── Cargo.toml          # workspace 根，只管编排
└── crates/
    ├── mcx-cli/        # 命令行入口
    ├── mcx-core/       # 核心引擎：agent 循环
    ├── mcx-protocol/   # 共享类型：Event、Op、配置
    ├── mcx-tools/      # 工具实现
    └── mcx-sandbox/    # 沙箱策略与平台实现
```

根目录 `Cargo.toml`：

```toml
[workspace]
resolver = "2"
members = [
    "crates/mcx-cli",
    "crates/mcx-core",
    "crates/mcx-protocol",
    "crates/mcx-tools",
    "crates/mcx-sandbox",
]

# 所有 crate 共享同一份依赖版本，避免同一个库编两遍
[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
async-trait = "0.1"
```

`resolver = "2"` 是新项目标配，让 feature 合并行为更可预测。**写上就对了。**

`[workspace.dependencies]` 是特别值得养成的习惯：子 crate 声明依赖时写 `tokio = { workspace = true }`，版本只在根里写一次。否则你会在 `Cargo.lock` 里发现两个版本的 `tokio`，编译时间翻倍，还会遇到“类型不匹配”这种诡异错误。

生成骨架：

```bash
cargo new --lib crates/mcx-core
cargo new --lib crates/mcx-protocol
cargo new --lib crates/mcx-tools
cargo new --lib crates/mcx-sandbox
cargo new --bin crates/mcx-cli
```

```bash
cargo build
```

五秒之内编完，什么都没发生。**这正是我们要的——一个能编译的骨架，比一堆跑不起来的代码有用。**

---

## 2.4 依赖方向：让编译器替你守架构

定死一条规则，本书全程遵守：

```
mcx-cli      →  依赖所有 crate
mcx-core     →  依赖 protocol、tools、sandbox
mcx-tools    →  依赖 protocol、sandbox
mcx-sandbox  →  依赖 protocol
mcx-protocol →  不依赖任何 workspace 内的 crate
```

画成图：

```
        mcx-cli
           │
           ▼
       mcx-core
      ╱    │    ╲
     ▼     ▼     ▼
mcx-tools  │  mcx-sandbox
     ╲     ▼     ╱
       mcx-protocol
           │
         (空)
```

**箭头只能向下。** `mcx-protocol` 是地基，它不认识任何人。

### 亲手验证

在 `crates/mcx-core/src/lib.rs` 里加一行：

```rust
use mcx_cli::something;  // 故意的反向依赖
```

编译：

```bash
cargo build -p mcx-core
```

```
error[E0432]: unresolved import `mcx_cli`
 --> crates/mcx-core/src/lib.rs:1:5
  |
1 | use mcx_cli::something;
  |     ^^^^^^^ no external crate `mcx_cli`
```

**记住这个错误长什么样。** 以后每次看到它，说明你的架构被守住了。

---

## 避坑专栏 #2：Cargo 守得住什么，守不住什么

很多资料会说“crate 边界让反向依赖编译失败”。**这句话只对了一半**，而那一半的误差会让你在第 22 章踩坑。

**Cargo 能强制的**：crate A 想 `use` crate B，必须在 A 的 `Cargo.toml` 里显式声明。没声明就 `use`，编译直接报错——就是上面那个 `E0432`。

这挡住了绝大多数**意外耦合**：你想用，就得先走一步“去 Cargo.toml 加一行”，而这一步在 code review 里极其显眼。

**Cargo 不能强制的**：如果有人（或某个 agent）真的去 `Cargo.toml` 加了一行反向依赖，**Cargo 不会阻止**。Rust 的类型系统管不到 crate 之间的依赖图。

所以真正的机械强制要靠 CI。手段有三档：

| 手段 | 挡什么 | 成本 |
|---|---|---|
| Cargo 显式声明（现在就有） | 挡**意外** | 零 |
| `cargo-deny` 配置禁止依赖 | 挡**故意** | 一个配置文件 |
| 自定义架构测试（解析 `Cargo.toml` 断言） | 完全可定制 | 一个测试文件 |

**好消息是第一档已经解决了 90% 的真实问题。** 剩下两档我们到第 22 章自举时装——那时候你才有真实的违规需要防。

> **为什么现在就要知道这个？** 因为如果你误以为“编译器已经保证了架构”，你就不会去写架构测试。等你发现 agent 偷偷加了一行反向依赖时，依赖图已经乱了。

---

## 2.5 如何读一个 90+ crate 的开源项目

这套方法不只适用于 codex-rs。

### 错误做法：从 `main.rs` 一路读下去

这是新手本能，也是最快的劝退方式。第 10 分钟撞进配置解析，第 20 分钟迷失在某个 trait 实现，第 40 分钟彻底放弃。**大型项目的调用深度 20 层起步，深度优先遍历必死。**

### 正确做法：四步，每步都有明确产出

**第 1 步：读类型定义，不读逻辑**

先找 `protocol` 或 `types` 这类 crate。在 codex-rs 里是 `codex-rs/protocol/src/protocol.rs`。

**为什么先读类型？** 因为类型是项目的骨架。看懂这两个 enum，整个系统的数据流就清楚了：

```rust
// 概念示意，非源码原文
pub enum Op {           // 下行：客户端发给引擎的指令
    UserInput { .. },
    Interrupt,
    Shutdown,
}

pub enum EventMsg {     // 上行：引擎报给界面的事件
    AgentMessageDelta(String),
    ExecCommandBegin { .. },
    ExecCommandEnd { .. },
    PatchApplyEnd { .. },
    TaskComplete,
    Error(..),
}
```

**看懂这两个 enum，你就知道这个系统怎么动了。** 执行逻辑只是“收到 Op 之后干什么”的填充。

> **产出**：一张表，列出核心类型及其变体。

**第 2 步：找主循环**

类型告诉你数据长什么样，主循环告诉你数据怎么流。在 codex-rs 里搜 `submission_loop`，你会找到三层循环：

```
submission_loop      会话生命期常驻，只等 Op
    └── turn loop    一次用户输入的完整处理
            └── tool loop   工具调用往返，直到模型说完成
```

**读到主循环，你就拿到了项目的心跳。**

> **产出**：一张流程图，标出三层循环的进入/退出条件。

**第 3 步：追踪一个工具的完整链路**

挑最简单的工具（比如 `read_file`），从“模型请求调用”追到“结果回传”：

```
模型返回 tool_call
  → core 解析参数
  → ToolRegistry 查表
  → Tool::call()
  → 沙箱包装 + 执行
  → 结果封装成 Event
  → 追加进历史
  → 回到模型
```

**这一条链路走通，你就理解了 80% 的模块协作方式。**

> **产出**：一张时序图，标出每一步在哪个 crate。

**第 4 步：横向铺开**

有了骨架和一条完整链路，现在才按兴趣展开：想懂沙箱读 `sandboxing/`，想懂 UI 读 `tui/`。**每次只攻一个子系统，且始终带着“它挂在第 2 步的哪个节点上”这个问题去读。**

### 三个必备工具

```bash
# 1. 看 crate 依赖图，一眼看清谁依赖谁
cargo tree -p codex-core --depth 2

# 2. 全局搜符号，比 IDE 跳转更快
rg "fn spawn_command_under" codex-rs/

# 3. 生成文档，本地起一个可跳转的 API 浏览器
cargo doc --open -p codex-core
```

**第三个被严重低估。** `cargo doc --open` 会编译所有依赖的文档，在浏览器里打开一个静态站点：左边是 crate 树，点进任意类型能看到所有方法签名，**而且每个类型名都能点进去跳转**。

读陌生 crate 时，它的效率比翻源码高一个量级——因为你不需要在 20 个文件之间跳来跳去确认“这个 `ExecRequest` 到底有哪些字段”，一眼就看到了。

### 一个加速技巧：看 git 历史

遇到看不懂的设计，试这个：

```bash
git log --oneline -- codex-rs/core/src/exec.rs | head -20
```

按时间倒序看这个文件的提交记录。**最近一次大重构的 commit message，往往直接写明了“为什么改成这样”。** 代码只告诉你结果，commit 告诉你理由。

---

## 2.6 Design Rationale

**Q：五个 crate 是不是太少/太多了？**

对教学项目，五个刚好：**一个入口、一个核心、一个共享类型、两个能力子系统**。

判断标准不是数量，是**每个 crate 能不能用一句话说清职责**。如果你说不清 `mcx-utils` 是干什么的，它就该被拆掉或合并。**`utils`、`common`、`helpers` 这类名字，几乎总是“设计还没想清楚”的信号。**

**Q：为什么 `mcx-protocol` 在最底层且不依赖任何东西？**

因为它是**所有 crate 都要用的公共词汇表**。如果它也依赖别人，依赖图可能出现环——而 Rust 不允许 crate 循环依赖，你会立刻编译失败。

更重要的是语义：事件类型和配置结构是系统里最稳定、最不该变的东西。让它们待在地基上，等于宣告“这里的改动会影响所有人”。

**Q：为什么 Codex 坚持“新增功能开新 crate 而不是塞进 core”？**

三个理由，按重要性排：

1. **编译时间。** 改 `tui` 不该重编 `core`。90 个 crate 的项目全量重编可能要十分钟——这会直接杀死迭代速度。
2. **边界即文档。** 一个叫 `execpolicy` 的 crate，比 `core/src/exec_policy.rs` 这个路径更能声明“这是一块独立的关注点”。
3. **对 agent 友好。** 这是 agent 时代的新理由：**给 agent 的任务越容易划出边界，它越不容易改错地方。** 你在 AGENTS.md 里写“文件改动限制在 `crates/mcx-tools/` 内”，比写“改 core 里的工具相关代码”可执行得多。

---

## 避坑专栏 #3：workspace 里的循环依赖

图方便让 `mcx-core` 依赖 `mcx-tools`，又让 `mcx-tools` 依赖 `mcx-core`：

```toml
# crates/mcx-core/Cargo.toml
[dependencies]
mcx-tools = { path = "../mcx-tools" }

# crates/mcx-tools/Cargo.toml
[dependencies]
mcx-core = { path = "../mcx-core" }
```

Cargo 直接拒绝：

```
error: cyclic package dependency: package `mcx-core v0.1.0` depends on itself.
```

**Rust 在编译期就禁止 crate 循环依赖。**

但真正难的是**逻辑上的循环**：crate 层面没环，可 A 的行为依赖 B 的回调，B 的行为又依赖 A 的状态。这种环编译器看不见，只能靠设计拆掉。

标准解法是**把共享部分下沉到第三层**——这正是 `mcx-protocol` 存在的理由：

```
错误：  core ⟷ tools
正确：  core → protocol ← tools
```

> **通用法则**：两个模块互相需要对方时，不要想办法让它们通信，要找出它们共同需要的东西，把它抽出来放到下面一层。

---

## AI 软件工程原理 #2

> **架构违规应该是编译错误，而不是 code review 的意见。**

这条原理在 agent 时代的分量，比在传统团队里重十倍。

**为什么？** 人类团队里，架构靠三样东西维持：code review、口头约定、老人带新人。**这三样对 agent 全部失效**：

- Agent 不参加设计评审会
- Agent 不看 Slack 里的架构讨论
- Agent 从上下文里学到的规范，下次会话就忘了

**Agent 唯一读得到、且无法绕过的东西，是代码本身的结构和编译器的报错。**

所以：

| | 传统团队 | Agent 团队 |
|---|---|---|
| 架构写在 | 文档里 | crate 边界里 |
| 违规在 | review 时被指出 | 编译时被拒绝 |
| 表达方式 | “别这么做” | “这么做编译不过” |

**Rust 在这里有天然优势。** crate 系统、显式的 `Cargo.toml` 依赖、`pub(crate)` 可见性、类型系统——这些本来为内存安全和工程性设计的东西，在 agent 时代意外地成了最强的架构执行工具。

这不是说 Rust 是唯一选择。它的意思是：**选技术栈时，“能不能机械强制执行架构”应该是一个显式的评估项**，而不只看性能和生态。

### 这条原理还有第二层：crate 边界是防熵的第一道墙

Agent 有一个隐蔽但持续的破坏方式：**它会复制仓库里已经存在的模式，哪怕那个模式是坏的。**

OpenAI 在内部实践中就撞上了这个——agent 复制了仓库里不均匀、不理想的写法，日积月累造成漂移。团队最初每周五花 20% 的时间人工清理“AI 垃圾”，很快就发现这不可持续。

**crate 边界在这里的作用是双重的**：

1. **限制复制范围。** 一个被限定在 `mcx-tools/` 内工作的 agent，就算学到了坏模式，危害也局限在这一个 crate。
2. **让偏离可检测。** 边界清晰时，“这个 crate 不该 import 那个”是可以被机械断言的；边界糊成一团时，你连违规都定义不出来。

熵管理是个大话题——OpenAI 最终的解法是把“golden principles”写进仓库，再跑定期清理 agent 扫偏离、开小重构 PR，**像 GC 一样小额高频还债**。我们会在第 22 章完整实现这套机制。

现在你只需要记住：**你在 2.4 节画的那些箭头，不只是架构图，它们是第一道防火墙。**

---

## Rust 修炼小结

| 概念 | 本章用法 | 后面在哪用到 |
|---|---|---|
| Cargo workspace | 拆分五个 crate | 全书 |
| `[workspace.dependencies]` | 统一依赖版本 | 全书，防重复编译 |
| `cargo new --lib` / `--bin` | 库与二进制 | lib 给内部用，bin 是入口 |
| `pub` / `pub(crate)` | 控制可见性 | `pub(crate)` 是 crate 内的“半公开” |
| crate 依赖图 | 架构约束 | 第 22 章的架构测试 |

补充一条可见性规则：

```rust
pub fn foo() {}        // 任何 crate 都能用
pub(crate) fn bar() {} // 只有本 crate 能用
fn baz() {}            // 只有本模块能用
```

**新人常见毛病是把所有东西都写成 `pub`。** 默认从最严格开始，需要了再放开——这样你的 `pub` 列表本身就成了一份“这个 crate 对外承诺什么”的文档。

---

## 章末验收

- [ ] `cargo build` 通过，workspace 内有 5 个 crate
- [ ] 在 `mcx-core` 里 `use mcx_cli::...` 会编译失败，且你认得 `E0432` 这个错误
- [ ] 你能画出 mini-codex 的依赖方向图，并说出 `mcx-protocol` 为什么在最底下
- [ ] 你能在 codex-rs 仓库里指出：哪个是核心引擎、哪些是表面、哪些是基础设施（各举两个）
- [ ] 你跑过 `cargo doc --open` 并成功跳转过一次

---

## 读者挑战

1. **五个 crate 里，你觉得哪个最先会成为瓶颈？** 提示：想想哪个被依赖次数最多，改它一次要重编多少东西。
2. **`mcx-protocol` 里该放什么、不该放什么？** 如果你把“工具的具体实现”放进去，会发生什么？
3. 用 2.5 节的四步法，去 codex-rs 里找出 `Op` 和 `EventMsg` 的完整定义，数一数各有几个变体。**这个数字会告诉你很多关于系统复杂度的信息。**

---

## 下一章预告：整个引擎，只有两条通道

用 2.5 节的四步法读完 `protocol` 之后，你会发现一件有趣的事：

**Codex 的整个引擎，只有两个通道。**

- **`Op` 下行**——客户端（TUI、CLI、CI、IDE）把用户指令塞进这条 channel
- **`Event` 上行**——引擎把流式文本、工具调用、错误、完成信号，全部从这条 channel 抛回来

就这两个 enum，撑起了五种表面、三层循环、以及后面所有的能力。

第 3 章我们要亲手实现它。你也会看到：**为什么“直接写个 `loop` 调模型”这种最直觉的写法，会在第 19 章做 TUI 时变成一个必须推倒重来的错误。**


# 第一部分　骨架：让一个循环跑起来（第 3–6 章）

> 前三章我们不追求“能干活”，只追求一件事：**把引擎和界面彻底分开。**
> 这个决定在第 19 章做 TUI 时会救你一命。

---


# 第 3 章　最小 Agent 循环：Op 与 Event

**本章任务**：跑通第一个 turn loop，并引入全书最重要的抽象——`Op` 与 `Event` 两条 channel。

这一章结束时，你的程序还是不能干活（没有工具），但它会有一个正确的骨架。**骨架对了，后面加什么都不用推倒重来。**

---

## 3.1 先写一个走不远的循环

按直觉，一个聊天 agent 长这样。我们把第 1 章的代码套个 `loop`：

```rust
// 反例：能跑，但走不远
async fn chat_forever() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        if line.trim() == "/quit" { break; }

        // ← 从这里到打印完成，整个程序是冻结的
        let reply = call_llm(&line).await?;

        println!("{reply}");
    }
    Ok(())
}
```

它确实能跑，而且代码最短。但有四个问题，每一个都足以在后面逼你重写：

| 问题 | 后果 | 什么时候爆炸 |
|---|---|---|
| **UI 冻结** | 模型思考的几秒里，用户敲键盘没反应、Ctrl+C 可能不生效 | 第 19 章做 TUI |
| **无法中途打断** | 用户发现 agent 跑偏了，只能杀进程 | 第 7 章跑长命令 |
| **无法被复用** | IDE、CI、MCP 想驱动它？没有入口 | 第 17 章做 app-server |
| **无法测试** | 逻辑依赖 stdin/stdout，测试里没法注入 | 第 20 章做回归评测 |

**这四个问题的共同根源**：引擎和界面缠在一起了。模型在慢慢吐字，用户在敲键盘，工具在跑 `npm install`——**三件事的速度完全不匹配，却挤在同一个执行流里。**

正确的做法是：把它们拆成两个独立任务，中间用两条 channel 通信。

---

## 3.2 Op 与 Event：两条 channel

这是 Codex 架构里最核心的一对抽象，也是全书出现频率最高的两个名字。

**一句话定义**：

- **`Op`（下行）**：客户端发给引擎的指令。用户敲了一句话、按了 Ctrl+C、点了关闭窗口。
- **`Event`（上行）**：引擎报给界面的事件。模型吐了几个字、开始执行命令、命令失败了、这轮结束了。

```
   TUI / CLI / CI / IDE / MCP          ← 五种"表面"
        │              ▲
    Op  │              │  Event
        ▼              │
   ┌──────────────────────┐
   │      Session         │  ← 核心引擎，不认识任何界面
   └──────────────────────┘
```

**关键性质**：引擎不知道界面是什么。它只管从 `Op` 收指令、往 `Event` 抛结果。

这一个性质，直接决定了后面这些能力能不能做出来：

- **TUI** —— 订阅 Event 流渲染，同时继续响应键盘（第 19 章）
- **exec / CI** —— 把 Event 流转成 JSONL 打进 stdout，不需要 UI（第 17 章）
- **app-server** —— 把 Op/Event 暴露成 JSON-RPC，IDE 和 SDK 就能驱动它（第 17 章）
- **回归评测** —— 录下 Event 流，回放对比两次运行的工具调用序列（第 20 章）

### 类型定义

`crates/mcx-protocol/src/lib.rs`：

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

/// 下行：客户端 → 引擎
#[derive(Debug)]
pub enum Op {
    /// 用户提交了一段输入
    UserInput { text: String },
    /// 用户想打断当前轮次
    Interrupt,
    /// 关闭会话
    Shutdown,
}

/// 上行：引擎 → 界面
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// 一轮开始
    TurnBegin { turn: usize },
    /// 模型吐出的一段文本（流式，会来很多次）
    AgentMessageDelta(String),
    /// 一轮结束，附完整文本
    TurnComplete { turn: usize, text: String },
    /// 出错了，但会话还能继续
    Error(String),
    /// 引擎已退出
    Shutdown,
}
```

注意 `Event` 是 `Clone + PartialEq` 而 `Op` 不是。为什么？

**因为 Event 需要被测试和回放，Op 不需要。** 这个 derive 的差异本身就是设计意图的表达——第 20 章做评测时，你会想 `assert_eq!(events, expected)`。

---

## 3.3 submission_loop：会话的心跳

Codex 有三层循环，从外到内是：

```
submission_loop      会话生命期常驻，只等 Op，直到 Op::Shutdown
    └── turn loop    处理一次用户输入：组装 prompt → 调模型 → 发事件
            └── tool loop   工具调用往返，直到模型说"我做完了"
```

**本章只实现前两层。** tool loop 要等第 6 章有了工具系统才能填——现在它是空的，但位置已经留好了。

`crates/mcx-core/src/session.rs`：

```rust
use mcx_protocol::{Event, Message, Op, Role};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub struct Session<C: LlmClient> {
    client: C,
    history: Vec<Message>,
    op_rx: mpsc::Receiver<Op>,
    event_tx: mpsc::Sender<Event>,
    cancel: CancellationToken,
    turn: usize,
}

impl<C: LlmClient> Session<C> {
    pub fn new(client: C, op_rx: mpsc::Receiver<Op>, event_tx: mpsc::Sender<Event>) -> Self {
        Self {
            client,
            history: Vec::new(),
            op_rx,
            event_tx,
            cancel: CancellationToken::new(),
            turn: 0,
        }
    }

    /// 第一层：会话主循环。整个会话生命期只跑这一个函数。
    pub async fn submission_loop(&mut self) {
        loop {
            // 收到 None 说明所有 Op sender 都被 drop 了 —— 界面全关了，该退了
            let Some(op) = self.op_rx.recv().await else { break };

            match op {
                Op::UserInput { text } => self.run_turn(text).await,
                Op::Interrupt => self.cancel.cancel(),
                Op::Shutdown => {
                    self.emit(Event::Shutdown).await;
                    break;
                }
            }
        }
    }

    /// 上报一个事件。发送失败不算错误 —— 界面关了而已。
    async fn emit(&self, ev: Event) {
        let _ = self.event_tx.send(ev).await;
    }
}
```

**`CancellationToken` 是干什么的？**

先补一个依赖：**它来自 `tokio-util`，不是 `tokio` 本体。**`tokio` 的 `full` feature 并不包含它，必须单独声明：

```toml
# 根 Cargo.toml 的 [workspace.dependencies]
tokio-util = "0.7"

# mcx-core/Cargo.toml
tokio-util = { workspace = true }
```

它是 tokio-util 提供的“可取消信号”：一个地方 `cancel()`，所有等待它的任务同时被唤醒。现在 `Op::Interrupt` 只是调了它、还没人监听——**这没问题，我们提前把位置占好**，第 7 章做超时取消时它会派上用场。

---

## 3.4 turn loop：一次对话的完整处理

```rust
impl<C: LlmClient> Session<C> {
    /// 第二层：处理一次用户输入。
    async fn run_turn(&mut self, text: String) {
        self.turn += 1;
        let turn = self.turn;
        self.emit(Event::TurnBegin { turn }).await;

        self.history.push(Message { role: Role::User, content: text });

        // 第二层循环目前只有一次迭代。
        // 第 6 章引入工具后，这里会变成 while 循环：
        //   模型要调工具 → 执行 → 结果追加进 history → 再问模型 → 直到不再要工具
        match self.client.complete(&self.history).await {
            Ok(reply) => {
                self.history.push(Message { role: Role::Assistant, content: reply.clone() });
                self.emit(Event::TurnComplete { turn, text: reply }).await;
            }
            Err(e) => {
                // 关键：出错不终止会话。用户应该还能继续说话。
                self.emit(Event::Error(e.to_string())).await;
                self.emit(Event::TurnComplete { turn, text: String::new() }).await;
            }
        }
    }
}
```

三个设计点，都是后面会反复用到的：

**① 出错不终止会话。** 一次 API 调用失败是常态（限流、网络抖动、模型拒绝），如果失败就让整个 agent 退出，用户会疯。**错误是一个 Event，不是一个 panic。**

**② 历史只追加。** `history.push` 从不修改已有内容。这在第 15 章会变成关键——**append-only 才能保住 prompt cache**，而 prompt cache 是长会话的成本命脉。

**③ 第三层循环的位置已经留好了。** 看那行注释。第 6 章你只会往这里加一个 `while`，不会动外层结构。**这就是骨架对了的收益。**

### 模型客户端抽象

引擎不该知道模型是 OpenAI 还是本地 Ollama。定义个 trait：

```rust
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("网络错误: {0}")]
    Network(#[from] reqwest::Error),
    #[error("响应格式无法解析: {0}")]
    Malformed(String),
    #[error("请求被取消（收到取消信号，见第 17 章）")]
    Cancelled,
}

#[async_trait]
pub trait LlmClient: Send + Sync {
    async fn complete(&self, messages: &[Message]) -> Result<String, LlmError>;
}
```

`#[async_trait]` 是必需的：Rust 原生不支持 trait 里的 `async fn`（返回类型是个匿名 Future，编译器没法确定大小）。这个宏帮你把它改写成返回 `Pin<Box<dyn Future>>`。**代价是每次调用有一次堆分配，对 agent 场景可以忽略。**

实现真实客户端（沿用第 1 章的 Responses API）：

```rust
pub struct OpenAiClient {
    http: reqwest::Client,
    api_key: String,
    model: String,
}

#[async_trait]
impl LlmClient for OpenAiClient {
    async fn complete(&self, messages: &[Message]) -> Result<String, LlmError> {
        let input: Vec<_> = messages
            .iter()
            .map(|m| serde_json::json!({ "role": m.role, "content": m.content }))
            .collect();

        let resp: serde_json::Value = self
            .http
            .post("https://api.openai.com/v1/responses")
            .bearer_auth(&self.api_key)
            .json(&serde_json::json!({ "model": self.model, "input": input }))
            .send()
            .await?
            .json()
            .await?;

        extract_text(&resp).ok_or_else(|| {
            LlmError::Malformed(resp.to_string())
        })
    }
}
```

（`extract_text` 就是第 1 章那个函数，搬进 `mcx-core` 即可。）

---

## 3.5 接上 CLI：输入任务与渲染任务分离

现在把三个角色拆成三个独立的 tokio 任务：

```rust
// crates/mcx-cli/src/main.rs
use mcx_core::{OpenAiClient, Session};
use mcx_protocol::{Event, Op};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (op_tx, op_rx) = mpsc::channel(16);
    let (ev_tx, mut ev_rx) = mpsc::channel(128);

    // 任务 A：引擎。不认识 stdin，也不认识 stdout。
    let mut session = Session::new(OpenAiClient::from_env()?, op_rx, ev_tx);
    tokio::spawn(async move { session.submission_loop().await });

    // 任务 B：渲染。只消费 Event，别的什么都不管。
    tokio::spawn(async move {
        while let Some(ev) = ev_rx.recv().await {
            match ev {
                Event::AgentMessageDelta(s) => {
                    print!("{s}");
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                }
                Event::TurnComplete { .. } => println!(),
                Event::Error(e) => eprintln!("\n[错误] {e}"),
                Event::Shutdown => break,
                _ => {}
            }
        }
    });

    // 任务 C：输入。读取一行，发一个 Op。
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    while let Some(line) = lines.next_line().await? {
        if line.trim() == "/quit" {
            op_tx.send(Op::Shutdown).await?;
            break;
        }
        op_tx.send(Op::UserInput { text: line }).await?;
    }
    Ok(())
}
```

跑起来：

```bash
cargo run
> 你好
你好！有什么可以帮你的？
> /quit
```

**现在 UI 永远不死。** 模型思考的时候，你还能敲下一句话——它会被排队，等这轮结束后立即处理。

---

## 3.6 用假模型测试整个循环

这是本章最值钱的一段。因为我们用了 trait + channel，**整个引擎可以在没有网络、没有 API key 的情况下被完整测试**。

```rust
pub struct ScriptedLlm {
    replies: std::sync::Mutex<std::collections::VecDeque<String>>,
}

impl ScriptedLlm {
    pub fn new(replies: Vec<String>) -> Self {
        Self { replies: std::sync::Mutex::new(replies.into_iter().collect()) }
    }
}

#[async_trait]
impl LlmClient for ScriptedLlm {
    async fn complete(&self, _messages: &[Message]) -> Result<String, LlmError> {
        Ok(self.replies.lock().unwrap().pop_front().unwrap_or_default())
    }
}
```

测试：

```rust
#[tokio::test]
async fn two_turns_are_processed_in_order() {
    let (op_tx, op_rx) = mpsc::channel(8);
    let (ev_tx, mut ev_rx) = mpsc::channel(64);
    let mut session = Session::new(
        ScriptedLlm::new(vec!["A".into(), "B".into()]),
        op_rx,
        ev_tx,
    );

    let handle = tokio::spawn(async move { session.submission_loop().await });

    op_tx.send(Op::UserInput { text: "1".into() }).await.unwrap();
    op_tx.send(Op::UserInput { text: "2".into() }).await.unwrap();
    op_tx.send(Op::Shutdown).await.unwrap();

    // 收集事件流，直到引擎说它退出了
    let mut events = Vec::new();
    while let Some(ev) = ev_rx.recv().await {
        let done = matches!(ev, Event::Shutdown);
        events.push(ev);
        if done { break; }
    }
    handle.await.unwrap();

    let replies: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            Event::TurnComplete { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(replies, vec!["A".to_string(), "B".to_string()]);
}
```

**这个测试在 30 毫秒内跑完，不花钱，永远稳定。** 而如果你的引擎和 stdin/stdout 缠在一起，这个测试根本写不出来。

> **记住这个模式**：`ScriptedLlm` + 收集 Event 流 + 断言。第 20 章的回归评测集，就是把这个模式放大 20 倍。

---

## 3.7 Design Rationale

**Q：为什么一上来就要两条 channel，而不是直接写 `loop`？**

因为**界面与引擎的速度不匹配是所有 agent 复杂性的总根源**。

模型每秒吐几十个字，用户每秒敲几个键，工具可能要跑三分钟。三者的时间尺度差了两个数量级。把它们塞进一个执行流，你就必须在每一处都手工处理“等等，现在该响应谁”——**这个复杂度是指数增长的**。

两条 channel 把这个问题的解法固定成了一种模式：**引擎只管产生事件，界面按自己的节奏消费。** 后面每加一个新表面（TUI、CI、IDE），都只是新增一个 Event 的消费者，引擎一行不改。

**Q：为什么 `Op::Interrupt` 现在什么也不做还要留着？**

因为**取消是最难后加的功能**。如果等到第 7 章才想起来“哦要能打断”，你会发现取消信号必须穿过模型调用、工具执行、子进程树三处——那时候加，等于重写。

先把类型留在这里，成本是零；等需要时再加，成本是整个引擎。

**Q：为什么用 bounded channel（`mpsc::channel(16)`）而不是 unbounded？**

`unbounded` 会让你失去背压：如果渲染任务卡住，事件会在内存里无限堆积，最终 OOM。**bounded channel 会让生产者在队列满时等待，这是特性不是缺陷**——它让“下游跟不上了”这个事实变得可见。

代价是可能死锁，见下面的避坑专栏。

---

## 避坑专栏 #4：send() 失败不等于出错

新手会这样写：

```rust
// 危险
self.event_tx.send(ev).await.unwrap();
```

然后在用户关闭窗口、或者测试里 drop 掉 receiver 时，**引擎 panic**。

`send()` 返回 `Err` 只说明一件事：**接收端没了**。这可能是正常的（用户退出），也可能是 bug（你提前 drop 了 receiver）。但无论哪种，**panic 都是错的反应**——引擎应该安静地停止上报。

所以我们在 `emit` 里写的是：

```rust
async fn emit(&self, ev: Event) {
    let _ = self.event_tx.send(ev).await;   // 忽略失败，这是有意的
}
```

**`let _ =` 在这里不是偷懒，是明确的设计意图。** 我建议在旁边写注释说明，否则下一个人会“好心”地帮你加上 `unwrap()`。

### 进阶：这个坑在第 10 章会变成真死锁

现在不会死锁，因为渲染任务只消费、不生产。

但第 10 章引入**审批**后，流程变成：

```
引擎 → 审批专用通道（unbounded，载荷 ApprovalRequest）→ 渲染任务
渲染任务 → 等用户点"允许" → Op::Approval → 引擎
```

如果 channel 是有界的、且容量设小了：

```
引擎阻塞在 send(ApprovalRequest)   ← 队列满
渲染任务阻塞在等待用户输入          ← 而 UI 需要引擎先吐完才能渲染提示
```

**真死锁。** 解法有三条，按推荐度排：

1. **审批请求用单独的 channel**，不和普通事件共用队列
2. **审批通道用 unbounded**（交互类消息量极小，无 OOM 风险）
3. **给 send 加超时**，超时就认为用户无响应

我们在第 10 章采用第 1 条。**现在提，是因为等你真遇到的时候，现象是“偶尔卡死”，极难定位。**

---

## Rust 修炼小结

| 概念 | 本章用法 | 后面在哪用到 |
|---|---|---|
| `tokio::spawn` | 拆成引擎/渲染/输入三个任务 | 全书 |
| `mpsc::channel` | Op 下行、Event 上行 | 全书，第 17 章的 RPC |
| `async_trait` | 模型客户端抽象 | 第 6 章的 `Tool` trait 同款手法 |
| `CancellationToken` | 取消信号（先占位） | 第 7 章超时、第 19 章 Ctrl+C |
| `let-else`（`let Some(x) = .. else break`） | 简洁地处理“没有了就退出” | 全书 |

补充：`let Some(op) = self.op_rx.recv().await else { break };` 是 Rust 1.65 引入的 **let-else** 语法。它比 `match` 更贴近意图——**“我要的是 Some，不是的话走人”**。读代码时遇到不认识的语法先别慌，Rust 的新语法大多是为了减少嵌套。

---

## AI 软件工程原理 #3

> **事件流是 agent 系统的真相来源。**

这句话有三层含义，一层比一层重要。

**第一层：可观测。** 有了事件流，你才知道 agent 在干什么。传统程序可以打日志，但日志是散落的；事件流是结构化的、有因果顺序的、完整的。

**第二层：可回放。** 事件流落盘之后，你就能把一次运行重放出来——**而这正是调试 agent 的唯一可靠方式**。因为 agent 的行为不可复现（模型有随机性），你没法像调试普通程序那样“再跑一次看看”。你只能回放记录下来的那次。

**第三层：可判定。** 这是最重要的一层。把一次运行的关键事实落到结构化记录后，你就能写出这样的断言：

```rust
// items 是带类型的历史条目（Item 的精确定义第 5 章展开）；
// 工具成败承载在契约 Item::ToolResult 上，而不是 Event 变体上
assert!(items.iter().any(|m| matches!(m, Item::ToolResult { is_error: true, .. })));
```

注意断言对象是 `Item` 而不是 `Event`：**工具结果的正文属于契约的 `Item` 侧**，事件流只记录“本轮调用了哪个工具”（第 6 章的 `Event::ToolCallRecord`）。这不削弱“事件流是真相来源”——事件流是可判定历史的转录底稿，重放它即可重建这份 `items`。

**“agent 有没有犯错”从一个主观判断，变成了一个可以放进 CI 的布尔值。**

回到原理 #1 的那个分水岭：传统工程靠 code review 兜质量，AI 工程靠机械验证兜质量。**而机械验证的前提，是你得有东西可验证——事件流就是那个“东西”。**

### 这条原理决定了后面很多决策

- 第 5 章为什么要把 Item 拆那么细（粗粒度的“一条消息”没法判定）
- 第 16 章为什么用 append-only JSONL（可 diff、可回放、崩溃友好）
- 第 20 章为什么评测集能自动化（对比两次运行的事件序列）

**它们都是同一件事的不同侧面：先把真相记录下来，才谈得上检验真相。**

---

## 章末验收

- [ ] `cargo run` 能连续对话，`/quit` 干净退出
- [ ] 模型思考时你仍能输入（输入被排队，不会丢）
- [ ] `cargo test` 通过，且**不依赖网络和 API key**
- [ ] 你能说清：如果现在要加一个“把对话记录打到文件”的功能，需要改几个文件？（答案应该是 0 个改动引擎，只需新增一个 Event 消费者）

---

## 读者挑战

1. 现在的引擎在 `Op::UserInput` 到达时会立刻处理，处理完才收下一个。**如果用户在模型思考时连发三句话，会发生什么？** 这是你想要的行为吗？
2. `Event::AgentMessageDelta` 目前一次都没被发出（我们只在 `TurnComplete` 里给了完整文本）。**为什么还要留着它？** 提示：想想下一章。
3. 如果 `submission_loop` 里 `run_turn` 执行到一半，用户发了 `Op::Shutdown`，会怎样？**这是 bug 还是设计？**

---

---


# 第 4 章　流式响应：SSE 解析这一关必须过

**本章任务**：让文字一个字一个字吐出来，并写出一个**在任意字节位置切分都不出错**的 SSE 解析器。

这一章看起来是“锦上添花”，实际是**整个项目最容易埋下长期 bug 的地方**。流式解析出错的表现是“偶发乱码”——它不会让你的程序崩溃，只会让你的输出偶尔多一个问号，然后在三个月后某个用户的机器上变成一片乱码。

---

## 4.1 为什么要流式

先说一个反直觉的事实：**流式不是为了好看。**

真正的理由是：**长任务下，用户需要能判断“它到底是在思考，还是卡死了”。**

一个 agent 跑一个复杂任务，可能 40 秒没有任何输出。如果是个转圈动画，用户会在第 15 秒开始怀疑，第 25 秒杀掉进程。如果屏幕上一直在吐字，用户能一直等下去。

**这不是 UI 问题，是产品能不能用的问题。**

第二个理由是**中断时机的判断**。用户看到第 3 行就发现方向错了，可以立刻打断——而不是等 40 秒后看到完整结果才发现全白干了。

---

## 4.2 SSE 协议的四个坑

SSE（Server-Sent Events）格式长这样：

```
data: {"type":"response.output_text.delta","delta":"你"}

data: {"type":"response.output_text.delta","delta":"好"}

data: [DONE]

```

看着简单，实则四个坑：

**坑 1：网络会把数据切成任意大小的块**

你觉得会收到 `data: {"delta":"你好"}`，实际上可能收到：

```
第 1 个 chunk:  data: {"delta":"\xe4\xbd
第 2 个 chunk:  \xa0"}
```

**一个 UTF-8 字符被从中间劈开。** 中文是三字节，被劈开的概率不低。

**坑 2：行尾可能是 `\n`，也可能是 `\r\n`**

不同服务器、不同中间代理的行为不一样。只按 `\n` 切分，你会留下一个尾巴上的 `\r`。

**坑 3：`data:` 后面可能有多行**

SSE 规范规定，一个事件里的多个 `data:` 行要用 `\n` 连接。JSON 被换行拆开是常见做法。

**坑 4：`[DONE]` 是字符串，不是 JSON**

你必须先判断它，否则会拿去 `serde_json::from_str`，然后得到一条毫无意义的解析错误。

### 关键洞察

注意坑 1 的解法：**如果你只在“收到完整的一帧”之后才把字节转成字符串，那部分 UTF-8 的问题就自动消失了。**

帧的分界是 `\n\n`，那是 ASCII 字节，不可能和 UTF-8 多字节序列混淆。所以只要**按字节找分隔符、按帧处理**，中文被劈开根本不是问题。

**很多手写的 SSE 解析器会先把 buffer 转成 `String` 再处理——那就是乱码的根源。** 记住：**在帧边界确定之前，那些字节不是文本，只是字节。**

---

## 4.3 增量解析器

`crates/mcx-core/src/sse.rs`：

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SseEvent {
    /// 一帧数据（通常是 JSON）
    Data(String),
    /// 收到 `[DONE]`，流结束
    Done,
}

#[derive(Debug, thiserror::Error)]
pub enum SseError {
    #[error("帧内容不是合法 UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("缓冲区超过上限 {0} 字节，可能是对端没有正确分帧")]
    BufferOverflow(usize),
}

pub struct SseParser {
    buffer: Vec<u8>,
    max_buffer: usize,
}

impl SseParser {
    pub fn new() -> Self {
        Self { buffer: Vec::new(), max_buffer: 8 * 1024 * 1024 }
    }

    /// 喂进任意大小的一块字节，返回这一块里能解析出的所有完整事件。
    pub fn feed(&mut self, chunk: &[u8]) -> Result<Vec<SseEvent>, SseError> {
        self.buffer.extend_from_slice(chunk);

        let mut events = Vec::new();
        while let Some((idx, sep_len)) = find_frame_end(&self.buffer) {
            // 把 buffer 拆成 [帧内容][分隔符][剩余]
            let mut tail = self.buffer.split_off(idx);   // buffer=帧内容, tail 从分隔符开始
            tail.drain(..sep_len);                        // 丢掉帧界分隔符（2 或 4 字节）
            let frame = std::mem::replace(&mut self.buffer, tail);

            if let Some(ev) = Self::parse_frame(&frame)? {
                events.push(ev);
            }
        }

        if self.buffer.len() > self.max_buffer {
            return Err(SseError::BufferOverflow(self.max_buffer));
        }
        Ok(events)
    }

    /// 流结束时调用，处理最后一帧（有些服务器末尾不发分隔符）。
    pub fn finish(&mut self) -> Result<Option<SseEvent>, SseError> {
        if self.buffer.is_empty() { return Ok(None); }
        let frame = std::mem::take(&mut self.buffer);
        Self::parse_frame(&frame)
    }

    fn parse_frame(frame: &[u8]) -> Result<Option<SseEvent>, SseError> {
        let text = String::from_utf8(frame.to_vec())?;

        let mut data = String::new();
        for line in text.split('\n') {
            let line = line.strip_suffix('\r').unwrap_or(line);   // 处理 CRLF
            if line.is_empty() || line.starts_with(':') {
                continue;   // 空行、注释（心跳）
            }
            if let Some(rest) = line.strip_prefix("data:") {
                let rest = rest.strip_prefix(' ').unwrap_or(rest);  // 容忍有无空格
                if !data.is_empty() { data.push('\n'); }            // 多行 data 用 \n 连接
                data.push_str(rest);
            }
        }

        if data.is_empty() { return Ok(None); }
        if data.trim() == "[DONE]" { return Ok(Some(SseEvent::Done)); }
        Ok(Some(SseEvent::Data(data)))
    }
}

/// 找帧界分隔符，返回（起点, 长度）。SSE 规范允许 `\r\n` 行尾，
/// 所以帧界可能是 `\n\n`（LF，长 2）或 `\r\n\r\n`（CRLF，长 4）。
/// CRLF 必须显式处理：`\r\n\r\n` 的字节是 `0D 0A 0D 0A`，并不包含连续两个 `\n`。
fn find_frame_end(buf: &[u8]) -> Option<(usize, usize)> {
    let lf = buf.windows(2).position(|w| w == b"\n\n").map(|i| (i, 2));
    let crlf = buf.windows(4).position(|w| w == b"\r\n\r\n").map(|i| (i, 4));
    match (lf, crlf) {
        (Some(a), Some(b)) => Some(if a.0 <= b.0 { a } else { b }),
        (a, b) => a.or(b),
    }
}
```

**几个实现细节值得琢磨**：

- `split_off` + `mem::replace` 这个组合，避免了 `Vec` 头部删除的 O(n) 拷贝。`split_off(idx)` 后 `self.buffer` 是 `[0, idx)`，`tail` 是 `[idx, ..)`；丢掉 tail 开头紧跟的帧界分隔符——LF 帧 `\n\n` 是 2 字节，CRLF 帧 `\r\n\r\n` 是 4 字节，这也是 `find_frame_end` 要返回分隔符长度的原因——再用 `replace` 把 tail 换进 buffer、把帧内容取出来。**零拷贝。**
- `max_buffer` 是防止对端不发分隔符导致内存无限增长。**所有从不信任来源读数据的解析器都必须有这个上限**——这是原理 #4 的直接体现。
- `finish()` 处理“最后一帧没有分隔符”的情况，以及心跳注释（`:ping`）。

---

## 4.4 解析 Responses API 的增量事件

有了帧，还要从 JSON 里抠出文本增量：

```rust
#[derive(serde::Deserialize)]
struct DeltaEnvelope {
    #[serde(rename = "type")]
    kind: String,
    delta: Option<String>,
}

/// 从一帧 SSE 数据里取出文本增量；不是文本增量则返回 None。
pub fn extract_delta(payload: &str) -> Option<String> {
    let env: DeltaEnvelope = serde_json::from_str(payload).ok()?;
    match env.kind.as_str() {
        "response.output_text.delta" => env.delta,
        _ => None,
    }
}
```

**注意 `.ok()?`**：解析失败就当这个事件不存在，不报错。

为什么这么宽容？因为**服务端会不断新增事件类型**。今天你不认识的 `response.reasoning.delta`，明天可能是重要功能。如果解析失败就让整个流终止，你的 agent 会在服务端升级那天集体崩溃（如果我们在解析上偷懒的话）。

> **这是原理 #4 的一条推论**：对不认识的东西要宽容，对格式错误的东西要严格。区别在于——**不认识的字段是“未来的功能”，畸形的帧是“当前的故障”。**

---

## 4.5 接进 Session

改造 `LlmClient`，让它能吐增量：

```rust
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// 逐块把文本增量推进 `delta_tx`，返回完整文本。
    async fn complete(
        &self,
        messages: &[Message],
        delta_tx: &mpsc::Sender<String>,
    ) -> Result<String, LlmError>;
}
```

HTTP 流式请求：

```rust
async fn complete(&self, messages: &[Message], delta_tx: &mpsc::Sender<String>)
    -> Result<String, LlmError>
{
    let mut resp = self
        .http
        .post("https://api.openai.com/v1/responses")
        .bearer_auth(&self.api_key)
        .json(&self.build_body(messages, /* stream = */ true))
        .send()
        .await?
        .bytes_stream();

    let mut parser = SseParser::new();
    let mut full = String::new();
    let mut finished = false;

    while let Some(chunk) = resp.next().await {
        let chunk = chunk?;
        for ev in parser.feed(&chunk)? {
            match ev {
                SseEvent::Data(payload) => {
                    if let Some(delta) = extract_delta(&payload) {
                        full.push_str(&delta);
                        let _ = delta_tx.send(delta).await;
                    }
                }
                SseEvent::Done => { finished = true; }
            }
        }
        if finished { break; }
    }

    // 收尾：处理没有分隔符的最后一帧
    if let Some(SseEvent::Data(payload)) = parser.finish()? {
        if let Some(delta) = extract_delta(&payload) {
            full.push_str(&delta);
            let _ = delta_tx.send(delta).await;
        }
    }
    Ok(full)
}
```

需要两处依赖改动，缺一不可：

```toml
# mcx-core/Cargo.toml
futures-util = { workspace = true }

# 根 Cargo.toml —— 注意 reqwest 必须开 stream feature，
# 否则 `.bytes_stream()` 会报 no method found
reqwest = { version = "0.12", default-features = false,
            features = ["json", "rustls-tls", "stream"] }
```

根 `[workspace.dependencies]` 里加 `futures-util = "0.3"`。

> **这个坑很典型**：`reqwest` 关掉 `default-features` 后，很多“以为默认就有”的能力其实都没有。`json`、`stream`、`multipart` 全都要显式开。遇到 `no method named X found for RequestBuilder/Response`，先去核对 feature 列表。

然后 `run_turn` 里把增量转成 `Event::AgentMessageDelta`：

```rust
let (delta_tx, mut delta_rx) = mpsc::channel(64);
let ev_tx = self.event_tx.clone();   // mpsc::Sender 是 Clone 的（多生产者）

// 一边等模型完成，一边把增量转发成 Event
let forward = tokio::spawn(async move {
    while let Some(delta) = delta_rx.recv().await {
        let _ = ev_tx.send(Event::AgentMessageDelta(delta)).await;
    }
});

let reply = client.complete(&history, &delta_tx).await?;
drop(delta_tx);           // 关掉 sender，让 forward 任务能退出
let _ = forward.await;
```

> **`mpsc::Sender` 是 `Clone` 的**——`mpsc` 就是 multi-producer, single-consumer 的缩写。所以你不需要把 `self.event_tx` 包进 `Arc`，直接 clone 一份进任务即可。

> **别忘了 `drop(delta_tx)`。** 否则 `delta_rx.recv()` 永远等不到 `None`，forward 任务永不退出。**这是 tokio 里最常见的任务泄漏**，详见避坑专栏。

---

## 4.6 测试：在任意字节处切分

现在写本章最重要的测试。它不依赖网络、不依赖 proptest，但覆盖能力极强：

```rust
#[test]
fn any_split_point_yields_same_events() {
    // 故意包含：ASCII、中文（三字节 UTF-8）、[DONE]
    let stream: Vec<u8> = b"data: {\"x\":1}\n\ndata: \xe4\xbd\xa0\xe5\xa5\xbd\n\ndata: [DONE]\n\n"
        .to_vec();

    let expected = vec![
        SseEvent::Data("{\"x\":1}".into()),
        SseEvent::Data("你好".into()),
        SseEvent::Done,
    ];

    // 在每一个可能的字节位置把它切成两半，结果必须一致
    for split_at in 0..=stream.len() {
        let mut p = SseParser::new();
        let mut got = p.feed(&stream[..split_at]).unwrap();
        got.extend(p.feed(&stream[split_at..]).unwrap());
        assert_eq!(got, expected, "在字节 {split_at} 处切分时结果不一致");
    }
}

#[test]
fn one_byte_at_a_time() {
    let stream = b"data: hi\n\ndata: [DONE]\n\n";
    let mut p = SseParser::new();
    let mut got = Vec::new();
    for b in stream {
        got.extend(p.feed(&[*b]).unwrap());
    }
    assert_eq!(got, vec![SseEvent::Data("hi".into()), SseEvent::Done]);
}

#[test]
fn handles_crlf_and_multiline_data() {
    // CRLF 帧界 `\r\n\r\n` 长 4 字节且不含 `\n\n`——必须被显式切出，不能指望 finish() 兜底
    let mut p = SseParser::new();
    let got = p.feed(b"data: {\"a\":1}\r\ndata:   \"b\":2\r\n\r\n").unwrap();
    assert_eq!(got, vec![SseEvent::Data("{\"a\":1}\n  \"b\":2".into())]);
}

#[test]
fn crlf_frame_boundary_split_across_chunks() {
    // `\r` 与后面的 `\n\r\n` 分属两个 feed 块时也必须找到帧界；
    // 否则纯 CRLF 事件会一直积压到 finish() 才被处理，CRLF 用例即失败
    let stream = b"data: hi\r\n\r\ndata: [DONE]\r\n\r\n";
    for split in 1..stream.len() {
        let mut p = SseParser::new();
        let mut got = p.feed(&stream[..split]).unwrap();
        got.extend(p.feed(&stream[split..]).unwrap());
        assert_eq!(got, vec![SseEvent::Data("hi".into()), SseEvent::Done],
                   "在字节 {split} 处切分时 CRLF 帧界被破坏");
    }
}

#[test]
fn detects_done_sentinel() {
    let mut p = SseParser::new();
    let got = p.feed(b"data: [DONE]\n\n").unwrap();
    assert_eq!(got, vec![SseEvent::Done]);
}

#[test]
fn oversized_buffer_is_rejected() {
    let mut p = SseParser::new();
    p.max_buffer = 16;                       // 测试里调小上限
    assert!(matches!(
        p.feed(&[b'x'; 1024]),
        Err(SseError::BufferOverflow(_))
    ));
}
```

第一个测试是全章的核心：**它穷举了所有可能的网络切分方式。** 只要有一个字节边界的处理是错的，这个测试就会挂。

**而且它跑起来只要几毫秒。** 这就是为什么值得把解析器单独抽成一个纯函数——可测试性不是副产品，是设计目标。

---

## 4.7 Design Rationale

**Q：为什么不直接用现成的 SSE 库？**

可以用（`eventsource-stream`、`reqwest-eventsource` 都不错）。我们手写有两个理由——**教学价值**（亲手踩一遍“面向不可靠输入”的四类坑，评估任何库时才知道该问什么）与**可控性**（缓冲区上限、自定义错误处理、解析失败时保留原始字节）——正文 4.2/4.3 已经论证过，这里不重复。留给第 18 章的只有一句：评估 MCP 阶段外部工具带来的流时，这段“先手写一次”的经验会直接变成提问清单。

**实际项目建议**：先用第三方库跑起来；等遇到它解决不了的问题，再换成自己写的（你已经写过一遍了，换起来很快）。

**Q：为什么 `complete()` 既返回完整文本、又推送增量？**

因为两者的用途不同：

- **增量**给界面——用户要看到字一个个冒出来
- **完整文本**给 `history`——下一轮要把它整段发给模型

如果只返回增量，调用方要自己拼装，容易漏；如果只返回完整文本，界面就没法流式。**两个都要，各司其职。**

**Q：为什么用 channel 推增量，而不是返回 `impl Stream`？**

主要理由是**背压**（正文与避坑 #5 已论证）：`mpsc::channel(64)` 让下游消费慢时，上游 `send().await` 会等待，自动保护内存；用 `impl Stream` 也能做，只是要把背压显式写出来，代码更绕。第 18 章做 MCP 时我们会换成 `Stream`——因为那里的消费方是协议层，不是 UI。

---

## 避坑专栏 #5：忘了 drop sender，任务永不退出

这段代码看起来没问题：

```rust
let (delta_tx, mut delta_rx) = mpsc::channel(64);
let forward = tokio::spawn(async move {
    while let Some(d) = delta_rx.recv().await {
        let _ = ev_tx.send(Event::AgentMessageDelta(d)).await;
    }
});
client.complete(&history, &delta_tx).await?;
// 忘了 drop(delta_tx)
forward.await;   // ← 永远卡在这里
```

`delta_rx.recv()` 只有在**所有 sender 都被 drop** 时才返回 `None`。`delta_tx` 还活着（哪怕 `complete` 已经结束了），所以 `forward` 任务永远阻塞在 `recv()` 上，`forward.await` 永不返回。

**症状**：程序不崩溃、不报错，就是卡住不动。CPU 占用 0%。极难定位。

**三个防御手段**：

```rust
// 1. 显式 drop（最直白）
drop(delta_tx);
let _ = forward.await;

// 2. 用作用域限制 sender 生命期（最可靠，编译器帮你保证）
{
    let (delta_tx, mut delta_rx) = mpsc::channel(64);
    // ... 用 delta_tx
}   // 出了作用域自动 drop

// 3. 给 await 加超时（最后防线）
if tokio::time::timeout(Duration::from_secs(5), forward).await.is_err() {
    eprintln!("[警告] 转发任务超时未退出");
}
```

**推荐 2 和 3 搭配**：用作用域管生命期，用超时兜底。

> **通用形式**：tokio 里“任务不退出”的 bug，八成是某个 sender 没被 drop。看到程序静静卡住，先去数 sender。

---

## Rust 修炼小结

| 概念 | 本章用法 | 后面在哪用到 |
|---|---|---|
| `Vec<u8>` 缓冲 + `split_off` | 零拷贝地切分帧 | 第 8 章的 patch 解析器 |
| `std::mem::replace` / `take` | 取出值同时留下默认值 | 全书 |
| `String::from_utf8` | 标准库提供的字节转文本安全入口 | 任何解析场景 |
| `strip_prefix` / `strip_suffix` | 干净地剥掉前后缀 | 第 8、12 章 |
| `thiserror` | 库级别的错误类型 | 全书 |
| `futures_util::StreamExt` | 消费异步字节流 | 第 18 章 MCP |

---

## AI 软件工程原理 #4

> **面向不可靠输入编程。**

agent 系统的输入有四样，全部不可信：

| 来源 | 不可信的原因 |
|---|---|
| **网络** | 分片、乱序、中断、中间代理改写 |
| **文件系统** | 权限、编码、符号链接、并发修改 |
| **子进程输出** | 任意大小、可能含控制字符、可能永不结束 |
| **模型自己的输出** | 非法 JSON、幻觉的 API、超长文本、中途截断 |

注意最后一条——**模型输出是不可信输入**。这一点很多人意识不到：你会给外部 API 的返回写校验，却默认模型返回的一定是合法 JSON。

### 三条具体纪律

**① 任何从外部读的东西都要有上限**

```rust
max_buffer: usize        // SSE 缓冲区
max_output_bytes         // 第 7 章：命令输出
max_file_bytes           // 第 9 章：读文件
max_iterations           // 第 6 章：工具循环
```

**没有上限的读取，就是一个内存耗尽型漏洞。** 而且它往往在你跑最大那个任务时才炸。

**② 对不认识的宽容，对畸形的严格**

- 不认识的事件类型 → 忽略（可能是未来功能）
- 非法的 UTF-8 帧 → 报错（这是故障）

判据是：**前者不会让系统处于不一致状态，后者会。**

**③ 失败要留下原始信息**

```rust
LlmError::Malformed(resp.to_string())   // 把原始响应塞进错误
```

解析失败时如果你只说“解析失败”，你会对着日志发呆半小时。**把导致失败的原始字节留下来**——第 20 章做 trace 导出时，这是救命的。

---

## 章末验收

- [ ] `cargo run` 时文字逐字输出，不是一次性刷出
- [ ] 中文输出无乱码（用 `你好世界` 之类反复测）
- [ ] `any_split_point_yields_same_events` 测试通过
- [ ] 拔网线 / 用一个会中途断开的假服务器，程序能干净报错而不是卡死
- [ ] 你能说出：为什么“先把 buffer 转成 String 再处理”是错的

---

## 读者挑战

1. 如果服务端在流的正中间断开（TCP 连接中断），`bytes_stream().next()` 会返回什么？**你的代码当前会怎么处理？** 用户看到的应该是什么？
2. `finish()` 我们只在流正常结束时调用。**如果流异常中断，buffer 里的半帧数据该怎么办？**（提示：想想审计日志）
3. 现在的 `max_buffer` 是 8MB。**这个值该定多大？** 依据是什么？

---

> **中场：这一整章，我们都在把模型的话逐字递给屏幕。但换个问题——这些字最后要不要存？**
>
> 渲染可以消费一个丢一个：屏幕刷新后，`AgentMessageDelta("你")` 就没有继续存在的理由。可会话是要能“回到上一轮”、能审计、能当评测证据的。逐字增量存下来，是一堆几十万字符的碎片；可若不存增量，又该存什么粒度？
>
> 这个问题我们不在本章回答——**它属于第 5 章**。你只需要带着它翻页：第 5 章会引入一个新概念 **Item**，把“流水线上流动的增量”（Event）和“仓库里定型的记录”（Item）正式分开。到第 15 章你会看到，这个区分最终决定了上下文压缩“能丢什么、不能丢什么”。

---

---


# 第 5 章　类型建模：Item、Event 与协议演进

**本章任务**：把整个会话建模成类型系统，并定义一套**能在不破坏旧数据的前提下演进**的 JSONL 事件格式。

这章决定了后面所有能力能不能做出来——**这一章偷懒，后面（第 19 章 UI、第 20 章评测）会加倍还回来。**

---

## 5.1 Event 是流，Item 是账本

上一章末尾我们留了一个问题：逐字 delta 不能都存，那该存什么？答案不是“存更少字”，而是**换一层东西存**——把运行时流动的信号（Event）和事后定型的记录（Item）分开，各司其职。两者确实容易混淆，先厘清：

| | `Event` | `Item` |
|---|---|---|
| 本质 | **运行时信号** | **会话记录** |
| 频次 | 高频（每个字一个 delta） | 低频（每条消息一个 item） |
| 是否持久化 | 大部分不落盘 | **全都要落盘** |
| 消费方式 | 流式 | 整体读取 |
| 用途 | 渲染、中断、进度 | 回放、审计、评测、压缩 |

**一个类比**：Event 是流水线的传送带上的东西，Item 是成品仓库里的库存。

- `AgentMessageDelta("你")`、`AgentMessageDelta("好")` → 这两个 Event 最终合成一个 `Item::AgentMessage { content: "你好" }`
- 你不应该把每个 delta 都存进历史——那会让你的 JSONL 文件变成几百 KB 的碎片

**这个区分在第 15 章会变得极其重要**：上下文压缩要挑出“哪些 Item 可以丢、哪些必须留”。如果历史是一堆 delta 碎片，你连挑选的粒度都没有。

---

## 5.2 Thread / Turn / Item 三层模型

Codex 的会话模型是三层：

```
Thread（会话）       一次完整的对话，有 id，可持久化、可续、可 fork
  └── Turn（轮次）   一次用户输入 + 模型响应 + 期间所有工具调用
        └── Item     最小单位的记录
```

为什么要有 Turn 这一层？

**因为压缩的切点必须在 Turn 边界上。** 第 15 章我们会看到：如果你在 Turn 中间切断历史，会造出“有问无答”的残缺上下文，模型下一轮会困惑甚至重复劳动。**Turn 是语义上最小的完整单元。**

```rust
// crates/mcx-protocol/src/thread.rs

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Thread {
    pub id: String,
    pub created_ms: u64,
    pub turns: Vec<Turn>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Turn {
    pub index: usize,
    pub items: Vec<Item>,
}
```

---

## 5.3 用 serde 建模 tagged enum

`Item` 是一个 enum，落盘时需要自描述类型。用**内部标签枚举**：

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Item {
    UserMessage { content: String },
    AgentMessage { content: String },
    Reasoning { summary: String },
    ToolCall { call_id: String, name: String, arguments: String },
    ToolResult { call_id: String, output: String, is_error: bool },
    /// 兜底：遇到不认识的 type 时落到这个变体
    #[serde(other)]
    Unknown,
}
```

序列化出来长这样：

```json
{"type":"user_message","content":"帮我改个 bug"}
{"type":"tool_call","call_id":"c1","name":"read_file","arguments":"{\"path\":\"a.rs\"}"}
```

**为什么用 snake_case？** 因为 JSON 生态的惯例是 snake_case，而 Rust 惯例是 PascalCase 的 variant 名。`rename_all` 一处声明，全 enum 生效。

**`#[serde(other)]` 是干什么的？**

它让反序列化在遇到不认识的 `type` 时，落到 `Unknown` 而不是报错。

这带来一个极其重要的性质：**旧版本的代码能读新版本写的日志**（顶多是不认识的内容变成 `Unknown`），**新版本的代码能读旧版本的日志**。这叫**前向兼容**，是持久化格式最值得优先保证的性质之一。

### 但 Unknown 是静默降级

`Unknown` 会让数据“消失”——你看不到它的内容了。所以必须记日志：

```rust
match serde_json::from_str::<Item>(line)? {
    Item::Unknown => {
        tracing::warn!(line = %line, "遇到了不认识的 Item 类型，已跳过");
    }
    item => items.push(item),
}
```

**静默降级 + 无日志 = 未来某天你会花三小时找“数据去哪了”。**

---

## 5.4 落盘：JSONL

为什么是 JSONL 而不是 SQLite？

| | JSONL | SQLite |
|---|---|---|
| 崩溃友好 | 追加写，最后一行可能不完整，前面全好 | 需要事务，崩溃可能损坏 |
| 可 diff | 纯文本，`git diff` 直接看 | 二进制 |
| 可 grep | 直接 grep | 要写 SQL |
| 可回放 | 顺序读即可 | 需要查询 |
| 结构化查询 | 需要自己建索引（第 16 章） | 原生支持 |

**JSONL 赢在前四项，而这四项正是 agent 会话最需要的。** 结构化查询需求（比如“列出所有会话”）交给旁挂的 SQLite 索引——第 16 章我们会加。

```rust
// crates/mcx-core/src/rollout.rs

#[derive(Debug, thiserror::Error)]
pub enum RolloutError {
    #[error("I/O 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("序列化错误: {0}")]
    Json(#[from] serde_json::Error),
}

/// 一条落盘记录。Item 嵌套在 `item` 字段里，不 flatten。
#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    /// schema 版本。旧数据没有这个字段时默认 1。
    #[serde(default = "default_version")]
    pub v: u32,
    pub ts_ms: u64,
    pub thread_id: String,
    pub turn: usize,
    pub item: Item,
}

fn default_version() -> u32 { 1 }

pub struct Rollout {
    writer: BufWriter<File>,
    path: PathBuf,
}

impl Rollout {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, RolloutError> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        Ok(Self { writer: BufWriter::new(file), path: path.as_ref().to_path_buf() })
    }

    /// 追加一条记录。
    pub fn append(&mut self, rec: &Record) -> Result<(), RolloutError> {
        serde_json::to_writer(&mut self.writer, rec)?;
        self.writer.write_all(b"\n")?;
        // 每行都 flush：agent 崩溃时，已发生的事件不能丢
        self.writer.flush()?;
        Ok(())
    }

    /// 读回全部记录。遇到无法解析的行，记录警告并跳过。
    pub fn read_all(path: impl AsRef<Path>) -> Result<Vec<Record>, RolloutError> {
        let text = std::fs::read_to_string(path)?;
        Ok(text
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| match serde_json::from_str::<Record>(l) {
                Ok(r) => Some(r),
                Err(e) => {
                    tracing::warn!(err = %e, "跳过无法解析的行");
                    None
                }
            })
            .collect())
    }
}
```

**两个决策点**：

**① 每行 flush，牺牲性能换不丢数据。** 一次会话可能有几百条记录，每秒 flush 几次的开销可以忽略——但**崩溃时丢失审计记录是不可接受的**（第 10 章的审批审计全靠它）。如果性能真的成了瓶颈，可以改成每 N 条 flush 一次，但要知道你在用数据安全性换速度。

**② 读的时候跳过坏行而不是整体失败。** 因为崩溃留下的最后一行就是残缺的。**如果读的时候报错，你的会话就永远恢复不了了。** 这比丢一行数据严重得多。

（第 16 章我们会用“写临时文件 + 原子 rename”彻底解决残缺行的问题。）

---

## 5.5 协议演进规则

一旦有数据落盘，你就背上了兼容性债。四条规则，必须遵守：

**规则 1：只增不改语义**

新增 variant 可以；把 `content` 改名成 `text` 不行；把 `output` 从 `String` 改成 `Option<String>` 要慎重。

**规则 2：新字段必须 `#[serde(default)]`**

```rust
#[serde(default)]
pub is_error: bool,
```

否则旧数据（没有这个字段）读不进来。

**规则 3：绝不删除 variant，改成 deprecated**

```rust
pub enum Item {
    // ...
    /// 已废弃：v2 之后改用 ToolCall。保留仅为兼容旧日志。
    #[serde(rename = "shell_call")]
    ShellCallLegacy { command: String },
}
```

删掉它的成本是“所有旧日志读不了”；留着它的成本是“多一个 match 分支”。**默认选择留着，除非字段带敏感或隐私风险。**

**规则 4：版本号只在破坏性变更时才升**

`v` 字段不是装饰品，它是你在读旧数据时判断该怎么处理的依据。只在真的无法兼容时才 +1。

### 演进检查清单

每次改 `Item`，问自己四个问题：

1. 旧代码读到新数据会怎样？（应该：落到 `Unknown`，不崩）
2. 新代码读到旧数据会怎样？（应该：新字段用默认值，不崩）
3. 有没有测试覆盖“读旧文件”？（应该：有，且旧文件是**真实的历史文件**，不是手写的假数据）
4. 我有没有删掉任何 variant 或改字段语义？

**第 3 条最容易被忽略。** 手写的“旧格式假数据”和你一年前真实写出去的数据不是一回事。正确做法是：**把真实的旧日志提交进测试固件目录**，像对待化石一样对待它们。

---

## 5.6 前向兼容测试

```rust
const LEGACY_V1: &str = r#"{"ts_ms":1700000000000,"thread_id":"t1","turn":0,"item":{"type":"user_message","content":"你好"}}"#;

#[test]
fn reads_legacy_record_without_version_field() {
    let rec: Record = serde_json::from_str(LEGACY_V1).unwrap();
    assert_eq!(rec.v, 1, "缺 v 字段时应默认为 1");
    assert_eq!(rec.item, Item::UserMessage { content: "你好".into() });
}

#[test]
fn unknown_item_type_falls_back_to_unknown() {
    // 模拟未来版本写出的、当前代码不认识的类型
    let line = r#"{"v":2,"ts_ms":1,"thread_id":"t","turn":0,
                   "item":{"type":"web_search","query":"rust sse"}}"#;
    let rec: Record = serde_json::from_str(line).unwrap();
    assert_eq!(rec.item, Item::Unknown);
}

#[test]
fn roundtrip_preserves_every_variant() {
    let items = vec![
        Item::UserMessage { content: "a".into() },
        Item::AgentMessage { content: "b".into() },
        Item::Reasoning { summary: "c".into() },
        Item::ToolCall { call_id: "1".into(), name: "read".into(), arguments: "{}".into() },
        Item::ToolResult { call_id: "1".into(), output: "d".into(), is_error: false },
    ];
    for item in items {
        let json = serde_json::to_string(&item).unwrap();
        assert_eq!(serde_json::from_str::<Item>(&json).unwrap(), item);
    }
}

#[test]
fn trailing_garbage_line_is_skipped_not_fatal() {
    let dir = std::env::temp_dir().join("mcx-test-rollout");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("rollout.jsonl");

    let rec = Record {
        v: 1, ts_ms: 1, thread_id: "t".into(), turn: 0,
        item: Item::UserMessage { content: "ok".into() },
    };
    let mut rollout = Rollout::open(&path).unwrap();
    rollout.append(&rec).unwrap();
    drop(rollout);

    // 模拟崩溃：追加半行垃圾
    use std::io::Write;
    std::fs::OpenOptions::new().append(true).open(&path)
        .unwrap()
        .write_all(b"{\"v\":1,\"ts_ms\":2,\"thr")
        .unwrap();

    let recs = Rollout::read_all(&path).unwrap();
    assert_eq!(recs.len(), 1, "残缺行应被跳过，且不影响前面的记录");
}
```

---

## 5.7 Design Rationale

**Q：为什么要把 Item 拆这么细？Event 为什么不直接落盘？**

一句话答案在 5.1 的表里已经给出——Event 是流动的运行时信号、大部分不落盘，Item 是定型的会话记录、全部落盘；这两个问题其实是同一分工的两面。这里只补 5.1 表没展开的角度：**渲染、审计、评测、压缩等消费方对粒度的要求不同**——TUI 渲染 diff 要知道“哪个文件的哪几行被改了”，审计要结构化的 `FileChange`，评测要独立的 `ToolCall` / `ToolResult`，压缩要知道哪些 Item 能丢、哪些必须留，fork 需要精确的切分点。粗粒度的“一条消息”同时喂不饱这些消费方，这才值得把 Item 拆细、并把高频 Event 挡在账本之外。Codex 的 `Event` 有几十个变体，正是被这些需求逼出来的（codex-rs 源码里的实际类型名是 `EventMsg`，以源码为准）。

**Q：为什么用 `tag = "type"` 的内部标签，而不是 `{"kind": ..., "data": ...}` 的外部标签？**

内部标签序列化出来更扁平：

```json
// 内部标签
{"type":"user_message","content":"hi"}
// 外部标签
{"kind":"user_message","data":{"content":"hi"}}
```

内部标签少一层嵌套，文件更小、可读性更好、和 Codex 的格式一致。

**代价**：内部标签枚举和 `#[serde(flatten)]` 不兼容（serde 的已知限制）。所以我们把 `Item` 嵌在 `Record.item` 字段里，而不是 flatten 进 `Record`。**这是有意为之的绕行，见避坑专栏。**

---

## 避坑专栏 #6：内部标签枚举的三个坑

**坑 1：忘了 `type` 字段，报错信息很误导**

```rust
// 这样写的代码
let item: Item = serde_json::from_str(r#"{"content":"hi"}"#)?;
// 报错
// Error: missing field `type`
```

错误信息说“缺 type”，但你可能以为是自己数据结构错了。**其实是你的输入少了一个字段。** 遇到 `missing field 'type'`，先去检查输入，不是改结构体。

**坑 2：`#[serde(flatten)]` 和内部标签枚举不共存**

```rust
#[derive(Deserialize)]
pub struct Record {
    pub ts_ms: u64,
    #[serde(flatten)]        // ← 和内部标签枚举一起用会炸
    pub item: Item,
}
```

serde 处理 flatten 时需要先把未知字段收集进一个 buffer，而内部标签枚举也依赖这个 buffer——两者冲突，会在**运行时**（不是编译时）产生诡异行为甚至 panic。

**解法**：嵌套，不要 flatten。

```rust
pub struct Record {
    pub ts_ms: u64,
    pub item: Item,     // {"ts_ms":..., "item":{"type":..., ...}}
}
```

**坑 3：`#[serde(other)]` 只能是单元变体**

```rust
#[serde(other)]
Unknown,                          // ✅ 可以
// Unknown { raw: Value },        // ❌ 编译不过
```

如果你想保留未知内容用于诊断，得换个思路：先反序列化成 `serde_json::Value`，判断 `type` 字段，再手动分发。

```rust
let value: serde_json::Value = serde_json::from_str(line)?;
match value["type"].as_str() {
    Some("user_message") => serde_json::from_value(value)?,
    other => {
        tracing::warn!(ty = ?other, raw = %value, "未知 Item 类型");
        continue;
    }
}
```

**代价是你要手写分发逻辑。** 选哪个取决于你要不要保留未知数据——** agent 场景下我倾向保留，因为你不知道服务端哪天会加什么。**

---

## Rust 修炼小结

| 概念 | 本章用法 | 后面在哪用到 |
|---|---|---|
| `#[serde(tag = "type")]` | 自描述的 enum 序列化 | 全书所有持久化格式 |
| `#[serde(rename_all)]` | 统一命名风格 | 全书 |
| `#[serde(default)]` | 前向兼容的关键 | 第 13 章配置分层 |
| `#[serde(other)]` | 未知变体兜底 | 第 18 章 MCP 动态工具 |
| `BufWriter` + `flush` | 落盘与持久性权衡 | 第 16 章原子写入 |
| 嵌套而非 flatten | 绕开 serde 限制 | 全书 |

---

## AI 软件工程原理 #5

> **先设计能被机器判定的产物，再设计产生它的代码。**

这句话是原理 #3（事件流是真相来源）的操作化版本。

**事件 schema 就是 agent 系统的验收标准。** 在你写任何 agent 逻辑之前，先问：**“跑完之后，我会留下什么东西？这个东西能不能被机械地判定对错？”**

### 为什么这个顺序重要

反过来做（先写 agent，再想怎么记录）的后果是：**你记录下来的东西，恰好是你当时想到的那些。** 而三个月后你要评估的新维度，恰好不在里面。

举几个真实的例子：

| 你后来想问的问题 | 需要提前记录什么 |
|---|---|
| “这次比上次好吗？” | 每次工具调用的耗时和结果 |
| “它是不是在原地打转？” | 工具调用序列（可 diff） |
| “它改了哪些文件？” | 结构化的 `FileChange`，不是 patch 文本 |
| “哪一步最贵？” | 每次调用的 token 用量 |
| “用户批准了什么？” | 审批决策的完整上下文 |

**这些字段，没有一个是“写完代码顺手加”能加出来的。** 你必须在设计阶段就想清楚。

### 一个自测问题

给你当前的事件 schema，问：**“我能不能只靠它，判断一次运行是成功还是失败？”**

- 如果答案是“能” → 你的 schema 可以支撑自动化评测
- 如果答案是“还得人工看一眼” → 你的 schema 有缺口，而且这个缺口会随着任务变复杂而放大

**这个自测在第 20 章会变成硬指标**：我们的 20 个回归任务，每一个都必须有机械判定的验收条件。做不到，就说明 schema 设计得不够。

---

## 章末验收

- [ ] `Item` 的每个变体都能 round-trip（序列化 → 反序列化 → 相等）
- [ ] 读取缺少 `v` 字段的旧记录能成功，`v` 默认为 1
- [ ] 遇到不认识的 `type` 落到 `Unknown` 并**打了警告日志**
- [ ] 日志里追加半行垃圾后，`read_all` 仍能读出前面的完整记录
- [ ] 你能说出：为什么 `#[serde(flatten)]` 不能和内部标签枚举一起用

---

## 读者挑战

1. 现在 `Item::Reasoning` 只有一个 `summary` 字段。**如果模型返回的是加密的推理内容（Codex 就是这么做的），你的 schema 该怎么设计？** 提示：想想“云端续会话”的场景。
2. `Rollout::read_all` 遇到坏行会跳过。**如果用户想知道“我的日志有损坏”呢？** 该怎么设计 API 才能既不破坏读取、又不隐藏问题？
3. 我们说“绝不删除 variant”。**如果某个 variant 存的是敏感信息（比如被误记下来的 API key），必须删除呢？**（这是真实会发生的事）

---

## 下一章预告：会聊天，不等于会干活

骨架有了，会话能跑，事件能存。但它还只是个聊天的——**它什么也做不了。**

第 6 章我们给它装上工具系统：

- 用 `trait Tool` 抽象出“能被模型调用的能力”
- 用 `Box<dyn Tool>` 做运行时注册表（为什么不用 enum？因为第 18 章 MCP 会在运行时塞进你没见过的工具）
- 把第 3 章留空的**第三层循环（tool loop）**填上

你会第一次看到 agent 真正“动起来”：它说要读一个文件，你替它读，它看完再说下一步。**那个循环，就是 agent 和 chatbot 的分界线。**


---

# 第 6 章　工具系统：Trait 与 Registry

**本章任务**：装上第一个工具，把第 3 章留空的第三层循环——tool loop——真正填上 `while`。从此 agent 从“会说”变成“会做”。

第 5 章末尾我们停在这样一个画面：引擎已经能流式对话、历史在追加、事件流在往外冒，但每一轮只有一次模型调用。模型说“我要读文件”，引擎听不懂；模型说“我来改代码”，引擎没有手。**这一章要补上的就是那双手。** 而关键不在于“加一个工具”，在于**加完之后 agent loop 一行都不用改**——这既是衡量本章设计是否正确的标尺，也是后面 13 章所有扩展的起点。

---

## 6.1 先把那个空位找出来

第 3 章的 `run_turn` 长这样（为聚焦，省略流式细节，沿用契约里的 `complete` 签名）：

```rust
// crates/mcx-core/src/session.rs（第 3 章版本）
async fn run_turn(&mut self, text: String) {
    self.turn += 1;
    let turn = self.turn;
    self.emit(Event::TurnBegin { turn }).await;
    self.history.push(Message { role: Role::User, content: text });

    // 第二层循环目前只有一次迭代。
    // 第 6 章引入工具后，这里会变成 while 循环：
    //   模型要调工具 → 执行 → 结果追加进 history → 再问模型 → 直到不再要调工具
    match self.client.complete(&self.history).await {
        Ok(reply) => {
            self.history.push(Message { role: Role::Assistant, content: reply.clone() });
            self.emit(Event::TurnComplete { turn, text: reply }).await;
        }
        Err(e) => { /* ... */ }
    }
}
```

那行注释就是本章全部工作的坐标。**改动前，一次 turn = 一次模型调用；改动后，一次 turn = “模型 ↔ 工具”反复往返，直到模型不再要工具。**

先想清楚往返长什么样。模型的输出不再是纯文本，而是一个“动作序列”：可能含若干工具调用，可能含一段自然语言回复，也可能两者都有。每个工具调用有名字、有参数（`arguments`，契约里是 `String`，即 JSON）、有 `call_id`。我们把调用记录成 `Item::ToolCall`，把执行结果记录成 `Item::ToolResult`，两者用 `call_id` 配对——这正是契约里那一对类型存在的理由。

```
模型输出
 ├─ ToolCall { call_id, name: "read_file", arguments }
 ├─ ToolCall { call_id, name: "list_dir",  arguments }
 └─ ToolResult { call_id, output }   ← 执行后塞回 history
```

**history 是模型与工具共享的唯一真相。** 模型看不到“副作用发生了”，它只能看到 history 里新增的 `ToolResult`。所以每执行一个调用，就必须往 `history` 追加对应的 `Item::ToolResult`——少一条，模型就活在另一个世界里。

---

## 6.2 工具抽象：`trait Tool`

工具在 mini-codex 里是一个统一接口：有名字、有给模型看的描述与参数 schema、能被异步调用。把它做成 trait，放进独立的 `mcx-tools` crate（依赖方向：`mcx-core → mcx-tools → mcx-protocol`，不反向）：

```rust
// crates/mcx-tools/src/lib.rs
use async_trait::async_trait;
use mcx_protocol::Item;
use serde_json::Value;

/// 一个可被模型调用的工具。
#[async_trait]
pub trait Tool: Send + Sync {
    /// 工具名，必须全局唯一（用于 Registry 的 key 与模型的 `name` 字段）
    fn name(&self) -> &str;

    /// JSON Schema，描述 arguments 的结构 + 自然语言描述。
    /// 这份 schema 会被序列化进系统提示词，模型靠它决定"要不要调、怎么填参数"。
    fn schema(&self) -> Value;

    /// 执行。arguments 是模型生成的 JSON 字符串（契约里 Item::ToolCall.arguments 就是 String）。
    /// 返回 (output, is_error)。即使工具"失败"也应返回 Ok，用 is_error=true 告诉模型去纠错——
    /// 因为工具执行失败是正常业务路径，不是引擎崩溃。
    async fn call(&self, arguments: &str) -> Result<ToolOutput, ToolError>;
}

#[derive(Debug, Clone)]
pub struct ToolOutput {
    pub output: String,
    pub is_error: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("参数无法解析: {0}")]
    BadArguments(String),
    #[error("工具内部错误: {0}")]
    Internal(String),
    #[error("参数不合法: 缺少字段或类型不符")]
    InvalidArgs, // 第 18 章 MCP 远程工具的参数校验使用
}
```

三点值得停下来看：

**① `arguments` 是 `String`，不是已解析的结构体。** 因为第 18 章 MCP 来的工具，参数 schema 在运行时才拿到，编译期没有对应的 Rust 类型。统一成 JSON 字符串，让所有工具——内置的和远程的——走同一条调用路径。代价是每次都要自己解析，收益是**接口永远闭合**。

**② `Tool: Send + Sync`。** 因为工具要被装进 `Session`（在 tokio 任务里跨 `await` 使用），且 Registry 会被多线程共享。`async_trait` 把 `async fn` 改写成返回 `Pin<Box<dyn Future>>`——和第 3 章 `LlmClient` 同款手法，代价是一次堆分配，对 agent 场景可忽略。

**③ 错误返回 `ToolOutput { is_error: true }` 而不是 `Err`。** 区分“工具崩了”（进程异常、解析失败 → 引擎层面）和“工具告诉模型它失败了”（文件不存在、命令返回非零 → 模型层面）。后者必须让模型看到，它才会自我纠正。**把错误变成对话，而不是终止对话。**

---

## 6.3 Registry：为什么是 `Box<dyn Tool>`，不是 enum

最容易想到的方案是“工具种类有限，用 enum 不就行了”：

```rust
// 反例：用 enum 枚举所有工具
pub enum Tool {
    ReadFile(ReadFileTool),
    ListDir(ListDirTool),
    Shell(ShellTool),
    // ...每加一个工具就改这里
}
```

它类型安全、零堆分配、调用不用虚表——听起来很 Rust。**但它过不了第 18 章那一关**：MCP（Model Context Protocol）允许一个外部进程在运行时把新工具注册进来，名字和参数直到连接建立才知道。你的二进制里根本没有对应变体，编译器也没法预知。`enum` 要求“编译期枚举所有可能性”，而这里的可能性是**开放的**。

结论：**凡是“运行时才知道有什么”的地方，都需要动态分发。** 这正是 trait object 出场的条件：

```rust
// crates/mcx-core/src/tools.rs
use std::collections::HashMap;

pub struct Registry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl Registry {
    pub fn new() -> Self { Self { tools: HashMap::new() } }

    /// 注册工具。返回旧的同名工具（若有），便于测试里临时替换。
    pub fn register(&mut self, tool: Box<dyn Tool>) -> Option<Box<dyn Tool>> {
        let name = tool.name().to_string();
        self.tools.insert(name, tool)
    }

    pub fn get(&self, name: &str) -> Option<&dyn Tool> { self.tools.get(name).map(|t| &**t) }

    /// 把所有工具的 schema 拼成一份清单，喂给模型。
    pub fn schemas_json(&self) -> serde_json::Value {
        let mut arr = serde_json::Value::Array(Vec::new());
        if let serde_json::Value::Array(a) = &mut arr {
            for tool in self.tools.values() {
                a.push(tool.schema());
            }
        }
        arr
    }
}
```

`HashMap<String, Box<dyn Tool>>` 的好处现在还不显眼，到第 18 章就变成决定性优势：**MCP 连上来，直接 `registry.register(boxed_remote_tool)`，agent loop 一个字节不用改。** 这就是 6.1 说的“加工具只改一处”——那“一处”就是 Registry 的注册点，而不是 loop。

> **Rust 新手最容易拧巴的点**：觉得 `Box<dyn Trait>` 是“退而求其次”。不是。它是**专门为“开放集”准备的正确工具**。用 enum 处理开放集，每加一种就改 N 个 `match`；用 trait object，扩展点只有一个。判断口诀：**可能性在编译期封闭 → enum；运行时开放 → trait object。**

---

## 6.4 工具描述即提示词

工具的 schema 不是给人看的文档，是**模型的输入**。它写得好不好，直接决定模型一次调对的概率。一份 schema 同时承担两件事：结构约束（参数名、类型、必填）和语义约束（什么时候该调、参数取什么值）。

以 `read_file` 为例：

```rust
// crates/mcx-tools/src/read_file.rs
use serde_json::json;

pub struct ReadFileTool { /* cwd, limits ... */ }

impl Tool for ReadFileTool {
    fn name(&self) -> &str { "read_file" }

    fn schema(&self) -> Value {
        json!({
            "name": "read_file",
            "description":
                "读取仓库内一个文本文件的全部或部分内容。\
                 路径相对于工作目录；二进制文件会被拒绝。\
                 优先用 list_dir 确认路径后再调用。",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "相对于工作目录的文件路径，如 src/main.rs"
                    },
                    "offset": {
                        "type": "integer",
                        "description": "起始行号（从 0 开始），用于大文件分段读取"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "最多读取的行数，默认 500"
                    }
                },
                "required": ["path"]
            }
        })
    }

    async fn call(&self, arguments: &str) -> Result<ToolOutput, ToolError> {
        let args: ReadArgs = serde_json::from_str(arguments)
            .map_err(|e| ToolError::BadArguments(e.to_string()))?;
        // ... 实际读取逻辑见第 9 章；此处先用桩
        Ok(ToolOutput { output: format!("(read {})", args.path), is_error: false })
    }
}

#[derive(serde::Deserialize)]
struct ReadArgs { path: String, offset: Option<usize>, limit: Option<usize> }
```

**注意 `description` 里的那句“优先用 list_dir 确认路径后再调用”。** 这不是注释，这是给模型的指令。模型没有项目的心理模型，它只能从工具描述里推断调用顺序。把组合策略写进描述，比在系统提示词里另起一段更精准——**描述附着在工具上，工具出现在上下文里，上下文就是模型的全部世界。**

同样，“参数取什么值”也要写进 description：`path` 是“相对路径，如 src/main.rs”——给一个例子，模型的遵循率显著高于只写类型。这些细节累积起来，就是“模型一次调对”和“反复纠错浪费五轮”的差距。

> **这就是原理 #6 的落地标准**：评判一个工具好不好，别问“实现简不简洁”，要问“**模型在没有额外提示的情况下，第一次就填对参数的概率有多高**”。参数名、示例值、合法取值范围、调用前置条件——凡是模型可能猜错的地方，都要写进 schema。

---

## 6.5 填上 while：tool loop 的完整形态

现在改 `Session`。给 `Session` 加一个 `tools: Registry` 字段，并把 `run_turn` 改成循环：

```rust
// crates/mcx-core/src/session.rs（本章改造后）
pub struct Session<C: LlmClient> {
    client: C,
    history: Vec<Message>,
    tools: Registry,                    // 新增
    op_rx: mpsc::Receiver<Op>,
    event_tx: mpsc::Sender<Event>,
    cancel: CancellationToken,
    turn: usize,
    /// 单 turn 内工具调用轮次上限，防失控（原理 #7 的前兆）
    max_tool_rounds: usize,
}

impl<C: LlmClient> Session<C> {
    pub fn new_with_tools(
        client: C, op_rx: mpsc::Receiver<Op>, event_tx: mpsc::Sender<Event>, tools: Registry,
    ) -> Self {
        Self { client, tools, op_rx, event_tx, cancel: CancellationToken::new(), turn: 0, max_tool_rounds: 25 }
    }

    async fn run_turn(&mut self, text: String) {
        self.turn += 1;
        let turn = self.turn;
        self.emit(Event::TurnBegin { turn }).await;
        self.history.push(Message { role: Role::User, content: text });

        // ★ 第三层循环：模型 ↔ 工具 往返，直到模型不再产生 ToolCall
        for _ in 0..self.max_tool_rounds {
            // 注：真实实现把 tools.schemas_json() 注入消息；这里沿用第 4 章 §4.5 的流式签名，
            // 每轮建一个增量通道，把文本块转成 Event::AgentMessageDelta
            let (delta_tx, mut delta_rx) = mpsc::channel::<String>(64);
            let ev_tx = self.event_tx.clone();   // mpsc::Sender 可 Clone：多生产者
            let forward = tokio::spawn(async move {
                while let Some(delta) = delta_rx.recv().await {
                    let _ = ev_tx.send(Event::AgentMessageDelta(delta)).await;
                }
            });

            let reply = match self.client.complete(&self.history, &delta_tx).await {
                Ok(r) => r,
                Err(e) => { self.emit(Event::Error(e.to_string())).await; break },
            };
            drop(delta_tx);           // 关掉 sender，让 forward 任务能退出
            let _ = forward.await;
            self.history.push(Message { role: Role::Assistant, content: reply.clone() });

            let calls = parse_tool_calls(&reply);      // 从模型输出里抠出 ToolCall
            if calls.is_empty() { break; }             // 没有工具调用 → 本轮结束

            let mut results = Vec::new();
            for call in calls {
                self.emit(Event::ToolCallRecord { turn, call_id: call.call_id.clone(),
                                                  name: call.name.clone() }).await;
                let output = self.execute_call(&call).await;
                results.push(Item::ToolResult {
                    call_id: call.call_id, output: output.output, is_error: output.is_error,
                });
            }
            // 所有结果一次性追加回 history，再问模型下一轮
            for item in &results {
                self.history.push(item_to_message(item));   // ToolResult → Message
            }
        }

        self.emit(Event::TurnComplete { turn, text: last_text(&self.history) }).await;
    }

    async fn execute_call(&self, call: &ToolCall) -> ToolOutput {
        match self.tools.get(&call.name) {
            Some(tool) => match tool.call(&call.arguments).await {
                Ok(out) => out,
                Err(e) => ToolOutput { output: format!("工具错误: {e}"), is_error: true },
            },
            None => ToolOutput { output: format!("未知工具: {}", call.name), is_error: true },
        }
    }
}
```

`Event::ToolCallRecord` 需在 `mcx-protocol` 的 `Event` 里新增一个变体（沿用 `#[derive(Clone, PartialEq)]`）。契约的 `Item::ToolCall`/`ToolResult` 已就绪，这里只是把它们接进事件流。

**三处改动，一个原则**：

1. **`for _ in 0..max_tool_rounds` 代替无界 `loop`。** 模型可能陷入“调工具→看结果→再调同一个”的死循环。上限是护栏，超了就截断本轮——第 7 章会把“每个副作用都要有边界”这条原则系统展开，这里先埋伏笔。
2. **`if calls.is_empty() { break }`。** 这是循环的退出条件：模型不再产出 `ToolCall`，就意味着“我做完了，这是给用户的回复”。**循环的终止由模型决定，不由我们预设的步骤数决定。**
3. **每轮先把所有 `ToolCall` 收集、执行、再统一把 `ToolResult` 回填 history，然后才进入下一轮。** 模型看到的是一整批结果，而不是交错的状态——这对应 OpenAI 的 “function calling” 语义，也让 history 保持干净的顺序。

### 改动前后对比：骨架对了的收益

| | 改动前 | 改动后 |
|---|---|---|
| `run_turn` 结构 | 一次 `complete` 即结束 | `for` 循环包裹 `complete` + 工具执行 |
| 新增工具 | — | 只在 Registry 注册，**`run_turn` 零修改** |
| 模型输出 | 纯文本 | 文本 + `ToolCall` 序列 |
| history | 只有 User/Assistant | 新增 `ToolCall`/`ToolResult` 配对 |
| 终止条件 | 固定（调用一次） | 模型不再产出调用 / 达上限 |

**这就是第 3 章那句“骨架对了，后面加什么都不用推倒重来”的具体兑现。** 三层循环的位置三年前就留好了，今天只往里填 `while`，外层 `submission_loop`、channel、事件流——全部不动。

> **加工具只改一处**：定义 struct 实现 `Tool` → `registry.register(Box::new(ReadFileTool::new()))`。agent loop、Session、CLI、事件流，一行不改。这条性质你可以立刻写成测试（见 6.7）。

---

## 避坑专栏 #7：`Box<dyn Tool>` 的 `'static` 陷阱

**错误写法**：

```rust
// 编译不过
pub struct Registry { tools: HashMap<String, Box<dyn Tool>> }
// error: trait `Tool` cannot be made into an object because it uses `Self` ...
```

或更隐蔽的：

```rust
async fn call(&self, args: &str) -> Result<ToolOutput, ToolError> {
    some_async().await;   // 借用 self 里的某个字段做异步状态机
}
// error: future cannot be sent between threads safely (`&` 生命周期不 'static)
```

**症状**： trait object 编译报错，或 `Box::new(tool)` 报 “expected a `'static` lifetime”。

**原因**：trait object 要放进 `HashMap` 就得拥有所有权（所以 `Box<dyn Tool>`），而 `Box<dyn Trait>` 默认要求 `dyn Trait: 'static`——即里面不能藏着任何外部生命周期的引用。`async fn` 同理，返回的 Future 若捕获了 `&self` 的非 `'static` 借用，`Send` 就丢了。

**解法**：

```rust
// 1. 让工具持有自己的数据（Clone 进 Arc 也行），不借用外部
pub struct ReadFileTool { cwd: Arc<PathBuf>, limits: Arc<ReadLimits> }

// 2. trait 加 Send + Sync 边界（已加），impl 里用 Arc<...> 共享可变配置
// 3. 异步状态需要 &mut self 时，改用内部 Mutex，方法签名仍 &self
async fn call(&self, args: &str) -> Result<ToolOutput, ToolError> {
    let mut st = self.state.lock().await;
    // ...
}
```

**通用形式**：**trait object + async = 永远 `'static` + `Send`**。工具需要上下文时，把上下文 `Arc` 进去，而不是借进去。一旦养成这个习惯，`Box<dyn Tool>`、`Box<dyn LlmClient>`、`Box<dyn SandboxPolicy>` 全是同一种手法。

---

## 6.6 用假工具跑完整 turn

沿用第 3 章的 `ScriptedLlm` 测试法（不依赖网络、不花钱、永远稳定）。这次预设模型的回复里“夹带”工具调用，断言引擎能执行并把结果回填：

```rust
// crates/mcx-core/src/session/tests.rs
use super::*;
use mcx_protocol::{Item, Message, Op};
use mcx_tools::{Registry, Tool, ToolOutput, ReadFileTool};
use std::sync::Arc;

/// 假模型：按预设队列依次回复，回复可含 ToolCall 文本
struct ScriptedLlm { replies: Mutex<VecDeque<String>> }

#[async_trait]
impl LlmClient for ScriptedLlm {
    async fn complete(&self, _msgs: &[Message]) -> Result<String, LlmError> {
        Ok(self.replies.lock().unwrap().pop_front().unwrap_or_default())
    }
}

#[tokio::test]
async fn tool_loop_executes_call_and_feeds_result_back() {
    let mut registry = Registry::new();
    registry.register(Box::new(ReadFileTool::new(Arc::new(PathBuf::from("/tmp")))));

    let (op_tx, op_rx) = mpsc::channel(8);
    let (ev_tx, mut ev_rx) = mpsc::channel(64);

    // 第一轮：模型说"我要读 a.rs"；第二轮：模型直接给结论（无 ToolCall → 循环退出）
    let llm = ScriptedLlm::new(vec![
        r#"[TOOL] read_file({"path":"a.rs"})"#.into(),
        "已读完，a.rs 是入口。".into(),
    ]);

    let mut session = Session::new_with_tools(llm, op_rx, ev_tx, registry);
    let handle = tokio::spawn(async move { session.submission_loop().await });

    op_tx.send(Op::UserInput { text: "看下 a.rs".into() }).await.unwrap();

    // 收集直到会话结束（这里靠 Shutdown 驱动；真实测试可用 channel 关闭）
    // ... 断言 history 里出现对应的 ToolResult
    let _ = handle; // 简化：实际测试里 drop op_tx + await
}
```

上面是骨架示意，**完整可运行版本**的关键断言应写成：

```rust
// 断言：history 中出现了 ToolCall("read_file") 及其配对的 ToolResult
let has_call = history.iter().any(|m| m.content.contains("read_file"));
let has_result = history.iter().any(|m| m.content.contains("(read a.rs)"));
assert!(has_call && has_result, "tool loop 必须把结果回填 history");
```

以及第 6.4 那个“加工具只改一处”的验收测试：

```rust
#[test]
fn adding_a_tool_requires_no_change_to_agent_loop() {
    // 若本测试需要修改 run_turn 或 Session 的任何代码，说明抽象失败。
    let mut reg = Registry::new();
    reg.register(Box::new(FakeTool::new("get_weather")));
    reg.register(Box::new(FakeTool::new("send_email")));
    assert_eq!(reg.schemas_json().as_array().unwrap().len(), 2);
}

struct FakeTool { name: String }
#[async_trait]
impl Tool for FakeTool {
    fn name(&self) -> &str { &self.name }
    fn schema(&self) -> Value { json!({"name": self.name, "description": "fake", "parameters": {}}) }
    async fn call(&self, _: &str) -> Result<ToolOutput, ToolError> {
        Ok(ToolOutput { output: "ok".into(), is_error: false })
    }
}
```

**这个测试的含金量不在断言，在它的存在本身**：它把一个架构性质（“扩展点是否唯一”）变成了可判定的检查。三个月后有人改 loop 时，CI 会告诉他“你破坏了开放封闭”。

---

## 6.7 Design Rationale

**Q：为什么用 trait object 而不是 enum 把所有工具列出来？**

因为工具集合是**开放的**。内置工具在第 7、8、9 章陆续加入，MCP 工具在第 18 章运行时注入——后者在编译期根本不存在。enum 要求穷举，每次扩展要改 N 个 `match`；trait object 把扩展收敛到一个点（Registry）。**性能损失是一次虚表调用，换来的是“不修改引擎就能接进任意来源的工具”**——对 agent 系统这是划算的交易。

**Q：为什么 `arguments` 是 JSON 字符串，而不是每个工具一个强类型参数 struct？**

强类型更 Rust、更省解析，但**强类型只在编译期已知的工具上成立**。MCP 工具的参数由远端 schema 描述，运行前没有 Rust 类型。统一成字符串后，内置工具（`ReadFileTool`）自己 `serde_json::from_str` 成私有结构体，远程工具直接把 JSON 透传给远端——**两条路径汇到同一个 `call(&str)`**。这是“统一接口”胜过“静态完美类型”的少数场景之一。

**Q：为什么工具失败返回 `Ok(ToolOutput { is_error: true })` 而不是 `Err(ToolError)`？**

因为“文件不存在”“命令返回非零”“参数越界”对模型来说是**预期内的、可恢复的信息**，它看到后通常会换个参数重试。若一律 `Err` 让引擎中断 turn，模型就失去了自我纠正的机会。**引擎崩溃和工具业务失败是两种语义，必须用两个通道表达。** 真正的 `Err` 只留给“引擎本身出问题”（解析崩了、panic 级错误），那时才该中断。

---

## AI 软件工程原理 #6

> **工具设计的第一原则是让模型容易生成对，不是让人容易实现。**

**评判工具好坏的标准，是模型一次调对的概率，不是实现代码行数。** 这一点反直觉，因为程序员本能地优化“我写起来方不方便”。

看三个对比：

| 设计取向 | 参数设计 | 模型一次调对率 | 后果 |
|---|---|---|---|
| 面向实现 | 复用既有内部函数签名，`offset: usize` 默认 0 | 低（模型常忘、常越界） | 反复纠错、多烧 3–5 轮 |
| 面向模型 | 参数带示例值、合法范围、默认值写进 description | 高 | 一轮到位 |
| 面向模型 | 把“调用前置条件”写进 description | 高 | 减少非法调用 |

**第二个推论：工具的描述是提示词的一部分。** 它和系统提示词、用户消息一起构成模型的输入预算。一份 schema 里“描述 80 字符 vs 800 字符”的差异，换来的是调用准确率，代价只是每次请求的几百 token——**这是全书性价比最高的 token 投入**，因为它省下的是整轮往返。

**第三个推论：工具数量要克制。** 每多一个工具，模型就要多一份“什么时候该用它”的判断，上下文也被 schema 清单占满。第 9 章会量化这个取舍。**不是工具越多 agent 越强，是“恰好够用且描述精准”的那组工具最强。**

这与第 5 章的“历史 append-only”、第 4 章的“宽容未知字段”一脉相承：**把不确定性留在模型能处理的地方，把确定性用类型和测试锁死。** 下一章（shell）就是这个原则的第一场硬仗——因为 shell 是副作用最猛的工具。

---

## Rust 修炼小结

| 概念 | 本章用法 | 后面在哪用到 |
|---|---|---|
| `trait` + `impl Trait` | `Tool` 抽象，统一所有工具接口 | 第 18 章 MCP 远程工具同款 |
| `Box<dyn Trait>` | Registry 存异质工具 | `LlmClient`、`SandboxPolicy` |
| `async_trait` | 异步 trait 方法 | 全书 |
| `'static` + `Send` 约束 | trait object 能跨线程 | 第 7 章 `CancellationToken` |
| `Arc<...>` 注入上下文 | 工具持有 cwd/配置 | 第 9 章多工具共享 |
| `thiserror` + `ToolError` | 区分业务失败与引擎崩溃 | 全书错误类型 |

---

## 章末验收

- [ ] `Registry::register` 后，模型能调用该工具，且 `run_turn` 代码零修改
- [ ] 有测试用 `ScriptedLlm` + 假工具跑完整 turn，不依赖网络
- [ ] 工具返回 `is_error: true` 时，turn 不中断，结果回填 history
- [ ] 未知工具名返回 `is_error: true`，不 panic
- [ ] 单 turn 工具轮次有上限（`max_tool_rounds`），死循环会被截断

---

## 读者挑战

1. 把 `max_tool_rounds` 设为 0，会发生什么？**这是“护栏缺失导致失控”的最直观演示。**
2. 试写一个工具，让模型在两轮内必然陷入重复调用（提示：返回值里不告诉它“已经做完了”）。**这揭示了工具输出设计的另一半——不仅要给事实，还要给“终止信号”。**
3. 若要求“工具调用必须按声明顺序串行执行”，`for call in calls` 的并行改造会破坏什么？**本书不给答案，提示：看 `CancellationToken`。**

---


# 第二部分　让它真能干活：三个核心工具（第 7–9 章）

> 第 6 章给了 agent 一双手，这一部分要给这双手配上三件真正能改世界的工具：`shell`（执行命令）、`apply_patch`（改写文件）、`read_file`/`list_dir`/`view_image`（感知环境）。
>
> 这三章有一个共同主题：**工具的输入格式要贴着模型的擅长区设计。** 模型会复述、不会计数；能匹配、不擅长精确构造。Codex 自创 `apply_patch` DSL 而不用 unified diff，就是这个原则最锋利的一次应用。

---

# 第 7 章　shell：进程、超时与进程树

**本章任务**：实现 `shell` 工具——装上它，agent 才算真正活了。但它的杀伤力也最大：**一条 `rm -rf`、一个跑飞的子进程、一份 100MB 的 stdout，都足以拖垮整个会话。** 所以这一章一半在写“怎么跑命令”，一半在写“怎么让命令永远跑不出界”。

上一章我们把第 3 章的占位 `CancellationToken` 接进了取消语义；这一章它终于要有听众了。

---

## 7.1 先看一个会出事的版本

直觉写法很直白：

```rust
// 反例：能跑，但生产环境是灾难
async fn run_cmd(cmd: &str) -> String {
    let out = tokio::process::Command::new("sh")
        .arg("-c").arg(cmd)
        .output().await.unwrap();          // ← 三个坑埋在这里
    String::from_utf8_lossy(&out.stdout).into_owned()
}
```

三件事会炸：

1. **没有超时**。`while true; do :; done` 让你的 agent 永远不返回。
2. **stdout/stderr 无上限**。`cat` 一个 100MB 文件，整个输出塞进 `ToolResult`，再塞进 history，再随下一次请求发回模型——**上下文被一次撑爆，账单和延迟同步起飞。**
3. **只杀父进程，不杀进程树**。`sh -c "sleep 100 &"` 里 `sleep` 是 `sh` 的子进程；你 kill 掉 `sh`，`sleep` 被 init 收养，继续跑——**孤儿进程积累，下次 `lsof` 一看满屏僵尸。**

再加两个：`Op::Interrupt`（Ctrl+C）现在**终于要产生效果了**——第 3 章的 `cancel` 字段一直没人监听，本章让它在工具执行期间被轮询；以及环境变量泄露 `HOME`、`SSH_AUTH_SOCK` 等敏感信息。

**结论：shell 工具 = 执行 + 四个边界（时间、输出、进程、环境）。** 四个边界少一个，agent 就可能在某个雨天跑飞。

---

## 7.2 设计：把边界全部显式化

```rust
// crates/mcx-tools/src/shell.rs
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

pub struct ShellTool {
    pub cwd: PathBuf,
    pub env_allowlist: Arc<Vec<String>>,   // 环境变量白名单
    pub stdout_cap: usize,                  // 单流输出上限（字节）
    pub wall_timeout: Duration,             // 总超时（含宽限期）
    pub graceful: Duration,                 // SIGTERM 宽限期
}
```

四个字段就是四道边界。注意它们**全是配置，不是硬编码**——第 10 章的安全审批会把其中一部分暴露给用户策略，这里先把接口留成可注入。

工具 schema 里的 description 同样服务于原理 #6：

```rust
fn schema(&self) -> Value {
    json!({
        "name": "shell",
        "description":
            "在受控工作目录中执行一条 shell 命令，返回其 stdout（到达上限即截断并标注 [truncated]）。\
             注意：教学版只捕获 stdout，stderr 直接透传给终端，不合并返回。\
             有执行超时与输出大小上限；超时被强制终止。\
             不要执行破坏性命令（rm -rf、git reset --hard 等）除非用户明确要求。",
        "parameters": {
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "要执行的完整命令，如 cargo build" },
                "timeout_secs": { "type": "integer", "description": "可选超时秒数，默认 120" }
            },
            "required": ["command"]
        }
    })
}
```

**“不要执行破坏性命令”这句不是装饰。** 模型没有“生产环境”的概念，它只看到 `command` 参数。把风险边界写进描述，是成本最低的第一道防线——当然，它不是安全机制，真正的拦截在第 10、11 章的沙箱与审批。

---

## 7.3 核心执行：超时三段式 + 进程树清理

超时不能“到点就 SIGKILL”。很多程序在 SIGTERM 后需要清理：写盘、关连接、删临时文件。**直接 SIGKILL 会留下半成品文件，而半成品文件是 agent 后续误判的根源**（它读到一半的内容，以为是真相）。

所以正确的时序是 **terminate → 宽限期 → hard kill**：

```
t=0        启动进程（并记录 pid + 所有子孙 pid）
t=timeout   SIGTERM（terminate）整棵进程树
t=timeout+grace  若仍在跑 → SIGKILL 整棵进程树
```

实现（Unix 为主，Windows 用 `job object` 同理，本章用 `#[cfg(unix)]` 隔离）：

```rust
use tokio::process::{Child, Command};
use tokio::sync::oneshot;

pub async fn run_shell(
    tool: &ShellTool, args: &ShellArgs, cancel: &CancellationToken,
) -> Result<ToolOutput, ToolError> {
    let timeout = args.timeout_secs
        .map(Duration::from_secs)
        .unwrap_or(tool.wall_timeout);

    // 启动：只把 stdout 用 pipe 捕获；stderr 直接继承到终端，
    // 避免“pipe 了却没人读”把子进程卡在写满的 stderr 管道上（教学取舍，见正文）
    let mut child = Command::new("sh")
        .arg("-c").arg(&args.command)
        .current_dir(&tool.cwd)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .kill_on_drop(true)                 // ← 关键：Child 被 drop 时自动 kill
        .spawn()
        .map_err(|e| ToolError::Internal(format!("spawn 失败: {e}")))?;

    let pid = child.id().expect("spawn 后 pid 必存在");

    // 三段式：用 select! 同时监听"正常退出 / 超时 / 取消"
    let outcome = tokio::select! {
        // 分支 A：进程自己跑完了
        exit = wait_with_capture(&mut child, tool.stdout_cap) => {
            Ok(exit_to_output(exit?))
        }
        // 分支 B：超时到点 → terminate，等宽限期，再 hard kill
        _ = tokio::time::sleep(timeout) => {
            terminate_tree(pid);
            match tokio::time::timeout(tool.graceful, child.wait()).await {
                Ok(Ok(status)) => Ok(ToolOutput {
                    output: format!("[超时] 命令被终止（已优雅退出，status={status}）"),
                    is_error: true,
                }),
                _ => {
                    hard_kill_tree(pid);
                    let _ = child.wait().await;   // 回收，避免僵尸
                    Ok(ToolOutput {
                        output: "[超时] 命令被强制终止（SIGKILL）".into(),
                        is_error: true,
                    })
                }
            }
        }
        // 分支 C：用户中断（Op::Interrupt → cancel.cancel()）
        _ = cancel.cancelled() => {
            terminate_tree(pid);
            let _ = child.wait().await;
            Ok(ToolOutput { output: "[已取消]".into(), is_error: true })
        }
    };

    // 收尾：确保没有任何子孙残留（kill_on_drop 是第二道防线）
    reap_descendants(pid);
    outcome
}
```

`wait_with_capture` 在读 pipe 时**边读边计数**，一旦超过 `stdout_cap` 就丢弃后续字节并在输出末尾标注 `[truncated]`——这是输出上限的实现要点：不是事后截断字符串，而是**从源头就不读进内存**。stderr 为什么不一并捕获？教学取舍：子进程的错误主要靠退出码与 stdout 里的报错来判断，把 stderr 原样交给终端最简单，也绕开“pipe 了却没人读”的背压复杂度（子进程向写满的 stderr 管道继续写会被阻塞）；生产实现若要错误也进 UI，再做双管道同时读，并在到达 cap 后继续排水直到子进程退出。

```rust
async fn wait_with_capture(child: &mut Child, cap: usize)
    -> Result<Captured, ToolError>
{
    let mut stdout = child.stdout.take().unwrap();
    let mut buf = Vec::new();
    let mut truncated = false;

    use tokio::io::AsyncReadExt;
    let mut chunk = vec![0u8; 8 * 1024];
    loop {
        let n = stdout.read(&mut chunk).await?;
        if n == 0 { break; }
        if buf.len() + n > cap { truncated = true; break; }  // ← 到顶即停
        buf.extend_from_slice(&chunk[..n]);
    }
    let status = child.wait().await?;
    Ok(Captured { bytes: buf, truncated, status })
}
```

**`kill_on_drop(true)` 是这段代码里最重要的五个字符。** 它让 `Child` 在离开作用域时自动被 kill——即使上面某个分支因 early return 或 panic 没走到显式清理，操作系统也会回收进程。**把“必须清理”交给 RAII，而不是交给程序员的记性。**

---

## 避坑专栏 #8：`Child` 不 `.wait()` 会留僵尸

**错误写法**：

```rust
let mut child = Command::new("sh").arg("-c").arg(cmd).spawn()?;
// 忘了 child.wait().await;
drop(child);   // kill_on_drop 会 kill，但若未启用，进程变僵尸
```

**症状**：命令早结束了，`ps` 里却还躺着 `<defunct>`。积累多了，进程表耗尽、PID 用尽。

**原因**：Unix 里父进程必须 `waitpid` 回收子进程的退出状态，否则它一直停在 zombie 态。`kill_on_drop` 只保证 kill，不保证回收；显式 `child.wait()` 才回收。

**解法**（通用形式）：**每次 `spawn` 都必须有一条路径走到 `child.wait().await`**。本章的三段式在每个分支末尾都写了 `let _ = child.wait().await;`。更稳妥是把 `Child` 包进一个 RAII 守卫：

```rust
struct ChildGuard(Child);
impl Drop for ChildGuard {
    fn drop(&mut self) { /* 同步 kill + 非阻塞回收不可行，tokio 推荐 await wait */ }
}
```

**tokio 的 Child 没有同步 Drop 回收能力**，所以最可靠的仍是“在 async 流程里保证 `wait`”。判断口诀：**spawn/wait 是配对的，像 lock/unlock 一样不能只写一半。**

---

## 7.4 杀死进程树，而不是单进程

`sh -c "gcc ..."` 里 `gcc` 是 `sh` 的子进程；`make -j` 会再 fork 出 N 个。只 kill `sh`（pid），`gcc`/`make` 被 PID 1（或 launchd）收养，继续跑——**孤儿进程积累，占用端口、锁文件、GPU 显存**。

Unix 的做法是按**进程组（process group）** 发信号。shell 会把整条管道放进同一个 pgid，我们只要对这个 pgid 发信号，整棵子树一起倒：

```rust
#[cfg(unix)]
fn terminate_tree(pid: u32) {
    // 负数 pid = 对进程组发信号。SIGTERM 允许清理。
    unsafe { libc::kill(-(pid as i32), libc::SIGTERM); }
}
#[cfg(unix)]
fn hard_kill_tree(pid: u32) {
    unsafe { libc::kill(-(pid as i32), libc::SIGKILL); }
}

#[cfg(windows)]
fn terminate_tree(pid: u32) {
    // Windows：用 Win32 Job Object 把子进程纳入同一 job，终止 job 即终止全树。
    todo!("mcx-sandbox 会在第 11 章提供跨平台实现");
}
```

**前提**：子进程必须和父在同一个进程组。我们的 `sh -c` 满足；但如果工具启动的是自己 `fork` 出的 daemon（它会 `setsid`），进程组法失效——这时要靠第 11 章的 cgroup / bwrap / Seatbelt 做**层级化沙箱**，从 cgroup 根一刀砍下去。

> **通用形式**：**能杀死“一棵树”的只有“包裹整棵树的那个容器”**——进程组、cgroup、job object、容器。单 PID 只能杀一个点。所以 shell 工具的完整性依赖第 11 章的沙箱，**本章先解决 90% 的常见情况，剩下的交给层级隔离。**

---

## 7.5 环境变量白名单

默认环境里装着 `HOME`、`SSH_AUTH_SOCK`、`AWS_PROFILE`、`GITHUB_TOKEN`……把整个 `env` 透传给子进程，等于把宿主机的凭证送给模型生成的命令。正确做法是**白名单**：

```rust
fn build_env(allow: &[String]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (k, v) in std::env::vars() {
        if allow.iter().any(|a| a == &k) {
            out.push((k, v));
        }
    }
    // 必须的基础变量，即使不在白名单也注入
    out.push(("PATH".into(), std::env::var("PATH").unwrap_or_default()));
    out.push(("LANG".into(), "C.UTF-8".into()));
    out
}
```

`shell_environment_policy.include_only` 这个概念来自 Codex 的配置。**默认只允许 `PATH`、`LANG`、显式声明的几个**——第 10 章会让用户审批“是否允许访问 `HOME/.gitconfig`”。

> **环境变量是 agent 最容易被忽略的攻击面。** 模型不需要“故意作恶”，它只需要跑一条 `env | curl`，你的凭证就出去了。白名单是默认安全的唯一形态；黑名单永远补不完。

---

## 7.6 接进 Session：让 CancellationToken 生效

第 3 章的 `cancel` 一直没人听。现在 `execute_call` 在跑 shell 时要把它传进去：

```rust
// Session::execute_call 改造（节选）
async fn execute_call(&self, call: &ToolCall) -> ToolOutput {
    match self.tools.get(&call.name) {
        Some(tool) => {
            // shell 需要取消信号；其他工具暂不需要
            if call.name == "shell" {
                // 用 child_token 让单次调用可被中断，不影响整个 Session
                let child_token = self.cancel.child_token();
                return run_shell_with_token(tool, &call.arguments, &child_token).await;
            }
            // ...普通调用
        }
        // ...
    }
}
```

**关键：`self.cancel.child_token()`。** `CancellationToken` 支持层级——父取消则所有子被取消，但子取消不影响父。这样一次 `Op::Interrupt` 只中断当前这条命令，会话还能继续。**这就是第 3 章“先占位”的价值：今天接进来，改动是局部的。**

用户按 Ctrl+C → CLI 发 `Op::Interrupt` → `submission_loop` 调 `self.cancel.cancel()` → shell 的 `cancel.cancelled()` 分支触发 → `terminate_tree` → 返回 `[已取消]` → 模型看到结果，决定下一步。**全链路零阻塞，UI 永远响应。**

---

## 7.7 测试：不依赖网络，但要真跑进程

```rust
#[tokio::test]
async fn timeout_kills_the_command() {
    let tool = ShellTool::with_limits(Duration::from_secs(2), Duration::from_millis(300), 4 * 1024);
    let out = run_shell(&tool, &ShellArgs { command: "sleep 30".into(), timeout_secs: Some(1) },
                        &CancellationToken::new()).await.unwrap();
    assert!(out.is_error);
    assert!(out.output.contains("终止"));
    // 验证无孤儿：给宽限期后再查（伪代码，实际用 libc::kill 探活）
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(!pid_alive(/* sleep 的 pid */));
}

#[tokio::test]
async fn large_output_is_truncated() {
    let tool = ShellTool::with_limits(Duration::from_secs(5), Duration::from_millis(100), 1 * 1024);
    let out = run_shell(&tool, &ShellArgs { command: "yes | head -c 1000000".into(), timeout_secs: None },
                        &CancellationToken::new()).await.unwrap();
    assert!(out.output.contains("[truncated]"));
    assert!(out.output.len() <= 1 * 1024 + 200);   // 上限未被突破
}

#[tokio::test]
async fn env_allowlist_drops_secrets() {
    std::env::set_var("SUPER_SECRET_X", "leaked-value");
    let tool = ShellTool::allowlist(vec!["PATH".into()]);
    let out = run_shell(&tool, &ShellArgs { command: "env".into(), timeout_secs: None },
                        &CancellationToken::new()).await.unwrap();
    assert!(!out.output.contains("SUPER_SECRET_X"));
}
```

三个测试分别对应三道边界：**超时、输出上限、环境隔离**。它们都跑真实进程（用 `sleep`/`yes`/`env`，不依赖网络、不花钱），几毫秒到一两秒完成。

---

## 7.8 Design Rationale

**Q：为什么超时要有宽限期，直接 SIGKILL 不行吗？**

直接 SIGKILL 确实简单，代价是**半成品文件**。想想 `cargo build` 被砍在写 `target/` 中间：下一次调用看到损坏的 `*.rlib`，报错千奇百怪，模型据此做出错误判断。**agent 的可靠性不只取决于“命令成不成功”，更取决于“文件系统是否始终自洽”**——后者被宽限期保护了。300ms 的代价换来的是可复现的工作树。

**Q：为什么用 `select!` 同时监听退出、超时、取消，而不是一个循环轮询？**

`select!` 是**事件驱动**——哪个先到就走哪个分支，零忙等、微秒级响应。`loop { 检查状态; sleep(100ms) }` 的轮询版本会漏掉“刚好在两个 tick 之间发生”的超时，且延迟上限等于 tick 间隔。**取消体验的好坏（Ctrl+C 后多久停下）直接取决于这里的响应速度。**

**Q：为什么 `kill_on_drop` 还不够，要手动 `terminate_tree`？**

`kill_on_drop` 只杀**直接子进程**，不管它的子孙；且它是 best-effort（drop 时机不确定）。我们的 `terminate_tree` 是按进程组整棵砍，**确定性高、作用域清晰**。两者是主防 + 兜底的关系，不是二选一。

---

## AI 软件工程原理 #7

> **每个副作用都要有边界。**

超时是时间的边界、输出上限是空间的边界、重试预算是次数的边界、迭代上限是深度的边界。**没有边界的 agent 一定会跑飞**——不是“可能”，是“必然”，区别只在是第 3 次还是第 300 次调用。

把本章的四道边界和第 6 章的 `max_tool_rounds` 摆一起看：

| 边界 | 本章实现 | 失控的后果 |
|---|---|---|
| 时间 | `wall_timeout` + 三段式 | 命令永远不返回，会话卡死 |
| 空间 | `stdout_cap` 源头截断 | 上下文/内存爆掉 |
| 进程 | 进程树清理 | 孤儿进程积累、端口/锁泄漏 |
| 环境 | 变量白名单 | 凭证外泄 |
| 深度 | `max_tool_rounds` | 工具死循环烧 token |

**“边界”是 AI 软件工程里最接近传统工程安全网的东西**——传统程序靠类型保证不越界，agent 靠**显式的 budget** 保证不失控。而且边界必须是**默认存在、显式放宽**的：把上限写成配置字段、给 `timeout_secs` 一个默认值，就是不信任调用方的设计。

**反面教训**：把边界写成“信任模型会自己停”= 没有边界。模型没有“够了”的概念，它只会按概率继续。**凡是模型能触发的副作用，都要有一个不依赖模型的、确定性的停止条件。**

下一章的 `apply_patch` 同样服从这条原理：原子落地、路径白名单、最大 patch 数——**都是边界**。

---

## Rust 修炼小结

| 概念 | 本章用法 | 后面在哪用到 |
|---|---|---|
| `tokio::process::Command` | 异步 spawn + pipe | 第 11 章沙箱执行 |
| `select!` | 退出/超时/取消三选一 | 第 19 章事件多路复用 |
| `CancellationToken::child_token` | 层级取消 | 第 12 章并发任务 |
| `kill_on_drop(true)` | RAII 清理兜底 | 全书 |
| `#[cfg(unix)]` | 平台差异隔离 | `mcx-sandbox` 跨平台 |
| `libc::kill(-pid, SIGTERM)` | 进程组整树终止 | 第 11 章 cgroup |

---

## 章末验收

- [ ] `sleep 100` 能在 1 秒内被中断，且 `ps` 查不到残留 `sleep` 进程
- [ ] `yes | head -c 1000000` 的输出被截断在 `stdout_cap` 内，不撑爆上下文
- [ ] 白名单外的环境变量不出现在 `env` 输出里
- [ ] `Op::Interrupt` 触发 `CancellationToken`，正在跑的 shell 返回 `[已取消]`
- [ ] `cargo test` 全部通过，不依赖网络

---

## 读者挑战

1. 把 `graceful` 设为 0，跑一个会写文件的命令（如 `dd`），观察产物是否损坏。**这演示了“为什么宽限期不是可有可无”。**
2. `sh -c "setsid sleep 100"` 会绕过进程组清理吗？**本书不给答案，提示：看 `setsid` 的语义，并预习第 11 章 cgroup。**
3. 若允许工具并发跑（第 6 章的“并行改造”），`kill_on_drop` 的时机会和 `select!` 冲突吗？**提示：`ChildGuard` + 共享 PID 集合。**

---


# 第 8 章　apply_patch：为什么不用 unified diff

**本章任务**：实现文件修改工具，让 agent 从此能真改代码。这是本书最具争议的一个设计选择——**不用标准的 unified diff，而用 Codex 自创的 patch DSL**。本章要把这个选择讲透，因为这个选择背后的思维方式，比代码本身更重要。

---

## 8.1 先承认一个反直觉

unified diff 是行业标准。`patch(1)`、Git、`difflib`、GitHub PR 都在用它。一个工具系统第一反应肯定是“直接包 `diff` 嘛”。

**但 Codex 没这么做。** 它发明了这样一套 DSL：

```
*** Begin Patch
*** Update File: src/lib.rs
@@ src/lib.rs
 class Foo {
+    pub fn new() -> Self { ... }
 }
*** End Patch
```

**理由只有一句，但足以推翻整个方案：行号是模型的致命弱点。**

模型不擅长“计数”。它能把一段代码**逐字复述**得几乎完美（它就是在海量代码上训练的，复述是本能），但它数不准“要改的那段在第 47 行还是第 48 行”。而 unified diff 的全部语义都押在行号上：

```
@@ -46,7 +46,8 @@
 context line 1
 context line 2
-old line
+new line
 context line 4
```

**一旦行号偏移 1，整个 hunk 失效，且往往以最糟糕的方式失效——静默地应用到错误的位置。** Git 会说 “patch does not apply”，还算温和；更糟的是 `fuzz` 成功但改错了地方，留下一个语义正确、位置错误的改动——**这是 review 几乎看不出来的那类 bug**。

模型的失败模式是确定的：**复述强、计数弱**。那工具接口就该只要求复述，不要求计数。这就是 DSL 的核心动机。

---

## 8.2 用“上下文匹配”代替“行号寻址”

DSL 的思想：与其说“在第 47 行删掉 X”，不如说“找到这一段**长得像 X 的上下文**，把它的中间改成 Y”。模型只需要能复述附近几行代码——**这是它最擅长的事**——定位交给精确字符串匹配。

四种指令，覆盖所有改动：

| 指令 | 语义 | 模型要提供 |
|---|---|---|
| `*** Update File: <path>` | 修改已有文件 | 上下文块（search） + 替换块（replace） |
| `*** Add File: <path>` | 新建文件 | 完整内容 |
| `*** Delete File: <path>` | 删除文件 | （无需内容） |
| `*** Move File: <from> -> <to>` | 移动/重命名 | 目标路径 |

一个真实例子。模型想给 `src/lib.rs` 的 `Foo` 加个方法，它生成：

```
*** Begin Patch
*** Update File: src/lib.rs
@@
 impl Foo {
     pub fn existing(&self) {}
+
+    pub fn new() -> Self {
+        Self {}
+    }
 }
*** End Patch
```

**注意 `existing` 那行是“锚点”**——模型复述一段保证存在于文件里的代码，工具靠精确匹配找到插入点。`+` 行是新增。模型**不需要知道这个 `impl` 块从第几行开始**。

```rust
// crates/mcx-tools/src/apply_patch.rs
use serde_json::Value;

pub struct ApplyPatchTool {
    pub root: PathBuf,
    pub max_bytes: usize,         // 单文件写入上限
    pub max_files: usize,         // 单次 patch 最多改动文件数
}

impl Tool for ApplyPatchTool {
    fn name(&self) -> &str { "apply_patch" }
    fn schema(&self) -> Value {
        json!({
            "name": "apply_patch",
            "description":
                "按 patch DSL 修改文件。用上下文匹配定位（复述附近代码作为锚点），不要依赖行号。\
                 一次可含多个 Update/Add/Delete。失败时会返回精确错误信息供你纠正。",
            "parameters": {
                "type": "object",
                "properties": {
                    "patch": { "type": "string", "description": "完整的 patch DSL 文本" }
                },
                "required": ["patch"]
            }
        })
    }
    async fn call(&self, arguments: &str) -> Result<ToolOutput, ToolError> { /* 见 8.5 */ }
}
```

---

## 8.3 解析器：状态机，不是正则表达式

DSL 看着像文本，但**它是结构化语言**——嵌套、转义、跨行。用正则解析必然在某个边界崩盘。**正确做法：手写状态机，逐行转移。**

```rust
#[derive(Debug, PartialEq)]
enum Stmt {
    BeginPatch,
    UpdateFile(PathBuf),
    AddFile(PathBuf),
    DeleteFile(PathBuf),
    MoveFile(PathBuf, PathBuf),
    Hunk(Vec<HunkLine>),
    EndPatch,
}

#[derive(Debug, PartialEq)]
enum HunkLine { Context(String), Add(String), Remove(String) }

/// 解析器状态：在哪一"层"
enum State {
    Top,                       // 等 Begin Patch
    InPatch,                   // 等 Update/Add/Delete 或 @@
    InHunk,                    // 收集 + - 行，直到空行或新指令
}

pub fn parse_patch(input: &str) -> Result<Vec<Stmt>, PatchParseError> {
    let mut stmts = Vec::new();
    let mut state = State::Top;
    let mut hunk = Vec::new();

    for (lineno, raw) in input.lines().enumerate() {
        let line = raw.trim_end();   // 保留左侧空白（缩进有意义）

        match state {
            State::Top => {
                if line == "*** Begin Patch" {
                    stmts.push(Stmt::BeginPatch);
                    state = State::InPatch;
                } else if !line.is_empty() {
                    return err(lineno, "期望 '*** Begin Patch'");
                }
            }
            State::InPatch => {
                if let Some(rest) = strip_prefix(line, "*** Update File:") {
                    flush_hunk(&mut hunk, &mut stmts);
                    stmts.push(Stmt::UpdateFile(path(rest)?));
                    state = State::InHunk;          // 紧接一个 @@ 块
                } else if let Some(rest) = strip_prefix(line, "*** Add File:") {
                    flush_hunk(&mut hunk, &mut stmts);
                    stmts.push(Stmt::AddFile(path(rest)?));
                    state = State::InHunk;          // Add 的内容也是 hunk 形式
                } else if let Some(rest) = strip_prefix(line, "*** Delete File:") {
                    flush_hunk(&mut hunk, &mut stmts);
                    stmts.push(Stmt::DeleteFile(path(rest)?));
                    // Delete 后面不跟 hunk
                } else if line == "*** End Patch" {
                    flush_hunk(&mut hunk, &mut stmts);
                    stmts.push(Stmt::EndPatch);
                    state = State::Top;
                } else if line == "@@" {
                    state = State::InHunk;          // 当前文件的 hunk 开始
                } else {
                    return err(lineno, "未知指令");
                }
            }
            State::InHunk => {
                if line.is_empty() {
                    flush_hunk(&mut hunk, &mut stmts);
                    state = State::InPatch;         // 空行 = 这个 hunk 结束
                } else if let Some(rest) = strip_prefix(line, "*** ") {
                    flush_hunk(&mut hunk, &mut stmts);
                    // 新的顶层指令：回退到 InPatch 处理（见下方说明）
                    state = State::InPatch;
                    // ... 用 peek 或更清晰的两步解析更好；此处为教学简化
                } else if let Some(s) = strip_prefix(line, "-") {
                    hunk.push(HunkLine::Remove(s.into()));
                } else if let Some(s) = strip_prefix(line, "+") {
                    hunk.push(HunkLine::Add(s.into()));
                } else {
                    hunk.push(HunkLine::Context(line.into()));
                }
            }
        }
    }
    flush_hunk(&mut hunk, &mut stmts);
    Ok(stmts)
}

fn flush_hunk(hunk: &mut Vec<HunkLine>, stmts: &mut Vec<Stmt>) {
    if !hunk.is_empty() {
        stmts.push(Stmt::Hunk(std::mem::take(hunk)));
    }
}
```

**状态机的价值**：每个状态只关心“我现在能合法看到什么”。错误报告天然带行号（`lineno`）——**这是关键，见 8.6**：模型纠错的输入就来自这里。

> **教学提示**：上面是教学版，生产实现建议把 `InHunk` 里“遇到 `***` 新指令”的处理改成**前瞻（peek）**——保留当前行，回退到 `InPatch` 重新分发。用 `peekable()` 迭代器一行搞定。

---

## 8.4 校验：路径穿越、规模、语义

解析通过后，落地前还要过三道关卡。**任何一道失败都不动磁盘**——这是“原子”的前提。

**① 路径穿越防护**。模型可能生成 `*** Update File: ../../etc/passwd` 或 `/etc/passwd`。必须在 `root` 内做规范化：

```rust
fn resolve_inside_root(root: &Path, raw: &str) -> Result<PathBuf, PatchError> {
    if raw.is_empty() { return Err(PatchError::EmptyPath); }
    let abs = if Path::new(raw).is_absolute() {
        PathBuf::from(raw)
    } else {
        root.join(raw)
    };
    // canonicalize 会解析 ".."；若文件尚不存在，先只规范化 components
    let cleaned = normalize_components(&abs)?;   // 手动处理 ".."，不越 root
    if !cleaned.starts_with(root) {
        return Err(PatchError::PathTraversal { attempted: raw.into() });
    }
    Ok(cleaned)
}

/// 逐段处理 ".."，不允许越过 root。不访问文件系统（新建文件尚不存在）。
fn normalize_components(path: &Path) -> Result<PathBuf, PatchError> {
    let mut out = PathBuf::new();
    for comp in path.components() {
        use std::path::Component::*;
        match comp {
            RootDir => out.push("/"),
            CurDir => {}
            ParentDir => { if !out.pop() { return Err(PatchError::PathTraversal { .. }) } }
            Normal(s) => out.push(s),
        }
    }
    Ok(out)
}
```

**② 规模上限**。`max_files`（防一次改 500 个文件）、`max_bytes`（防写出 1GB）、hunk 数量上限。**每道都是原理 #7 的边界。**

**③ 语义校验**。Update 的每个 hunk 必须有 Context 或 Remove（纯 Add 的 hunk 没有锚点，无法定位——这是 DSL 的规则，要在校验里说"No）。Delete 的目标必须存在。Move 的源必须存在、目标不存在。

---

## 8.5 原子落地：先写临时文件，再 rename

**最怕的场景**：patch 改了 5 个文件，第 3 个写一半时校验失败。磁盘上留下**部分改动**——比什么都不改糟得多，因为下次重试连上下文都对不上了。

解法：**两阶段提交**。

```rust
pub async fn apply_patch(tool: &ApplyPatchTool, patch_text: &str)
    -> Result<ToolOutput, ToolError>
{
    let stmts = parse_patch(patch_text)?;
    validate(&stmts, tool)?;                      // ① 全量校验，不动磁盘

    // ② 对每个要改/要建的文件：写临时文件到同一目录
    let mut staged: Vec<(PathBuf, Option<PathBuf>)> = Vec::new(); // (final, tmp)
    for stmt in &stmts {
        if let Stmt::UpdateFile(p) | Stmt::AddFile(p) = stmt {
            let final_path = p.clone();
            let tmp = final_path.with_extension(
                final_path.extension().unwrap_or_default().to_str().unwrap_or("")
            ).with_file_name(format!("{}.tmp.{}",
                final_path.file_name().unwrap_or_default().to_str().unwrap_or("x"),
                std::process::id()));
            write_new_content(&tmp, &final_path, stmt, tool).await?;
            staged.push((final_path, Some(tmp)));
        }
    }

    // ③ 全绿后，一次性 rename 进正式位置（rename 是原子的）
    for (final_path, tmp) in &staged {
        if let Some(tmp) = tmp {
            tokio::fs::rename(tmp, final_path).await?;
        }
    }
    // Delete / Move 也在此时生效（Move = rename）

    Ok(ToolOutput {
        output: format!("已应用：{} 个文件改动", staged.len()),
        is_error: false,
    })
}
```

**`write_new_content` 是“在内存里算出完整新文件，一次性落盘”**——对于 Update，做法是：把原文件按行读入，用上下文匹配找到 hunk 位置，应用 `-`/`+` 生成新内容，写到 tmp。**rename 在同一文件系统内是原子操作**，所以“半个文件”对外不可见。

**被删除文件的原始内容要保留**——`git diff` 看得到，审计也看得到。这是契约里 `Item::ToolResult` 可以承载的（输出里附上 diff 摘要）。

> **“先 tmp 再 rename”是本书最重要的文件系统技巧之一**，第 16 章的 JSONL 落盘同样用它。**任何非 append 的写操作都该是原子的**，否则崩溃窗口 = 数据损坏概率。

---

## 8.6 失败时给模型足够信息自我纠正

**这是 DSL 相对 unified diff 的第二大优势**：错误信息是**结构化的、定位精确的**，可以直接喂回给模型。

```
[行 12] Update File "src/lib.rs"：未找到上下文匹配
  期望找到：
     pub fn existing(&self) {}
+
+    pub fn new() -> Self {
 实际文件在该位置不匹配。请重新读取文件确认最新内容后重试。
```

模型拿到这段，会立刻 `read_file` 重新确认，再生成正确的 patch。**这个闭环成立的前提是：错误信息足够具体，让模型能定位到“是哪一段复述错了”。** 抽象地报 “patch failed” 等于没报。

```rust
#[derive(Debug, thiserror::Error)]
pub enum PatchError {
    #[error("路径为空，拒绝处理")]
    EmptyPath,
    #[error("[行 {line}] 解析错误: {msg}")]
    Parse { line: usize, msg: String },
    #[error("路径穿越被拒绝: {attempted}")]
    PathTraversal { attempted: String },
    #[error("[行 {line}] Update File {path}: 未找到上下文匹配\n  期望:\n{context}")]
    ContextNotFound { line: usize, path: String, context: String },
    #[error("hunk 缺少上下文（纯 + 行无法定位）")]
    HunkHasNoAnchor,
    #[error("改动文件数 {n} 超过上限 {max}")]
    TooManyFiles { n: usize, max: usize },
}
```

**注意**：这类错误返回 `ToolOutput { is_error: true, output: 详细诊断 }`，**不是 `Err`**——因为它要走 history、给模型看。第 6 章的原则再次生效：**工具失败是对话的一部分。**

---

## 8.7 与整文件重写的取舍

DSL 不是万能的。**改动占比很大时，复述整段上下文的 token 成本比“直接重写整个文件”更高**。决策点：

| 场景 | 推荐 | 理由 |
|---|---|---|
| 小改动（加减几行、改一处逻辑） | **patch DSL** | 省 token，上下文精确 |
| 大改动（重构整个文件、换实现） | **整文件重写**（`write_file`） | 复述上下文成本 > 全文重写 |
| 新建文件 | **Add File**（DSL 或 write） | 无上下文问题 |
| 删除大段 | DSL Remove hunk | 锚点明确即可 |

**阈值**：上下文复述超过“新文件内容 × 1.5”就改用整文件重写。这个判断由**模型自己**根据工具描述里的指引来做——schema 里写清两种工具的适用场景，模型按改动比例自选。**提供两种原语、让模型选，比逼它只用一种更可靠。**

> **unified diff 真正的对手不是 DSL，是“整文件重写”**。三者各有领地。Codex 选 DSL 是因为它处理的绝大多数是**小粒度、高精度的代码编辑**——这恰好是 agent 最常见的工作单元，也恰好是行号最容易错的场景。

---

## 8.8 测试：穷举 patch 的合法与非法

```rust
#[test]
fn parse_update_with_context_and_add() {
    let input = "\
*** Begin Patch
*** Update File: src/lib.rs
@@
 impl Foo {
     pub fn existing(&self) {}
+
+    pub fn new() -> Self { Self {} }
 }
*** End Patch
";
    let stmts = parse_patch(input).unwrap();
    assert_eq!(stmts.len(), 3);   // Begin + UpdateFile + Hunk(+End)
    assert!(matches!(stmts[2], Stmt::Hunk(_)));
}

#[test]
fn context_must_match_exactly() {
    let tool = ApplyPatchTool::in_dir("fixtures/repo");
    let patch = "\
*** Begin Patch
*** Update File: lib.rs
@@
 this line does not exist in the file
+
+added
*** End Patch
";
    let out = run_apply(&tool, patch);
    assert!(out.is_error);
    assert!(out.output.contains("未找到上下文匹配"));   // 精确诊断
}

#[test]
fn path_traversal_is_rejected() {
    let tool = ApplyPatchTool::in_dir("/tmp/safe_root");
    let patch = "*** Begin Patch\n*** Update File: ../../etc/passwd\n@@\nold\n+new\n*** End Patch\n";
    let err = validate_only(patch, &tool).unwrap_err();
    assert!(matches!(err, PatchError::PathTraversal { .. }));
}

#[test]
fn twenty_consecutive_patches_on_same_file_succeed() {
    // 章末验收项：连续 20 个 patch 不出错
    let mut content = String::from("fn main() {}\n");
    for i in 0..20 {
        let patch = format!(
            "*** Begin Patch\n*** Update File: a.rs\n@@\nfn main() {{}}\n+\n+// edit {i}\n*** End Patch\n"
        );
        content = apply_to_string(&content, &patch).unwrap();
    }
    assert_eq!(content.matches("// edit").count(), 20);
}
```

第四个测试直接对应**章末验收的“连续 20 个 patch 不出错”**——它验证的是**累积正确性**：每个 patch 的上下文匹配都基于上一轮的真实文件，任何一次“差一错位”都会在这里暴露。

---

## 避坑专栏 #9：临时文件要和正式文件同目录

**错误写法**：

```rust
let tmp = std::env::temp_dir().join("mcx-XXXX");   // /tmp 下
tokio::fs::rename(&tmp, &final_path).await?;        // 跨文件系统 → 退化成 copy+unlink
```

**症状**：多数时候正常，但 `/tmp` 和项目目录不在同一挂载点时，`rename` 返回 `EXDEV`，或 silently 变成“复制整个文件再删除”——**不再原子**，崩溃窗口重现。大文件时还会卡顿。

**解法**：**tmp 放在目标文件同一目录**：

```rust
let tmp = final_path.with_file_name(format!("{}.tmp.{}",
    final_path.file_name().unwrap().to_str().unwrap(),
    std::process::id()));
// 同目录 → 同一文件系统 → rename 是 O(1) 原子的
```

**通用形式**：**“写临时 + rename 保证原子”这个技巧，要求 tmp 与正式文件同挂载点。** 判断口诀：看到 `rename` 就检查两边路径的 `df -h`，不在同一行就是 bug。

---

## 8.9 Design Rationale

**Q：为什么不用 unified diff？模型完全可以用行号啊。**

因为模型的失败模式是**稳定的、可预测的**：它在复述代码上接近完美，在精确计数上系统性地差。unified diff 把全部赌注押在“行号精确”上——**这正是模型最弱的一环**。DSL 把赌注改押到“上下文复述”——**模型最强的一环**。这是一笔确定性的交易，不是审美偏好。

**Q：上下文匹配会不会“误匹配到第二个相同块”？**

会，如果代码里真有两段一模一样的上下文。解法：**要求上下文块包含足够多的行（默认 ≥3 行非空）**，并在匹配到多处时报错让模型加更多上下文。**这是把“歧义”变成显式错误，比静默选第一个安全得多。**

**Q：为什么不直接用 `diff_match_patch` 之类的库做自动对齐？**

自动对齐在“大段移动”时很香，但**它把不确定性藏在库里**。agent 改代码的场景，模型给出的意图是精确的（“在这段后面加”），用上下文匹配足以还原；引入模糊对齐反而可能在边界情况选错。**确定性工具配确定性接口**——这是本章贯穿的选择。

---

## AI 软件工程原理 #8

> **把不确定性从模型的弱项转移到强项。**

模型的能力分布极不均匀：**复述、模式匹配、按示例泛化是强项；精确计数、严格地址、全局一致的状态维护是弱项。** 好的工具设计就是把任务**重新表述**，让它只要求强项、不要求弱项。

把三个章节的接口摆一起看这个迁移：

| 任务 | 弱项方案（不用） | 强项方案（采用） | 不确定性的去向 |
|---|---|---|---|
| 改文件 | 行号寻址（unified diff） | 上下文复述匹配（DSL） | 从“计数”到“模式匹配” |
| 读文件 | “记住整个仓库” | 先 list_dir 建立索引再读 | 从“全局记忆”到“按需检索”（第 9 章） |
| 调工具 | 复杂参数对象 | 带示例的 schema 描述 | 从“猜结构”到“看示例” |

**这条原理是“上下文工程”的另一种说法**：不只是“往窗口里塞什么”，更是“**把问题编码成模型容易答对的形式**”。接口设计者要为模型的认知特点负责，而不是假定一个理想化的全能模型。

**反面教训**：让模型“自己维护行号”“自己记住 20 个文件的修改状态”“自己估算 token 余量”= 把强约束交给弱能力 = 必然漂移。**凡是要精确状态的地方，用代码和外部存储（history、文件系统、数据库）兜底；模型只做模式判断。**

下一章的 `list_dir` + `read_file` 组合，是同一原理的又一应用：**让模型先建立索引，再按需读取**——把“全局记忆”换成“检索”，继续把不确定性留在模型的强项区。

---

## Rust 修炼小结

| 概念 | 本章用法 | 后面在哪用到 |
|---|---|---|
| 手写状态机 | DSL 解析器 `State` 枚举 | 第 13 章 plan 解析 |
| `Peekable` 迭代器 | hunk 间前瞻 | 全书 |
| `Path::components` + `ParentDir` | 路径穿越防护 | `mcx-sandbox` |
| `tokio::fs::rename` 原子性 | 两阶段提交 | 第 16 章 JSONL |
| `thiserror` 结构化错误 | 精确诊断喂回模型 | 全书 |
| 两阶段提交 | tmp + rename | 第 15 章 checkpoint |

---

## 章末验收

- [ ] 对同一文件连续打 20 个 patch，最终结果正确无错位
- [ ] `*** Update File: ../../etc/passwd` 被拒绝，`is_error: true`
- [ ] 上下文不匹配时返回的 `ToolOutput` 含行号与期望片段
- [ ] patch 中途校验失败时，磁盘上的原文件未被修改
- [ ] `cargo test` 通过

---

## 读者挑战

1. 构造一个文件含**两段完全相同**的上下文块，让单次 hunk 匹配到多处。**工具该怎么报错？本书不给答案。**
2. 把 Add File 和 Update File 合并成“若文件不存在则创建”的单指令，**这会丢失什么安全性？**
3. DSL 用 `@@` 和 `***` 两种前缀，能否统一成一种？**提示：考虑“以 `+` 开头的代码行”与前缀冲突。**

---


# 第 9 章　读文件、列目录与看图片

**本章任务**：补齐 agent 的“感知三件套”——`read_file`、`list_dir`、`view_image`。读到此章，mini-codex 的工具箱就齐了：**感知（读/列/看）+ 行动（shell/apply_patch）**，一个能真正在仓库里干活的 agent 成型。

但本章的重点不是“三个函数怎么写”，而是**它们该怎么组合、以什么顺序、花多少 token**。这引出了全书关于上下文的最关键一章。

---

## 9.1 先问一个被忽略的问题：模型怎么“知道”项目长什么样

设想你把 agent 丢进一个陌生仓库，说“找到处理鉴权的文件”。

**直觉方案**：直接 `grep` / 直接 `read_file Cargo.toml`。**但模型不知道要 grep 什么关键字，也不知道有哪些文件可读**——它对仓库的全部认知是空的。

**Codex 的做法**：启动时先 `list_dir` 一遍，把目录结构（尊重 `.gitignore`）喂给模型，让它**先建立心理模型**，再决定读哪个文件。**这一步带来的收益是显著的**——把“盲目检索”变成“有依据的导航”。

```
启动 → list_dir("/") → 模型看到  src/  tests/  Cargo.toml  .github/
                          ↓
      模型决策："鉴权通常在 middleware，先 read_file src/middleware/auth.rs"
                          ↓
      定向 read_file → 拿到真实内容 → 开始改
```

**顺序就是一切**：先建立索引，再按需读取。这不是优化技巧，这是**模型在信息不足时唯一可靠的行动策略**。

---

## 9.2 list_dir：尊重 .gitignore

`list_dir` 的第一个设计点：**默认忽略不该看的东西**。一个典型的仓库里，`target/`（Rust 构建产物）动辄几百 MB、几百万行——把它列进上下文等于自毁。

```rust
// crates/mcx-tools/src/list_dir.rs
use ignore::WalkBuilder;   // 第三方 crate：尊重 .gitignore / .ignore / 嵌套规则
use serde_json::Value;

pub struct ListDirTool {
    pub root: PathBuf,
    pub max_entries: usize,     // 单目录条目上限
    pub max_depth: usize,       // 递归深度上限
}

impl Tool for ListDirTool {
    fn name(&self) -> &str { "list_dir" }
    fn schema(&self) -> Value {
        json!({
            "name": "list_dir",
            "description":
                "列出目录内容，用于建立项目结构心理模型。默认遵循 .gitignore/.ignore。\
                 优先列根目录和关键子目录（src/、crates/），再深入。\
                 不要一次递归过深；发现目录过大时分批列。",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "相对于工作目录的目录路径，默认 \".\"" },
                    "depth": { "type": "integer", "description": "递归深度，默认 2，最大 4" }
                }
            }
        })
    }
}
```

**`ignore` crate 的好处**：它和 `ripgrep` 同源，正确处理 `.gitignore` 的**嵌套语义**（子目录的 `.gitignore` 只作用于自身及以下）、`.ignore`、`.git/info/exclude`，以及 `!` 取反规则。**自己写递归遍历 = 重新发明一套不完整的 ignore 逻辑**，边界极多。

```rust
async fn call(&self, arguments: &str) -> Result<ToolOutput, ToolError> {
    let args: Args = serde_json::from_str(arguments)?;
    let depth = args.depth.unwrap_or(2).min(self.max_depth);
    let start = self.root.join(args.path.as_deref().unwrap_or("."));

    let mut walker = WalkBuilder::new(&start);
    walker.max_depth(Some(depth + 1))
        .standard_filters(true)      // 启用 .gitignore 等
        .git_ignore(true)
        .git_global(false)
        .git_exclude(true);

    let mut entries = Vec::new();
    for entry in walker.build().filter_map(Result::ok) {
        if entries.len() >= self.max_entries { break; }
        let rel = entry.path().strip_prefix(&self.root).unwrap_or(entry.path());
        let kind = if entry.file_type().map(|f| f.is_dir()).unwrap_or(false) { "d" } else { "f" };
        // 用制表符缩进表达层级，模型对对齐格式极敏感
        let indent = "  ".repeat(rel.components().count().saturating_sub(1));
        entries.push(format!("{indent}{kind} {rel}", rel = rel.display()));
    }

    let mut out = entries.join("\n");
    if entries.len() >= self.max_entries {
        out.push_str("\n[截断] 条目过多，请缩小 path 或 depth 后分批列");
    }
    Ok(ToolOutput { output: out, is_error: false })
}
```

**注意输出格式**：`d`/`f` 前缀 + 缩进对齐。模型对**等宽、对齐、有规律的文本**理解最好（第 8 章的 DSL 也是这个思路）。**工具输出的格式，是 schema 之外第二重要的“提示词”。**

---

## 9.3 read_file：范围控制与二进制检测

`read_file` 看似最简单，三个坑都在细节：

**坑 1：大文件**。`Cargo.lock` 上万行、生成的 `bindings.rs` 几十万行——全读 = 上下文爆。解法：**`offset`/`limit` 分段读取**（默认 500 行），返回里带 `[共 N 行，已显示第 1–500 行]` 这样的 **1-based 闭区间**，让模型知道还有后续。参数 `offset` 本身仍是“跳过多少行”的 0-based 索引；模型翻页时用 `offset = 上次页脚里的最后行号`（即 `上次 offset + 实际显示行数`），不会因“行号 vs 偏移”差一位。

**坑 2：二进制文件**。`read_file icon.png` 会把乱码塞进 history，token 计费器起飞，模型也看不懂。解法：**读前嗅探（NUL 字节、过高非 ASCII 比例）**，命中就返回 `[二进制文件，请改用 view_image]`。

**坑 3：行号对齐**。模型引用“第 42 行”时，必须和实际内容对得上。**输出的每一行都带行号前缀**——这是给下一章 `apply_patch` 的上下文复述打地基。

```rust
// crates/mcx-tools/src/read_file.rs
pub struct ReadFileTool { pub root: PathBuf, pub line_cap: usize }

async fn call(&self, arguments: &str) -> Result<ToolOutput, ToolError> {
    let args: Args = serde_json::from_str(arguments)?;
    let path = resolve_inside_root(&self.root, &args.path)?;   // 同第 8 章穿越防护
    let bytes = tokio::fs::read(&path).await?;

    if looks_binary(&bytes) {
        return Ok(ToolOutput {
            output: "[二进制文件，无法以文本读取；若为图片请用 view_image]".into(),
            is_error: false,
        });
    }

    let text = String::from_utf8_lossy(&bytes);
    let total = text.lines().count();
    let offset = args.offset.unwrap_or(0);
    let limit = args.limit.unwrap_or(self.line_cap).min(self.line_cap);

    let mut out = String::new();
    for (i, line) in text.lines().skip(offset).take(limit).enumerate() {
        // 行号前缀：固定宽度，模型可直接引用
        out.push_str(&format!("{:>5} | {}\n", offset + i + 1, line));
    }
    // 页眉行号与正文行号同是 1-based 闭区间：显示第 first–last 行
    let first = offset + 1;
    let shown = limit.min(total.saturating_sub(offset));
    let last = offset + shown;
    out.push_str(&format!(
        "--- 共 {total} 行，显示第 {first}–{last} 行；翻页请用 offset={last} ---",
    ));
    Ok(ToolOutput { output: out, is_error: false })
}

/// 二进制嗅探：前 8KB 内出现 NUL，或非 ASCII 比例过高
fn looks_binary(bytes: &[u8]) -> bool {
    let head = &bytes[..bytes.len().min(8192)];
    if head.contains(&0) { return true; }
    let non_ascii = head.iter().filter(|&&b| b > 0x7F).count();
    non_ascii * 100 / head.len().max(1) > 30
}
```

**行号前缀是刻意为之**：`apply_patch` 虽然不靠行号定位，但**模型在推理时会引用行号**（“在第 42 行附近加一个函数”）。让行号始终可见，减少一次“数错行”的机会。**一致性比纯洁性重要**——既然模型会数行号，就给它对的行号。

---

## 9.4 view_image：多模态输入的边界

`view_image` 把图片塞进请求。它有两个成本必须显式管理：

1. **token 成本**。一张 4K 截图经多模态编码可能上千 token，而且**图片 token 通常比文本贵**。
2. **传输成本**。几 MB 的 PNG 不该反复随每个请求上传。

实现要点：**缩放 + 格式转换 + 尺寸上限**。超过阈值就下采样到最大边长（如 1568px，约对齐常见多模态模型的预处理），并以 PNG（无损、token 效率优于 JPEG for diagrams）输出：

```rust
pub struct ViewImageTool {
    pub root: PathBuf,
    pub max_side_px: u32,      // 长边上限
    pub max_bytes: usize,      // 解码后字节上限
}

async fn call(&self, arguments: &str) -> Result<ToolOutput, ToolError> {
    let args: Args = serde_json::from_str(arguments)?;
    let path = resolve_inside_root(&self.root, &args.path)?;

    // 真实实现用 image crate；此处给出结构
    let img = load_image(&path)?;
    let resized = resize_to_fit(&img, self.max_side_px);
    let png = encode_png(&resized)?;

    if png.len() > self.max_bytes {
        return Ok(ToolOutput { output: "[图片过大，已拒绝]".into(), is_error: true });
    }

    // 通过协议层把二进制作为多模态 part 注入下一条请求；
    // ToolResult.output 只放摘要 + 占位引用。
    Ok(ToolOutput {
        output: format!("![image](ref={}, {}x{}, {}KB)",
                        args.path, resized.width(), resized.height(), png.len() / 1024),
        is_error: false,
    })
}
```

**协议层怎么注入图片？** 这需要扩展 `Message`/`complete`——第 17 章做 app-server 时会正式支持多模态 part。本章先把工具接口和**边界**（`max_side_px`、`max_bytes`）定好，落地时复用同一套机制。

> **工具组合的红线：图片永远不该被当成文本塞进 `ToolResult.output`。** 那是 base64 灾难（一次几十 KB 的 base64 在上下文里膨胀 1.33×，还吃掉模型的视觉注意力）。正确做法是**引用 + 协议外通道**。

---

## 避坑专栏 #10：`std::fs` 与 `tokio::fs` 混用不会报错，但会阻塞

**错误写法**：

```rust
async fn call(&self, args: &str) -> Result<ToolOutput, ToolError> {
    let bytes = std::fs::read(&path)?;   // ← 同步 IO，在 async 任务里阻塞线程
    // ...
}
```

**症状**：单文件读取时毫无感觉；一旦并发跑 10 个工具或读一个 NFS 挂载的慢文件，**整个 tokio 工作线程卡住**，所有其他任务（流式、事件转发）停摆。响应延迟从毫秒变秒。

**原因**：`std::fs::read` 是同步系统调用，会占住当前线程直到磁盘返回。tokio 的线程池有限（默认 ≈ CPU 核数），一个阻塞 = 少一个 worker。

**解法**：**工具内部一律 `tokio::fs`**：

```rust
let bytes = tokio::fs::read(&path).await?;          // ✓
let mut f = tokio::fs::File::open(&path).await?;     // ✓
```

**只有两种情况可用 `std::fs`**：(a) 真的只在同步上下文；(b) 用 `tokio::task::spawn_blocking` 显式搬到专用线程池。**判断口诀：在 `async fn` 里看到 `std::fs` / `Mutex::lock` / 长时间 CPU 计算 → 三个都是阻塞信号。**

---

## 9.5 工具组合的经济学

现在三件套齐了，真正的课题浮现：**一次往返的 token 成本**。

| 调用 | 典型输出 | 是否进 history | 成本性质 |
|---|---|---|---|
| `list_dir depth=2` | 几百条目 | 是 | **一次性、可复用**（结构不常变） |
| `read_file` | 500 行 | 是 | 按需、可分页 |
| `view_image` | 引用摘要 | 是（正文），图片走外通道 | 单次贵，控制频次 |
| `shell`（grep） | 命中行 | 是 | 比全读省，但 grep 关键字依赖模型判断 |

**核心矛盾**：信息越多模型越准，但**每多一单位信息都要付 token + 时间，且稀释掉真正重要的内容**（“大海捞针”效应）。所以策略不是“能拿多少拿多少”，而是**最小化必要信息量**。

```text
理想路径（3 轮内定位）：
  list_dir . (1)  → 模型建立结构
  list_dir src    → 定位候选
  read_file src/auth.rs → 拿到真相

反模式：
  read_file Cargo.toml
  read_file src/main.rs
  read_file src/lib.rs
  ... 盲目逐个读，token 烧完还没找到
```

**`list_dir` 先行 = 用 O(目录规模) 的廉价信息，换掉 O(全仓库) 的昂贵读取。** 这正是章末验收“陌生仓库 3 轮内定位”的经济学依据。

> **一张图（概念）**：
> ```
> token 预算
> ┌──────────────────────────────┐  ← 上限
> │ list_dir        ████  廉价索引 │
> │ read_file ×3    ██████████  按需 │
> │ view_image ×1   ██████  控制频次 │
> │ 推理余量        ███████████████│
> └──────────────────────────────┘
> ```

**这是“上下文工程”的操作化定义**——下一节把它写成原理。

---

## 9.6 接进 Registry：工具装配点

三件套注册进 `Registry`（`mcx-tools/src/lib.rs`）：

```rust
pub fn builtin_tools(root: PathBuf) -> Registry {
    let mut reg = Registry::new();
    let root = Arc::new(root);
    reg.register(Box::new(ListDirTool::new(root.clone(), 2000, 4)));
    reg.register(Box::new(ReadFileTool::new(root.clone(), 500)));
    reg.register(Box::new(ViewImageTool::new(root.clone(), 1568, 8 * 1024 * 1024)));
    reg.register(Box::new(ShellTool::default(root.clone())));          // 第 7 章
    reg.register(Box::new(ApplyPatchTool::new(root.clone())));        // 第 8 章
    reg
}
```

**这一行行 `register`，就是全书第 6 章承诺的“加工具只改一处”的最终形态。** 第 3 章的 `Session` 对这套装配一无所知，它只认 `Registry`。

`Session::new_with_tools(client, op_rx, ev_tx, builtin_tools(workspace))`——CLI 里一行搞定。**所有工具共享同一个 `root: Arc<PathBuf>`**（原理 #6 里的“上下文注入”模式），各自持有自己的边界配置。

---

## 9.7 测试：导航行为可判定

```rust
#[tokio::test]
async fn lists_respects_gitignore() {
    let tmp = setup_repo(&[
        ("src/main.rs", "fn main(){}"),
        ("target/debug/big", "ignored"),
        (".gitignore", "target/\n"),
    ]);
    let out = run_list_dir(tmp.path(), ".", 2).await;
    assert!(out.output.contains("src/main.rs"));
    assert!(!out.output.contains("target/debug/big"));   // 被 ignore 过滤
}

#[tokio::test]
async fn read_file_is_paginated_and_numbered() {
    let tmp = setup_repo(&[("a.rs", &(0..2000).map(|i| format!("line {i}\n")).collect::<String>())]);
    let out = run_read_file(tmp.path(), "a.rs", 0, 3).await;
    assert!(out.output.contains("    1 | line 0"));
    assert!(out.output.contains("共 2000 行，显示 0–3"));
}

#[tokio::test]
async fn binary_file_is_rejected_with_hint() {
    let tmp = setup_repo(&[("logo.png", &[0u8, 1, 2, 0, 4][..])]);
    let out = run_read_file(tmp.path(), "logo.png", 0, 500).await;
    assert!(out.output.contains("二进制"));
}

#[tokio::test]
async fn agent_locates_target_in_three_rounds_or_less() {
    // 章末验收：陌生仓库 3 轮内定位
    let repo = make_fixture_with_feature("auth_middleware");
    let scripted = ScriptedLlm::from_actions(vec![
        Action::ListDir { path: ".".into() },
        Action::ListDir { path: "src/middleware".into() },
        Action::ReadFile { path: "src/middleware/auth.rs".into() },
    ]);
    let (events, _history) = run_turn_with(repo.path(), scripted);
    let read_calls: Vec<_> = history_tool_calls(&events, "read_file");
    assert!(read_calls.len() <= 3, "调用序列: {read_calls:?}");
    assert!(read_calls.iter().any(|c| c.contains("middleware/auth.rs")));
}
```

最后一个测试把**“工具组合策略”变成了可判定的验收**：不是“模型理解了项目”（主观），而是“`read_file` 调用次数 ≤3 且命中正确路径”（客观）。**这就是原理 #3“事件流是可判定的真相来源”在工具层的延伸。**

---

## 9.8 Design Rationale

**Q：为什么让 agent 先 `list_dir` 而不是直接 `grep`？**

因为 grep 的前提是**你知道要搜什么关键字**——而模型对陌生仓库没有这个知识。list_dir 建立的是**结构先验**（“这个项目的代码在 `src/`，配置在 `config/`”），它让后续每个决策有据可依。**先索引、后检索，是把“盲目探索”变成“受控导航”**。实测收益显著，这正是 Codex 启动映射的设计。

**Q：为什么要自己写分页（`offset`/`limit`），不用 ripgrep 一把梭？**

ripgrep 适合“已知关键字”的精准搜索；但 agent 的工作流是**先概览再深入**。分页让模型**按需加载**，把上下文占用压到最低。`list_dir` 给出候选 → `read_file` 取一段 → 需要时翻页。**两种检索原语并存，按阶段选用。**

**Q：行号前缀会不会浪费 token？**

会，每行几个字符。但它换来的是**模型引用行号时的准确率**——而一次“数错行 → apply_patch 失败 → 重新读取”的往返，成本远超行号前缀。**这是花小钱省大钱。** 且行号只在 `read_file` 输出里出现，不会污染其他工具。

---

## AI 软件工程原理 #9

> **上下文工程 = 在正确的时间把正确的信息放进窗口。**

**从 agent 视角，它访问不到的东西就等于不存在。** 模型没有“文件系统”的概念，只有“上一次 `list_dir`/`read_file` 返回的内容”。所以**信息获取的顺序、粒度、预算，本身就是推理的一部分**——不是附属的 IO。

把这个原理展开成三条操作规则：

| 规则 | 反模式 | 正解（本章） |
|---|---|---|
| **先建索引** | 直接 grep/直接读 | `list_dir` 先行 |
| **按需加载** | 一次读完整个目录树 | `offset`/`limit` 分页 |
| **控制单价** | 无脑读二进制、堆图片 | 二进制检测、尺寸上限 |

**第二条推论：上下文不是越多越好，是“信噪比”越高越好。** 每多一段无关文本，就稀释掉真正关键的几行（大海捞针），同时多付一份 token。**“什么该放进去”的决策，和“放进去之后怎么推理”同等重要。** 这是 agent 工程区别于普通 LLM 应用的核心——普通应用把整段文档塞 prompt，agent 必须自己决定读什么。

**第三条推论：工具的输出格式本身就是上下文工程。** `list_dir` 的对齐缩进、`read_file` 的行号、`apply_patch` 的错误诊断——**这些格式决定了模型能不能正确消化信息**。把工具输出当成“给模型的 UI”来设计，而不是“函数的返回值”。

**与前后章节的呼应**：原理 #6（让模型容易生成对）是本地的、单工具的；原理 #9 是全局的、跨工具调用的——它管的是**整个上下文窗口的生命周期**。下一章进入**安全篇**，把“敢让它做什么”做成配置（审批、沙箱、网络策略）——因为到这里为止，我们的 agent 已经**有能力**做任何事，接下来要回答的是“该允许它做哪些”。

---

## Rust 修炼小结

| 概念 | 本章用法 | 后面在哪用到 |
|---|---|---|
| `ignore::WalkBuilder` | 尊重 `.gitignore` 遍历 | 第 13 章仓库检索 |
| `Arc<PathBuf>` 共享 root | 多工具共享工作目录 | 全书 |
| `tokio::fs` 全套 | 异步文件 IO | 第 16 章落盘 |
| `String::lines` + `skip/take` | 分页读取 | 第 20 章评测 |
| 二进制嗅探（NUL + 比例） | 防止乱码进上下文 | 通用文件处理 |
| 行号前缀格式化 | 对齐输出、辅助模型引用 | `apply_patch` 诊断 |

---

## 章末验收

- [ ] 在陌生仓库里，agent 能在 3 轮内定位到指定功能的实现文件（有测试断言）
- [ ] `list_dir` 正确过滤 `.gitignore` 里的 `target/` 等条目
- [ ] `read_file` 对 2000 行文件只返回 `limit` 行，并带行号前缀
- [ ] 二进制文件返回提示而非乱码，`view_image` 能控制尺寸与字节上限
- [ ] `cargo test` 通过，不依赖网络

---

## 读者挑战

1. `list_dir` 的 `max_entries` 触发截断时，模型怎么知道“该缩窄 path 还是加深 depth”？**提示：输出文案本身就是提示词。**
2. 若 `read_file` 每次都返回行号前缀，而 `apply_patch` 不用行号——**这套接口里是否藏着不一致？如何消除？**
3. 让 `view_image` 和 `read_file` 共用一个“资源引用”机制，**这预示了协议层的什么改动？** 本书不给答案，预告第 17 章。

---

## 下一章预告：一个什么都能做的 agent，没人敢让它运行

到这里，mini-codex 已经**什么都能做**了：`shell` 能跑任意命令、`apply_patch` 能改任意文件、`read_file` 能读任意文本。它活了。**但一个能 `rm -rf`、能读 `~/.ssh/`、能访问生产数据库的 agent，没人敢让它真正运行。**

这一部分要解决的问题恰好是前面四章**刻意留白**的部分：

- **第 10 章　审批（Approval）**：破坏性操作在真正执行前，把决定权交给用户。这里会首次引入“**同步等待用户点头**”的交互模式——而第 3 章的 bounded channel 埋下的那个死锁伏笔，也将在这一章正面解决。
- **第 11 章　沙箱（Sandbox）**：用 cgroup / bwrap / Seatbelt 把 shell 进程**真正隔离**——第 7 章的“进程树清理”只是第一道防线，层级化沙箱才是确定性隔离。
- **第 12 章　网络与凭证策略**：环境变量白名单只是起点，真正的安全需要“**这个工具、这次调用、能不能访问那个主机**”的细粒度决策。

你会发现一个贯穿这一部分的主题：**前面每加一个能力，安全面就扩大一圈；安全不是事后贴的创可贴，而是和每一步能力同步生长的护栏。** 第 10 章的第一把锁，就从那个“等用户点允许”的 channel 死锁开始讲起。

---

## 引用来源

[1] 《用 Rust 造一个 Codex：AI Agent 系统设计与 AI 软件工程》全书大纲，第 150–238 行（第 6–9 章大纲）
> 工具系统：Trait 与 Registry；shell：进程、超时与进程树；apply_patch：为什么不用 unified diff；读文件、列目录与看图片。

[2] 《用 Rust 造一个 Codex》`_API契约.md`（mcx-protocol / mcx-core / mcx-tools 类型与三层循环定义）
> `Item::ToolCall { call_id, name, arguments }`、`Item::ToolResult { call_id, output, is_error }`、`Session::submission_loop` / `run_turn`、`CancellationToken`。

[3] 《用 Rust 造一个 Codex》第 3–5 章样章（第 3.1、3.6、4.2、4.6、5.3、5.7 节教学法）
> “先写个错的”开场、ScriptedLlm 假模型测试、`let _ =` emit、状态机与 tagged enum、Design Rationale Q&A。

[4] OpenAI Codex 应用 patch DSL 设计（Codex 仓库 `apply_patch` 实现）
> `*** Begin Patch` / `*** Update File` / `*** Add File` / `*** Delete File` 指令集，上下文匹配定位。

[5] OpenAI Codex 沙箱与进程管理（`external-sandbox`、bwrap/Seatbelt）
> 超时三段式 terminate → 宽限期 → hard kill；杀死进程树而非单进程；环境变量白名单 `shell_environment_policy.include_only`。

[6] ignore crate（ripgrep 项目）—— `WalkBuilder` 文档
> 尊重 `.gitignore` / `.ignore` / `.git/info/exclude` 的嵌套语义与 `!` 取反规则。

[7] LangChain 代理 harness 改进：启动时映射目录结构带来显著收益
> “代理应先建立项目结构的心理模型，再决定读取哪些文件”——工具组合经济学。


# 第三部分　安全：把“敢让它做什么”做成配置（第 10–12 章）

> 前九章的 mini-codex 已经拥有了完整的能力闭环：`shell` 能执行任意命令，`apply_patch` 能改写任意受控路径，`read_file`/`list_dir` 能读取环境信息。能力越强，风险面越大；本部分不再新增业务功能，而是把“模型想做什么”“系统允许做什么”“用户愿意承担什么”拆成三个独立的决策层。第 10 章管人的价值判断，第 11 章管操作系统的强制边界，第 12 章管可复用的声明式规则。三者合起来，才是一个敢在生产仓库里跑的 agent。

---

# 第 10 章　审批策略：自主性是个可调旋钮

**本章任务**：把“是否询问用户”从工具实现里抽出来，做成纯函数式策略；再给第 3 章埋下的审批死锁伏笔一个确定解。读完本章，mini-codex 将拥有四档 `ApprovalPolicy`、四档 `SandboxPolicy`，并且能明确回答：什么时候自动执行、什么时候询问、什么时候允许在沙箱外重试。

第 9 章末尾预告得没错：到这里为止，agent“什么都能做”并不是好事。我们要把它变成“能做的范围由配置决定，并且每一次越界都可见、可问、可拒绝”。

## 10.1 先写一个会把系统卡死的审批器

最直观的审批器长这样：工具调用前先发一个事件，等 UI 回复，再决定执不执行。

```rust
// 反例：会把引擎任务与渲染任务锁成死锁
// 注：Event::ExecApprovalRequest 并不存在于第 3 章的 Event 定义里——
// 它是这里为了让反例成立而“拍脑袋”造出的变体，本章后半正是要论证它不该被发明
async fn request_approval(
    session: &Session<impl LlmClient>, call: &ToolCall,
) -> Decision {
    session
        .event_tx
        .send(Event::ExecApprovalRequest { /* ... */ })
        .await
        .unwrap(); // ← 队列满时永久阻塞

    let response = session.approval_rx.recv().await.unwrap();
    interpret(response)
}
```

这段代码继承了第 3 章的两条常识：用 `Event` 向 UI 报告、用 `Op` 把用户动作送回引擎。但它恰恰踩中了避坑专栏 #4 预告的升级版问题：**审批响应不是普通事件，而是当前工具调用的控制依赖。**

```
引擎任务：send(ApprovalRequest) ──▶ 事件队列 ──▶ 渲染任务
渲染任务：读事件 → 显示提示 → 等键盘 → send(Op::Approval)
```

若事件队列是有界 `mpsc`，且渲染任务正准备把提示渲染到底层 UI，流程就会变成：引擎等队列腾出空间，队列等渲染任务消费，渲染任务又等用户交互——**但用户根本看不到提示，因为提示本身还在满队列后面**。队列容量越小、审批越频繁，死锁越容易出现；它通常不是每次都炸，而是“偶尔卡死”，所以比 panic 更难排查。

这里的根本矛盾是：**普通 `Event` 是“通知”，审批请求是“同步闸门”。** 把闸门塞进通知队列，就让两者共享了同一个背压点。

## 10.2 把审批拆成纯策略和可等待令牌

先修正 `mcx-protocol` 的事件集合。我们保留事件流的“可观测”用途，但审批请求必须进入独立通道。

```rust
// crates/mcx-protocol/src/lib.rs（增量）
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    // ...既有变体
    /// 仅用于展示，不再承担“请回复我”的控制语义
    ToolCallRecord { turn: usize, call_id: String, name: String },
    /// 审批结果已经确定后的审计事件
    ApprovalRecord { turn: usize, call_id: String, decision: DecisionKind },
}

/// 审批请求不再借用 Session，也不是 Event。
#[derive(Debug, Clone)]
pub struct ApprovalRequest {
    pub call_id: String,
    pub tool: String,
    pub summary: String,
    pub risk: RiskLevel,
}
```

`Decision` 也不该知道 stdin、tokio 或用户界面。它是策略内核的产物，只描述“允许怎么做”。

```rust
// crates/mcx-core/src/approval.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApprovalPolicy {
    /// 只有已知只读/安全命令自动放行，其余均询问
    Untrusted,
    /// 沙箱内默认放行；沙箱内失败时才升级询问
    OnFailure,
    /// 模型可显式请求审批；其余按风险规则处理
    OnRequest,
    /// 永远不询问；拒绝直接作为工具错误返回模型
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny,
    Ask,
    /// 允许，但必须在沙箱内运行
    AllowInSandbox,
    /// 沙箱内尝试失败，可申请在沙箱外重试
    EscalateOutOfSandbox,
}

impl ApprovalPolicy {
    /// 基础档：没有规则命中时的缺省决策，等价于 `decide(cmd, /* known_safe = */ false)`。
    /// 第 12 章 `policy_decision` 在 execpolicy 未命中规则时调用它。
    pub fn base_decision(self, cmd: &CommandView) -> Decision {
        self.decide(cmd, false)
    }

    /// 规则档：execpolicy 命中 `effect = "prompt"` 后，策略决定是否真的打断用户。
    /// `never` 把“询问”关掉并转成 `Deny`；其余策略都把选择权交回给用户。
    pub fn decide_prompt(self, cmd: &CommandView) -> Decision {
        match self {
            ApprovalPolicy::Never => Decision::Deny,
            _ => Decision::Ask,
        }
    }

    /// 纯函数：同一输入永远得到同一输出。
    pub fn decide(self, cmd: &CommandView, known_safe: bool) -> Decision {
        use ApprovalPolicy::*;
        match self {
            Untrusted => {
                if known_safe && !cmd.has_external_effect {
                    Decision::AllowInSandbox
                } else {
                    Decision::Ask
                }
            }
            OnFailure => {
                // 默认先相信沙箱；失败由执行层转换为 EscalateOutOfSandbox
                if cmd.requires_sandbox_bypass {
                    Decision::Ask
                } else {
                    Decision::AllowInSandbox
                }
            }
            OnRequest => {
                if cmd.explicit_approval_request {
                    Decision::Ask
                } else if known_safe {
                    Decision::AllowInSandbox
                } else {
                    Decision::Deny
                }
            }
            Never => {
                if cmd.requires_sandbox_bypass {
                    Decision::Deny
                } else {
                    Decision::AllowInSandbox
                }
            }
        }
    }
}
```

`CommandView` 是从 `shell` 参数、`apply_patch` 的解析结果等归一化出来的只读视图：

```rust
// crates/mcx-core/src/approval.rs
#[derive(Debug, Clone, Default)]
pub struct CommandView {
    pub tool: String,
    pub executable: Option<String>,
    pub argv: Vec<String>,
    pub writes_outside_workspace: bool,
    pub touches_git_metadata: bool,
    pub has_external_effect: bool,
    pub explicit_approval_request: bool,
    pub requires_sandbox_bypass: bool,
}
```

**关键点：策略不查数据库、不发请求、不读时钟。** 它的测试因此可以直接枚举 `command × policy`；它的日志可以直接变成审计记录；它也无法偷偷等待 UI。第 12 章的 `execpolicy check` 就是对同一类纯评估的离线复用。

## 10.3 四档策略的真实含义

四档不是“打扰程度由高到低”的肤浅滑块，而是四套不同的失败哲学：

| 策略 | 默认行为 | 适合场景 | 拒绝时的表现 |
|---|---|---|---|
| `untrusted` | 只读/安全操作自动放行，其余询问 | 陌生仓库、公共 CI、评审前试跑 | 让用户看到“为什么需要批准” |
| `on-failure` | 沙箱内自动放行，失败时升级 | 本地可信开发、长任务 | 默认不打扰，越界才问 |
| `on-request` | 模型显式请求才询问；其他命令要么安全放行要么拒绝 | 半自动审查、精细工作流 | 不出现弹窗式疲劳 |
| `never` | 完全静默，拒绝转成模型可见错误 | CI、脚本化回归、受更高层隔离保护的执行器 | 不能依赖人类救场 |

特别注意 `never`：**它不是“允许一切”。** 它只是把“询问人类”这种交互手段关掉。网络越界、写越界、沙箱绕过请求仍然会被 `Deny`，只是不再问你，而是把错误交还给模型处理。若把 `never` 误配成“静默放行”，agent 会把拒绝悄悄吞掉——所以在策略层返回 `Deny`，比在 UI 层隐藏提示更安全。

> **价值判断必须外置。** “改 `Cargo.toml` 要不要问”“访问 `HOME` 算不算副作用”“模型请求发布到 npm 能否自动同意”——这些都不是引擎该写死的逻辑。它们属于 `ApprovalPolicy` 和项目配置。引擎只负责把事实归一化成 `CommandView`，再忠实执行 `Decision`。

## 10.4 审批与沙箱正交：两个旋钮，不是一条开关

第 3 章的 `Event`/`Op` 分离教过我们：两个变化频率不同的维度，别捏成一个类型。这里同样如此：

- **`ApprovalPolicy`**：要不要打断用户、把控制权交出去；
- **`SandboxPolicy`**：即使自动执行，最坏能造成什么破坏。

```rust
// crates/mcx-protocol/src/lib.rs（增量）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SandboxPolicy {
    /// 只读，默认断网；测试、代码搜索
    ReadOnly,
    /// 工作区/CWD/tmp 可写；`.git`、`.codex` 等仍受保护
    WorkspaceWrite,
    /// 外部已有隔离，mini-codex 不再施加本地限制
    ExternalSandbox,
    /// 完全不隔离；仅用于受控执行器或明确的 escape hatch
    DangerFullAccess,
}
```

![审批与沙箱是两个独立维度](assets/fig_policy_sandbox.png)
*图 1：四档审批策略与四档沙箱策略正交组合；每一格都是一个明确可测试的执行模式。数据来源：本书第 10–11 章策略模型，基于 OpenAI Codex 官方安全文档。*

把它们耦合成一个枚举，比如 `enum Mode { SafeAsk, AutoSandboxed, FullAuto }`，很快会遇到无法表达的需求：我想**只读且不打扰**，或者**工作区可写但每次都问**。拆开之后，`--full-auto` 就不再是一个神秘安全等级，而是：

```toml
# ~/.codex/profiles/full_auto.toml
approval_policy = "on-failure"
sandbox_policy = "workspace-write"
```

```rust
// crates/mcx-cli/src/profile.rs
pub fn apply_full_auto(cfg: &mut SessionConfig) {
    cfg.approval_policy = ApprovalPolicy::OnFailure;
    cfg.sandbox_policy = SandboxPolicy::WorkspaceWrite;
}
```

**这就是“`--full-auto` 是预设，不是机制”的含义。** 机制是“审批独立于沙箱”；预设只是把两个常用值绑在一起。后面若需要“只读 CI 模式”，可以再加 `read-only + never`，而无需改引擎匹配逻辑。

`ExternalSandbox` 尤其体现正交性：它声明“外层执行环境已隔离”，所以 mini-codex **不再施加本地 mount/seccomp 限制**，但仍会把 `has_full_network_access`、`sandbox_label` 等信息传给工具与 MCP server。反过来，`DangerFullAccess` 不是“自动开网络”，只是不再由本地沙箱设置屏障；它应要求显式 `--dangerously-disable-sandbox` 之类确认，避免成为默认值。

## 10.5 升级流程：失败不是终点，而是一次受控越界

`on-failure` 最有价值的部分是 **escalation**。流程必须是一个状态机，而不是简单的“失败后重试”。

```rust
// crates/mcx-core/src/execution.rs
pub enum ExecutionOutcome {
    Success(ToolOutput),
    SandboxDenied { attempt: SandboxedAttempt },
    OtherError(ToolOutput),
}

pub async fn execute_with_escalation(
    ctx: &ExecutionContext, call: &ToolCall,
) -> ToolOutput {
    let decision = ctx.policy.decide(&call.view(), is_known_safe(call));

    let first = match decision {
        Decision::AllowInSandbox | Decision::EscalateOutOfSandbox => {
            run_sandboxed(ctx, call, decision).await
        }
        Decision::Allow => run_unsandboxed(ctx, call).await,
        Decision::Ask => return await_user_approval(ctx, call).await,
        Decision::Deny => return denied_output(call),
    };

    match first {
        ExecutionOutcome::SandboxDenied { attempt } => {
            // 只有策略允许升级，且用户明确批准，才离开沙箱
            if !attempt.policy.allows_escape() {
                return attempt.into_denied_output();
            }
            let approved = request_escalation(ctx, call, &attempt).await;
            if !approved { return attempt.into_denied_output(); }
            ctx.audit.record_escape(call, &attempt.reason);
            run_unsandboxed(ctx, call).await.into_output()
        }
        ExecutionOutcome::Success(out) => out,
        ExecutionOutcome::OtherError(out) => out,
    }
}
```

升级的语义极其重要：**默认路径永远先进入沙箱；用户批准的那一次，才在沙箱外重试。** 因此用户不是在“允许 agent 乱来”，而是在为这一次具体失败承担明确责任。审计记录要包含原命令、命中规则、失败原因、批准者与时间戳——这正是第 3 章“事件流是真相来源”的安全版。

不过，escalation 有清晰的边界：

1. **不能自动循环**：批准一次 ≠ 永久允许同模式命令；授权应按 `(command hash, scope, expiry)` 缓存。
2. **不能跨越更外层隔离**：`ExternalSandbox` 下即使批准 escape，也不能假装有主机权限。
3. **不能以“失败”冒充“需要权限”**：超时、segfault、退出码 1 都不是越界证据；只有沙箱明确拒绝（denial、权限错误、seccomp/SBPL 命中）才进入升级。

## 10.6 兑现第 3 章伏笔：三条解法与本书选择

回到避坑专栏 #4。审批导致死锁有三条通用解法：

| 方案 | 优点 | 代价 | 评价 |
|---|---|---|---|
| ① 审批请求用独立 channel | 背压点分离，语义清晰 | 多一对 channel | **推荐** |
| ② 审批通道用 unbounded | 实现简单 | 交互消息理论无界，错误可无限堆积 | 可辅助，不应单靠 |
| ③ 给 send 加超时 | 一定能解除阻塞 | 超时后“用户批准”语义不完整 | 适合作为最后防线 |

**本书采用方案 ①，并用 ③ 兜底。** 具体结构如下：

```rust
// crates/mcx-core/src/session.rs（与第 3 章结构衔接）
pub struct Session<C: LlmClient> {
    client: C,
    history: Vec<Message>,
    tools: Registry,
    op_rx: mpsc::Receiver<Op>,
    event_tx: mpsc::Sender<Event>,          // 普通事件：有界、可丢
    approval_tx: mpsc::Sender<ApprovalRequest>, // 独立闸门通道
    approval_rx: mpsc::Receiver<ApprovalResponse>,
    cancel: CancellationToken,
    turn: usize,
    policy: Arc<PolicySet>,
}
```

UI 侧从“消费单一 Event 流”变成两个角色：

```rust
// crates/mcx-cli/src/main.rs（节选）
let (approval_tx, mut approval_rx) = mpsc::channel(64);
let (approval_reply_tx, approval_reply_rx) = mpsc::channel(64);

// 审批前台：专门把 ApprovalRequest 显示成交互提示
tokio::spawn(async move {
    while let Some(req) = approval_rx.recv().await {
        if ask_user(&req).await {
            let _ = approval_reply_tx.send(ApprovalResponse::allow(req.call_id)).await;
        } else {
            let _ = approval_reply_tx.send(ApprovalResponse::deny(req.call_id)).await;
        }
    }
});
```

引擎请求审批时，**等待回复也受 `CancellationToken` 与超时保护**：

```rust
async fn await_user_approval(ctx: &ExecutionContext, call: &ToolCall) -> ToolOutput {
    let request = ApprovalRequest::from(call);
    if ctx.approval_tx.send(request).await.is_err() {
        return ToolOutput::cancelled("审批前台已关闭");
    }
    let wait = async {
        loop {
            match ctx.approval_rx.recv().await {
                Some(resp) if resp.matches(call) => return resp.decision,
                Some(_) => continue, // 旧请求被取消，忽略过期回复
                None => return Decision::Deny,
            }
        }
    };
    match tokio::time::timeout(ctx.approval_timeout, wait).await {
        Ok(Decision::Allow) => execute_approved(ctx, call).await,
        _ => ToolOutput::denied("用户未批准或等待超时"),
    }
}
```

`approval_tx` 仍用有界 channel，不是无脑 unbounded：如果前台已退出而引擎继续发请求，填满后 `send` 会失败，我们直接返回“无法审批”，不会反向拖死工具循环。超时后必须忽略过期 `ApprovalResponse`，否则用户晚点点的“允许”会被错误复用到下一个命令。

> **通用形式：同步闸门必须拥有独立的有界队列 + 取消路径。** 事件流负责“发生了什么”，审批流负责“现在能不能继续”；前者可丢失，后者不可默默阻塞。第 3 章的 `let _ = event_tx.send(...)` 仍然正确，因为它只用于通知。

## 10.7 把决策变成可测试、可审计的产物

策略纯函数化的最大红利是表驱动测试：

```rust
// crates/mcx-core/src/approval/tests.rs
#[test]
fn decisions_are_deterministic_and_cover_policy_sandbox_matrix() {
    let cases = [
        (ApprovalPolicy::Untrusted, view("ls", &[]), Decision::AllowInSandbox),
        (ApprovalPolicy::Untrusted, view("rm", &["-rf", "/"]), Decision::Ask),
        (ApprovalPolicy::OnFailure, view("cargo", &["build"]), Decision::AllowInSandbox),
        (ApprovalPolicy::OnRequest, view("gh", &["pr", "merge"]), Decision::Deny),
        (ApprovalPolicy::Never, view("curl", &["https://x"]), Decision::Deny),
    ];
    for (policy, cmd, expected) in cases {
        assert_eq!(policy.decide(&cmd, false), expected,
                   "policy={policy:?} cmd={cmd:?}");
    }
}
```

```rust
#[tokio::test]
async fn approval_channel_deadlock_is_impossible_on_closed_ui() {
    let (tx, rx) = mpsc::channel(2);
    drop(rx); // UI 已经关闭
    let result = tx.send(ApprovalRequest::dummy()).await;
    assert!(result.is_err(), "send 应立刻失败，而不是阻塞等待不存在的前台");
}
```

后一个测试看起来平凡，却正是第 3 章伏笔的反证：把 `unwrap()` 换成“失败即拒绝”，并给接收配超时，死锁就不再可能成立。章末验收还要求把 `ApprovalRecord` 写入 `Rollout`，使“当时批准了什么”可回放——这正是原理 #3 的可判定事件流在安全层的延伸。

## 避坑专栏 #11：把“询问”伪装成普通日志事件

**错误做法**是把“请用户批准”渲染成普通 `Event::Info`，再用全局 `event_tx` 等待一个不存在的 reply channel。现象是：队列满时引擎卡住，UI 看起来正常，但用户从没看到提示。根源是把**同步控制流**伪装成了**异步通知流**。

**解法**是类型层面的隔离：`ApprovalRequest` 不是 `Event`；它走独立 `approval_tx/approval_rx`；等待时使用 `CancellationToken`、超时和幂等 `call_id`。普通事件仍可派生 `Event::ApprovalRecord` 用于展示和审计，但**那是决策的副作用，不是决策本身**。

**通用形式：凡是有“必须收到回复才能继续”语义的消息，都不要用普通日志 channel 承载。**

## 10.8 Design Rationale

**Q：为什么不把 `untrusted`/`never` 做成每个工具内部的 `if`？**

因为这样会让每个工具都复制一份价值判断。新工具出现时，作者会忘记询问；同一组织也无法统一收紧。把四档策略放在 `mcx-core`，工具只产出 `CommandView`，就能让“敢不敢”与“怎么做”分别演进。工具可以新增，策略矩阵无需重写。

**Q：为什么允许在沙箱外重试，而不是直接禁止？**

因为 agent 的真实工作不是停留在“被拒绝”，而是完成已获授权的任务。一次 `cargo build` 因内存限制被 cgroup 拒绝，与试图 `curl` 密钥，性质完全不同。`on-failure` 的升级只服务于**明确的、用户批准的、单次 escape**，并把证据留在审计里。它把“默认安全”和“可恢复性”同时保住。

**Q：为什么 `never` 不等于“放行一切”？**

因为 `never` 的契约是“不询问用户”，不是“放弃所有规则”。它关闭的是人机交互通道；网络、写入、权限仍受沙箱和 `execpolicy` 约束。CI 尤其需要这种区别：失败必须成为模型可处理的 `ToolOutput`，而不是进程挂起等人工确认。

## AI 软件工程原理 #10

> **把价值判断外置成配置，把安全边界内置成机制。**

人的偏好——“这个仓库可不可信”“发布前要不要问”“失败能否重试”——属于配置。进程隔离、写范围、网络规则、审批超时，属于机制。把前者写进工具 `if`，会随工具膨胀失控；把后者完全做成 YAML，又会让恶意配置关闭真实屏障。正确分工是：**配置选档位，机制执行不可逆限制，审计记录每一次跨越边界。**

这解释了为什么审批与沙箱必须正交：价值判断和安全边界的变化频率不同，故障模式也不同。它也解释了为什么第 12 章要把规则做成可离线询问——配置若不能被检验，就不是可管理的安全边界。

## Rust 修炼小结

| 概念 | 本章用法 | 后面在哪用到 |
|---|---|---|
| 纯函数式策略 | `(command, policy) -> Decision` | 第 12 章离线评估 |
| 独立 `mpsc` | 审批闸门与事件流分离 | 第 17 章 RPC 双向请求 |
| `CancellationToken` | 审批等待可被中断 | 第 7 章工具超时 |
| `#[derive(Serialize)]` | 决策、审计可落盘 | 第 16 章 JSONL |
| 表驱动测试 | 覆盖 policy × command | 第 20 章回归评测 |

## 章末验收

- [ ] `cargo test` 覆盖四种 `ApprovalPolicy` × 四种 `SandboxPolicy` 的代表组合
- [ ] 关闭审批 UI 后，引擎在有限时间返回“无法审批”，不卡死
- [ ] `on-failure` 仅在沙箱明确拒绝时升级；普通非零退出不触发 escape
- [ ] 每次 `Allow`/`Deny`/`Escalate` 都生成 `ApprovalRecord` 并写入 rollout
- [ ] `--full-auto` 只是 `workspace-write + on-failure` 的预设，可用 `mcx-cli config show` 打印

## 读者挑战

1. 若允许“同一命令批准一次后永久免询问”，攻击者让模型反复执行同一条命令即可绕过审批。**请设计基于命令哈希、作用域和 TTL 的缓存键，并写测试证明重复批准不能跨越仓库。**
2. 把审批通道改成 unbounded 似乎能“永远不阻塞”。**什么故障会让内存无界增长？如何用背压保护它？**
3. 模型可能在工具参数里伪造 `explicit_approval_request: true`。**这个字段应由谁生成，怎样避免模型自我批准？**

## 下一章预告：Codex 的沙箱不是容器

审批只能决定“要不要问”，不能阻止被批准的命令破坏主机。第 11 章将进入全书技术密度最高的部分：用 Linux bubblewrap、macOS Seatbelt、Windows restricted token，把同一个 `SandboxPolicy` 落到三个操作系统。我们将正面回答一个反直觉事实——Codex 的沙箱不是容器，而是一个普通用户即可启动、毫秒级生效的内核机制组合。

---


# 第 11 章　沙箱：三个操作系统，一个抽象

**本章任务**：把上一章的 `SandboxPolicy` 变成操作系统无法撤销的限制。本章是全书的招牌章节，因为这里有一个足以解释全部架构选择的题眼：

> **限制必须在 `exec` 之前施加，且不可撤销。**

它会一次性说明：为什么需要独立 helper 二进制，为什么依赖 user namespace，以及为什么“用一个 Rust crate 在库里封一下”从根上做不到。

## 11.1 破直觉：它不是 Docker，也不是一条 `chroot` 调用

看到“沙箱”，最容易想到的是 Docker：`docker run --rm -v $PWD:/work`。但对一个 CLI agent 而言，容器是错的默认选择：

- **需要常驻 daemon 或 root 权限**的运行时，和用户本地的开发环境冲突；
- **启动慢、镜像依赖重**，一次 `cargo check` 都可能被拖成分钟级；
- **安全模型外置**：容器内仍是完整 rootfs，配置错误就形同虚设；
- **无法低成本表达“工作区可写、`.git` 只读”**这类细粒度规则。

Codex 的路线相反：不创建容器，而是直接调用三套平台原生内核机制，用一个统一策略把它们包起来。于是 mini-codex 的安装负担只有“系统里有 bwrap / `sandbox-exec` / 合适的 Windows 能力”，启动只是一次进程派生。

先建立统一抽象。`mcx-sandbox` 不负责业务逻辑，只把 `SandboxRequest` 翻译成平台原语：

```rust
// crates/mcx-sandbox/src/lib.rs
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct WritableRoot {
    pub path: PathBuf,
    /// 该根下重新设为只读的子路径，如 `.git`、`.codex`、解析后的 gitdir
    pub read_only_subpaths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Default)]
pub struct SandboxRequest {
    pub policy: SandboxPolicy,
    pub cwd: PathBuf,
    pub writable_roots: Vec<WritableRoot>,
    pub network: NetworkAccess,
    pub extra_read_paths: Vec<PathBuf>,
    pub command: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkAccess {
    Disabled,
    Permitted,
    Proxied,
}

#[async_trait::async_trait]
pub trait SandboxRunner: Send + Sync {
    async fn spawn(&self, req: &SandboxRequest) -> Result<SandboxedProcess, SandboxError>;
}
```

`SandboxPolicy` 放在 `mcx-protocol`，因为配置和事件都要引用它；平台 runner 放在 `mcx-sandbox`，因为它依赖 `cfg(target_os)`、FFI、`bubblewrap` 发现等重型细节；策略**决策**仍留在 `mcx-core`，避免核心逻辑被平台代码反向污染。这正是 API 契约规定的依赖方向：`mcx-core → mcx-sandbox → mcx-protocol`。

## 11.2 题眼：限制为什么必须在 `exec` 之前、且不可撤销

先看一个看起来很美的“库式沙箱”：

```rust
// 反例：根本不成立
pub fn with_sandbox<F>(policy: SandboxPolicy, f: F)
where F: FnOnce() -> Output
{
    set_no_new_privs();      // 太晚：当前进程已经打开文件、加载动态库
    drop_root();             // 太晚：setuid 已在启动时完成
    restrict_fs(&policy);    // 往往不可能：POSIX 没有“本线程仅可写这些目录”
    f()
}
```

三个词解释它为什么错：**时机、原子性、可撤销性。**

1. **时机**：Rust 程序在 `main()` 之前已经由动态链接器映射了库、打开了 `/dev/urandom`、继承了文件描述符。等 `with_sandbox` 开始执行，攻击者早就能通过已存在的 fd、加载器或环境变量取得控制权。
2. **原子性**：真正的隔离需要把一组操作一次性生效——新 mount namespace、新 user namespace、seccomp、no-new-privs、关闭 fd、重置 env。分开设置会留下“设置一半”的窗口。
3. **可撤销性**：如果限制是普通库函数，目标命令也能调用同一个库撤销它。**安全边界必须位于目标进程无法触达的位置：内核命名空间、exec 不可逆转的过滤器、父进程持有的 cgroup。**

因此架构是固定形态：

```
helper 启动
  → unshare / set up namespaces / apply seccomp
  → 关闭多余 fd、清理 LD_*、设置 PR_SET_DUMPABLE=0
  → execvp(user command)      // ← 此后命令无法撤销前置限制
```

`codex-linux-sandbox` 正是这个独立可执行文件。它不是“为了方便分发”，而是**唯一能把限制放到 exec 之前的位置**。

> **通用形式：凡要求“子进程无法退出安全上下文”的限制，都不能由子进程自己调用库函数实现。** 必须由父进程或 exec 链上游的固定二进制施加。

## 11.3 Linux：bubblewrap 为主，Landlock 退居 legacy

从 v0.115 起，Linux 默认路径是 bubblewrap（bwrap）。它利用 **user namespace + mount namespace** 在普通用户下重塑文件系统，无需 daemon、无需 root、毫秒级启动。

```rust
// crates/mcx-sandbox/src/linux/bwrap.rs
use std::path::PathBuf;

pub struct BwrapCommandBuilder {
    helper: PathBuf,
    args: Vec<String>,
}

impl BwrapCommandBuilder {
    pub fn apply_policy(&mut self, req: &SandboxRequest) {
        // 1) 全盘只读：所有已存在路径的统一基础视图
        self.args.extend(["--ro-bind", "/", "/"]);

        // 2) 可写根：更具体的可写层
        for root in &req.writable_roots {
            let p = root.path.to_string_lossy();
            self.args.extend(["--bind", &p, &p]);

            // 3) 关键：保护路径再次盖回只读
            for ro in &root.read_only_subpaths {
                let s = ro.to_string_lossy();
                self.args.extend(["--ro-bind", &s, &s]);
            }
        }

        // 4) 网络：受限且无代理时隔离网络命名空间
        if req.network == NetworkAccess::Disabled {
            self.args.extend(["--unshare-net", "--unshare-user", "--unshare-pid"]);
        }

        self.args.extend(["--proc", "/proc", "--dev", "/dev"]);
    }
}
```

![bwrap 后挂的更窄规则覆盖更宽规则](assets/fig_bwrap_layers.png)
*图 2：Linux bwrap 的典型挂载顺序。`/repo` 先可写，`.git`/`.codex` 随后以更具体规则重新设为只读。数据来源：OpenAI Codex 上游 `codex-linux-sandbox` 实现说明。*

图中顺序不是装饰，而是 bubblewrap 的语义：**后出现的、更具体的绑定覆盖先前的较宽规则。** 如果先 `--bind /repo/.git`，再 `--bind /repo`，后者会重新让 `.git` 可写；Codex 因此先建全盘只读，再叠加 `WritableRoot`，最后用 `.git`、解析后的 `gitdir:`、`.codex` 的只读绑定“钉死”敏感目录。

这就是为什么 agent 不能改 git hooks：若 `.git/hooks/post-commit` 可写，脚本下一次提交时以用户身份执行，等于 agent 通过 Git 自我提权。`.codex` 同理——它一旦可写，agent 可改自己的配置、放宽规则、关掉 hooks，使整个策略形同虚设。**保护这些路径不是洁癖，而是安全模型闭环的必要条件。**

### 路径特异性：窄路径优先

重叠策略按路径特异性排序，窄者胜出。下列三者可共存：

| 路径 | 规则 |
|---|---|
| `/repo` | write |
| `/repo/a` | none |
| `/repo/a/b` | write |

bwrap 会按特异性排序后生成 mount 序列，因此“先 `--bind /repo`、后 `--ro-bind /repo/a`、再 `--bind /repo/a/b`”不是矛盾，而是精确的黑白名单组合。实现时注意：

- **必须对真实路径规范化后再排序**，符号链接需按最终目标或明确拒绝，避免 `repo/.git` 指向 `/tmp/attacker`；
- **不存在的受保护路径也要声明**：bwrap 可将占位挂载盖在尚未创建的目录上，防止命令先建目录再取得写权；
- **symlink-in-path 必须失败或显式解析**：否则“可写根下的只读子路径”会被软链接绕开。

### bwrap 发现链

Codex 的发现链是工程健壮性的典范：

```rust
// crates/mcx-sandbox/src/linux/discovery.rs
pub fn resolve_bwrap() -> Result<PathBuf, SandboxError> {
    if let Some(p) = which_outside_cwd("bwrap")? {
        if supports_argv0(&p) { return Ok(p); }
        return Ok(compat_no_argv0(p)); // 兼容旧版本：走内层 re-exec
    }
    if let Some(vendored) = find_vendored_bwrap() {
        emit_startup_warning("使用自带的 vendored bwrap；建议安装系统 bubblewrap");
        return Ok(vendored);
    }
    Err(SandboxError::BwrapUnavailable)
}
```

顺序是：

1. **PATH 上、且位于当前目录之外的 `bwrap`**——避免 `./bwrap` 被恶意替换；
2. 存在但**太旧、不支持 `--argv0`**：保留系统 bwrap，切到兼容调用；
3. 缺失：回退到 **vendored bwrap** 并发出启动警告；
4. 连备用方案都没有：明确拒绝，而不是悄悄降级成主机执行。

“警告而非静默”是本书反复强调的可观测原则。用户需要知道正在用哪个后端，否则故障排查会变成猜谜。

### `AF_UNIX` 必须豁免

一个容易让 agent 跑不起来的细节：**网络受限时，必须允许本地 Unix socket。** 构建工具、LSP、IPC、某些包管理器都依赖 `/tmp/*.sock` 或 `/var/run` 的 `AF_UNIX`。若 seccomp 把所有 `socket(AF_UNIX)` 一刀切，命令会在看似无关的调用中失败。

策略应是：**禁止 `AF_INET`/`AF_INET6` 的 connect/bind/sendto，但放行 `AF_UNIX`；再配合 namespace 隔离，避免它访问不该访问的 socket。** 这正是“默认拒绝，但显式放行必要路径”的体现。

### WSL2 可用，WSL1 不可支持

bwrap 需要 user namespace。WSL2 使用真实 Linux 内核，走正常 Linux 路径；WSL1 不具备所需的命名空间能力。因此对 WSL1 应直接拒绝带沙箱的执行：

```rust
pub fn detect_linux_capability() -> LinuxCapability {
    if wsl1_detected() {
        LinuxCapability::Unsupported {
            reason: "WSL1 不能创建 user namespace；请在 WSL2 中运行，或使用 external-sandbox",
        }
    } else if !unshare_user_works() {
        LinuxCapability::Warning {
            reason: "user namespace 不可用；bwrap 无法安全启动",
        }
    } else {
        LinuxCapability::Bwrap
    }
}
```

**拒绝比悄悄主机执行好。** 宁可让命令返回明确错误，也不要让用户以为自己在沙箱里。

### Landlock 为何降级为 legacy

Landlock 是强大的内核机制，但它不是 bwrap 的完全替代品：精细的文件系统规则映射、复杂只读覆盖、动态路径策略、与旧版本兼容，都需要更多工程。Codex 把它保留为显式 fallback：

```toml
[features]
use_legacy_landlock = true
```

启用条件是**拆分策略能等价映射回旧模型**；若策略使用了只有 bwrap 能表达的嵌套例外，就继续走 bwrap，不为了“用新 API”牺牲精确性。这个取舍说明一个深层原则：**安全代码的价值不在于用了最新 syscall，而在于约束是否准确且可验证。**

## 11.4 macOS：动态生成 SBPL，而不是写死 profile

macOS 没有 bubblewrap，但有 Seatbelt。`sandbox-exec -p <sbpl>` 把策略编译进命令的安全上下文。我们的 runner 不维护一个大而全的静态 profile，而是**按请求参数生成 SBPL**：

```rust
// crates/mcx-sandbox/src/macos/seatbelt.rs
pub fn build_sbpl(req: &SandboxRequest) -> String {
    let mut p = String::from("(version 1)\n(deny default)\n");
    p.push_str("(allow process-fork process-exec)\n");
    p.push_str("(allow file-read* (subpath \"/\"))\n"); // 默认全盘读

    for root in &req.writable_roots {
        let s = esc(root.path.to_string_lossy());
        p.push_str(&format!("(allow file-write* (subpath \"{s}\"))\n"));
        for ro in &root.read_only_subpaths {
            let r = esc(ro.to_string_lossy());
            p.push_str(&format!("(deny file-write* (subpath \"{r}\"))\n"));
        }
    }
    if req.network == NetworkAccess::Disabled {
        p.push_str("(deny network*)\n");
    }
    // 允许必要基础能力
    p.push_str("(allow sysctl-read)\n(allow signal (target self))");
    p
}
```

**`.git` 和 `.codex` 作为参数化只读子路径注入**是关键。若把 profile 硬编码成“允许 `/Users/me/project”，攻击者控制的子目录规则就难以审计；把当前 `cwd`、配置根、gitdir 作为参数传给生成器，才能在每次执行前断言：“这个具体路径必须只读”。

Seatbelt 是**默认拒绝、显式放行**的系统，因此规则顺序与 bwrap 一样重要：`file-read* (subpath "/")` 先给出基础读，随后为可写根放开写，最后把敏感子路径 `deny`。生成的 profile 应当打印到调试日志，让用户能用 `codex sandbox macos --log-denials -- ls` 复现。

macOS 的边界也要诚实说明：Seatbelt 策略通过环境变量和进程上下文传递，**目标是开发工具隔离，不是抵御本机恶意软件**。完整安全还依赖没有不受信任代码已在本机长期运行、用户凭证不被环境变量泄露等前提。

## 11.5 Windows：受限令牌、ACL 与环境塑形

Windows 不保证 POSIX 风格的一刀切内核沙箱。Codex 用三件事组合：

1. **受限令牌（restricted token）**：从当前用户令牌派生一个权限更少、组更少、完整性更低的令牌，`CreateProcessAsUser` 启动命令；
2. **ACL / 路径塑形**：对可写根授予写 ACL，对 `.git`、`.codex` 等保持只读，对其他路径仅保留读；
3. **环境隔离**：清空或重写 `HTTP_PROXY`、`HTTPS_PROXY`、`NO_PROXY`，删掉 `SSH_AUTH_SOCK` 等敏感变量，并可选地用 PATH stub。

```rust
// crates/mcx-sandbox/src/windows/token.rs（教学版骨架）
pub struct RestrictedLauncher {
    pub writable_roots: Vec<PathBuf>,
    pub read_only_roots: Vec<PathBuf>,
    pub network_policy: NetworkAccess,
}

impl RestrictedLauncher {
    pub fn prepare(&self, req: &SandboxRequest) -> Result<PreparedCommand, SandboxError> {
        let token = self.create_restricted_token()?;
        apply_path_acls(&token, &req.writable_roots, &req.read_only_roots)?;
        let env = shape_environment(req, /* drop secrets */ true);
        Ok(PreparedCommand { token, env, ..Default::default() })
    }
}
```

**必须诚实：Windows 的网络阻断通常不是内核级过滤。** 它可能是代理环境变量、假 PATH、假 `curl` 封装或防火墙规则。若命令绕过环境、直接连接 IP、使用另一条代理链，限制就可能失效。因此 Windows 上更强烈推荐：

- **优先 WSL2**，继承 Linux bwrap 语义；
- 或在**受管理的执行器**中使用，由外部防火墙/网络策略兜底；
- 永远不要宣称“Windows 环境变量隔离 = 网络绝对不可达”。

这也是“三个操作系统，一个抽象”的真实代价：抽象层可以统一 API，却不能把平台保证吹成一致。不同 backend 的 `Unsupported`/`Warning` 必须暴露给 CLI。

## 11.6 进程加固：在 `main()` 之前消灭继承攻击面

即使命令还没 exec，主进程或 helper 也可能继承危险状态。Codex 用独立 crate `codex-process-hardening`，借助 `#[ctor::ctor]` 在 `main()` 之前执行：

```rust
// crates/mcx-process-hardening/src/lib.rs
use ctor::ctor;

#[ctor]
unsafe fn harden_before_main() {
    // 1) 禁止 core dump，减少内存镜像泄露凭证
    set_rlimit_core_zero();
    // 2) 禁止 ptrace/调试器附加
    set_no_dumpable();
    // 3) 剥离动态链接器注入变量
    remove_ld_vars();
    // 4) 关闭不必要的继承 fd（具体平台实现）
    close_inherited_fds();
}
```

`#[ctor::ctor]` 不是日常 Rust，但在安全工具里恰好正确：**目标是在任何用户代码、任何 `lazy_static`、任何测试框架运行前收紧进程。** 风险也要坦白：ctor 的顺序未定义，不能与依赖初始化的代码抢跑；因此该 crate 只用同步 syscall，不触碰 allocator、日志或网络。

第 7 章的 `kill_on_drop` 负责子进程；本章的 helper 负责 **exec 前原子限制**；两者是“生命周期边界”和“权限边界”的互补，不是替代关系。

## 11.7 统一 runner 与四条诚实边界

把平台细节藏在一个构造函数后：

```rust
// crates/mcx-sandbox/src/lib.rs
pub fn make_runner() -> Result<Box<dyn SandboxRunner>, SandboxError> {
    #[cfg(target_os = "linux")]   { return LinuxRunner::detect().map(to_box); }
    #[cfg(target_os = "macos")]   { return Ok(Box::new(MacOSRunner::new())); }
    #[cfg(target_os = "windows")] { return Ok(Box::new(WindowsRunner::new())); }
    Err(SandboxError::UnsupportedPlatform)
}
```

调用方只看到 `spawn(&SandboxRequest)`。无论 Linux、macOS 还是 Windows，`Session` 都不写平台 `if`。

但**沙箱不是绝对真理**。本章必须写下四条边界：

1. **全盘读访问通常恒为 true。** 为让编译器、LSP、包管理器可用，agent 默认可读 `/usr`、`/etc`、注册表等大量主机状态。只读不等于无害：配置文件可泄露 token、构建缓存可泄露源码。
2. **沙箱检测是启发式的。** bwrap 可用性、WSL 版本、Seatbelt 可用性、Windows token 权限，都可能因环境变化而失败；我们显式拒绝或警告，而非默默降级。
3. **`ExternalSandbox` 绕过本地强制。** 这是设计，不是漏洞：它把强制责任交给外层。CLI 必须清楚标注“当前不施加本地限制”，并把网络状态传给工具。
4. **升级流程是刻意设计。** 批准后离开沙箱是有意 escape，必须有审计、TTL、作用域，不能把 escalation 伪装成普通重试。

> **诚实的威胁模型比“宣称绝对安全”更有价值。** 如果用户知道 Windows 网络是环境级、知道全盘读默认开放，就能把 mini-codex 放进更强外层；如果文档假装万事大吉，部署就会在错误的地方失败。

## 11.8 测试：真实进程，明确断言

平台代码离不开 OS，但策略小单元仍可纯测试。这里给出可在 Linux CI 跑的集成测试骨架：

```rust
// crates/mcx-sandbox/tests/linux.rs
use mcx_sandbox::{SandboxRequest, WritableRoot, SandboxPolicy, NetworkAccess};
use std::path::PathBuf;

fn req_with_git(tmp: &std::path::Path) -> SandboxRequest {
    SandboxRequest {
        policy: SandboxPolicy::WorkspaceWrite,
        cwd: tmp.to_path_buf(),
        writable_roots: vec![WritableRoot {
            path: tmp.to_path_buf(),
            read_only_subpaths: vec![tmp.join(".git"), tmp.join(".codex")],
        }],
        network: NetworkAccess::Disabled,
        ..Default::default()
    }
}

#[tokio::test]
#[cfg(target_os = "linux")]
async fn workspace_writable_but_git_remains_read_only() {
    let tmp = tempdir();
    std::fs::write(tmp.path().join("ok.txt"), b"x").unwrap();
    std::fs::create_dir_all(tmp.path().join(".git")).unwrap();

    let runner = mcx_sandbox::make_runner().unwrap();
    let status = runner
        .spawn(&req_with_git(tmp.path()))
        .await
        .unwrap()
        .wait_for_exit()
        .await
        .unwrap();

    // 试图写 .git/HEAD 应被拒绝（denial / 权限错误）
    assert!(status.exit_code != 0 || status.stderr.contains("denied"),
            "agent 不应能改写 .git");
}
```

更完整的测试应按平台 `cfg`：

- Linux：写 `/tmp`/CWD 成功，写 `/etc` 失败；受限网络下连接回环 socket 成功、`AF_INET` 失败。
- macOS：用 `sandbox-exec` 生成 profile；`.git`/`.codex` 为参数化只读子路径。
- Windows：受限令牌成功创建；网络隔离在明确“环境级”的测试标记下运行。

无法跨平台取得同等保证时，**不要伪造一致性**。用 `cfg` 和 `#[ignore]` 明确标注平台依赖，比用一个假的 mock runner 掩盖差异更诚实。

## 避坑专栏 #12：库函数“沙箱”总能在子进程里被撤销

**错误写法**是让工具直接调用一个 `set_sandbox()` 函数：

```rust
// 危险：限制施加在自身，且可由后续代码撤销
fn run(cmd: &str) {
    set_rlimit(); set_no_new_privs();
    std::process::Command::new("sh").arg("-c").arg(cmd).spawn().unwrap();
}
```

问题在于：`set_no_new_privs` 对已有进程有效，但子进程还能 `exec` setuid、仍继承 fd、仍受动态链接器影响；若限制是普通 Rust API，被执行的命令也可能链接同库撤销它。

**正确结构是独立 helper 二进制 + exec 链**：父进程先做完 namespace、mount、seccomp、fd 清理、env 剥离，再 `execvp` 目标命令。目标进程从出生就在受限上下文里，不能回到之前状态。

**通用形式：不可撤销的安全边界 = 内核机制 + exec 前一次性施加。**

## 11.9 Design Rationale

**Q：为什么不用 Docker 做默认隔离？**

因为容器解决的是“环境可复现”，不是“用户本机毫秒级命令隔离”。它需要镜像、daemon、权限、网络配置，且无法天然表达“只读 `.git`、可写工作区”。bwrap/user namespace 直接满足 agent 的精细文件系统和快速派生需求。

**Q：为什么一定要独立 helper，而不是库 crate？**

因为限制点只有一个：`exec` 之前。库函数无法保证“在所有语言运行时初始化之后、目标命令执行之前”的原子顺序；helper 二进制把顺序变成进程启动协议。此外，helper 可由 setuid/文件 capability 管理，而库调用无法把权限收回。

**Q：为什么全盘读默认允许？**

因为彻底只读会导致编译器、包管理器、共享缓存全部不可用，agent 的实用性归零。安全模型因此选择“**读面广、写面窄、网络明确、敏感目录钉死**”。这也意味着凭证防护不能只靠沙箱：还需 env 白名单、secret scanning、外层网络策略。

## AI 软件工程原理 #11

> **沙箱是默认开启的，不是可选增强。**

若安全必须用户主动记得开启，真实运行就会长期停留在“关着更方便”的状态。因此：未配置时应默认 `read-only` 或 `workspace-write`；无法建立隔离时应**拒绝执行**而不是静默主机运行；escalation 必须显式确认；网络默认关闭，写范围默认最小。

默认开启还意味着可观测：每次启动打印 backend、版本、writable roots、network 状态。安全机制若安静失败，等于不存在。第 7 章的时间/输出/进程/环境边界是工具层的护栏；本章的 namespace、seccomp、Seatbelt、restricted token 是操作系统层的护栏——**两者共同默认生效，agent 才可交付。**

## Rust 修炼小结

| 概念 | 本章用法 | 后面在哪用到 |
|---|---|---|
| `#[cfg(target_os)]` | 三平台 runner 隔离 | 第 18 章平台工具 |
| `#[ctor::ctor]` | main 前进程加固 | 启动期安全初始化 |
| `execvp`/helper 二进制 | exec 前原子施加限制 | CLI 子命令 |
| trait object runner | 统一 `SandboxRequest` | 第 12 章策略评估 |
| 平台条件测试 | Linux/macOS/Windows 各有断言 | CI 矩阵 |

## 章末验收

- [ ] `workspace-write` 下，写 `/tmp`/CWD 成功，写其他路径失败
- [ ] 网络受限时，`AF_UNIX` 本地 socket 可用，`AF_INET` 外连失败
- [ ] `.git`、`.codex` 即使在可写根内仍不可写
- [ ] bwrap 不可用时打印警告并拒绝，不静默退化为主机执行
- [ ] Linux/macOS/Windows 至少各有 1 条集成测试；不支持环境明确返回 `Unsupported`

## 读者挑战

1. 设计一个“符号链接绕过”测试用例：`/repo/.codex -> /tmp/evil`。**怎样在施加挂载前解析并拒绝？**
2. bwrap 需要 user namespace；某些 hardened 容器禁止它。**如何检测并优雅降级为 `ExternalSandbox`，同时不谎报安全等级？**
3. Windows 网络隔离依赖环境变量。**请列出至少三种绕过方式，并为每一种设计可观测的防御层。**

## 下一章预告：规则的价值，在被执行之前

沙箱解决了“最坏能坏到哪”，审批解决了“要不要问人”，但项目还需要“`rm -rf` 永远禁止、`gh pr view` 先提示”这类可复用规则。第 12 章将把 `SafetyRule` 工业化：声明式 `execpolicy`、离线检查、PreToolUse/PostToolUse hooks，以及项目信任模型。规则的价值不只是被执行，更是能被提前询问。

---


# 第 12 章　execpolicy 与 hooks：声明式规则与生命周期拦截

**本章任务**：把“哪些命令允许、提示、禁止”外置成可测试的规则文件，再在工具调用前后插入可信的 `PreToolUse`/`PostToolUse` 钩子。第 1 章的 `SafetyRule` 只是一个类型；本章让它成为可组合、可离线评估、可审计的策略系统。

## 12.1 先写一个事后才发现问题的策略

很多项目的第一版安全规则长这样：

```rust
// 反例：散落在调用点，无法离线验证
async fn run(cmd: &str) -> ToolOutput {
    if cmd.contains("rm -rf") {
        return ToolOutput::denied("禁止 rm -rf");
    }
    if cmd.starts_with("gh pr view") && !user_approved("gh") {
        ask_user(); // 同步询问，混在业务里
    }
    run_shell(cmd).await
}
```

它有三个致命问题：**规则藏在函数里，无法列出**；**只有真正执行时才知后果**，CI 不能预检；**正则散落各处**，团队无法评审、无法版本化、无法复用。

正确目标是：把“命令会怎样”从“命令有没有跑”中拆出来。第 1 章的 `SafetyRule` 此时应升级为：

```rust
// crates/mcx-core/src/execpolicy.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleEffect {
    Allow,
    Prompt,
    Forbidden,
}

#[derive(Debug, Clone)]
pub struct SafetyRule {
    pub id: String,
    pub description: String,
    pub pattern: Vec<PatternToken>,
    pub effect: RuleEffect,
    pub justification: String,
    /// 自测样例：必须/不得匹配
    pub examples: RuleExamples,
}
```

这正是第 1 章 `SafetyRule` 的“工业化版本”：从“有这样一个结构体”变成“有 ID、有理由、有样例、可合并、可询问”。

## 12.2 `execpolicy check`：规则必须能被离线询问

核心 API 是纯评估：

```rust
// crates/mcx-core/src/execpolicy.rs
#[derive(Debug, Clone, Default)]
pub struct EvalResult {
    pub effect: RuleEffect,
    pub matched: Vec<RuleMatch>,
}

#[derive(Debug, Clone)]
pub struct RuleMatch {
    pub rule_id: String,
    pub justification: String,
    pub severity: Severity,
}

pub fn evaluate(rules: &[SafetyRule], command: &CommandView) -> EvalResult {
    let mut matched = Vec::new();
    let mut effect = RuleEffect::Allow;

    for rule in rules {
        if rule.matches(command) {
            matched.push(RuleMatch::from(rule));
            effect = effect.combine(rule.effect);
        }
    }
    EvalResult { effect, matched }
}
```

合并语义采用**最严格规则优先**：`Forbidden > Prompt > Allow`。这避免“一条宽松规则盖过一条严格规则”。若命中存在矛盾的规则，诊断信息必须同时列出二者，而不是悄悄选一个。

```rust
impl RuleEffect {
    fn combine(self, other: RuleEffect) -> RuleEffect {
        use RuleEffect::*;
        let order = |e: RuleEffect| match e {
            Forbidden => 3, Prompt => 2, Allow => 1,
        };
        if order(self) >= order(other) { self } else { other }
    }
}
```

CLI 子命令让用户在跑 agent 前自检：

```rust
// crates/mcx-cli/src/execpolicy_cmd.rs
async fn check_command(rules: &Path, command: &[String]) -> Result<(), CliError> {
    let rules = load_rules(rules)?;
    let view = CommandView::from_argv(command);
    let result = evaluate(&rules, &view);

    let out = serde_json::json!({
        "effect": result.effect,
        "matched": result.matched,
        "command": command,
    });
    println!("{}", serde_json::to_string_pretty(&out)?);

    if result.effect == RuleEffect::Forbidden {
        std::process::exit(2);
    }
    Ok(())
}
```

```bash
$ mini-codex execpolicy check --rules .codex/rules/base.rules -- rm -rf /
{
  "effect": "forbidden",
  "matched": [
    {
      "rule_id": "no-destructive-rm",
      "justification": "递归删除可能摧毁工作区外文件；请改用受控清理工具",
      "severity": "high"
    }
  ],
  "command": ["rm", "-rf", "/"]
}
```

**这就是“规则要能被询问”的落地。** CI 可以把 `execpolicy check` 放在 pre-commit，项目维护者可以验证新增规则，agent 也能在执行前展示同一结果。安全策略若只能等事故时被动触发，就无法被工程化采纳。

## 12.3 规则文件、匹配与样例自测

规则格式采用 TOML，比内嵌 JSON 更适合人类评审；解析后归一化为 `SafetyRule`。

```toml
# .codex/rules/base.rules（Starlark 风格伪代码对应到可执行 TOML）
[[rule]]
id = "read-only-git-inspect"
description = "只读 Git 检视可放行"
pattern = ["git", ["status", "diff", "log", "show"]]
effect = "allow"
justification = "不修改仓库元数据"
match = ["git status", "git log --oneline"]
not_match = ["git push", "git config user.email evil@x"]

[[rule]]
id = "no-destructive-rm"
description = "禁止递归删除"
pattern = ["rm", ["-rf", "-fr", "-r", "-f"]]
effect = "forbidden"
justification = "使用受控清理工具；破坏性操作需要明确路径白名单"
match = ["rm -rf node_modules", "rm -fr /"]
not_match = ["rm -r .tmp-build", "rm -f Cargo.lock.bak"]

[[rule]]
id = "gh-read-needs-prompt"
description = "读取远端仓库信息前先询问用户"
pattern = ["gh", ["pr", "issue", "repo"], ["view", "diff"]]
effect = "prompt"
justification = "gh 会向外部网络发请求并展示远端他人代码，先问用户再执行"
match = ["gh pr view 7888", "gh pr diff 123"]
not_match = ["gh auth status", "gh version"]
```

第三条的 `effect = "prompt"` 把“读取远端”放进询问档：这样 12.8 章末验收 `gh pr view 7888` 才会命中规则并输出 prompt 提示，而不是落入默认 `Allow` 后静默放行。

`PatternToken` 支持字面量和“该位置可选集”：

```rust
// crates/mcx-core/src/execpolicy.rs
#[derive(Debug, Clone)]
pub enum PatternToken {
    Literal(String),
    OneOf(Vec<String>),
}

impl SafetyRule {
    fn matches(&self, cmd: &CommandView) -> bool {
        let argv: Vec<&str> = cmd.argv.iter().map(String::as_str).collect();
        if self.pattern.len() > argv.len() { return false; }
        for (tok, actual) in self.pattern.iter().zip(&argv) {
            if !tok.matches(actual) { return false; }
        }
        true
    }
}

impl PatternToken {
    fn matches(&self, s: &str) -> bool {
        match self {
            PatternToken::Literal(l) => l == s,
            PatternToken::OneOf(v) => v.iter().any(|x| x == s),
        }
    }
}
```

> **规则评估必须在规范化后的 argv 上做，而不是原始命令行字符串。** `shell` 传入 `command` 后要先解析 `sh -c`，`apply_patch` 则直接用结构化参数构造 `CommandView`。否则 `rm -rf /` 可被空格、引号、变量、`bash -lc` 绕过。规则是安全层，不是文本过滤。

加载时立即校验 `match`/`not_match`：

```rust
impl SafetyRule {
    pub fn validate(&self) -> Result<(), RuleError> {
        for example in &self.examples.must_match {
            if !self.matches(&CommandView::from_argv(example)) {
                return Err(RuleError::ExampleMismatch {
                    rule: self.id.clone(),
                    example: example.join(" "),
                });
            }
        }
        for counter in &self.examples.must_not_match {
            if self.matches(&CommandView::from_argv(counter)) {
                return Err(RuleError::CounterexampleMatched {
                    rule: self.id.clone(),
                    counter: counter.join(" "),
                });
            }
        }
        Ok(())
    }
}
```

这条校验极其重要：**规则文件在加载时就自证。** “`sudo` 禁止但 `sudoku` 不应命中”这类错误，会在配置启动阶段而非某次执行时暴露。

```toml
# 测试样例必须随规则一起存在
[examples]
match = ["sudo rm x", "sudoedit /etc/hosts"]
not_match = ["sudoku solve", "git status"]
```

## 12.4 把规则接入工具循环

第 10 章的 `Decision` 现在可以由 `execpolicy` 参与构造：

```rust
// crates/mcx-core/src/execution.rs
pub fn policy_decision(
    approval: ApprovalPolicy, rules: &[SafetyRule], cmd: &CommandView,
) -> Decision {
    let result = evaluate(rules, cmd);
    if result.effect == RuleEffect::Forbidden {
        return Decision::Deny;
    }
    if result.effect == RuleEffect::Prompt {
        // 规则要求提示；审批策略决定是否真的询问
        return approval.decide_prompt(cmd);
    }
    approval.base_decision(cmd)
}
```

接入第 9 章的 `Session::execute_call`：

```rust
// crates/mcx-core/src/session.rs（节选，沿用第 6 章 Registry）
async fn execute_call(&self, call: &ToolCall) -> ToolOutput {
    let view = CommandView::from_tool_call(call);
    let decision = policy_decision(self.policy.approval, &self.policy.rules, &view);

    match decision {
        Decision::Deny => {
            self.emit_audit(call, &view, "denied-by-execpolicy");
            return ToolOutput::denied(rule_summary(&view));
        }
        Decision::Ask => return request_approval(self, call).await,
        Decision::AllowInSandbox => {
            return run_sandboxed(self, call, view).await.into_output();
        }
        Decision::EscalateOutOfSandbox => {
            return request_escalation(self, call).await;
        }
        Decision::Allow => return run_unsandboxed(self, call).await,
    }
}
```

注意顺序：**先 `execpolicy`（声明式、确定、可审计），再审批（人的价值判断），再 sandbox（OS 机制），最后 PreToolUse hooks（项目生命周期）。** 拒绝不应等到 hook；hook 负责的是“在放行前后增加检查/动作”，不是主决策。

## 12.5 PreToolUse / PostToolUse：可信生命周期拦截

hooks 配置从 `hooks.json` 或 `config.toml` 加载：

```toml
# .codex/hooks.toml
[[hooks.PreToolUse]]
matcher = "^shell$"
hooks = [
  { type = "command", command = "/usr/bin/python3 .codex/hooks/block_curl.py",
    timeout = 5, status_message = "检查 shell 命令是否访问密钥" }
]

[[hooks.PostToolUse]]
matcher = "apply_patch"
hooks = [
  { type = "command", command = "cargo check --message-format=json",
    timeout = 120, status_message = "校验补丁可编译" }
]
```

```rust
// crates/mcx-core/src/hooks.rs
#[derive(Debug, Clone)]
pub struct Hook {
    pub event: HookEvent,
    pub matcher: regex::Regex,
    pub command: String,
    pub timeout: Duration,
    pub status_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookEvent { PreToolUse, PostToolUse }

#[derive(Debug, Clone)]
pub struct HookInput {
    pub tool: String,
    pub call_id: String,
    pub arguments: serde_json::Value,
    pub result: Option<ToolOutput>,
}

#[derive(Debug, Clone)]
pub enum HookVerdict {
    Continue,
    Block(String), // 仅 PreToolUse 有效
}
```

失败路径归 `HookError`——它只描述 **hook 机制本身坏了**（超时、被取消、底层进程错误），与 `Block`（hook 的业务判决）严格分开：

```rust
#[derive(Debug, thiserror::Error)]
pub enum HookError {
    #[error("hook 执行超时: {command}")]
    Timeout { command: String },
    #[error("hook 被用户取消")]
    Cancelled,
    #[error("hook 底层进程错误: {0}")]
    Io(#[from] std::io::Error),
}
```

PreToolUse 在执行前运行；任一 hook 返回 block 即中止。PostToolUse 在工具成功后运行，可用于编译检查、静态扫描、通知，但**默认不修改已有结果**，避免把“观察”伪装成“拦截”。

```rust
pub async fn run_pre_hooks(
    hooks: &[Hook], input: &HookInput, cancel: &CancellationToken,
) -> Result<HookVerdict, HookError> {
    for hook in hooks.iter().filter(|h| h.event == HookEvent::PreToolUse)
                       .filter(|h| h.matcher.is_match(&input.tool))
    {
        let verdict = run_one(hook, input, cancel).await?;
        if matches!(verdict, HookVerdict::Block(_)) {
            return Ok(verdict);
        }
    }
    Ok(HookVerdict::Continue)
}

async fn run_one(hook: &Hook, input: &HookInput, cancel: &CancellationToken)
    -> Result<HookVerdict, HookError>
{
    let mut child = tokio::process::Command::new("sh")
        .arg("-c").arg(&hook.command)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    write_json_stdin(child.stdin.as_mut().unwrap(), input).await?;

    let output = tokio::select! {
        r = child.wait_with_output() => r?,
        _ = tokio::time::sleep(hook.timeout) => {
            child.kill().await.ok();
            return Err(HookError::Timeout { command: hook.command.clone() });
        }
        _ = cancel.cancelled() => {
            child.kill().await.ok();
            return Err(HookError::Cancelled);
        }
    };

    if output.status.success() {
        Ok(parse_verdict(&output.stdout))
    } else {
        Ok(HookVerdict::Block(format!("hook 拒绝：{}", String::from_utf8_lossy(&output.stderr))))
    }
}
```

三个细节决定健壮性：

- **超时不可缺**：hook 跑慢了不能拖死整个 turn；默认合理上限，SessionEnd 类事件更短。注意超时语义：**引擎主动 kill hook 并上报 `HookError::Timeout`，这不是 hook 给出的 `Block` 判决**——上层要把“hook 拖死被引擎杀”和“hook 明确拒绝”当成两条不同的失败路径。
- **取消信号传递**：用户 Ctrl+C 必须能终止 hook，复用第 7 章的 `CancellationToken`。
- **stdin 传结构化事件**：hook 读取 `tool`、`arguments`、`result`，不靠环境变量猜测。

## 12.6 信任模型：项目级配置必须“受信任”才加载

这是最容易让“声明式安全”自我否定的地方：如果项目仓库里的 hooks 在 clone 后自动执行，攻击者只需提交一个恶意 `.codex/hooks.toml`，就会在用户首次运行时被 `run_one` 直接执行——**规则文件本身成了 RCE 载体。**

解法不是“禁用 hooks”，而是把信任边界显式化：

```rust
// crates/mcx-core/src/trust.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrustLevel { Trusted, Untrusted }

pub struct TrustStore {
    pub projects: std::collections::HashMap<PathBuf, TrustLevel>,
}

impl TrustStore {
    pub fn load_project_policy(&self, project_root: &Path)
        -> Result<ProjectPolicy, PolicyError>
    {
        let trusted = self.projects.get(project_root)
            .copied() == Some(TrustLevel::Trusted);

        if !trusted {
            // 仅加载用户全局规则；项目 hooks/规则需明确批准
            return Ok(ProjectPolicy::user_only());
        }
        Ok(ProjectPolicy::load_from(project_root.join(".codex"))?)
    }
}
```

规则：

1. **用户全局规则默认加载**；项目级 `.codex/rules`、`hooks.toml` 只在项目标记为 `Trusted` 时加载。
2. **首次进入项目时显示差异**：本次新增/修改了哪些 hook、规则、可执行命令；批准后才写入 trust store。
3. **hooks 的 `command` 必须可审计**：列出绝对路径或已注册脚本，避免 `$PATH` 劫持。
4. **失败默认关闭**：hook 超时、解析错误、找不到脚本时，按“安全侧”处理——PreToolUse 默认阻止，PostToolUse 默认只记录，具体策略可配置。

> **配置即代码，不等于配置即可信。** 第 11 章保护操作系统边界；本节保护“谁有权给 agent 增加可执行代码”。两者缺一，声明式安全都会被信任链反噬。

## 12.7 把整套链路连起来

最终的执行顺序是确定状态机：

```
CommandView
  │
  ├─ 1. execpolicy evaluate      → Forbidden / Prompt / Allow
  │
  ├─ 2. ApprovalPolicy           → Deny / Ask / Allow / Escalate
  │
  ├─ 3. SandboxPolicy runner     → read-only / workspace-write / external / full
  │
  ├─ 4. PreToolUse hooks         → Continue / Block
  │
  ├─ 5. 真实工具执行（shell / apply_patch / …）
  │
  └─ 6. PostToolUse hooks        → 观察、编译检查、审计；默认不改变结果
```

```rust
pub async fn execute_tool(ctx: &ExecutionContext, call: &ToolCall)
    -> ToolOutput
{
    let view = CommandView::from_tool_call(call);

    // 1+2：声明式规则 + 审批
    let decision = policy_decision(ctx.policy.approval, &ctx.policy.rules, &view);
    let decision = match decision {
        Decision::Deny => return ToolOutput::denied(rule_summary(&view)),
        Decision::Ask => request_approval(ctx, call).await,
        other => other,
    };

    // 3：OS 边界
    let sandbox = match decision {
        Decision::AllowInSandbox => Some(prepare_sandbox(ctx, call, &view).await),
        _ => None,
    };

    // 4：PreToolUse（信任模型已过滤项目 hook）
    if let Some(bind) = pre_hooks_block(ctx, call).await {
        return bind;
    }

    // 5：真实执行
    let mut output = spawn_with(ctx, call, sandbox).await;

    // 6：PostToolUse；错误默认不覆盖主结果
    if let Err(e) = run_post_hooks(ctx, call, &output).await {
        ctx.audit.record_hook_failure(call, &e);
    }
    ctx.audit.record(call, &view, &output);
    output
}
```

`Record` 与 `Event::ApprovalRecord` 让整个链可回放：命中的规则 ID、审批决定、sandbox backend、hook 输出全部结构化落盘。原理 #3“事件流是可判定的真相来源”在此达到完整闭环——你可以写一个测试，断言“某次运行中 `rm -rf` 被 `no-destructive-rm` 拒绝，且未执行任何 PreToolUse hook 后的命令”。

## 12.8 测试：离线评估与超时隔离

`execpolicy` 的优势是可纯测试：

```rust
// crates/mcx-core/src/execpolicy/tests.rs
#[test]
fn forbidden_wins_over_allow_and_prompt() {
    let rules = vec![
        SafetyRule::allow(vec!["git", "push"]),
        SafetyRule::prompt(vec!["git", OneOf(vec!["push".into()])]),
        SafetyRule::forbidden(vec!["git", "push"]),
    ];
    let result = evaluate(&rules, &cmd("git", &["push", "origin"]));
    assert_eq!(result.effect, RuleEffect::Forbidden);
    assert!(result.matched.iter().any(|m| m.rule_id == "forbid-git-push"));
}
```

```rust
#[test]
fn rule_examples_are_validated_at_load_time() {
    let toml = r#"
      [[rule]]
      id = "no-sudo"
      pattern = ["sudo"]
      effect = "forbidden"
      match = ["sudo rm x"]
      not_match = ["sudo", "ku"]   # 错误：应为 "sudoku"，分词不符
    "#;
    let err = load_rules_str(toml).unwrap_err();
    assert!(format!("{err}").contains("反例被匹配"));
}
```

hooks 用超时测试防止 turn 被拖死：

```rust
#[tokio::test]
async fn slow_hook_is_killed_and_does_not_deadlock_turn() {
    let hook = Hook {
        event: HookEvent::PreToolUse,
        matcher: regex::Regex::new("^shell$").unwrap(),
        command: "sleep 10".into(),
        timeout: Duration::from_millis(100),
        status_message: None,
    };
    let input = HookInput::shell("echo hi");

    // 超时不是 hook 给出的 Block 判决，而是引擎级错误：
    // run_one 在超时分支 kill 掉子进程并返回 Err(HookError::Timeout)
    let started = std::time::Instant::now();
    let err = run_one(&hook, &input, &CancellationToken::new()).await.unwrap_err();
    assert!(matches!(err, HookError::Timeout { .. }),
        "超时后应返回 Err(HookError::Timeout)，不是 HookVerdict::Block");
    assert!(started.elapsed() < Duration::from_secs(2),
        "hook 必须在上限附近被 kill，不能真等 sleep 10 跑完拖死 turn");
}
```

```toml
# 章末验收可直接调用
[checks]
must_pass = [
  "mini-codex execpolicy check --rules .codex/rules/base.rules -- rm -rf /",
  "mini-codex execpolicy check --rules .codex/rules/base.rules -- gh pr view 7888"
]
```

`--pretty` 输出应明确：最终 `effect`、所有命中规则、每条规则的 `justification`。对 `forbidden`，退出码应非零，方便 CI 阻断。

## 避坑专栏 #13：项目 hook 自动加载 = 自动 RCE

**错误写法**是 `Repository::load(".codex/hooks.toml")` 后立即 `tokio::process::Command::new(hook.command)`。现象是：用户刚 clone 一个看起来正常的项目，mini-codex 首次运行就执行仓库里的任意脚本。

**解法**是信任链：项目配置默认不加载；首次发现时打印 diff 并要求明确批准；批准后把项目根写入 trust store；加载期校验脚本路径、超时、参数模板，禁止未声明变量。用户全局 hooks 与项目 hooks 分层。

**通用形式：凡是“来自不可信输入的代码/命令/序列化器”，都必须有一个比加载更靠前的信任决策。**

## 12.9 Design Rationale

**Q：为什么不直接把规则写进 Rust 的 `match`？**

因为规则是配置，不是引擎。项目维护者、企业管理员、CI 都需要增加例外而不重编译；规则还要能被离线评估、被 PR 审查、被样例自测。Rust 代码适合实现 evaluator 和合并语义，不适合承载“允许 `gh pr view`”这种业务策略。

**Q：为什么要有 `execpolicy check`，而不等 agent 执行时再判断？**

因为安全策略的价值一半在于**预测性**。用户要能在改规则前知道“这条命令会怎样”，CI 要在提交前拒绝危险规则，agent 要在 UI 展示命中原因。只有被执行时才报错，等于把策略验证成本转嫁给事故。

**Q：为什么 PostToolUse 默认不改变工具结果？**

因为后置观察的职责是审计、校验、通知。若 hook 随意改写 `ToolOutput`，模型看到的就不是真实执行结果，事件流真实性被破坏，回放也就失效。需要修改结果时应使用专门的、显式批准的 transformer，而不是普通 hook。

## AI 软件工程原理 #12

> **规则要能被询问，而不只是被执行。**

可执行不等于可管理。声明式 `execpolicy` 把“会怎样”变成可离线计算的值；`match`/`not_match` 把规则正确性变成启动期检查；`execpolicy check` 把它变成 CI 门禁；命中原因把它变成用户看得懂的解释。可解释、可预测、可回放，才是安全机制能被团队持续采用的前提。

这条原理是原理 #10 的延续：第 10 章把价值判断外置成配置，第 12 章把配置变成可询问的对象；第 11 章的机制则保证即使配置被误判，OS 层仍有不可撤销底线。**配置管决策、机制管执行、审计管事后验证**——三者缺一，安全系统就会在某次变更中悄悄退化。

## Rust 修炼小结

| 概念 | 本章用法 | 后面在哪用到 |
|---|---|---|
| `serde`/`toml` | 规则与 hook 配置 | 第 16 章配置版本化 |
| 正则 matcher | 工具名/事件过滤 | 第 18 章 MCP 过滤 |
| 纯函数 evaluator | `execpolicy check` | 第 20 章策略回归 |
| `tokio::select!` | hook 超时 + 取消 | 第 7 章工具超时 |
| 信任存储 | 项目配置加载门槛 | 第 15 章工作区信任 |

## 章末验收

- [ ] `mini-codex execpolicy check "rm -rf /"` 输出明确拒绝，并列出 `no-destructive-rm`
- [ ] `execpolicy check` 对 `gh pr view` 输出 `prompt` 及命中规则；CI 可用退出码阻断
- [ ] 规则的 `match`/`not_match` 在加载期自测；反例被错误命中时报错
- [ ] PreToolUse hook 超时不会拖死 turn，且超时记录进审计
- [ ] 未信任项目的 `.codex/hooks.toml` 不加载；信任批准可持久化

## 读者挑战

1. 写一个规则：`cargo publish` 禁止，但 `cargo publish --dry-run` 允许。**注意 argv 前缀与标志组合，别让 `--dry-run` 被忽略。**
2. 若用户全局规则与项目规则冲突，应该谁赢？**请为“企业强制策略不可被项目覆盖”设计优先级与显式错误。**
3. PostToolUse 想自动修复 lint，但又要保持事件流真实。**设计一种显式“二次工具调用”方案，而不是静默改写原 `ToolResult`。**

## 下一章预告：长会话里，遗忘才是真正的敌人

第三部分给 mini-codex 装上了完整的“敢做什么”的控制面：第 10 章决定要不要问人，第 11 章用三个操作系统的机制执行不可撤销边界，第 12 章把命令规则与生命周期拦截做成可询问的策略。但还有一个问题没解决：长会话里，agent 会遗忘、重复检索、把上下文塞爆。

第四部分将从第 13 章开始回答：**如何把“记住什么、何时忘、怎样检索”也做成可靠系统。** 记忆不是更大的 prompt，而是一套与事件流、工具结果和持久存储协同的工程结构——它和第 10–12 章的安全控制面一样，需要在可控边界内工作。

---

## 引用来源

[1] https://developers.openai.com/codex/reference/agent-approvals-and-security/
> “Codex keeps older codex exec --full-auto invocations as a deprecated compatibility path and prints a warning.”

[2] https://developers.openai.com/codex/reference/agent-approvals-and-security/
> “macOS uses Seatbelt policies and runs commands using sandbox-exec with a profile (-p) that corresponds to the --sandbox mode you selected.”

[3] https://developers.openai.com/codex/reference/rules/
> “Use codex execpolicy check to test how your rules apply to a command.”

[4] https://developers.openai.com/codex/reference/rules/
> “When you use forbidden, include a recommended alternative in the justification when appropriate.”

[5] https://developers.openai.com/codex/reference/hooks/
> “timeout is in seconds. If timeout is omitted, Codex uses 600 seconds for most hooks.”

[6] https://developers.openai.com/codex/reference/hooks/
> “Commands run with the session cwd as their working directory.”

[7] https://developers.openai.com/codex/reference/agent-approvals-and-security/
> “Windows uses the Linux sandbox implementation when running in Windows Subsystem for Linux 2 (WSL2).”

[8] https://developers.openai.com/codex/reference/agent-approvals-and-security/
> “starting in 0.115, the Linux sandbox moved to bwrap, so WSL1 is no longer supported.”

[9] https://github.com/openai/codex/tree/main/codex-rs/linux-sandbox
> “the whole filesystem is bound read-only with --ro-bind / /. When bubblewrap is active, writable roots are layered with --bind.”

[10] https://github.com/openai/codex/tree/main/codex-rs/linux-sandbox
> “Overlapping split-policy entries are applied in path-specificity order so narrower writable children can reopen broader read-only or denied parents.”

[11] https://github.com/openai/codex/tree/main/codex-rs/linux-sandbox
> “WSL1 is not supported for bubblewrap sandboxing because it cannot create the required user namespaces.”

[12] https://github.com/openai/codex/tree/main/codex-rs/macos-sandbox
> “The base policy defaults to deny, with explicit allowances for PTYs, basic system calls, and safe sysctls.”

[13] https://github.com/openai/codex/tree/main/codex-rs/windows-sandbox-rs
> “Uses environment-level offline controls instead of the dedicated offline-user firewall rule.”

[14] https://deepwiki.com/yulin0629/codex/5.4-tool-orchestration-and-approval-flow
> “Never: Never prompts the user. Failures are returned directly to the model without escalation.”

[15] https://deepwiki.com/yulin0629/codex/5.4-tool-orchestration-and-approval-flow
> “OnFailure: Auto-approves all commands running in the sandbox. If sandbox execution fails, escalates to the user for approval to run without sandbox restrictions.”


# 第四部分　记忆：让长会话不崩（第 13–16 章）

> 安全篇结束时，mini-codex 已经能把“敢做什么”拆成审批、沙箱和规则三层。但它还记不住一件事：**你在这个仓库、这次任务、这条命令里，究竟想要什么。** 记忆篇不负责增加新工具，它负责让 agent 的配置可复现、项目知识可发现、上下文可持续、历史可回放。没有这四件事，长任务只会在第一百轮悄悄失控。

---

# 第 13 章　配置分层与 profiles

**本章任务**：给 mini-codex 写一个可信、可合并、可观测的配置加载器。它要同时做到三件事：让个人偏好、团队约定和临时调试各得其所；让 `--profile review` 一键切换整套行为；把项目级配置中试图篡改凭据、provider 或 profile 的恶意键无情丢弃。

安全篇的 `--full-auto` 已经暴露出一个事实：执行策略不能散落在工具内部，而必须是一组可选择的档位。现在要把这层“选择”做成第一等配置，并明确**配置能选什么、不能选什么**。

## 13.1 一个会咬人的“简单配置”

最朴素的实现是：启动时读一个 TOML，把字段塞进全局 `Config`。

```rust
// 反例：把所有来源拍扁成一个结构体
let mut cfg: SessionConfig = toml::from_str(&file)?;
cfg.api_key = std::env::var("CODEX_API_KEY").ok();
cfg
```

这段代码的问题不是“不能跑”，而是**它没有信任边界**。克隆一个恶意仓库后，项目里的 `.codex/config.toml` 只要写上别的 `api_key`、换掉 `provider` 或 `model`，就会在用户毫无察觉时把请求发到攻击者控制的 endpoint；写上 `profile = "full-auto"`，就把本地安全档位一并劫持。Codex 的设计因此把配置分成“普通设置”和“受保护设置”，后者只能来自用户或 CLI，不能由项目文件决定[citation:7]。

mini-codex 沿用同一思路。配置文件不是脚本，也不该拥有机器；它只是把你曾经做出的选择固化下来。所以本章的第一个设计准则是：

> **来源决定信任，信任决定覆盖权。**

## 13.2 四层来源与合并顺序

配置来源从低到高如下。后者只覆盖前者“允许覆盖”的字段，合并结果必须可打印、可复现。

1. **内置默认值**：保证即使没有任何文件，CLI 也能启动。
2. **用户级配置** `~/.codex/config.toml`：个人偏好，如默认 model、主题、自己的 key 来源。
3. **profile 层**：从 `~/.codex/profiles/<name>.toml` 读出的命名预设。
4. **项目级配置**：从仓库根向当前目录逐层读取 `.codex/config.toml`，更近者胜。
5. **CLI 参数**：一次性、最高优先级；环境变量并入这一层。

“从仓库根走到当前目录”意味着子目录可以细化规则，但不能反噬上级目录。例如仓库根的 `.codex/config.toml` 规定 Rust 项目默认用 `cargo test`，而 `crates/mcx-cli/.codex/config.toml` 只补充 CLI 专属命令。读取时必须先 canonicalize 路径，再确认每一层都在仓库根之下，防止 `../.codex/config.toml` 或符号链接指向工作区外。

```rust
// crates/mcx-config/src/lib.rs
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum ConfigSource {
    Defaults,
    UserFile(PathBuf),
    Profile(String, PathBuf),
    ProjectFile(PathBuf),
    Cli,
}

#[derive(Debug, Clone, Default)]
pub struct SessionConfig {
    pub model: Opt<String>,
    pub base_url: Opt<Url>,
    pub context_window: Opt<usize>,
    pub reserve_tokens: Opt<usize>,
    pub approval_policy: Opt<ApprovalPolicy>,
    pub sandbox_policy: Opt<SandboxPolicy>,
    /// 受保护字段：只能来自用户/CLI/profiles，项目文件不得设置。
    pub provider: Locked<Provider>,
    pub api_key_ref: Locked<Option<String>>,
    pub profile: Locked<Option<String>>,
    /// 项目层可追加，不能删除上层。
    pub extra_agents_paths: BTreeMap<PathBuf, Vec<PathBuf>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Opt<T> {
    /// 该层完全没写这个键
    Unset,
    /// 该层显式写 `key = null`
    ExplicitNone,
    /// 该层给了具体值
    Value(T),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Locked<T>(Option<T>);

impl<T> Opt<T> {
    /// 合并右值：只有它“更确定”时才覆盖。
    pub fn merge(&self, right: Opt<T>) -> Opt<T> {
        match (self, &right) {
            (_, Opt::Value(_)) => right,
            (Opt::Unset, Opt::ExplicitNone) => right,
            _ => self.clone(),
        }
    }
}
```

`Opt<T>` 就是本章最重要的 Rust 细节。**“未设置 / 显式 null / 有值”不是文字游戏，而是三种不同的用户意图**：

| 状态 | 含义 | 合并结果 |
|---|---|---|
| `Unset` | 该层没提这个键 | 可被更高层覆盖 |
| `ExplicitNone` | 用户明确清空 | 覆盖普通值，但不等于“没设置” |
| `Value(v)` | 有明确选择 | 覆盖前两者 |

若只用普通的 `Option<T>`，`Some(None)` 与 `None` 无法区分：上层就无法表达“我就是要清空模型”。第 5 章的 `#[serde(default)]` 与此同构——旧数据没有字段时，不能把它误判为“用户主动删除”。

## 13.3 安全红线：项目配置只能装饰，不能夺权

`SessionConfig::merge_from` 把“覆盖规则”集中在一个函数里。项目层的 `base_url`、模型偏好、工具白名单可以合并；一旦碰到受保护键，就**忽略并警告**，绝不悄悄采纳。

```rust
impl SessionConfig {
    pub fn merge_from(&mut self, layer: &Layer, source: ConfigSource) {
        let allow_protected = matches!(
            source,
            ConfigSource::UserFile(_) | ConfigSource::Profile(_, _) | ConfigSource::Cli
        );

        self.model = self.model.merge(layer.model.clone());
        self.base_url = self.base_url.merge(normalize_url(layer.base_url.clone()));
        self.context_window = self.context_window.merge(layer.context_window);
        self.reserve_tokens = self.reserve_tokens.merge(layer.reserve_tokens);
        self.approval_policy = self.approval_policy.merge(layer.approval_policy);
        self.sandbox_policy = self.sandbox_policy.merge(layer.sandbox_policy);

        if let Some(p) = &layer.provider {
            if allow_protected {
                self.provider = Locked(Some(p.clone()));
            } else {
                warn_ignored("provider", source);
            }
        }
        if let Some(k) = &layer.api_key_ref {
            if allow_protected { self.api_key_ref = Locked(Some(k.clone())); }
            else { warn_ignored("api_key_ref", source); }
        }
        if let Some(name) = &layer.profile {
            if allow_protected { self.profile = Locked(Some(name.clone())); }
            else { warn_ignored("profile", source); }
        }
    }
}
```

为什么要警告而不是“静默成功”？因为静默会让攻击者精确知道你没采纳，也会让善意用户以为配置生效。告警要带文件路径、行号（TOML 解析时保留位置）和建议：`cannot be set by a project file; move it to ~/.codex/config.toml`。

这条红线恰好覆盖第 10–12 章的安全策略：`approval_policy`、`sandbox_policy` 允许项目设置，因为团队需要声明“本仓库默认只读、需要审批”；`provider`、`api_key_ref` 和“当前 profile 选择”不允许，因为那等于让代码仓库挑选身份和执行档位。规则文件、hooks 同样只在项目受信任时加载——配置加载器只负责提供“信任来源”，不负责决定仓库是否可信。

## 13.4 兑现第 1 章避坑 #1 的伏笔：base_url 的斜杠必须归一化

第 1 章避坑专栏 #1 埋下 base_url 末尾斜杠的坑，第 4 章又用 SSE 帧边界演示过一次“错误常常来自边界”；现在同样的边界问题落在 URL 末尾斜杠上。用户写 `https://api.example.com/v1/`，项目写 `https://api.example.com/v1`，若直接字符串比较，profile 切换会看似失效；若拼接 `/responses` 前不统一，就得到 `.../v1//responses`。

归一化必须在**配置层**发生，而不是在每个 HTTP 调用点重复处理。一旦存进 `SessionConfig`，后续代码可以放心拼接。

```rust
fn normalize_url(opt: Opt<String>) -> Opt<Url> {
    match opt {
        Opt::Unset => Opt::Unset,
        Opt::ExplicitNone => Opt::ExplicitNone,
        Opt::Value(s) => {
            let mut s = s.trim().to_string();
            if !s.ends_with('/') { s.push('/'); }
            match Url::parse(&s) {
                Ok(u) => Opt::Value(u),
                Err(e) => {
                    tracing::warn!(url = %s, err = %e, "忽略无法解析的 base_url");
                    Opt::ExplicitNone
                }
            }
        }
    }
}
```

这里有三层防御：**trim** 去掉复制粘贴的空格；**末尾 `/`** 让“目录式 base”统一；**解析失败**变成告警而不是 panic。测试要覆盖 ` trailing slash`、`no slash`、`double slash after merge` 和 `invalid scheme`。这和第 4 章“在帧边界确定前只处理字节”一脉相承：先把外部输入规整成可信内部表示，后面的系统才不会到处 if。

## 13.5 profiles：把一套意图绑成一个名字

profile 不是 config 里的另一个表，而是**独立文件**。`~/.codex/profiles/review.toml` 可以设置“只读沙箱、永不自动批准、更长的思考时间”；`~/.codex/profiles/full_auto.toml` 则对应安全篇的预设。独立文件有两个好处：CLI 可直接 `--profile review`；仓库不能把自己的名字塞进你的 profile 文件。

```toml
# ~/.codex/profiles/review.toml
model = "gpt-5-codex"
approval_policy = "never"
sandbox_policy = "read-only"
context_window = 200000
reserve_tokens = 16000
```

```rust
pub fn load(
    cwd: &Path, repo_root: &Path, cli: &Layer,
) -> Result<SessionConfig, ConfigError> {
    let mut cfg = SessionConfig::defaults();

    if let Some(p) = &user_config_path() { cfg.merge_file(p, "user")?; }
    if let Some(name) = &cli.profile {
        let p = profile_path(name)?;
        cfg.merge_file(&p, "profile")?;   // profile 属于用户来源
    }
    for dir in ancestors_between(repo_root, cwd).chain(std::iter::once(repo_root.to_path_buf())) {
        let p = dir.join(".codex").join("config.toml");
        if p.is_file() { cfg.merge_file(&p, "project")?; }
    }
    cfg.merge_layer(cli, ConfigSource::Cli);

    cfg.validate()?;   // 例如 reserve_tokens < context_window
    Ok(cfg)
}
```

`--profile` 是 CLI 层，因此优先级高于项目文件；但 profile 文件本身来自用户家目录，可以设置 provider 等受保护字段。**这两件事不矛盾**：项目不能替你选 profile，但你可以显式选择自己的预设。若 CLI 同时给出 `--profile` 和 `--model`，后者胜出，因为 CLI 是最具体的意图。

## 13.6 可观测的合并结果

“配置即意图的持久化”有一个直接推论：**别人应能重现你的运行环境。** `mini-codex config show --sources` 要打印最终值及每一项的最后来源。

```text
model = "gpt-5-codex"   (profile: review)
approval_policy = "never"   (project: .codex/config.toml)
sandbox_policy = "read-only"   (profile: review)
provider = "openai"   (user: ~/.codex/config.toml)
base_url = "https://api.example.com/v1/"   (normalized)
```

没有这个命令，用户只能靠猜：“到底是项目配置没加载，还是 profile 覆盖了它？”它也是 CI 的救命稻草：把 show 输出作为日志，一次诡异失败就能被复现。

## 13.7 完整测试：恶意项目 + profile 切换

测试不联网、不读真实家目录，所有路径用临时目录构造。

```rust
#[test]
fn project_cannot_override_provider_or_api_key() {
    let tmp = tempdir();
    let project = tmp.path().join(".codex/config.toml");
    write(&project, r#"
        provider = "rogue"
        api_key_ref = "env:ROGUE_KEY"
        profile = "full-auto"
        model = "tiny"
    "#).unwrap();

    let mut cfg = SessionConfig::defaults();
    cfg.merge_file(&project, "project").unwrap();

    assert_eq!(cfg.provider, Locked(None), "项目不得设置 provider");
    assert_eq!(cfg.api_key_ref, Locked(None));
    assert_eq!(cfg.profile, Locked(None));
    assert_eq!(cfg.model, Opt::Value("tiny".into()), "普通字段仍合并");
    assert!(warnings().iter().any(|w| w.contains("provider")));
}

#[test]
fn profile_switches_whole_behavior_predictably() {
    let tmp = tempdir();
    write(tmp.path().join("review.toml"), r#"
        approval_policy = "never"
        sandbox_policy = "read-only"
    "#).unwrap();
    write(tmp.path().join("dev.toml"), r#"
        approval_policy = "on-failure"
        sandbox_policy = "workspace-write"
    "#).unwrap();

    let review = load_with_profile(tmp.path(), "review").unwrap();
    let dev = load_with_profile(tmp.path(), "dev").unwrap();

    assert_eq!(review.approval_policy, Opt::Value(ApprovalPolicy::Never));
    assert_eq!(dev.sandbox_policy, Opt::Value(SandboxPolicy::WorkspaceWrite));
}

#[test]
fn base_url_normalization_handles_slash_variants() {
    let cases = [
        ("https://a.com/v1", "https://a.com/v1/"),
        ("https://a.com/v1/", "https://a.com/v1/"),
    ];
    for (input, want) in cases {
        assert_eq!(normalize_one(input), want);
    }
}
```

第一个测试是全章核心：它同时验证“忽略”和“告警”，而不仅是字段相等。第二个测试证明 profile 是一等切换单位，不是几行宏。第三个兑现第 1 章避坑 #1 埋下的斜杠坑。

## 避坑专栏 #14：用 `Option::or` 合并配置，会悄悄复活旧值

常见错误是把 `Opt<T>` 降级成 `Option<T>`：

```rust
// 危险：无法区分“没写”和“显式清空”
config.model = config.model.or(layer.model);
```

后果是：用户在项目文件写 `model = null` 想清空，但上层已经设过 model，结果旧值永远留下。更糟的是，项目层写 `provider = "rogue"` 会因为 `Some` 直接覆盖受保护字段。

**通用形式**：配置合并必须按“来源信任 × 字段状态”两张表决策，不能退化为 `a.or(b)`。本项目用 `Opt<T>::merge` 表示“是否更确定”，用 `Locked<T>` 表示“谁有权设置”；二者缺一不可。

## 13.8 Design Rationale

**Q：为什么不把所有配置塞进一个全局 TOML，再用 `--flag` 覆盖？**

因为**一个文件无法同时表达“我的偏好”和“这个目录的特殊性”**。你想让 `~/.codex/config.toml` 长期稳定，又想让 `crates/api/.codex/config.toml` 只在进入该目录时生效。把二者合成一个文件，就需要手写条件、环境变量和大量注释；分层则由加载顺序天然表达。

**Q：为什么 profile 必须是独立文件，而不能是 `[profile.review]` 表？**

表形式的诱惑在于“一个文件全看见”，但实践中你会复制粘贴、忘记切档，最终形成一个 200 行的 config。独立文件让 profile 成为可版本控制的资源，也能被 `--profile` 精确选择；更关键的是，它避免项目文件通过“定义同名 profile”间接夺权。

**Q：为什么 `provider`/`api_key_ref`/`profile` 必须锁死，而 `approval_policy` 可以项目设置？**

因为前者决定**身份和信任根**，后者决定**在工作区内的风险档位**。团队对“是否需要审批”有合理共识；但没有人该让克隆来的仓库挑选 API endpoint 和 key。这个边界越清晰，配置越不容易成为攻击面。

## AI 软件工程原理 #13

> **配置即意图的持久化。**

你调好的那套工作流——使用哪个模型、默认如何审批、哪些路径只读、上下文留多少余量——不是一串临时 CLI 参数，而是一段可复现的意图。好的配置系统要做到：把这份意图按来源分层、按信任锁定、按 scope 生效，并能一眼看出最后由谁决定。

这解释了为什么“项目层不能覆盖身份”、为什么 profile 是一等文件、为什么合并结果必须可打印：配置不是给程序读的便利，而是给团队和未来自己审计的契约。下一章将看到，仓库中的“项目知识”也遵循同样的真相源原则。

## Rust 修炼小结

| 概念 | 本章用法 | 后面在哪用到 |
|---|---|---|
| `enum Opt<T>` | 三态合并语义 | 第 14 章知识层覆盖 |
| `Locked<T>` | 保护高信任字段 | 第 16 章受保护审计字段 |
| `Path::canonicalize` | 防越界、防符号链接 | 第 14 章 AGENTS.md 发现 |
| `toml::from_str` + 位置信息 | 报告“哪个文件哪行被忽略” | 第 12 章规则诊断 |
| `BTreeMap` | 稳定、可打印的合并结果 | 第 16 章索引元数据 |

## 章末验收

- [ ] `~/.codex/config.toml` → profile → 项目层 → CLI 的顺序有文档化测试
- [ ] 项目配置写 `provider/api_key_ref/profile` 时被忽略，并输出带文件路径的警告
- [ ] `base_url` 的 `有斜杠/无斜杠/首尾空格` 归一化为同一内部值
- [ ] `--profile review` 切换后，所有相关字段按预设改变
- [ ] `mini-codex config show --sources` 能复现最终配置的来源链

## 读者挑战

1. 若允许项目层设置 `profile = "review"`，攻击者只需在仓库里写这一行即可改变用户档位。**请设计一个“受信任仓库”机制，使项目选择 profile 需要一次显式确认，并写测试证明确认不能跨仓库复用。**
2. 子目录配置想“撤销”根配置的某个字段，例如清空 `extra_agents_paths`。**`ExplicitNone` 应如何与 map 合并交互，才能既支持清空又不让项目删除用户全局路径？**
3. `config show` 会暴露敏感值。**如何让 key 显示为 `<set>` 的同时仍能让用户校验“是否来自正确来源”？**

## 下一章预告：让仓库替 agent 记住不变量

配置解决了“我想怎么跑”，但 agent 还需要知道“这个仓库有什么不变量”。下一章的分层 AGENTS.md 就是项目自己的记忆：它比配置更自由，比聊天历史更持久，也比一份巨大的 system prompt 更经济。我们将让“在根目录”和“在子目录”加载到不同规则，并说明为什么 32KB 的硬限制不是小气，而是上下文预算的第一道防线。

---


# 第 14 章　AGENTS.md：把项目知识写进仓库

**本章任务**：实现分层 AGENTS.md 加载器。它从全局个人偏好走到仓库根团队约定，再走到当前子目录的特性规则；合并时要安全、有上限、可观测。写完之后，同一段代码在不同目录会呈现不同“项目记忆”，而不会因为文件过大直接挤爆上下文。

第 13 章解决的是用户的意图，本章解决的是**项目的意图**。二者必须分开：你可以信任一个仓库的编码规范，却不代表它有权替你选 provider。

## 14.1 一份“越大越好”的知识文件为什么注定失败

先写一个看起来很勤奋的版本：

```rust
// 反例：把能找到的 AGENTS.md 全部拼接
let mut buf = String::new();
for p in glob("**/AGENTS.md")? { buf.push_str(&read(p)?); }
system_prompt.push_str(&buf);
```

问题不在拼接，而在**范围和预算脱节**。一个 monorepo 有 30 个服务，每个服务都写 2KB 约定；在 `services/api/` 下改一个小 bug 时，你根本不需要 `services/billing/` 的支付规则。把 60KB 一股脑塞进 system prompt，既浪费 token，又稀释真正相关的约束，还会让后续第 15 章的压缩更早发生。

正确做法是**按工作目录向上收集，越近越优先；跨目录的兄弟节点默认不加载**。这和第 13 章的“从仓库根走到当前目录”方向相反但互补：配置是“上层默认、就近覆盖”，项目知识是“当前范围为主、向上补上下文”。

## 14.2 三层结构与发现顺序

约定如下：

| 层 | 路径 | 典型内容 | 规模 |
|---|---|---|---|
| 全局个人 | `~/.codex/AGENTS.md` | 你的写作口吻、通用禁忌 | ~20 行 |
| 仓库根 | `<repo>/.codex/AGENTS.md` | 构建命令、分支策略、全局不变量 | ~40 行 |
| 子目录 | `<repo>/<sub>/.../.codex/AGENTS.md` | 该模块边界、测试入口、特性禁忌 | 按需 |

Codex 的真实实现也采用“用户级 + 仓库根 + 子目录”的层级，且子目录规则会覆盖更宽泛的规则；项目约定通常消耗 500–5000 token[citation:2][citation:4]。我们的实现把发现结果规整为 `KnowledgeBundle`，而不是直接返回字符串。

```rust
// crates/mcx-knowledge/src/lib.rs
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct KnowledgeFile {
    pub path: PathBuf,
    pub scope: Scope,
    pub body: String,
    pub bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Scope {
    Global,
    RepoRoot,
    Directory(PathBuf),
}

#[derive(Debug, Default)]
pub struct KnowledgeBundle {
    pub files: Vec<KnowledgeFile>,
    pub total_bytes: usize,
}

impl KnowledgeBundle {
    /// 从“当前目录向仓库根”收集；兄弟目录不进入。
    pub fn load(
        cwd: &Path, repo_root: &Path, global: Option<&Path>,
        limit: ByteLimit,
    ) -> Result<Self, KnowledgeError> {
        let mut bundle = Self::default();
        if let Some(g) = global { bundle.try_add_global(g, limit)?; }

        for dir in ancestors_up_to(cwd, repo_root).chain(std::iter::once(repo_root.to_path_buf())) {
            let candidate = dir.join(".codex").join("AGENTS.md");
            if candidate.is_file() { bundle.try_add(candidate, limit)?; }
        }
        bundle.files.sort_by_key(|f| f.scope.rank());
        Ok(bundle)
    }
}
```

`rank()` 保证全局最先、仓库根其次、最近目录最后。这样拼接后，**越近的规则在 prompt 末尾**，模型更容易把它当作当前指令。若同一层出现冲突，不是“后写覆盖前写”，而是显式报告冲突键——知识文件应该被审阅，而不是被悄悄裁决。

## 14.3 32KB 是安全闸门，不是写作配额

默认上限 32KB 来自“上下文预算意识”：一个中等规模的 system prompt 约 2000–3000 token，加上工具定义、Skills 和对话历史后，项目知识若再无限增长，很快就没有空间留给真正的工作[citation:4]。这里 32KB 是**进入上下文的闸门**，不是文件大小限制：仓库仍可以拥有 200KB 的 AGENTS.md，但加载器会截断并告警，且保留“已截断”标记。

```rust
#[derive(Debug, Clone, Copy)]
pub struct ByteLimit {
    pub soft: usize,
    pub hard: usize,
}

impl KnowledgeBundle {
    fn try_add(&mut self, path: PathBuf, limit: ByteLimit) -> Result<(), KnowledgeError> {
        let raw = std::fs::read(&path)?;
        let bytes = raw.len();
        if self.total_bytes.saturating_add(bytes) > limit.hard {
            return Err(KnowledgeError::HardLimitExceeded {
                path, current: self.total_bytes, limit: limit.hard,
            });
        }
        let body = String::from_utf8_lossy(&raw).into_owned();
        self.files.push(KnowledgeFile { path, scope: Scope::from_path(&path), body, bytes });
        self.total_bytes += bytes;
        Ok(())
    }
}
```

“超限降级”有两种策略，不能混用：

| 策略 | 行为 | 适合场景 |
|---|---|---|
| `truncate_with_marker` | 保留前 N KB，附 `[TRUNCATED: see <path>]` | 长历史、CI |
| `reject` | 返回错误并要求人工拆分 | 交互式、评审 |

mini-codex 默认在交互模式截断并警告，在严格模式拒绝。绝不能“静默丢弃末尾”，因为那可能正好丢掉最关键的不变量。

## 14.4 合并不是拼接：覆盖、追加与冲突

AGENTS.md 是 Markdown，不能像 TOML 字段那样机械 merge。我们采用**块级策略**：默认追加；以 `#` 标题为块；后加载的同名标题覆盖先加载的；其余块保留顺序。

```rust
pub fn render(&self) -> String {
    let mut out = String::new();
    for file in &self.files {
        out.push_str(&format!("\n<!-- source: {} -->\n", file.path.display()));
        out.push_str(&file.body);
        if !file.body.ends_with('\n') { out.push('\n'); }
    }
    out
}
```

渲染时插入 `<!-- source: ... -->` 注释是低成本但高价值的可观测性：压缩、调试或评测时，你能知道哪句话来自哪个文件。比“智能合并”更重要的是**可预测**——第 5 章的前向兼容也遵循同一原则：宁可显式保留 Unknown，也不要偷偷改语义。

标题覆盖可由独立函数实现，但本书选择“默认追加、冲突显式报告”，避免解析 Markdown 的复杂度。真正的项目应把“合并策略”写成规则，并在 CI 校验。

## 14.5 路径安全：符号链接、越界与 canonicalize

第 11 章的沙箱反复强调“路径必须真实存在且受控”，知识加载也一样。一个恶意的 `services/api/.codex/AGENTS.md` 通过符号链接指向 `/etc/shadow` 或工作区外秘密，不应被读成项目知识。

```rust
fn assert_within_repo(path: &Path, repo_root: &Path) -> Result<(), KnowledgeError> {
    let canonical = path.canonicalize().map_err(KnowledgeError::Io)?;
    let root = repo_root.canonicalize().map_err(KnowledgeError::Io)?;
    if !canonical.starts_with(&root) {
        return Err(KnowledgeError::EscapesRepo { path: path.to_path_buf(), root });
    }
    Ok(())
}
```

测试构造符号链接时必须跳过无权限环境（Windows CI 常有此限制），用 `cfg(unix)` 或在失败时跳过。另一件事是**不要执行 AGENTS.md**：它只产生文本，绝不能包含 `curl ... | bash`。加载器只读取、不解释代码块，工具的授权仍由第 10 章的审批策略负责。

## 14.6 怎么写好 AGENTS.md：写不变量，不写通用常识

最常见的坏文件长这样：

```markdown
# 项目说明
- 我们使用 Rust
- 使用 cargo build 构建
- 函数命名用 snake_case
```

模型知道 Rust 用 snake_case；更糟的是，这种信息在所有 Rust 项目里都重复，白白消耗预算。好文件写的是**只有这个仓库成立、且违反会导致事故**的事实：

```markdown
# services/api 规则
- 所有数据库迁移必须先在 `migrations/` 创建正向与回滚两文件；只改 schema 不提交数据修复脚本会阻塞发布。
- `UserId` 与 `ExternalUserId` 不可互相 `into()`；跨边界必须走 `UserId::from_external`。
- 测试默认走 `cargo test -p api --lib`；集成测试需要 Postgres，不得在 CI 缓存外新建数据库。
- 禁止在 handler 中直接读取 `std::env`；配置只能通过 `config::AppConfig`。
```

四条规则都满足“不变量或禁忌、模型可能不知道、违反代价高”。这也解释了第 15 章的预算压力：AGENTS.md 的价值密度比“如何使用框架”高得多，应优先保留。

## 14.7 测试：范围决定内容

下面测试不依赖网络，也不依赖真实仓库：

```rust
#[test]
fn subdir_sees_only_ancestors_not_siblings() {
    let tmp = tempdir();
    let root = tmp.path().join("repo");
    write(root.join(".codex/AGENTS.md"), "# root\n").unwrap();
    write(root.join("services/api/.codex/AGENTS.md"), "# api\n").unwrap();
    write(root.join("services/billing/.codex/AGENTS.md"), "# billing\n").unwrap();

    let bundle = KnowledgeBundle::load(
        &root.join("services/api/handlers"),
        &root,
        None,
        ByteLimit::unlimited(),
    ).unwrap();

    let texts: Vec<&str> = bundle.files.iter().map(|f| f.body.trim()).collect();
    assert_eq!(texts, vec!["# root", "# api"]);
}

#[test]
fn hard_limit_degrades_with_warning_not_panic() {
    let tmp = tempdir();
    write(tmp.path().join("AGENTS.md"), "x".repeat(40_000)).unwrap();
    let err = KnowledgeBundle::load(
        tmp.path(), tmp.path(), None,
        ByteLimit { soft: 16_000, hard: 32_000 },
    ).unwrap_err();
    assert!(matches!(err, KnowledgeError::HardLimitExceeded { .. }));
}
```

第一个测试证明“上下文预算跟任务范围成比例”：在 `api` 下工作不会加载 `billing`。第二个测试证明超限是优雅错误，不是崩溃。二者共同满足章末验收。

## 避坑专栏 #15：把“最近文件”误当“最高权限”

一个容易写反的逻辑是：

```rust
// 危险：把文件路径排序当信任排序
files.sort_by_key(|f| f.path);   // 字典序不能表达覆盖关系
```

后果是：仓库根规则因为文件名巧合排在子目录规则之后，子目录想覆盖的“禁止直接读 env”被根文件再次声明覆盖回去。正确做法是按 `Scope` 排序：Global < RepoRoot < Directory，且**越靠近 cwd 的目录越靠后**。如果未来支持兄弟目录的“标签引用”（例如显式 `imports = ["../shared/AGENTS.md"]`），那也是声明式依赖图，而不是自动发现。

**通用形式**：文件系统的邻近性不等于语义优先级；当二者冲突时，显式 `Scope` 排序是唯一可靠答案。

## 14.8 Design Rationale

**Q：为什么不把 AGENTS.md 全部塞进 system prompt，让模型自己挑？**

因为 prompt 不是无限资源。第 15 章将量化这一点：系统指令、工具定义、Skills 和对话历史都在争同一窗口。无关知识越多，关键约束越容易被稀释；长会话还必须更早压缩。分层是按范围做“按需加载”，把预算留给当前任务。

**Q：为什么不做一个“智能摘要器”先压缩 AGENTS.md？**

因为摘要会丢掉不变量，而 AGENTS.md 恰恰是最不该丢的部分。更好的顺序是：先用范围过滤，再按预算截断；只有当明确启用“远程/服务端压缩”时，才对**已加载但暂不活跃**的知识做摘要，并保留原文引用。摘要知识永远标记 `derived=true`，不能与原始规则混为一谈。

**Q：为什么不让项目文件覆盖用户全局偏好？**

因为全局层代表**操作者身份**，项目层代表**代码库约定**。你想让仓库规定“提交前跑哪些测试”，但不希望它改变你的审批风格。覆盖规则应与第 13 章一致：项目可补充、可细化，不可篡改信任根。

## AI 软件工程原理 #14

> **仓库是唯一真相来源。**

如果关键知识只存在于 Google Docs、Slack 或某位维护者的记忆里，agent 就无法稳定遵守它。AGENTS.md 的价值在于：把“这个仓库不能做什么、必须怎样做”写成可被加载器发现、被工具尊重、被压缩策略保留的版本化文本。

这条原理有三个推论。第一，**可发现**：从当前工作范围向上收集，避免遗漏根规则。第二，**高价值优先**：只写不变量和禁忌，不重复通用常识。第三，**可审计**：每个块标注来源，超限时显式降级。第 16 章的回放能力将再次用到这些来源标记。

## Rust 修炼小结

| 概念 | 本章用法 | 后面在哪用到 |
|---|---|---|
| `Path::ancestors` | 从 cwd 向 repo root 收集 | 第 13 章项目配置 |
| `canonicalize` | 防符号链接越界 | 第 11 章沙箱路径 |
| `from_utf8_lossy` | 容忍非严格 UTF-8 文本 | 日志、文件读取 |
| `ByteLimit` | 软硬阈值与降级策略 | 第 15 章上下文预算 |
| 块级合并 | Markdown 标题覆盖 | 第 15 章 Item 保留策略 |

## 章末验收

- [ ] 在 `services/api/` 下只加载 `api` 及其祖先，不加载 `billing`
- [ ] 仓库根与子目录规则按 scope 排序，近处规则在 prompt 末尾
- [ ] 文件超过 32KB 时按模式截断或拒绝，并产生明确告警
- [ ] 符号链接指向仓库外时拒绝加载
- [ ] 渲染结果含每个来源的路径注释

## 读者挑战

1. monorepo 的 `services/shared/` 想被多个服务引用，但默认“不加载兄弟目录”。**请设计一个显式 `imports` 字段，保证循环引用能被检测，且导入顺序可预测。**
2. 子目录想**撤销**根目录的一条规则，例如“根规则禁止网络，但 api 服务必须允许”。**如何用块标题表达撤销，而不让任意文件都能覆盖安全红线？**
3. AGENTS.md 里可能误提交 token。**加载器应在哪一阶段、以何种方式脱敏？是否允许脱敏结果进入摘要？**

## 下一章预告：上下文不是无限水箱

现在 agent 有了配置意图和项目知识，但 system prompt、AGENTS.md、MCP 工具定义、Skills、读过的文件和对话历史都在抢同一个窗口。下一章是本章批的重头戏：我们要量化每一笔开销，定义“何时必须压缩”，并回答一个反直觉问题——**为什么切点只能在 user turn 边界**。切错位置，模型会带着“有问无答”的残缺记忆继续工作。

---


# 第 15 章　上下文预算与压缩

**本章任务**：给 mini-codex 装上上下文预算器与压缩器。预算器要按角色分配 system、knowledge、tools、skills、history 和文件内容；压缩器要在窗口余量不足时，优先保留不变量、约束和近期活动，并在**完整的 user turn 边界**把旧历史交接出去。这是记忆篇最难的一章，因为它要在“省钱、保真、可回放”三者间取舍。

前面的 AGENTS.md 只是预算的一个输入；真正让长会话崩掉的，是所有输入加起来超过模型窗口。第 3 章的“history 只追加”在这里成为硬约束：追加能保住 prompt cache，但无限制追加最终会把窗口撑爆。

## 15.1 先数一数上下文都花在哪

如果不测量，就只能凭感觉压缩。先把固定开销和可变开销分开：

| 组成 | 典型量级（教学值） | 特点 |
|---|---:|---|
| 系统指令 | 2000–3000 token | 相对稳定，优先完整保留 |
| AGENTS.md | 500–5000 token | 按目录范围加载，本章核心变量之一[citation:4] |
| MCP 工具定义 | 每 server 500–3000 token | 工具越多越膨胀 |
| Skills 元数据 | 数百 token | 可按任务激活 |
| 对话历史 | 随轮次线性增长 | 长任务的主犯 |
| 读过的文件内容 | 0–数万 token | 一次粘贴就可能爆窗 |

数字会因 tokenizer 和模型不同而变化，所以不要把它们写死成真理；要写进可配置的 `ContextBudget`，并允许单测注入计数器。关键结论是：**历史不是唯一开销，但它是最容易失控的那一项。**

```rust
// crates/mcx-context/src/budget.rs
#[derive(Debug, Clone)]
pub struct ContextBudget {
    pub context_window: usize,
    pub reserve_tokens: usize,
    pub system: usize,
    pub knowledge: usize,
    pub tool_defs: usize,
    pub skills: usize,
    pub files: usize,
    pub history: usize,
}

impl ContextBudget {
    pub fn used(&self) -> usize {
        self.system + self.knowledge + self.tool_defs + self.skills + self.files + self.history
    }
    pub fn available_for_history(&self) -> usize {
        self.context_window.saturating_sub(
            self.system + self.knowledge + self.tool_defs + self.skills + self.files
        )
    }
    pub fn should_compact(&self) -> bool {
        self.used() + self.reserve_tokens > self.context_window
    }
}
```

`reserve_tokens` 是为输出、工具调用和安全边际预留的“不可挪用款”。触发条件是 `used + reserve > window`，而不是 `used > window`：等到正好满才压缩，下一轮模型要输出长答案时就会失败。这也是第 13 章把 `context_window` 和 `reserve_tokens` 做成可配置字段的原因——不同模型、不同任务的风险容忍度不同。

## 15.2 估算器：宁可稳定，不可精确

token 计数最准确的方式是模型提供商的 tokenizer，但它需要网络或绑定，而且每次按键都重算太贵。mini-codex 使用**可插拔 estimator**：本地默认用“字节/4 加标点修正”，测试注入精确计数器。

```rust
pub trait TokenEstimator: Send + Sync {
    fn estimate(&self, text: &str) -> usize;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ByteDivFour;

impl TokenEstimator for ByteDivFour {
    fn estimate(&self, text: &str) -> usize {
        // 教学近似：每 4 字节算 1 token，最少 1（非空）。
        (text.len().saturating_add(3) / 4).max(text.is_empty() as usize)
    }
}
```

这个近似显然不精确，但它有两个优点：**确定性**和**无外部依赖**。压缩决策最怕“两次估算结果不一致导致抖动”：若每次都差几十 token，切点会在相邻 turn 间反复横跳，prompt cache 被打碎。生产实现可换成 `tiktoken-rs` 或服务端计数，但必须保证同一会话使用同一 estimator。

## 15.3 Turn 是语义最小完整单元：反直觉的切点规则

第 5 章说过：Thread → Turn → Item 中，Turn 是“一次用户输入 + 模型响应 + 期间工具调用”。现在兑现这个伏笔。

错误做法是“按 token 数量从最老开始丢”：

```rust
// 反例：在工具调用中间切开历史
history.drain(0..cut);
```

假设第 7 轮模型调用 `read_file(a.rs)`，结果只回来一半，你正好把后半段和 `ToolResult` 切掉。下一轮模型看到的“记忆”是：

```text
[Turn 7] 用户：检查 a.rs 的并发问题
[ToolCall] read_file(a.rs)
```

于是它会**再次调用同一个工具**，或基于不存在的结果推断，或认为任务已完成——三种后果都浪费 token 且可能破坏仓库。这就是“有问无答”的残缺历史。

**正确做法是：切点必须是完整 user turn 的边界。** 即保留从某一轮 `UserMessage` 开始直到其所有 `ToolCall`/`ToolResult`、`AgentMessage`、审批结果全部闭合的完整片段；只把更早的完整 turn 压缩。

```rust
fn find_compaction_boundary(turns: &[Turn], target_drop: usize) -> usize {
    let mut dropped = 0;
    for (i, turn) in turns.iter().enumerate() {
        if i + 1 >= turns.len() { break; } // 永远不切最后一轮
        dropped += turn.estimated_tokens();
        if dropped >= target_drop {
            // 被压缩（移出窗口）的是 turn i 之前的更早 turn；窗口从 turn i 起保留为完整语义单元
            return i;
        }
    }
    turns.len().saturating_sub(1)
}
```

为什么要“从 user turn 开始”而不是“到 tool 调用结束”？因为**用户请求是意图的起点**。即使上一轮最后一条 item 是 `ToolResult`，那也是该轮的一部分；压缩单元必须是整轮。第 5 章的 `Item` 细粒度现在发挥作用：你可以按 turn 找边界，再按 item 类型决定哪些保留、哪些进摘要。

## 15.4 两种压缩与“append-only 交接”

压缩不是删历史。mini-codex 支持两种模式：

1. **本地交接摘要**：把将被移出窗口的早期 turn 压缩成一段结构化摘要，作为新的 system 附件**追加**到 prompt。
2. **服务端压缩**：把完整 thread 发给支持服务端上下文管理的模型，本地只保留引用。此时仍要记录“压缩请求/响应”，不能假装历史消失。

无论哪种，原始 JSONL **绝不修改**。第 3 章“history 只追加”在这里有两层收益：其一，前缀稳定，prompt cache 更可能命中；其二，回放和 rollback（第 16 章）能回到压缩前。

```rust
// crates/mcx-context/src/compact.rs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HandoffSummary {
    pub id: String,
    pub turn_range: std::ops::RangeInclusive<usize>,
    pub decisions: Vec<String>,
    pub invariants: Vec<String>,
    pub remaining_steps: Vec<String>,
    pub compressed_items: usize,
}

pub enum CompactOutcome {
    /// 什么都不删，只追加一段摘要锚点
    Appended { anchor: Item, summary: HandoffSummary },
    /// 服务端接管：保留引用，旧片段只在索引中可见
    ServerManaged { reference: String },
}
```

摘要不是“把旧对话重写一遍”。它必须保留四类高价值信息：**已做的决策、不可违反的不变量、剩余步骤、重要工具结果的关键引用**。模板如下：

```markdown
<compacted-turns 1..=6>
## 已确认决策
- 使用 `UserId::from_external` 跨 API 边界；不再尝试 `into()`。
## 不变量
- migrations 必须有正向 + 回滚；禁止 handler 直接读 env。
## 剩余步骤
- 在 `services/api` 为 `list_users` 补充权限测试。
## 关键产物
- 修改了 `domain/user.rs`；测试 `cargo test -p api --lib` 已通过。
</compacted-turns>
```

`Item::AgentMessage` 的冗长解释可以压缩；`ToolCall`/`ToolResult` 中的**关键文件与命令**要留引用；纯 `Reasoning` 摘要可以丢；**审批、拒绝、错误信息必须保留**——它们恰恰是最容易重复犯的错。

## 15.5 保留策略：Item 的挑选粒度

第 5 章区分 Event 与 Item，正是为了让压缩有“挑选粒度”。粗粒度的“一整条消息”只能整块保留或整块丢弃；分层 Item 可以区分“这条工具结果 20KB，但只有最后 3 行相关”。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Retention {
    MustKeep,      // 约束、审批、错误、最后一轮
    Summarize,     // 长对话、冗长推理
    DropCandidate, // 重复读文件、已 summarised 的原始输出
}

impl Item {
    pub fn retention(&self, in_last_kept_turn: bool) -> Retention {
        match self {
            Item::UserMessage { .. } => Retention::MustKeep,
            Item::ToolCall { .. } | Item::ToolResult { is_error: true, .. } => Retention::MustKeep,
            Item::AgentMessage { .. } => {
                if in_last_kept_turn { Retention::MustKeep } else { Retention::Summarize }
            }
            Item::Reasoning { .. } => Retention::Summarize,
            _ => Retention::DropCandidate,
        }
    }
}
```

`MustKeep` 并不意味着永久留在窗口，而是**至少保留到下一个压缩点**，并以结构化字段进入摘要。读过的文件内容通常属于 `DropCandidate`：原文仍在磁盘和 JSONL 中，需要时重新读取，比永久占着 prompt 更划算。这个“丢掉内容、保留决策”的取舍，正是长任务可持续的关键。

## 15.6 完整的 Compactor：不破坏可回放性

`Event` 侧也要新增一个变体，把“压缩点”变成可回放的事实——TUI 能据此画出“哪几轮被收成了摘要”，回放也知道那一段历史换了一种形态：

```rust
pub enum Event {
    // ...既有变体
    /// 第 15 章新增：压缩收走了旧轮次；kept_turns 是压缩后窗口还剩的轮数
    Compacted { kept_turns: usize },
}
```

触发点在每轮 turn 结束后检查上下文预算：

```rust
impl<C: LlmClient> Session<C> {
    async fn maybe_compact(&mut self) -> Result<(), LlmError> {
        if !self.budget.should_compact() { return Ok(()); }

        let boundary = find_compaction_boundary(&self.thread.turns, self.budget.history / 2);
        if boundary == 0 { return Ok(()); } // 一轮都压不动，停止增长更优先

        let old_turns = self.thread.turns.drain(0..boundary).collect::<Vec<_>>();
        let summary = self.build_handoff(&old_turns).await?;

        // 追加，不是修改已有消息！
        self.history.insert(
            0,
            Message { role: Role::System, content: render_summary(&summary) },
        );
        self.thread.turns.insert(0, Turn::from_summary(summary.clone()));

        self.rollout.append(&Record::from_summary(&summary))?;
        self.emit(Event::Compacted { kept_turns: self.thread.turns.len() }).await;
        Ok(())
    }
}
```

注意三处刻意设计：

1. **先找边界，再 drain**：保证被移除的是完整 turn。
2. **摘要插入为 system message**：新内容出现在历史前部，不修改原有后缀，prompt cache 前缀稳定。
3. **原始 turn 仍写入 Rollout**：JSONL 是真相源，内存只是当前窗口视图。

若把摘要直接覆盖旧消息，你就失去了第 16 章的 rollback 能力，也让“压缩前后是否丢约束”无法审计。

## 15.7 快照测试：压缩不能悄悄丢约束

本章最关键的测试不是“token 数下降”，而是“第 5 轮定下的约束在第 100 轮仍能被复述”。我们使用不依赖网络的 `ScriptedLlm`，让它返回固定摘要。

```rust
#[tokio::test]
async fn never_cuts_mid_tool_call() {
    let turns = vec![
        Turn { index: 0, items: vec![Item::UserMessage { content: "看 a.rs".into() }] },
        Turn { index: 1, items: vec![
            Item::UserMessage { content: "继续".into() },
            Item::ToolCall { call_id: "c1".into(), name: "read_file".into(), arguments: "{}".into() },
        ]},
    ];
    let boundary = find_compaction_boundary(&turns, /* target */ 1);
    assert_eq!(boundary, 1, "不能把 ToolCall 单独留在被压缩侧");
}

#[tokio::test]
async fn invariants_survive_100_turns() {
    let scripted = ScriptedLlm::new(vec![
        serde_json::json!({
            "decisions": ["拒绝自动发布"],
            "invariants": ["UserId 不可 from_external 之外构造"],
            "remaining_steps": ["补权限测试"]
        }).to_string(),
    ]);
    let mut session = make_session(scripted, /* small window */ 800, /* reserve */ 200);

    for i in 0..100 {
        session.submit(format!("step {i}")).await;
    }
    let last_prompt = session.last_built_prompt();
    assert!(last_prompt.contains("UserId"), "约束必须保留");
    assert!(last_prompt.contains("拒绝自动发布"));
}

#[test]
fn token_curve_drops_after_compaction() {
    let mut b = ContextBudget { context_window: 4000, reserve_tokens: 400, ..Default::default() };
    b.history = 3800;
    assert!(b.should_compact());
    // 压缩后：旧 turn 由摘要替代，约 400 token
    b.history = 400;
    assert!(!b.should_compact());
}
```

第二个测试用 `ScriptedLlm` 生产确定性摘要，验证约束留存；它和第 3 章的测试哲学一致：不联网、不花钱、永远稳定。第三个测试证明 token 曲线会下降。真正的“100 轮后准确复述”还需要一个断言脚本：从最终 prompt 抽取约束集合，与压缩前的 `thread` 中 `AGENTS.md + decisions` 求差。

```text
token 使用量（示意）
  │      /╲
  │     /  ╲___/╲
  │    /         ╲___/╲
  │   /              ╲___
  └──────────────────────── 轮次
     触发阈值 → 压缩 → 下降 → 再增长
```

*图 1：每次在 user turn 边界压缩后，上下文使用量回落；阈值预留保证输出空间。数据来源：本章 `ContextBudget::should_compact` 规则。*

## 15.8 prompt cache 可观测性

append-only 的经济收益必须可度量，否则团队会怀疑“压缩到底省没省”。每次构建 prompt 时记录：

```rust
#[derive(Debug, Clone, Default, Serialize)]
pub struct CacheStats {
    pub prompt_prefix_tokens: usize,
    pub changed_suffix_tokens: usize,
    pub estimated_cache_hit: bool,
}
```

规则很简单：若本次 prompt 与前一次相比，**除最近追加部分外前缀完全相同**，则 `estimated_cache_hit = true`。频繁在旧消息中间插入摘要会破坏这个前缀；把摘要固定放在前部、把新活动放在后部，前缀才稳定。这是“上下文是预算，不是缓存”的另一面：**为了保住缓存，也必须遵守 append-only。**

## 15.9 Design Rationale

**Q：为什么不每轮都压缩一点，而是等到阈值再批量处理？**

因为摘要本身有成本和不稳定性。频繁压缩会让前缀反复变化，反而伤害 cache；批量压缩以完整 turn 为单元，边界清晰、可审计，也更符合模型一次“交接班”的认知。阈值是 `context_window - reserve_tokens`，不是某个神秘常数。

**Q：为什么不在工具调用中间切，即使那样能省更多 token？**

因为省下的几十 token 远不抵“模型重新执行同一工具”或“基于缺失结果做错决定”的代价。Turn 是语义最小完整单元；切在中间造出的残缺历史会让长任务在原地打转。第 5 章埋下的 Turn 层，正是为了给今天这个判断提供类型依据。

**Q：为什么原始 JSONL 永远不删？**

因为压缩摘要是**派生数据**。派生数据可以丢失、可以重算；原始事件流不行。服务端压缩也只把“可见窗口”交给模型，本地索引仍指向完整 rollout。这个原则将在第 16 章成为 resume/fork/rollback 的地基。

## 避坑专栏 #16：把摘要插在历史末尾，以为“最新优先”

常见写法：

```rust
// 危险：破坏 prompt cache 前缀，且让摘要远离系统上下文
self.history.push(Message::System(summary));
```

后果有两条。第一，所有既有前缀都成了“旧后缀”，cache 命中率骤降。第二，模型更容易把摘要当作最近闲聊，而不是长期约束。正确做法是把交接摘要作为**最前面的系统锚点**之一，并在其中显式写 `turn_range`、决策和不变量。

**通用形式**：日志/历史的派生视图必须 append 到稳定位置，不能回写原记录。第 3 章 append-only 与第 5 章版本化 Record 都是同一原则。

## AI 软件工程原理 #15

> **上下文是预算，不是缓存。**

长会话的第一号生产力杀手通常不是模型变笨，而是窗口被系统提示、项目知识、工具定义、文件内容和历史共同撑爆。把它当作“缓存”会诱使你无限留存；把它当作预算，则会迫使你分配固定开销、按范围加载知识、在完整语义边界交接。

这条原理连接三件事：

- **预算**：第 14 章的 32KB 闸门与本章 `ContextBudget` 是同一资源的不同层级；
- **边界**：第 5 章的 Turn、Item 粒度使压缩能挑选而非乱砍；
- **可恢复性**：append-only 的交接摘要让压缩可重放、可审计，并导向第 16 章。

压缩的质量决定了长任务的寿命；而压缩是否正确，只能用完整事件流验证。

## Rust 修炼小结

| 概念 | 本章用法 | 后面在哪用到 |
|---|---|---|
| trait `TokenEstimator` | 可注入、确定性估算 | 第 20 章评测成本 |
| `RangeInclusive` | 表达压缩的 turn 区间 | 第 16 章 fork/rollback |
| `Vec::drain` | 在边界移出旧 turn | 第 5 章事件折叠 |
| append-only 摘要 | 保 prompt cache、可重放 | 第 16 章 JSONL |
| 快照测试 | 断言约束留存 | 第 20 章回归集 |

## 章末验收

- [ ] 100 轮会话后，第 5 轮的不变量仍在最终 prompt 中可匹配
- [ ] 压缩切点总是完整 user turn 边界，工具调用不被切断
- [ ] `context_window - reserve_tokens` 触发后 token 曲线明显回落
- [ ] 每次构建 prompt 输出 cache 前缀/后缀统计
- [ ] JSONL 中原始 turn 在压缩后仍可完整回放

## 读者挑战

1. 一个长工具调用返回 50KB 日志，但只有前 20 行相关。**请设计“文件级保留、内容级摘要”的两级策略，并证明它不会把错误日志误删。**
2. 服务端压缩返回的“引用”失效了怎么办？**本地应保留哪些最小锚点，才能保证 resume 不依赖服务端状态？**
3. 若摘要模型本身出错，把“禁止发布”写成“允许发布”。**如何用语义校验或原文回链检测这种反转？**

## 下一章预告：会话会死，历史不该死

压缩已经产生派生摘要，但我们还需要一套持久化机制，让“发生了什么”在进程崩溃、会话中断、方案分叉后仍然可信。下一章将兑现第 5 章的残缺行伏笔：JSONL 主存、写临时文件再原子 rename；并基于事件流实现 resume、fork、rollback。JSONL 负责真相，SQLite 只做旁挂索引。

---


# 第 16 章　会话持久化：resume、fork 与 rollback

**本章任务**：把第 5 章的 `Rollout` 从“能追加的日志”升级成可恢复的会话存储。崩溃时即使最后一刻被打断，文件也必须是完整 JSONL；恢复时能从事件流重建 Thread；分叉时开出独立分支；回滚时回到指定 turn。SQLite 只做旁挂索引，不参与真相来源。

第 15 章的压缩依赖可重放的原文；本章就是它的基础。若持久化不可靠，“上下文压缩是否正确”“这次是否比上次好”都无从验证。

## 16.1 兑现第 5 章的残缺行伏笔

第 5 章已经解释：每行 flush，牺牲性能换取不丢数据；读时跳过坏行，换取可恢复。但“最后一行写一半就 kill”仍会留下半行，迫使读取逻辑容忍畸形。更彻底的做法是**先写临时文件，再原子 rename**。

为什么不用“读取时修复”？因为修复会让真相源发生歧义：到底原本是这一行，还是你猜的？原子替换让主文件**永远只包含完整行**；临时文件要么是完整新版本，要么被丢弃。

```rust
// crates/mcx-core/src/rollout.rs
use std::io::BufWriter;

pub struct Rollout {
    writer: BufWriter<File>,
    path: PathBuf,
    tmp_suffix: &'static str,
}

impl Rollout {
    pub fn append(&mut self, rec: &Record) -> Result<(), RolloutError> {
        let mut line = serde_json::to_vec(rec)?;
        line.push(b'\n');

        // 1) 写临时文件：包含全部已提交行 + 新行
        let tmp = self.tmp_path();
        let mut tmp_file = OpenOptions::new().create(true).truncate(true).write(true).open(&tmp)?;
        // 把既有内容复制过来（小文件实现）；生产可用 hardlink/copy-on-write。
        if self.path.exists() {
            std::io::copy(&mut File::open(&self.path)?, &mut tmp_file)?;
        }
        tmp_file.write_all(&line)?;
        tmp_file.flush()?;
        tmp_file.sync_all()?;

        // 2) 原子替换：POSIX 上 rename 是原子操作
        std::fs::rename(&tmp, &self.path)?;
        self.writer = BufWriter::new(File::open(&self.path)?);
        Ok(())
    }

    fn tmp_path(&self) -> PathBuf {
        let mut p = self.path.clone();
        p.set_extension(format!("{}.tmp", std::process::id()));
        p
    }
}
```

这里有两处刻意保留的工程取舍：

- **每次 append 重写整个文件**教学成本最低、最易证明正确性；高频场景应改为“追加普通日志 + 周期快照/checkpoint”，但 checkpoint 本身仍用临时文件替换。
- **`sync_all`** 保证 OS 缓冲区落盘；只在事务性事件（审批、压缩、关键 tool result）路径上付出该成本，流式 delta 不值得逐条 fsync。

无论哪种优化，**真相源始终是完整 JSONL 文件**，不是内存中的 `BufWriter`。

## 16.2 为什么 JSONL 主存、SQLite 只做索引

第 5 章已给出对比：JSONL 崩溃友好、可 diff、可 grep、可顺序回放；SQLite 擅长“列出所有会话、按时间范围查 tool call”。因此 mini-codex 的分工是：

| 职责 | 存储 | 是否真相源 |
|---|---|---|
| 事件流、Item、压缩原始记录 | `rollout-<thread>.jsonl` | 是 |
| 会话列表、turn 起止、工具时间线 | `mcx-index.sqlite` | 否（可重建） |
| 当前 Thread 快照 | 内存 + JSONL 重建 | 由 JSONL 派生 |

“旁挂”是可重建性的关键。删除 SQLite，mini-codex 仍能逐行回放所有会话；删除 JSONL，SQLite 只是无法解释的碎片。**事件溯源在单机上的轻量实践**正是如此：真相是追加日志，索引是为查询便利建立的缓存。

```sql
CREATE TABLE IF NOT EXISTS threads(
    id TEXT PRIMARY KEY, created_ms INTEGER NOT NULL, cwd TEXT
);
CREATE TABLE IF NOT EXISTS turns(
    thread_id TEXT NOT NULL, turn INTEGER NOT NULL,
    started_ms INTEGER NOT NULL, ended_ms INTEGER,
    PRIMARY KEY(thread_id, turn)
);
CREATE TABLE IF NOT EXISTS tool_calls(
    thread_id TEXT NOT NULL, turn INTEGER NOT NULL, call_id TEXT PRIMARY KEY,
    name TEXT NOT NULL, started_ms INTEGER, ended_ms INTEGER, is_error INTEGER NOT NULL DEFAULT 0
);
PRAGMA journal_mode=WAL;
```

`WAL` 让读不阻塞写、写不阻塞读，适合 CLI 与后台索引并发。但 WAL 不解决“两个 mini-codex 同时写同一 thread”的语义冲突，那是下一层锁文件的责任。

## 16.3 并发：锁文件 + WAL，而不是“SELECT FOR UPDATE”

```rust
pub struct IndexGuard<'a> {
    conn: &'a Connection,
    _lock: File,
}

impl IndexGuard<'_> {
    pub fn acquire(path: &Path, conn: &Connection) -> Result<Self, IndexError> {
        let lock_path = path.with_extension("lock");
        let lock = OpenOptions::new().create(true).write(true).open(&lock_path)?;
        // 进程级互斥；超时返回错误而非死等
        flock_exclusive(&lock, Duration::from_secs(5))?;
        Ok(Self { conn, _lock: lock })
    }
}
```

锁文件只保护**索引更新**这一临界区；JSONL 本身靠 append + atomic rename 保持崩溃安全。不能只用 SQLite 的忙重试：如果两个进程同时 rename 临时文件，后到的会覆盖前者，除非你以锁串行化。锁获取失败要返回“另一实例持有锁”，让用户选择 resume 只读模式，而不是无限重试。

## 16.4 持久化分级：Limited 与 Extended

并非所有运行都需要完整时间线。第 12 章的审计要求越高，记录越完整；CI 或短命令可以选择省空间。

| 策略 | 记录内容 | 适合 | 可恢复性 |
|---|---|---|---|
| `Limited` | UserMessage、AgentMessage、关键 ToolCall/ToolResult、审批 | CI、快速命令 | 可 resume 对话，但无逐字节输出 |
| `Extended` | 加上 PatchApplyEnd、ExecCommandEnd、Reasoning、指标、完整文件快照引用 | 本地审计、调试、评测 | 可完整导出时间线 |

分级发生在**写入端**，而不是读取端。不能“先全记内存、再按模式挑”，因为崩溃时最需要的审计恰恰会丢失。Extended 记录的额外字段仍遵循第 5 章的演进规则：新增 Item variant、旧代码遇到就 `Unknown`。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Durability {
    Limited,
    Extended,
}

impl Record {
    fn permitted_in(self, mode: Durability) -> bool {
        match (self.item, mode) {
            (Item::Reasoning { .. }, Durability::Limited) => false,
            (_, _) => true,
        }
    }
}
```

## 16.5 从事件流重建：resume

resume 不是“把文件尾读出来”，而是**确定性重放**：从第一条 Record 重建 Thread、审批状态、工具调用结果索引，然后只让模型继续未完成的最后一轮（若有）。

```rust
impl Thread {
    pub fn replay(records: impl IntoIterator<Item = Record>) -> Result<Self, ReplayError> {
        let mut thread = Thread::default();
        for rec in records {
            thread.apply(rec)?; // 纯函数：把 Item 放进对应 turn
        }
        Ok(thread)
    }

    fn apply(&mut self, rec: Record) -> Result<(), ReplayError> {
        let turn = match self.turns.last_mut().filter(|t| t.index == rec.turn) {
            Some(t) => t,
            None => {
                if rec.turn != self.turns.len() {
                    return Err(ReplayError::TurnOutOfOrder(rec.turn));
                }
                self.turns.push(Turn { index: rec.turn, items: Vec::new() });
                self.turns.last_mut().unwrap()
            }
        };
        if matches!(rec.item, Item::Unknown) {
            tracing::warn!(?rec, "未知 Item 降级为 Unknown，不进入 turn");
            return Ok(());   // 前向兼容：未知 variant 不破坏重建
        }
        turn.items.push(rec.item);
        Ok(())
    }
}
```

关键细节：`apply` 是**纯函数**。只要 JSONL 不变，重建结果就不变；这也让 fork 与测试容易实现。遇到 `Item::Unknown` 不报错，而是沿用第 5 章的宽容策略：未知的未来 variant 不影响旧代码重放。

resume 的最终动作取决于最后一轮是否闭合：

- **已闭合**（最后一条是 `AgentMessage` 或最终 `ToolResult`）：直接开始新 turn。
- **未闭合**（最后一条是 `ToolCall`，或摘要丢失）：向用户报告“会话在工具执行中中断”，让其在安全模式下选择“重放工具结果 / 跳过 / 人工检查”，绝不自动伪造结果。

## 16.6 fork：分叉不是复制文件

fork 的直觉是“复制一份继续改”，但如果只 `cp rollout.jsonl`，两个进程会共享锁、共享 SQLite、互相覆盖。正确的语义是：**复用既有事件流的可见前缀，分配新 thread id，从分叉点起独立追加。**

```rust
pub struct ForkPoint {
    pub thread_id: String,
    pub turn: usize,   // 包含这一轮
}

impl Store {
    pub fn fork(&self, point: ForkPoint, label: &str) -> Result<Thread, StoreError> {
        let base = self.load(&point.thread_id)?;
        let prefix = base.turns.iter()
            .take_while(|t| t.index <= point.turn)
            .cloned()
            .collect::<Vec<_>>();

        let new_id = format!("{}.fork-{label}", point.thread_id);
        let new_thread = Thread { id: new_id.clone(), created_ms: now_ms(), turns: prefix };

        // 新线程的物理日志独立；源 JSONL 只读
        let new_path = self.rollout_path(&new_thread.id);
        let mut out = Rollout::create(&new_path)?;
        for turn in &new_thread.turns {
            for item in &turn.items {
                out.append(&Record::from_item(now_ms(), &new_thread.id, turn.index, item.clone()))?;
            }
        }
        self.index.insert_thread(&new_thread)?;
        Ok(new_thread)
    }
}
```

> **`Record::from_item` 是这里的落盘辅助函数**：它等价于字面量构造 `Record { thread_id: &new_thread.id, turn: turn.index, item: item.clone(), created_ms: now_ms() }`。前缀里的每条 `Item` 都要写进新线程的物理日志，新分支才能脱离源 JSONL 独立重放——16.6 承诺的“分支可独立重放”靠的就是这一步。

注意：**fork 不是硬链接或 copy-on-write**。它把前缀作为新事件流重新落盘，代价是磁盘，收益是两条分支可以独立重放、独立压缩、独立删除。源线程保持 append-only，不被分叉污染。这与 Git 的“共享对象、分叉引用”不同：agent 会话更看重审计独立性。

## 16.7 rollback：回到某轮，但真相不消失

rollback 听起来像“undo”，但若真删除之后的记录，就破坏了可回放性。mini-codex 的 rollback 实际是**创建一个回滚后的 fork**：

```bash
mini-codex rollback <thread> --before-turn 8 --label experiment
```

实现把 `[0..=7]` 作为新线程前缀；第 8 轮及之后仍完整保留在原 `rollout-<thread>.jsonl` 中。于是：

- `thread` 原记录不变，满足 append-only；
- `thread.rollback-before-8.experiment` 成为可 resume 的新分支；
- 索引记录 `parent`、`fork_point`、`label`，形成可视图。

如果确实要“永久删除”，那是单独的 `forget` 命令，需显式确认并脱敏审计；它不属于 rollback。这个区分很重要：**回滚是创建工作分支，遗忘才是破坏真相源。**

## 16.8 完整测试：崩溃、恢复、分叉、回滚

```rust
#[test]
fn kill_after_rename_leaves_intact_jsonl() {
    let dir = tempdir();
    let path = dir.path().join("rollout.jsonl");
    let mut rollout = Rollout::open(&path).unwrap();

    rollout.append(&rec(1, "ok")).unwrap();
    // 模拟：临时文件存在，但主文件已完成原子替换
    drop(rollout);
    assert!(path.is_file());
    assert!(!path.with_extension("jsonl.tmp").exists(),
            "崩溃遗留临时文件应在启动时清理，或主文件仍完整");

    let recs = Rollout::read_all(&path).unwrap();
    assert_eq!(recs.len(), 1);
}

#[tokio::test]
async fn resume_continues_after_replaying_full_event_stream() {
    let recs = vec![rec(0, "hi"), rec(0, "hello"), rec(1, "do thing")];
    let thread = Thread::replay(recs).unwrap();
    assert_eq!(thread.turns.len(), 2);

    let mut session = Session::from_thread(ScriptedLlm::one("done"), thread);
    session.submit("继续".into()).await;
    assert!(last_turn_in(&session).contains("done"));
}

#[test]
fn fork_has_independent_log_and_preserves_source() {
    let store = make_store();
    let base = store.create_thread("t").unwrap();
    store.append(base.id(), rec(0, "a")).unwrap();
    store.append(base.id(), rec(1, "b")).unwrap();

    let fork = store.fork(ForkPoint { thread_id: base.id().into(), turn: 0 }, "try-b").unwrap();
    store.append(&fork.id, rec(1, "c")).unwrap();

    assert_eq!(store.load(base.id()).unwrap().turns.len(), 2);
    assert_eq!(store.load(&fork.id).unwrap().turns.len(), 2); // 0 轮 + 新第 1 轮
}

#[test]
fn rollback_creates_branch_and_keeps_original() {
    let store = make_store();
    let t = store.create_thread("main").unwrap();
    for i in 0..5 { store.append(t.id(), rec(i, "x")).unwrap(); }

    let rb = store.rollback(t.id(), 3, "safe").unwrap();
    assert_eq!(store.load(t.id()).unwrap().turns.len(), 5, "原始不缩短");
    // 按 16.7 的语义：--before-turn 8 → 前缀 [0..=7]，即保留 before_turn 轮
    assert_eq!(store.load(&rb).unwrap().turns.len(), 3);
}
```

第一个测试兑现第 5 章“写临时文件再 rename”的承诺；第二个证明 resume 由事件流重建；第三个和第四个证明 fork/rollback 是创建分支，而非篡改历史。

## 16.9 导出完整工具调用时间线

旁挂 SQLite 的价值就在于这种查询：

```rust
pub fn export_tool_timeline(conn: &Connection, thread_id: &str) -> Vec<ToolEvent> {
    conn.prepare(
        "SELECT turn, call_id, name, started_ms, ended_ms, is_error
         FROM tool_calls WHERE thread_id = ? ORDER BY started_ms"
    ).unwrap()
    .query_map([thread_id], |r| Ok(ToolEvent::from_row(r))).unwrap()
    .filter_map(Result::ok)
    .collect()
}
```

导出可以从 SQLite 快速生成；若 SQLite 缺失，退化方案是顺序扫描 JSONL，按 `ToolCall`/`ToolResult` 的 `call_id` 配对。后者更慢但仍是唯一真相。这也说明索引为何是“旁挂”：它可以被重建，JSONL 不能。

## 16.10 Design Rationale

**Q：为什么不直接用 SQLite 存所有事件？**

因为 agent 会话最自然的形态是追加日志，而日志最自然的问题是“我那一千行到底发生了什么”。JSONL 对崩溃、diff、grep、顺序回放天然友好；SQLite 的强项——索引、聚合、并发查询——交给旁挂层。把二者职责倒置，会得到一个既难手工检查、又需事务保护的“主数据库”。

**Q：为什么 fork 要复制前缀，而不是引用？**

因为引用会制造共享可变状态：源线程压缩、遗忘或重写时，分支突然失效。复制让每条线程独立可回放，代价是磁盘；对于本地 agent 会话，这个代价通常可接受。若未来需要空间优化，也应先加 content-address 的共享 item 池，而不是让 fork 共享可写日志。

**Q：为什么 rollback 不直接删除记录？**

因为删除破坏 append-only，也让“比较两次运行”不可能。rollback-as-branch 同时保留旧事实和新选择；它与版本控制中的“revert 提交”不同，后者保留历史、但这里是 agent 分支管理，创建分支更清晰。

## 避坑专栏 #17：用 `BufWriter::flush` 假装原子持久化

错误写法：

```rust
// 危险：flush 只保证进入 OS；崩溃后文件可能缺最后一行
self.writer.write_all(line)?;
self.writer.flush()?;
```

即使 flush 成功，临时状态、rename 和索引更新之间仍有窗口。更糟的是，如果直接在原文件追加，写到一半的进程被杀，主文件就留下畸形行。

**解法**：先写 `<file>.tmp`、fsync，再 `rename` 替换；启动时清理遗留 `.tmp`；索引更新放在锁内。`flush` 仍保留用于“尽力实时”，但不能被当作事务提交。

**通用形式**：对外承诺“崩溃后可恢复”的文件，必须有临时 + 原子替换 + 崩溃清理三件套。第 3 章的“事件流是真相来源”因此才真正成立。

## AI 软件工程原理 #16

> **可回放是评测和调试的前提。**

没有完整事件记录，你连“这次比上次好还是差”都无法可靠回答。JSONL 作为真相源、压缩只产生可重算的派生摘要、fork 创建独立分支、rollback 保留原历史，这一切都服务于同一目标：让任意一次运行可被精确重建。

这条原理连接全书：

- 第 3 章的 Event/Op 分离，使事件流可被独立消费；
- 第 5 章的 Item 细粒度与 append-only，使回放有挑选粒度且安全；
- 第 15 章的压缩可在原事件流上重算，不必修改历史；
- 第 20 章的回归评测将直接比较两次回放的 tool timeline。

若持久化有一处“聪明地改写历史”，这条证据链就会断。因此宁可多一份 fork，也不少一段原始记录。

## Rust 修炼小结

| 概念 | 本章用法 | 后面在哪用到 |
|---|---|---|
| 临时文件 + `rename` | 原子替换 JSONL | 配置快照、规则更新 |
| `sync_all` | 事务性落盘 | 审批审计 |
| SQLite WAL | 查询索引并发 | 第 17 章服务元数据 |
| 文件锁 | 串行化索引更新 | 多实例会话管理 |
| 纯函数 `Thread::apply` | resume/fork/rollback 复用 | 第 20 章重放评测 |

## 章末验收

- [ ] `mini-codex resume` 从 JSONL 重建 Thread 并继续新 turn
- [ ] `fork` 生成独立 `rollout-*.jsonl`，源线程不被修改
- [ ] `rollback --before-turn N` 保留原始记录，只创建新分支
- [ ] 模拟 `SIGKILL` 后，主 JSONL 仍可完整读取且无残缺行
- [ ] `export-tool-timeline` 能输出按时间排序的完整调用序列

## 读者挑战

1. 会话在“ToolCall 已发出、ToolResult 尚未写盘”时崩溃。**resume 应如何区分“结果丢失”和“结果已落盘但 JSONL 未更新”？提示：考虑幂等 call_id。**
2. 两个 CLI 同时对同一 thread 写 Extended 记录。**锁粒度应到 thread、session 还是文件？能否允许只读 resume 并发？**
3. JSONL 中误记录了 API key。**设计一个 `forget` 命令，使其既破坏可回放性又留下审计痕迹；它是否违反本章原理？**

## 下一章预告：同一颗引擎，服务无数个前端

记忆篇结束后，mini-codex 已拥有可信配置、项目知识、可持续上下文和可回放历史。但它仍是单用户 CLI。这一部分的第一件事是把 Op/Event 队列对升级成可被 IDE、桌面端和 SDK 驱动的协议：JSON-RPC over stdio/WebSocket、thread/turn/item 的查询 API、以及“服务端如何安全持有多个会话”。你会发现，第 3 章的解耦在这里终于兑现——引擎几乎不用改，只是多了一组消费者。

---

## 引用来源

[1] /data/workspace/_API契约.md
> `pub struct Record { pub v: u32, pub ts_ms: u64, pub thread_id: String, pub turn: usize, pub item: Item }`

[2] /data/workspace/用Rust造AI_Agent_全书大纲.md
> 三层：全局 `~/.codex/AGENTS.md`（个人偏好，约 20 行）→ 仓库根（团队约定，约 40 行）→ 子目录（特性规则）

[3] /data/workspace/第3-5章_样章.md
> **② 历史只追加。** `history.push` 从不修改已有内容。这在第 15 章会变成关键——**append-only 才能保住 prompt cache**

[4] /data/workspace/第3-5章_样章.md
> **`Event` 和 `Item` 的分工**：Event 负责“现在发生了什么”，Item 负责“最终留下了什么”。

[5] /data/workspace/第3-5章_样章.md
> `pub struct Rollout { writer: BufWriter<File>, path: PathBuf }`

[6] /data/workspace/第3-5章_样章.md
> **第 3 条最容易被忽略。** 手写的“旧格式假数据”和你一年前真实写出去的数据不是一回事。

[7] /data/workspace/ch10-12.md
> **机制是“审批独立于沙箱”；预设只是把两个常用值绑在一起。**

[8] /data/workspace/ch10-12.md
> `mcx-sandbox` 不负责业务逻辑，只把 `SandboxRequest` 翻译成平台原语

[9] /data/workspace/_API契约.md
> `pub fn append(&mut self, rec: &Record) -> Result<(), RolloutError>;`

[10] /data/workspace/用Rust造AI_Agent_全书大纲.md
> **切点优先选在 user turn 边界**，保留近期上下文

[11] /data/workspace/第3-5章_样章.md
> **为什么用 `tag = "type"` 的内部标签，而不是 `{"kind": ..., "data": ...}` 的外部标签？**

[12] /data/workspace/第3-5章_样章.md
> 遇到无法解析的行，记录警告并跳过。

[13] /data/workspace/第3-5章_样章.md
> `Item` 是一个 enum，落盘时需要自描述类型。用**内部标签枚举**：

[14] /data/workspace/_API契约.md
> `pub struct Session<C: LlmClient> { client: C, history: Vec<Message>, op_rx: mpsc::Receiver<Op>, event_tx: mpsc::Sender<Event>, cancel: CancellationToken, turn: usize }`

[15] /data/workspace/_API契约.md
> `pub struct Thread { pub id: String, pub created_ms: u64, pub turns: Vec<Turn> }`

[16] /data/workspace/_API契约.md
> `pub struct Turn { pub index: usize, pub items: Vec<Item> }`

[17] /data/workspace/第3-5章_样章.md
> **不认识的事件类型 → 忽略（可能是未来功能）**
[18] /data/workspace/_写作规范.md
> 每章有**可运行的代码增量**，读完 22 章手上有真东西。
[19] /data/workspace/_写作规范.md
> 关键论点用 `**加粗**`；重要警示用 `>` 引用块。
[20] /data/workspace/_写作规范.md
> 测试命名用完整句子描述行为，如 `any_split_point_yields_same_events`


# 第五部分　长成真系统（第 17–22 章）

> 第 16 章把会话、索引和回放固化成了可审计的 JSONL。现在 mini-codex 已经会思考、会记忆、也会留下证据；但它仍然只认识一种入口：一个读 stdin、写 stdout 的 CLI。本部分要把这层壳剥掉——不是因为它不好，而是因为**同一颗引擎要同时被 IDE、CI、SDK、远程服务和人眼使用**。如果每次换界面都要复制一遍引擎逻辑，系统就会从“一个 agent”退化成“一堆互相抄袭的脚本”。
>
> 前半三章分别解决三件事：**第 17 章把引擎变成可驱动的状态机，第 18 章让工具能力在运行时生长，第 19 章把事件流接成人类能操作的终端界面。** 它们的共同前提是第 3 章那个看似过度的决定：引擎不认识界面，只通过 `Op` 与 `Event` 两条 channel 和世界通信。

---

# 第 17 章　队列对协议与 app-server

**本章任务**：给 mini-codex 加一个稳定的 JSON-RPC 服务边界，让 Python/TypeScript SDK、IDE 插件、WebSocket 前端和既有 CLI 都驱动同一个 `Session`。核心不是“再多一个 main”，而是把 `Op`/`Event` 的完整队列语义固定下来。

---

## 17.1 一个能跑的 CLI，为什么还不能直接嵌入 IDE

第 3 章留下了一个诊断表：把模型调用、键盘输入和终端输出塞进一个 `loop`，会在 TUI、取消、复用和测试四个地方爆炸。前 16 章已经解决了其中大部分，但“**可被复用**”这一项还没有真正兑现。现在假设你要做一个 VS Code 插件。最省事的方案是直接 `spawn("./mcx")`，然后解析它的 stdout：

```rust
// 反例：把交互协议伪装成 API
let mut child = Command::new("mcx")
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()?;
child.stdin.as_ref().unwrap().write_all(b"fix the panic\n")?;
```

这个方案能演示，却经不起维护。第一，CLI 的 stdout 是给人看的：颜色、换行、进度条、错误提示随时会变，任何改动都是 breaking change。第二，多轮会话状态藏在子进程内存里，调用方只能靠“猜现在该读哪一行”来同步。第三，取消、关闭、背压和异常退出无法映射到结构化语义；你最终会发明一套藏在 ANSI 转义序列里的私有协议。

正确问题不是“怎么从 IDE 调用 CLI”，而是“**引擎已经有哪些不依赖界面的操作**”。答案就是 `Op`。只要外部世界能提交 `Op`、订阅 `Event`，它就不必关心引擎是跑在终端、浏览器还是 Kubernetes Pod 里。

```text
   Python SDK      TS SDK      IDE        WebSocket UI     CLI
        │            │          │             │            │
        └────────────┴──────────┴─────────────┴────────────┘
                              │  Op   ▲  Event
                              ▼        │
                        ┌───────────────────┐
                        │   JSON-RPC bridge │
                        └────────┬──────────┘
                                 │  Op   ▲  Event
                                 ▼        │
                        ┌───────────────────┐
                        │      Session      │
                        └───────────────────┘
```

这就是“引擎与表面分离”的具体形状：SDK 不导入 `Session`，只导入协议；协议只描述消息，不导入渲染器。本章所有代码都放在新 crate `mcx-protocol` 和 `mcx-server` 中，前者继续不依赖任何 workspace crate，后者依赖 `mcx-core`。

> **协议层存在的意义，不是让远程调用变优雅，而是让界面成为可替换零件。** 如果跳过这一层，第 19 章每改一次组件，都要同步修改 SDK；反过来，只要协议稳定，CLI 甚至可以是服务器的一个薄前端。

---

## 17.2 `Op`/`Event` 队列对的完整形态

第 3 章已经定义：

```rust
pub enum Op {
    UserInput { text: String },
    Interrupt,
    Shutdown,
}

pub enum Event {
    TurnBegin { turn: usize },
    AgentMessageDelta(String),
    TurnComplete { turn: usize, text: String },
    Error(String),
    Shutdown,
}
```

那时只用到 `UserInput`、`Shutdown` 和一个占位 `Interrupt`。要支撑 RPC，还缺三件事：**会话标识、异步结果关联、服务端主动通知**。于是协议从“两个枚举”升级为“两个队列 + 两类消息”。

| 方向 | 类型 | 必须携带 | 性质 |
|---|---|---|---|
| Client → Server | `Op` | `thread_id`、请求 id | 命令；可要求回应 |
| Server → Client | `Event` | `thread_id`、请求/订阅 id | 状态变化；可多播 |
| Server → Client | `Result` | 原请求 id | 一次请求的终态 |
| Client → Server | `Cancel` | 原请求 id | 通知，不要求回应 |

第 5 章说过：Thread 是会话，Turn 是“一次用户输入及其完整处理”，Item 是最小记录。这里要把它们从落盘结构升格为 API 资源。为保持契约兼容，不修改已有枚举，而是加一个请求信封：

```rust
// crates/mcx-protocol/src/lib.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequestId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum ClientMessage {
    /// 创建或接管一个 thread
    #[serde(rename = "thread/create")]
    ThreadCreate { id: RequestId, params: CreateThread },
    /// 向 thread 提交一轮用户输入
    #[serde(rename = "thread/prompt")]
    ThreadPrompt { id: RequestId, params: PromptParams },
    /// 取消某个仍在运行的 prompt
    #[serde(rename = "request/cancel")]
    RequestCancel { id: RequestId, params: CancelParams },
    /// 列出历史 turn 与 item
    #[serde(rename = "thread/get")]
    ThreadGet { id: RequestId, params: GetThread },
    /// 优雅关闭整个 server
    #[serde(rename = "server/shutdown")]
    ServerShutdown { id: RequestId },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateThread {
    pub thread_id: Option<String>,
    pub resume_from: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptParams {
    pub thread_id: String,
    pub text: String,
    /// 本次请求的服务端句柄；取消用它关联
    pub request_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelParams {
    pub thread_id: String,
    pub request_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetThread {
    pub thread_id: String,
    pub after_turn: Option<usize>,
}
```

`RequestId` 是 JSON-RPC 的请求 id；`request_id` 是业务取消句柄。二者不能合并：一个 `thread_prompt` 请求在其整个生命周期内会触发很多 `Event`，调用方取消的是“这个 prompt”，不是“这条通知”。这个区分在 HTTP 场景尤其重要——请求可能早已返回，但流式任务仍在继续。

**`Session::submission_loop` 的契约必须写成永不主动退出，除非收到 `Op::Shutdown`。** 这是全章最重要的不变式：

```rust
// crates/mcx-core/src/session.rs
impl<C: LlmClient> Session<C> {
    pub async fn submission_loop(&mut self) {
        loop {
            let Some(op) = self.op_rx.recv().await else {
                // 所有 sender 都已关闭：连 server 都不要它了
                break;
            };
            match op {
                Op::UserInput { text } => self.run_turn(text).await,
                Op::Interrupt => self.cancel.cancel(),
                Op::Shutdown => {
                    self.emit(Event::Shutdown).await;
                    break;
                }
            }
        }
    }
}
```

`run_turn` 是第二层循环，工具往返是第三层循环；前两层早已存在，第三层由第 6 章的工具系统闭合。RPC 层绝不绕过这两层去直接调用模型，否则“取消”“限流”“审计”和“事件顺序”会立即失效。

---

## 17.3 把引擎包成 JSON-RPC 2.0 服务

JSON-RPC 2.0 是传输无关的：同进程、socket、HTTP、stdio 都能承载[citation:12]。它的三种消息正好对应 agent 的三种交互：**请求要结果、通知不答复、流式进度走服务端通知**。mini-codex 采用“一个请求一个 JSON-RPC id，多次事件共享订阅 id”的映射。

```json
→ {"jsonrpc":"2.0","id":1,"method":"thread/create","params":{}}
← {"jsonrpc":"2.0","id":1,"result":{"thread_id":"thr_01"}}
→ {"jsonrpc":"2.0","id":2,"method":"thread/prompt",
   "params":{"thread_id":"thr_01","text":"加一个超时测试","request_id":"r1"}}
← {"jsonrpc":"2.0","method":"event","params":{"thread_id":"thr_01","request_id":"r1",
   "event":{"TurnBegin":{"turn":1}}}}
← {"jsonrpc":"2.0","method":"event","params":{"thread_id":"thr_01","request_id":"r1",
   "event":{"AgentMessageDelta":"先看"}}}
← {"jsonrpc":"2.0","id":2,"result":{"status":"completed","turn":1}}
```

服务端把 `client_message` 翻译为 `Op`；反过来，把 `Event` 翻译为通知。注意 `Event` 早就是 `Clone + PartialEq`，这正是第 3 章埋下的回报：一个事件可以同时发给 WebSocket 订阅者、文件回放器和测试收集器。

```rust
// crates/mcx-server/src/bridge.rs
use mcx_core::{Event, Op, Session};
use mcx_protocol::{ClientMessage, RequestId, Thread};
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};

pub struct ThreadHandle {
    op_tx: mpsc::Sender<Op>,
    cancel: tokio_util::sync::CancellationToken,
    next_turn: usize,
}

pub struct AppServer {
    threads: HashMap<String, ThreadHandle>,
    event_tx: mpsc::Sender<WireEvent>,
    // request_id → 该 prompt 的完成信号
    pending: HashMap<String, oneshot::Sender<PromptOutcome>>,
}

#[derive(Debug, Clone, Serialize)]
struct WireEvent {
    thread_id: String,
    request_id: String,
    event: Event,
}
```

线程不是“每个请求 new 一个 Session”。正确所有权是：**一个 thread 有一个常驻 `submission_loop`，一个 server 持有向其投递 `Op` 的 sender**。这样同一个 WebSocket 连接断开后重连，也能通过 `thread_id` 继续投递；引擎不知道有网络存在。

```rust
impl AppServer {
    fn spawn_thread(&mut self, id: String, session: Session<impl LlmClient>) {
        let (op_tx, op_rx) = mpsc::channel(16);
        let (event_tx, mut event_rx) = mpsc::channel(128);
        let mut session = session.with_channels(op_rx, event_tx.clone());

        // 任务 A：引擎，和 CLI 用的是完全相同的 Session
        tokio::spawn(async move { session.submission_loop().await });

        // 任务 B：把引擎事件扇出给所有订阅者
        let subscribers = self.subscribe_table.clone();
        tokio::spawn(async move {
            while let Some(wire) = event_rx.recv().await {
                subscribers.send(&wire.thread_id, &wire.request_id, wire.event).await;
            }
        });

        self.threads.insert(id, ThreadHandle { op_tx, .. });
    }
}
```

### 翻译层：唯一允许“懂两种语言”的地方

```rust
async fn handle(&mut self, msg: ClientMessage) -> JsonRpcResponse {
    match msg {
        ClientMessage::ThreadPrompt { id, params } => {
            let Some(handle) = self.threads.get(&params.thread_id) else {
                return error(id, -32602, "unknown thread");
            };
            let (done_tx, done_rx) = oneshot::channel();
            self.pending.insert(params.request_id.clone(), done_tx);

            // 非阻塞投递：背压发生在 op channel，不在 HTTP 线程
            if handle.op_tx.try_send(Op::UserInput { text: params.text }).is_err() {
                self.pending.remove(&params.request_id);
                return error(id, -32000, "engine overloaded");
            }
            // 立即返回“已接受”，不等待整个 turn
            JsonRpcResponse::Accepted(id, params.request_id, done_rx)
        }
        ClientMessage::RequestCancel { id, params } => {
            if let Some(handle) = self.threads.get(&params.thread_id) {
                handle.cancel.cancel();
            }
            JsonRpcResponse::result(id, serde_json::json!({}))
        }
        _ => error(id, -32601, "method not found"),
    }
}
```

> **其余分支从略**：`ThreadCreate`/`ThreadGet`/`ServerShutdown` 与上面两个分支模式相同（查 thread → 投 `Op` → 回结果或等通知），为避免重复不再展开，读者可照 `ThreadPrompt` 分支补齐。不要把协议里真实存在的方法落进 `_ => -32601` 的兜底——那是给“协议里根本没有的方法”准备的。

`Op::Interrupt` 现在终于有活干了：它取消当前 turn，但不会删除 thread。这样“取消一轮”和“关闭会话”成为两种语义，而不是让 Ctrl+C 既停止又销毁。

> **所有入站命令都必须通过 `Op`。** 如果某个 RPC 方法直接调用 `session.run_turn`，你就重新制造了第 3 章那锅“谁都能驱动引擎”的意大利面。

---

## 17.4 stdio、WebSocket 与三级资源模型

JSON-RPC 消息层不变，变化的只是字节搬运工。stdio 用换行分隔；WebSocket 用单条 JSON 帧；两者共用同一个 `AppServer`。

```rust
// crates/mcx-server/src/transports/stdio.rs
pub async fn serve_stdio(server: AppServer) -> std::io::Result<()> {
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() { continue; }
        match serde_json::from_str::<ClientMessage>(&line) {
            Ok(msg) => server.dispatch(msg).await,
            Err(e) => server.write_error(None, -32700, e.to_string()).await,
        }
    }
    Ok(())
}
```

stdio 的纪律很简单：**stderr 归日志，stdout 只放 JSON-RPC**。子进程的诊断信息一旦混入 stdout，就会污染协议流；同理，不要在 stdout 打印 banner。HTTP/WebSocket 则要注意：连接断开不等于 thread 死亡，因此不能直接把 `event_tx` 的生命期绑到 WebSocket 句柄上。

三级 API 的命名来自数据生命周期，而不是 URL 美学：

| 级别 | 资源 | 典型方法 | 生命周期 |
|---|---|---|---|
| Thread | `thread/create`、`thread/get` | 跨请求 | 会话级，可恢复 |
| Turn | `thread/prompt`、`event` | 一次请求 | 可被取消 |
| Item | `thread/get(after_turn=...)` | 增量查询 | 不可变、append-only |

一个 IDE 的典型流程是：`thread/create` → 多次 `thread/prompt` → 订阅事件 → 断线后用 `thread/get` 补齐缺失 turn。这恰好对应第 5 章的持久化模型：**Thread 是目录，Turn 是文件，Item 是 JSONL 行**。API 只是给外部世界一张查询这张表的门票。

---

## 17.5 SDK 为什么“免费”：一个五行的 Python 驱动脚本

协议层把界面彻底推到另一侧，SDK 就退化为“读写 JSON 的客户端”。下面脚本不依赖任何 Rust 绑定：

```python
import json, subprocess

p = subprocess.Popen(
    ["mcx", "serve", "--stdio"],
    stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True,
)

def rpc(method, params, id):
    p.stdin.write(json.dumps({"jsonrpc": "2.0", "id": id, "method": method, "params": params}) + "\n")
    p.stdin.flush()
    return json.loads(p.stdout.readline())

created = rpc("thread/create", {}, 1)
thread_id = created["result"]["thread_id"]
rpc("thread/prompt", {"thread_id": thread_id, "text": "解释 main.rs", "request_id": "r1"}, 2)

for line in p.stdout:
    msg = json.loads(line)
    if msg.get("method") == "event":
        ev = msg["params"]["event"]
        if "AgentMessageDelta" in ev:
            print(ev["AgentMessageDelta"], end="", flush=True)
    elif msg.get("id") == 2:
        break
```

若某天把传输换成 WebSocket，只改传输对象；`created`、`thread_id`、`event` 的形状完全相同。这正是第 3 章诊断表中“**无法被复用**”这一项的兑现：不是给每种宿主写一个 `Session` wrapper，而是给所有宿主一个 `Op`/`Event` 语言。

---

## 17.6 取消、背压与优雅关闭：协议最容易烂的地方

### 取消是请求级，关闭是会话级

`CancellationToken` 由第 3 章占位至今。现在把它接到真正的生命周期：每个 `thread_prompt` 创建一个子 token，取消消息只触发对应 token；`Op::Shutdown` 才关闭整个 server。不要用“全局 Ctrl+C 布尔值”同时表达两者。

```rust
async fn run_turn(&mut self, text: String) {
    self.turn += 1;
    let turn = self.turn;
    self.emit(Event::TurnBegin { turn }).await;

    // 每个 turn 用独立 child token，取消只影响这一轮
    let guard = self.cancel.child_token();
    let completed = tokio::select! {
        r = self.execute_turn(&text, &guard) => Ok(r),
        _ = guard.cancelled() => Err(LlmError::Cancelled),
    };
    // 无论完成、失败还是取消，都发 TurnComplete，让订阅者能闭合 UI
    ...
}
```

### 背压：有界队列不是缺陷，是报警器

`op_rx` 用 `mpsc::channel(16)`、`event_tx` 用 128。当渲染器慢，`Event` 会在 server 端排队；当引擎慢，RPC 层 `try_send` 会失败。后者是好事：它明确告诉客户端“服务器繁忙”，而不是默默吃掉内存。

```rust
// 危险：unbounded 把“下游跟不上”藏进内存
let (tx, rx) = mpsc::unbounded_channel();
```

第 10 章的审批会放大这个坑：审批请求是“引擎等用户、用户等渲染”的双向等待。普通事件与同步审批共用一个有界队列，就可能死锁。**通用解法是为同步审批单独建 channel**，而不是无限扩容通用队列。

### 优雅关闭：先停止收新请求，再排空事件

```rust
pub async fn shutdown(self) {
    // 1. 关闭入站；正在处理的 JSON-RPC 仍会收到结果
    self.inbound.close();
    // 2. 向所有 thread 发 Op::Shutdown
    for (_, handle) in self.threads {
        let _ = handle.op_tx.send(Op::Shutdown).await;
    }
    // 3. 等待引擎把剩余 Event 发完，再关 event channel
    self.event_drain.await;
}
```

顺序不能颠倒：先关 event 订阅者，未发出的 `TurnComplete` 就会被丢弃，SDK 永远等不到终态。

---

## 17.7 用假模型和 ScriptedLlm 测整条 RPC

第 3 章承诺的“不依赖网络的测试”现在开始复利。直接用内存通道模拟一个 JSON-RPC 客户端：

```rust
#[tokio::test]
async fn prompt_completes_with_events_then_can_resume() {
    let (to_server, from_client) = mpsc::channel(16);
    let (to_client, mut from_server) = mpsc::channel(64);
    let session = Session::new(
        ScriptedLlm::new(vec!["v1".into(), "v2".into()]),
        from_client, to_client,
    );

    let server = AppServer::new_for_test(session);
    tokio::spawn(async move { server.run().await });

    to_server.send(ClientMessage::ThreadPrompt {
        id: RequestId("1".into()),
        params: PromptParams { thread_id: "t".into(), text: "hi".into(), request_id: "r1".into() },
    }).await.unwrap();

    let mut deltas = String::new();
    while let Some(msg) = from_server.recv().await {
        match msg {
            ServerMessage::Event { event: Event::AgentMessageDelta(d), .. } => deltas.push_str(&d),
            ServerMessage::Result { id, .. } if id.0 == "1" => break,
            _ => {}
        }
    }
    assert_eq!(deltas, "v1");

    // 同一 thread 再发一轮：状态没有被销毁
    to_server.send(ClientMessage::ThreadPrompt {
        id: RequestId("2".into()),
        params: PromptParams { thread_id: "t".into(), text: "again".into(), request_id: "r2".into() },
    }).await.unwrap();
    // 收集到 TurnComplete { turn: 2, ... }
}
```

这个测试的价值在于：它同时验证**事件顺序、线程复用、流式结束和协议映射**。若有人将来把 `Session` 改成“每请求瞬时对象”，测试会立即失败——这比文档更能守住架构。

```bash
cargo test -p mcx-server
# test prompt_completes_with_events_then_can_resume ... ok
```

---

## 避坑专栏 #18：`Op::Shutdown` 之后，事件去哪儿了？

新手常写成这样：

```rust
// 危险：drop 事件订阅者后还指望对方收到 Shutdown
let (event_tx, _event_rx) = mpsc::channel(128);
session.event_tx = event_tx;
session.submission_loop().await;
```

现象是：客户端偶尔收不全最后的 `TurnComplete` 或 `Shutdown`，WebSocket 端永远挂起。原因不是“channel 有 bug”，而是**drop 接收端会让后续 `send` 立即失败**；而第 3 章的 `emit` 故意忽略失败，所以引擎安静地丢掉了终态。

正确做法是让关闭沿依赖图反向进行：

```rust
// 1. 停止入站请求
drop(inbound);
// 2. 通知引擎；等 submission_loop 返回
handles.shutdown_all().await;
// 3. 此时仍可能有 in-flight Event，最后才关 event_tx
drop(event_bus);
```

**通用形式**：在“生产者会在关闭前发最后一条消息”的系统里，关闭顺序必须是 `停止输入 → 等待生产者 → 关闭输出`。任何“先关输出再等生产者”的代码，都只是把竞争条件推迟到高负载时暴露。

---

## 17.8 Design Rationale

**Q：为一个教学项目做完整 RPC 层，是不是过度设计？**

不是，因为收益不在“远程调用”，而在**强制核心逻辑成为无界面状态机**。若没有这层，新增 TUI、IDE、CI 时最容易的做法是复制 main；有了 `Op`/`Event`，它们都是协议消费者。第 3 章把骨架做对，第 17 章才只需加“翻译”，不用重写。

**Q：为什么不直接让 SDK 调用 Rust 函数？**

进程内绑定最快，但有四个代价：SDK 必须与 mini-codex 同进程；模型调用、工具执行和 UI 共享一个线程池；版本升级变成 ABI 问题；无法在远程机器上托管 agent。JSON-RPC 把“能力”变成消息，反而让 CLI、SDK、IDE 平权。

**Q：为什么事件用通知而不是批量塞进请求结果？**

一次 prompt 会产生不确定数量、不确定时延的增量。把它们全攒到 HTTP 响应里，等于把流式体验重新变成“等 40 秒再输出”；塞进数组也无法支持取消。JSON-RPC 请求负责“开始并获得终态”，通知负责“过程”，二者职责正交[citation:12]。

**Q：为什么 thread 是长生命周期，而 turn 可以取消？**

因为 thread 对应项目工作、可恢复；turn 对应一次意图及其工具链。若取消粒度粗到 thread，用户打断一次就会丢失上下文；若细到每个 token，又无法表达“停止整个计划”。三级资源正好落在三种自然边界上。

---

## AI 软件工程原理 #17

> **引擎与表面分离，能力就能被复用。**

第 3 章已经证明：模型、用户、工具的节奏不同，必须拆成 channel。原理 #17 是它的产品级推论。CLI 只是第一种表面；当协议层稳定后，IDE 插件、CI runner、Python notebook、手机 App 都只是新的 `Event` 消费者与 `Op` 生产者。

这解释了三层结构为何不能省略：**核心引擎不依赖 stdio，RPC bridge 不依赖具体 UI，传输不依赖业务状态**。第 18 章的动态工具和第 19 章的 TUI 都建立在这条边界上；若今天把这层省掉，下一章每加一个前端都会重新发明协议。

---

## Rust 修炼小结

| 概念 | 本章用法 | 后面在哪用到 |
|---|---|---|
| `tokio::mpsc` | Op 下行、Event 上行、扇出 | 第 19 章 TUI 双循环 |
| `oneshot` | 单次 prompt 终态 | 审批、工具调用 |
| `CancellationToken::child_token` | 请求级取消 | 第 19 章 Ctrl+C |
| `HashMap<String, Handle>` | thread 注册表 | 第 18 章 MCP server registry |
| `serde(tag = "method")` | JSON-RPC 消息判别 | 事件序列化、回放 |

## 章末验收

- [ ] `mcx serve --stdio` 能被上面五行的 Python 脚本驱动，并完成一轮任务
- [ ] 中途发送 `request/cancel`，当前 turn 停止，但同一 `thread_id` 仍能继续
- [ ] 发送 `server/shutdown` 后，客户端收到最后事件且进程退出码为 0
- [ ] `cargo test -p mcx-server` 全部通过，不访问网络或 API key
- [ ] 直接关闭 stdin（EOF），正在运行的 turn 仍能发完 `TurnComplete`

## 读者挑战

1. 现在同一个 `thread_id` 只允许一个活跃 prompt。**设计一个排队/抢占策略，使“紧急输入”能取消旧请求并立即开始新轮，同时保留旧轮已产生的 Item。** 写测试证明不会出现两个 turn 并发修改 `history`。
2. WebSocket 断线后重连：若客户端错过事件，`thread/get(after_turn)` 返回的增量如何与实时事件合并才不会重复渲染？请给“事件序号”设计一份不依赖系统时钟的方案。
3. `Event` 是 `Clone + PartialEq`，但带大块 tool result 时克隆很贵。**引入 `Arc<Event>` 或只克隆订阅所需字段，分别测量吞吐与代码复杂度，并说明你选哪个。**

## 下一章预告：你写死的工具表，不是能力的边界

mini-codex 现在能被任何程序驱动，但它的工具表仍是编译期写死的。下一章接入 Model Context Protocol：一个第三方进程可以突然带来“查数据库”“翻 Jira”“操作浏览器”几十种工具，而你不必改一行 `Session` 代码。第 6 章选择 `Box<dyn Tool>` 而非枚举的理由，终于要兑现——同时也把“外部进程提供的工具凭什么能执行”这个问题摆上台面。

---


# 第 18 章　MCP：让工具在运行时长出来

**本章任务**：实现 MCP 客户端，让 mini-codex 在运行时发现、调用、更新和撤销远程工具；并把所有外部能力统一成第 6 章的 `Box<dyn Tool>`。核心命题不是“支持更多工具”，而是“**把扩展性从代码移到协议**”。

---

## 18.1 一个会过时的工具枚举

第 6 章之后，内置工具大概长这样：

```rust
// 反例：编译期封闭的工具世界
pub enum ToolName {
    ReadFile,
    EditFile,
    RunCommand,
    QueryPostgres,
    JiraTransition,
}
```

每次接入新系统都要改枚举、加 match、重新发布。更糟的是，团队 A 想让 agent 查内部 Grafana，团队 B 想让它操作 Stripe，而你并不想让 mini-codex 的依赖树变成“全世界 API 的总和”。

MCP 的解决方案是：工具定义、调用、结果都通过 JSON-RPC 消息交换；client 只负责连接，server 才拥有能力。mini-codex 因此不需要知道“查 Jira”怎么实现，只需要知道它叫什么、参数 schema 是什么、该不该让用户审批。

```text
Session ──► ToolRegistry ──► LocalTool(Box<dyn Tool>)
                        └──► McpTool(Box<dyn Tool>) ──► JSON-RPC ──► 子进程/HTTP/SSE
```

> **工具是协议对象，不是枚举变体。** 这解释了第 6 章的伏笔：`Registry` 用 `Box<dyn Tool>` 持有能力，让运行时注册任意来源；若用枚举，本章每接入一种传输都要修改核心 crate。

---

## 18.2 三种传输，同一个协议

MCP 的基线是 JSON-RPC 2.0：请求有 id、结果回相同 id、通知无 id[citation:16][citation:17]。传输只决定“消息如何从 A 到 B”，不影响工具语义。

| 传输 | 建立方式 | 适合 | 关键风险 |
|---|---|---|---|
| stdio | 启动本地子进程，stdin/stdout 传 JSON-RPC | 本地、单用户、低延迟 | 子进程崩溃、stderr 污染 stdout |
| HTTP（Streamable HTTP） | POST 请求 + 可流式响应 | 远程服务、网关、认证 | 会话恢复、超时、授权 |
| SSE | POST 请求 + 服务端事件流返回 | 长任务、旧版远程 server | 双向流限制、代理缓冲 |

本章统一接口为 `Transport`：既能发请求，也能订阅服务端通知（工具列表变更、日志、进度）。

```rust
// crates/mcx-mcp/src/transport.rs
#[async_trait]
pub trait Transport: Send + Sync {
    async fn open(&mut self) -> Result<(), McpError>;
    async fn request(&self, method: &str, params: Value) -> Result<Value, McpError>;
    async fn notify(&self, method: &str, params: Value) -> Result<(), McpError>;
    fn events(&self) -> mpsc::Receiver<ServerNotification>;
    async fn close(&self) -> Result<(), McpError>;
}
```

stdio 的 `open` 是 `Command::spawn`；HTTP 的 `open` 是握手并创建按 session 复用的 client。无论哪种，错误帧都转成统一 `McpError`，`Session` 从不需要 `match` 传输类型。

```rust
pub enum McpError {
    Transport(String),
    Protocol { code: i64, message: String, data: Option<Value> },
    InvalidParams(String),
    ServerDied,
    Timeout,
}
```

---

## 18.3 生命周期：initialize、能力协商、关闭

MCP 不像普通 RPC“连上就调用”。client 先发送 `initialize`，交换协议版本和能力；server 回应后，client 发 `notifications/initialized` 才算握手完成。mini-codex 把这次往返封装为状态机：

```rust
pub struct ClientState {
    server_name: String,
    capabilities: ServerCapabilities,
    transport: Box<dyn Transport>,
}

impl ClientState {
    pub async fn connect(mut transport: Box<dyn Transport>, name: &str)
        -> Result<Self, McpError>
    {
        transport.open().await?;
        let result = transport.request("initialize", serde_json::json!({
            "protocolVersion": "2025-11-25",
            "clientInfo": { "name": "mini-codex", "version": env!("CARGO_PKG_VERSION") },
            "capabilities": { "roots": {}, "sampling": {} }
        })).await?;
        let capabilities = serde_json::from_value(result["capabilities"].clone())?;
        transport.notify("notifications/initialized", Value::Object(Default::default())).await?;
        Ok(Self { server_name: name.into(), capabilities, transport })
    }
}
```

能力协商的意义是**避免“版本越高越能瞎调”**。如果 server 不声明 `tools`，client 就不发 `tools/call`；如果只声明旧式 `sampling`，就不能假定支持 prompts。协商结果应被记录下来——第 16 章的回放需要它还原运行环境。

```rust
pub struct ServerCapabilities {
    pub tools: Option<ToolsCapability>,
    pub resources: Option<ResourcesCapability>,
    pub prompts: Option<PromptsCapability>,
    pub logging: bool,
}

pub struct ToolsCapability {
    pub list_changed: bool, // 是否可能发 notifications/tools/list_changed
}
```

关闭要成对：先 `shutdown` 请求，再关闭传输。不要直接 kill 子进程——那样正在写的 JSONL 会半截结束。

---

## 18.4 工具发现：`tools/list` 与动态注册

握手后调用 `tools/list`，把每个声明转成 `Tool` trait 对象：

```rust
pub struct McpToolRef {
    server: String,
    name: String,
    description: String,
    input_schema: Value,
    annotations: ToolAnnotations,
}
```

```rust
async fn refresh_tools(state: &mut ClientState, registry: &mut ToolRegistry)
    -> Result<(), McpError>
{
    let list = state.transport.request("tools/list", Value::Null).await?;
    let declared: Vec<ToolDeclaration> = serde_json::from_value(list["tools"].clone())?;

    registry.remove_server(&state.server_name);
    for d in declared {
        let tool = Box::new(McpToolRef {
            server: state.server_name.clone(), name: d.name,
            description: d.description, input_schema: d.input_schema,
            annotations: d.annotations,
        });
        registry.register(tool)?;
    }
    Ok(())
}
```

`Tool` 的接口沿用第 6 章：

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    async fn invoke(&self, args: Value, ctx: &ToolContext) -> Result<ToolOutput, ToolError>;
}
```

`McpToolRef::invoke` 只是把参数再编码成 JSON-RPC，发给对应 server：

```rust
#[async_trait]
impl Tool for McpToolRef {
    async fn invoke(&self, args: Value, ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let progress = ctx.progress.clone();
        let params = serde_json::json!({
            "name": self.qualified_name(),
            "arguments": normalize_args(args, &self.input_schema),
            "_meta": { "progressToken": progress.token() },
        });
        let result = ctx.transport.request("tools/call", params).await?;
        Ok(ToolOutput::from_mcp(result))
    }
}
```

限定名 `server__tool` 是关键。模型看到的工具名可能冲突（“list_issues”），但 `(server, tool)` 二元组不会。`Registry::resolve` 必须返回来源信息，审批、审计、回放都依赖它。

> **`Box<dyn Tool>` 的回报就在这里**：本地 `ReadFile`、MCP `github__list_issues`、未来远程沙箱工具都实现同一 trait。`Session` 的 tool loop 只认识 trait，不认识 MCP——它甚至不知道对方是子进程。

---

## 18.5 资源、资源模板与按需上下文

MCP 不止工具。`Resources` 是只读上下文：文件、数据库记录、API 响应；`ResourcesTemplates` 则把“参数化资源”声明成 URI 模板，例如 `db://users/{id}/orders`。

```rust
pub struct Resource {
    pub uri: String,
    pub name: String,
    pub mime_type: Option<String>,
    pub description: Option<String>,
}

pub struct ResourceTemplate {
    pub uri_template: String,
    pub name: String,
    pub description: Option<String>,
}
```

资源不是“自动塞进 prompt 的魔法”。正确流程是：**发现元数据 → 按当前任务订阅 → 模型决定读取 → 内容进入上下文预算**。这直接呼应第 15 章：MCP 每 server 增加 500–3000 token 的工具定义，资源内容更可能以万 token 计[citation:15]。

```rust
async fn load_resource(state: &ClientState, uri: &str) -> Result<String, McpError> {
    let result = state.transport.request("resources/read", serde_json::json!({ "uri": uri })).await?;
    let contents = result["contents"].as_array().ok_or(McpError::Protocol {
        code: -32603, message: "missing contents".into(), data: None,
    })?;
    Ok(contents.iter().filter_map(|c| c["text"].as_str()).collect())
}
```

模板的价值是**避免枚举宇宙**。不要为了“查订单”预先把十万用户展开成资源；只注册模板，需要时再 `resources/read`。第 14 章的 AGENTS.md 也适用同样原则：按当前目录范围加载，而不是把全仓库知识压进 system prompt。

---

## 18.6 工具太多时：索引、检索与惰性完整化

当一个 server 声明 80 个工具，全部塞进 system prompt 会迅速吃掉上下文预算[citation:15]。mini-codex 分三步处理：

1. **始终携带“索引工具”**：`mcp__search_tools(query)`、`mcp__describe_tool(qualified_name)`。
2. **默认只放摘要**：名称、一句话用途、危险标签；完整 JSON Schema 按需取。
3. **按任务激活**：读取当前 turn 的线索，把命中分数高的 N 个完整化。

```rust
pub struct ToolIndex {
    entries: Vec<ToolSummary>,
    embedding: Option<SimpleEmbedder>,
}

pub struct ToolSummary {
    pub qualified_name: String,
    pub short_description: String,
    pub keywords: Vec<String>,
    pub annotations: ToolAnnotations,
    pub full_schema: Value, // 惰性加载时可暂缺
}
```

```rust
pub async fn select_for_prompt(&self, task: &str, budget: &mut ContextBudget)
    -> Vec<Value>
{
    let mut ranked = self.entries.clone();
    ranked.sort_by_key(|e| relevance(task, e));
    let mut chosen = Vec::new();
    for entry in ranked {
        let schema = match entry.full_schema {
            Some(s) => s,
            None => self.registry.describe(&entry.qualified_name).await,
        };
        let cost = budget.estimate_tool(&schema);
        if budget.available_for_tools() < cost { break; }
        chosen.push(schema);
        budget.record_tool(cost);
    }
    chosen
}
```

`search_tools` 本身是个普通 `Tool`：

```rust
async fn invoke(&self, args: Value, _ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
    let q = args["query"].as_str().ok_or(ToolError::InvalidArgs)?;
    let hits = self.index.search(q, 20);
    Ok(ToolOutput::json(serde_json::json!({ "tools": hits })))
}
```

这里必须避免“检索器成为新瓶颈”。索引只在 `tools/list` 或 `list_changed` 时重建；全文匹配对教学实现足够，生产可换 embedding。但**不能让模型为了找工具而连续调用搜索二十次**——那等于把上下文问题换成工具预算问题。

---

## 18.7 信任与审批：外部 server 提供的工具凭什么执行

MCP server 是外部进程，甚至可能来自 `npx` 随机包。它能提供工具，不代表它有权读写你的仓库或调用生产 API。**“工具存在”和“工具被允许”是两件事。**

mini-codex 的信任模型有四层：

| 层 | 决策 | 例子 |
|---|---|---|
| 来源 | 允许连接这个 server 吗？ | 内置白名单、用户确认、签名 |
| 身份 | 它是谁？ | server name、版本、命令哈希 |
| 工具 | 单次调用允许吗？ | read-only 自动，destructive 审批 |
| 参数 | 参数在策略内吗？ | 路径不出 workspace、禁止 `rm -rf` |

第 6 章的 `ToolAnnotations` 终于派上用场：

```rust
pub struct ToolAnnotations {
    pub read_only: bool,
    pub destructive: bool,
    pub idempotent: bool,
    pub open_world: bool, // 参数是否可能访问任意 URL/主机
}
```

```rust
pub enum Decision {
    Allow,
    Deny(String),
    Ask(ApprovalRequest),
}

pub fn evaluate(server: &ServerTrust, call: &ToolCall) -> Decision {
    if !server.allowed { return Decision::Deny("server not trusted".into()); }
    if call.annotations.destructive || call.annotations.open_world {
        return Decision::Ask(ApprovalRequest::from(call));
    }
    if server.mode == Mode::FullAuto && call.annotations.read_only {
        return Decision::Allow;
    }
    Decision::Ask(ApprovalRequest::from(call))
}
```

审批请求是 `Event` 的一种吗？**不是普通事件，而是同步请求。** 第 17 章说过，普通事件用共享有界队列；审批是“引擎停在这等用户”的控制流，必须用独立 `oneshot` 通道，否则会和第 10 章的死锁陷阱重逢。

```rust
// 引擎侧（简化）
let decision = approval_policy.evaluate(&trust, &call);
let approved = match decision {
    Decision::Allow => true,
    Decision::Deny(reason) => return ToolOutput::error(reason),
    Decision::Ask(req) => {
        let (tx, rx) = oneshot::channel();
        self.approval_tx.send(req.with_reply(tx)).await?;
        rx.await.unwrap_or(ApprovalAnswer::Deny)
    }
};
```

审批 UI 展示必须包含**足以判断的信息**：server 名、限定工具名、参数摘要、参数是否包含工作区外路径、预估副作用。只显示 `RunCommand: cargo test` 是不够的——攻击者会命名为 `run_safe_tests`。

> **对 MCP 来说，默认拒绝不是保守，是正确。** 一个未审查的 server 即使声称 `read_only`，也可能触发网络请求、读取 `~/.ssh` 或消耗大量 API 配额。工具注解是 server 的自述，不是证明。

### 沙箱隔离：本地 server 也不可全信

stdio server 默认继承父进程环境。mini-codex 应：只传白名单环境变量；以受限用户/工作区目录启动；对命令类工具复用第 11 章的沙箱；网络类工具按 profile 配置出口规则。HTTP MCP 还需要鉴权与 TLS、按用户 token 而不是 server 全局 secret 调用——一个内部 server 有权限，不等于当前用户有权限。

---

## 18.8 健壮性：重连、列表变更与 server 崩溃隔离

server 崩溃不能拖垮主会话。每个 MCP 连接有独立任务；失败只是“移除这批工具”，不会让 `Session` panic。

```rust
async fn supervise(name: String, mut state: ClientState, registry: Arc<Mutex<ToolRegistry>>) {
    let mut events = state.transport.events();
    loop {
        tokio::select! {
            // 服务端通知：工具表变了，重新拉取
            Some(notif) = events.recv() => {
                if notif.method == "notifications/tools/list_changed" {
                    let _ = refresh_tools(&mut state, &mut registry.lock().await).await;
                }
            }
            // 传输层报告致命错误
            Err(e) = state.transport.liveness() => {
                registry.lock().await.remove_server(&name);
                tracing::warn!(server = %name, err = %e, "mcp server down; tools detached");
                break;
            }
        }
    }
}
```

重连策略要指数退避，但**不能重放未确认的工具副作用**。只读操作可重试；转账、删除、发布必须返回原错误，让模型或用户决定。超时用 `tokio::time::timeout` 包裹 `tools/call`；超过阈值先取消，再按 schema 判断幂等性。

```rust
let result = tokio::time::timeout(call_timeout, transport.request("tools/call", params)).await
    .map_err(|_| McpError::Timeout)?;
```

---

## 18.9 测试：假 transport + ScriptedLlm + 工具计数

把 `Transport` 做成 trait 的好处是测试可注入。假 server 按预设脚本回 JSON-RPC：

```rust
struct ScriptedTransport {
    script: Mutex<VecDeque<Value>>,
    events: mpsc::Sender<ServerNotification>,
}

#[async_trait]
impl Transport for ScriptedTransport {
    async fn request(&self, _method: &str, _params: Value) -> Result<Value, McpError> {
        self.script.lock().unwrap().pop_front().ok_or(McpError::ServerDied)
    }
    ...
}
```

```rust
#[tokio::test]
async fn dynamic_tool_is_visible_and_callable_without_recompile() {
    let transport = ScriptedTransport::with_script(vec![
        // initialize 结果
        serde_json::json!({"capabilities": {"tools": {"listChanged": true}}}),
        // tools/list 结果：一个“未知”工具
        serde_json::json!({"tools": [{
            "name": "query_invoices",
            "description": "query invoices by status",
            "inputSchema": {"type":"object","properties":{"status":{"type":"string"}}},
            "annotations": {"readOnly": true}
        }]}),
        // tools/call 结果
        serde_json::json!({"content": [{"type":"text","text":"INV-1"}]}),
    ]);

    let mut registry = ToolRegistry::default();
    let mut state = ClientState::connect(Box::new(transport), "billing").await.unwrap();
    refresh_tools(&mut state, &mut registry).await.unwrap();

    assert!(registry.resolve("billing__query_invoices").is_some());
    let out = registry.invoke("billing__query_invoices", json!({"status":"paid"})).await.unwrap();
    assert!(out.text.contains("INV-1"));
}
```

第二个测试模拟崩溃隔离：

```rust
#[tokio::test]
async fn server_failure_detaches_tools_but_keeps_session_alive() {
    let (tx, rx) = mpsc::channel(16);
    let mut registry = ToolRegistry::default();
    registry.register(local_echo_tool());
    registry.attach_mcp("flaky", rx);

    // 模拟监督任务发现 server 死亡
    registry.handle_liveness("flaky", McpError::ServerDied).await;
    assert!(registry.resolve("flaky__x").is_none());
    assert!(registry.invoke("local_echo", json!("ok")).await.is_ok());
}
```

这两个测试不依赖真实 server，却覆盖最关键承诺：**新增能力无需重编译；外部故障不会让核心崩溃。**

---

## 避坑专栏 #19：把 MCP 结果直接拼进 prompt，会把一半工具变成“盲调用”

常见错误是 `ToolOutput` 无上限：

```rust
// 危险：20MB 的 resources/read 直接 push 进 history
history.push(Message { role: Assistant, content: result.output });
```

后果是下一轮触发第 15 章的压缩，甚至超过窗口。症状是“明明读了文件，agent 却像没看到”。解法是在工具边界做结构化裁剪：

```rust
pub struct ToolOutput {
    pub text: String,
    pub truncated: bool,
    pub citations: Vec<Citation>,
}
```

策略是：**保留前 N 行与最后 M 行，保存完整内容到索引，返回可点击 citation**。若工具结果包含二进制、超长堆栈或未知字段，标记 `derived=false` 并保留原始 JSONL；摘要只能用于辅助展示，不能冒充工具真相。

**通用形式**：所有外部数据进入上下文前，必须经过“大小上限 + 结构摘要 + 来源引用”三关。否则工具越强，记忆系统越早失控。

---

## 18.10 Design Rationale

**Q：为什么第 6 章选 `Box<dyn Tool>` 而不是 `enum ToolKind`？**

因为枚举是封闭集合，每加一种来源都要修改 `Session` 依赖的所有 match；trait object 是开放集合，只要实现接口就能在运行时注册。MCP 恰恰在编译期未知：工具名、schema、server 都是配置决定的。动态分发的一次堆分配，换来的是“不重新编译就能接入世界”。

**Q：为什么消费方是协议层，不用第 4 章的 channel 推增量？**

第 4 章的 `AgentMessageDelta` 面向人眼，用有界 mpsc 获得背压。MCP 的工具结果是请求/响应，且需要按 id 精确关联；它天然是 stream/await 结构。两者不是矛盾，而是同一原则的两个应用：**谁消费、谁负责背压**。第 4 章伏笔在此兑现。

**Q：为什么不把所有 MCP 工具默认自动批准？**

因为“能描述”不等于“可执行”。read-only 注解来自不可信 server；HTTP server 还涉及用户身份和出口网络。安全边界必须建立在 mini-codex 的策略上，而不是 server 的自报家门上。

**Q：为什么支持三种传输而不是只做 stdio？**

stdio 适合本地、最低运维成本；HTTP/SSE 适合团队共享和远程升级。限定一种会逼用户把安全模型削足适履。统一 `Transport` trait 后，增加 WebSocket 只加一个实现文件，核心 tool loop 零修改。

---

## AI 软件工程原理 #18

> **扩展性要落在协议上，不要落在代码里。**

`enum` 把能力编译进二进制；`trait + 协议` 把能力交给运行时的消息交换。前者是 fork、适配器和版本地狱，后者是“只要遵守 JSON-RPC 与 schema，任何人都能接入”。MCP 不是第一批插件协议，却第一次把工具、资源、提示、生命周期和通知放进同一套可协商消息。

对 mini-codex 而言，这意味着第 6 章的一个 `Box<dyn Tool>` 可以代表本地函数、子进程、远程服务甚至第 17 章的 SDK 调用。代价是必须补上信任、版本协商、超时和列表变更——协议化不消灭复杂度，只是把它从“改 core”迁移到“验证边界”。

---

## Rust 修炼小结

| 概念 | 本章用法 | 后面在哪用到 |
|---|---|---|
| `async_trait` | `Tool`、`Transport` | 新的能力来源 |
| `Box<dyn Trait>` | 运行时异构工具 | 插件、远程工具 |
| `oneshot` | 同步审批答复 | 第 19 章快捷键确认 |
| `Arc<Mutex<Registry>>` | 多 server 共享注册表 | 热加载 |
| `tokio::select!` | 事件 vs 存活信号 | 取消、超时 |

## 章末验收

- [ ] 配置一个第三方 MCP server，agent 能发现并调用新工具，不改 `mcx-core` 代码
- [ ] server 进程被 kill 后，主会话仍可继续，且其工具从注册表移除
- [ ] 工具数超过预算时，默认只暴露 `search_tools`，按需完整化
- [ ] 对 destructive 工具弹出审批；read-only 可按策略自动放行
- [ ] `cargo test -p mcx-mcp` 使用假 transport，不联网

## 读者挑战

1. 设计一个 `tools/list_changed` 的**原子切换**方案：在刷新期间，正在执行的旧工具调用必须要么继续看到旧 schema，要么明确失败；不能半新旧混合。提示：考虑 Arc 快照。
2. 参数策略如何表示“允许访问 `./target`，但禁止 `../.env`”？请把它写成可组合的谓词，并写测试覆盖符号链接绕过。
3. 当同一工具在 server A、B 中都叫 `search`，但语义不同，**模型选错的概率如何测量**？设计一个给 `qualified_name` 与摘要的实验，并说明何时必须要求用户消歧。

## 下一章预告：没有事件流，就没有能打断的界面

MCP 让能力无限增长，但人类仍然需要一个能看、能打断、能批准的界面。下一章做 ratatui TUI：聊天流、diff、审批面板和状态栏。届时你会看到，第 3 章那个“四个问题”的表不是危言耸听——没有 `Op`/`Event` 和三层循环，流式吐字、键盘响应、取消与审批会在同一个 `loop` 里互相阻塞。

---


# 第 19 章　TUI：把引擎接到人脸上

**本章任务**：用 ratatui 实现一个终端界面，让模型流式输出时 UI 仍能响应键盘，并把 diff、审批和上下文余量实时呈现。本章是全书最明确的一次“兑现承诺”：如果第 3 章用了那个直接 `loop`，现在必须推倒重来。

---

## 19.1 回到第 3 章那张“四个问题”的表

第 3 章展示过这个反例：

```rust
// 反例：直接 loop，正是第 3 章警告的写法
async fn chat_forever() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        if line.trim() == "/quit" { break; }

        // ← 模型思考的几十秒里，整个终端冻结
        let reply = call_llm(&line).await?;
        println!("{reply}");
    }
    Ok(())
}
```

当时说它会“在第 19 章爆炸”。现在具体看怎么爆。

| 第 3 章问题 | TUI 中的真实症状 |
|---|---|
| UI 冻结 | 模型吐字时 `crossterm` 读不到键，Ctrl+C 要等下一轮 |
| 无法中途打断 | 正在跑 `cargo test` 的 agent，只能关闭终端 |
| 无法被复用 | 渲染逻辑与 stdin 耦合，CI/IDE 各抄一份 |
| 无法测试 | UI 断言依赖真实终端，回归只能靠截图 |

而且 TUI 比 CLI 更难：屏幕必须保留聊天历史、显示正在生成的文本、高亮 diff、弹出模态审批，同时每一毫秒都在等待用户输入。**如果所有状态都在一个 `async fn` 的栈帧里，任何 `await` 都会让键盘任务饿死。**

mini-codex 不用重写，因为它早就是两张队列：

```text
键盘/鼠标 ──Op──▶ Session ──Event──▶ 渲染器
                        ▲                  │
                        └──────── 审批答复 ─┘
```

本章只新增两样东西：**事件循环**与**渲染循环**。引擎、工具、MCP、记忆系统一行不改。

---

## 19.2 双层架构：渲染循环与事件循环分离

第一版 TUI 容易写成“一个 big loop，里面又读键又画屏”。正确拆法是两任务并发：

```rust
// crates/mcx-tui/src/main.rs
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (op_tx, op_rx) = mpsc::channel(16);
    let (event_tx, event_rx) = mpsc::channel(128);

    // 任务 A：引擎（与 CLI、server 完全相同）
    let session = Session::new(build_client()?, op_rx, event_tx.clone());
    tokio::spawn(async move { session.submission_loop().await });

    // 任务 B：事件循环——把 Event 送进 UI 状态
    let (action_tx, action_rx) = mpsc::channel(64);
    let state = Arc::new(Mutex::new(AppState::default()));
    spawn_event_loop(event_rx, state.clone(), action_tx);

    // 任务 C：键盘输入——把按键翻译成 Op/审批答复
    spawn_input_loop(op_tx.clone());

    // 主线程：渲染循环；只在 Terminal 上阻塞
    run_render_loop(state, action_rx)?;
    Ok(())
}
```

三个任务的角色不可混淆：

| 任务 | 拥有 | 可以 await | 不可做 |
|---|---|---|---|
| 引擎 | `Session` | 模型、工具、MCP | 直接访问终端 |
| 事件循环 | `AppState` | channel、审批 oneshot | 每事件全屏重绘 |
| 渲染循环 | `Terminal` | 仅极短绘制 | 网络、文件、模型调用 |

> **模型慢慢吐字时，键盘仍能响应；审批弹出时，流还能继续更新背景。** 这两个“仍能”正是双层架构存在的理由。

---

## 19.3 `AppState`：UI 唯一可信状态

渲染器只从 `AppState` 读，事件循环只通过受控方法修改它。不要在事件处理里直接 `println!` 或 `terminal.draw`——那会让并发与测试失去锚点。

```rust
pub struct AppState {
    pub threads: BTreeMap<String, ThreadView>,
    pub current: String,
    pub status: StatusBar,
    pub modal: Option<Modal>, // 审批、命令面板、错误
    pub last_render: Instant,
}

pub struct ThreadView {
    pub turn: usize,
    pub items: Vec<Item>,
    pub draft: String,        // 正在生成的 AgentMessage
    pub scroll: usize,
}

pub struct StatusBar {
    pub context_used: usize,
    pub context_budget: usize,
    pub sandbox: &'static str,
    pub approval_mode: &'static str,
    pub mcp_servers: Vec<String>,
}
```

事件归约函数必须纯：`apply_event(state, event)`。这样同一份 JSONL 能重放成完全相同的界面状态，第 16 章的回放能力也就直接变成“离线复现 bug”。

```rust
fn apply_event(state: &mut AppState, ev: Event) {
    let view = state.threads.get_mut(&state.current).unwrap();
    match ev {
        Event::TurnBegin { turn } => {
            view.turn = turn;
            view.draft.clear();
        }
        Event::AgentMessageDelta(delta) => view.draft.push_str(&delta),
        Event::TurnComplete { turn, text } => {
            view.items.push(Item::AgentMessage { content: text });
            view.draft.clear();
        }
        Event::Error(e) => view.items.push(Item::Reasoning { summary: format!("error: {e}") }),
        Event::Shutdown => state.modal = Some(Modal::Exiting),
    }
}
```

注意 `TurnBegin`/`TurnComplete` 是事件流里的**语义边界**。第 15 章说过：压缩只能在完整 turn 边界发生；TUI 同理——不要在 `AgentMessageDelta` 中间把“当前消息”切成两条，否则滚动、复制和 diff 定位都会错。

---

## 19.4 事件循环：扇入、节流与审批桥

事件循环不是“收到就立即重绘”。它有三个职责：更新状态、触发必要的副作用、通知渲染器。

```rust
async fn spawn_event_loop(
    mut event_rx: mpsc::Receiver<Event>,
    state: Arc<Mutex<AppState>>,
    action_tx: mpsc::Sender<UiAction>,
) {
    while let Some(ev) = event_rx.recv().await {
        {
            let mut s = state.lock().await;
            apply_event(&mut s, ev.clone());
            update_status(&mut s);
        }
        // 每事件都请求重绘；真正限流在渲染端
        let _ = action_tx.send(UiAction::Redraw).await;
    }
}
```

为什么限流不在事件循环？因为**事件频率不是渲染瓶颈，帧时间才是**。事件循环若每毫秒只放行一帧，会丢失“最后一次 delta”；渲染循环按时间窗口合并，则既能保持交互流畅，又能保证最终帧完整。

```rust
fn run_render_loop(state: Arc<Mutex<AppState>>, mut action_rx: mpsc::Receiver<UiAction>)
    -> Result<(), Box<dyn std::error::Error>>
{
    let mut terminal = init_terminal()?;
    let tick = Duration::from_millis(16); // ~60fps 上限
    let mut last = Instant::now();

    loop {
        let guard = state.lock().unwrap();
        let needs = should_draw(&guard, last);
        drop(guard);

        // 有动作或达到节流周期才画
        if needs || last.elapsed() >= tick {
            let guard = state.lock().unwrap();
            terminal.draw(|f| render(f, &guard))?;
            last = Instant::now();
        }

        match action_rx.try_recv() {
            Ok(UiAction::Quit) => break,
            _ => {}
        }
        std::thread::sleep(Duration::from_millis(2));
    }
    restore_terminal(terminal)?;
    Ok(())
}
```

更鲁棒的做法是用 `tokio::time::interval` + `Notify`，把“事件到达”和“帧时钟” `select` 在一起。关键不变式是：**任何单次 token 绝不会导致完整重排所有历史。** 下面 19.6 节专门解决这个问题。

---

## 19.5 组件：聊天流、diff、审批、状态栏

### 聊天流：只追加，不重排

```rust
fn render_chat(f: &mut Frame, area: Rect, view: &ThreadView) {
    let lines: Vec<Line> = view.items.iter().flat_map(item_to_lines).collect();
    let mut text = Text::from(lines);
    if !view.draft.is_empty() {
        text.push_line(Line::from(view.draft.clone()).yellow());
    }
    let paragraph = Paragraph::new(text)
        .scroll((view.scroll as u16, 0));
    f.render_widget(paragraph, area);
}
```

滚动位置必须属于状态，不能在 `draw` 里临时算：否则窗口 resize、diff 展开、审批弹窗都会把视图“弹回底部”。只在用户向上滚动时暂停自动跟随底部。

### Diff 渲染：把 `Item` 变成彩色行

第 16 章的 `Item` 已经区分 `ToolCall`、`ToolResult`，因此 TUI 不必重新解析字符串。命令类工具的 result 若含 unified diff，就渲染成增删行；其余保持原文。

```rust
fn render_diff(f: &mut Frame, area: Rect, result: &str) {
    let mut lines = Vec::new();
    for raw in result.lines() {
        let line = match raw.as_bytes().first() {
            Some(b'+') => Line::from(raw).green(),
            Some(b'-') => Line::from(raw).red(),
            Some(b'@') => Line::from(raw).cyan(),
            _ => Line::from(raw).gray(),
        };
        lines.push(line);
    }
    f.render_widget(Paragraph::new(Text::from(lines)), area);
}
```

这里要避免把大段 result 每行都分配 `Line`。教学实现可接受；生产可只高亮首个屏幕、按需展开。无论优化如何，**不能把完整输出再塞回 `AgentMessage`**——事件类型是第 5 章留下的精确审计粒度。

### 审批面板：同步控制流，独立 channel

审批是“引擎等用户”的交互。它由 `Event::ApprovalRequest` 进入事件循环，事件循环通过**独立审批通道**发 oneshot，而不是普通 `Event` 队列：

```rust
pub enum UiAction {
    Redraw,
    Approval { request: ApprovalRequest, reply: oneshot::Sender<ApprovalAnswer> },
    Quit,
}
```

```rust
async fn handle_approval(reply: oneshot::Sender<ApprovalAnswer>, answer: ApprovalAnswer) {
    // 超时也不重发；引擎侧已有默认值
    let _ = reply.send(answer);
}
```

键盘任务读取 `j/k/Enter/Esc` 后调用 `handle_approval`。如果用普通有界 `Event` 队列，审批请求可能在满队列后阻塞引擎，形成第 3 章避坑专栏警告的经典死锁。**通道语义必须匹配等待语义**：流式事件异步，审批同步。

### 状态栏：让不可见风险可见

```rust
fn render_status(f: &mut Frame, bar: &StatusBar) {
    let ratio = bar.context_used as f64 / bar.context_budget.max(1) as f64;
    let warn = if ratio > 0.85 { "!! CONTEXT" } else { "" };
    let line = Line::from(vec![
        format!(" ctx {}/{} {} ", bar.context_used, bar.context_budget, warn).into(),
        format!("sandbox={} ", bar.sandbox).blue().into(),
        format!("approval={} ", bar.approval_mode).yellow().into(),
        format!("mcp={}", bar.mcp_servers.join(",")).gray().into(),
    ]);
    f.render_widget(LineGauge::default().line(line), Rect::default());
}
```

状态栏不是装饰。第 15 章的预算、第 11 章的沙箱、第 13 章的 profile 都汇聚于此。**当用户看到 `ctx 182000/200000` 时，他才能在压缩前主动开新 thread**；当 `approval=never` 却连接了未知 MCP server 时，状态栏应变成红色警告。

---

## 19.6 流式性能：每个 token 都重绘全屏会炸

一个朴素实现在 `AgentMessageDelta` 时调用 `terminal.draw(render_entire_app)`。100 token/s × 全量 diff × 长历史，会迅速把终端渲染变成 CPU 热点，并在 ssh 下产生可观网络流量。

正确策略是**脏区域 + 增量文本 + 帧节流**：

1. 聊天区维护“已绘制的 draft 长度”。新 delta 只追加，不重建整段。
2. 只有滚动、diff 折叠、审批显隐才标记为“需要全量布局”。
3. 渲染循环合并短时间内的多个 `Redraw`，以帧周期为上限。

```rust
pub struct ChatBuffer {
    stable: Vec<Item>,
    draft: String,
    rendered_draft_len: usize,
    dirty: DirtyFlags,
}

impl ChatBuffer {
    fn append_delta(&mut self, delta: &str) {
        self.draft.push_str(delta);
        // 只是文字增长，不需要重排历史
        self.dirty.insert(DirtyFlags::DRAFT_APPEND);
    }

    fn flush(&mut self, f: &mut Frame, area: Rect) {
        if self.dirty.contains(DirtyFlags::LAYOUT_CHANGED) {
            render_full(f, area, &self.stable, &self.draft);
        } else if self.dirty.contains(DirtyFlags::DRAFT_APPEND) {
            render_append(f, area, &self.draft[self.rendered_draft_len..]);
        }
        self.rendered_draft_len = self.draft.len();
        self.dirty = DirtyFlags::empty();
    }
}
```

教学版可以把“append-only 文本区”直接交给 ratatui 的 `Paragraph`；只要不每次重建 `Text`，通常已足够。重点是**架构承诺**：`AppState` 记录已渲染范围，渲染器只做最小补丁。若以后换成 GPU 终端或 Web canvas，替换的是 `flush`，不是事件系统。

> **性能边界也是架构边界。** “每 token 全量重绘”看起来只是慢，实际会让 Ctrl+C 的键盘任务被渲染任务饿死——第 3 章的问题 1 在 TUI 中以更隐蔽的方式回归。

---

## 19.7 键盘、取消与优雅退出

键盘任务读 raw mode 输入，然后发 `Op` 或本地 UI 命令：

```rust
async fn spawn_input_loop(op_tx: mpsc::Sender<Op>) {
    loop {
        if let Some(key) = read_key().await {
            match key {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    op_tx.send(Op::Interrupt).await.ok();
                }
                KeyCode::Esc if modal_open() => {
                    send_approval_answer(ApprovalAnswer::Deny).await;
                }
                KeyCode::Enter if modal_open() => {
                    send_approval_answer(ApprovalAnswer::Allow).await;
                }
                _ => handle_navigation(key),
            }
        }
    }
}
```

`Ctrl+C` 不再直接杀进程，而是 `Op::Interrupt`。这正是第 3 章占位 `CancellationToken` 的归宿：模型调用、MCP 请求、命令执行都在同一取消树下协作退出。模态审批打开时，`Esc/Deny` 和 `Enter/Allow` 是**同一控制流的两个终态**，绝不能用普通事件排队，否则快速按键会导致重复提交。

退出顺序沿用第 17 章：

```rust
async fn shutdown(op_tx: &mpsc::Sender<Op>, state: &AppState) {
    op_tx.send(Op::Shutdown).await.ok();
    // 等待引擎发完 Event::Shutdown，再 restore_terminal
    while !matches!(last_event(), Event::Shutdown) { yield_once().await; }
}
```

`restore_terminal` 必须在**所有 channel 关闭后**执行。若提前恢复 canonical mode，最后一段错误日志会把终端弄乱。

---

## 19.8 测试：ScriptedLlm + 状态归约，不需要终端

TUI 最容易被误认为“无法单测”。其实只要把终端抽成 trait，`AppState` 归约就是纯逻辑：

```rust
#[tokio::test]
async fn streaming_does_not_block_approval_and_quit() {
    let (op_tx, op_rx) = mpsc::channel(16);
    let (event_tx, mut event_rx) = mpsc::channel(128);
    let mut session = Session::new(
        ScriptedLlm::new(vec!["long stream".into()]),
        op_rx, event_tx,
    );
    tokio::spawn(async move { session.submission_loop().await });

    let state = Arc::new(Mutex::new(AppState::default()));
    op_tx.send(Op::UserInput { text: "plan".into() }).await.unwrap();

    // 不等模型“完成”，立刻模拟 Ctrl+C
    let before = Instant::now();
    op_tx.send(Op::Interrupt).await.unwrap();
    let answered = oneshot_approval("allow").await;

    // UI 状态应在短时间收敛：draft 停止增长，模态关闭
    while before.elapsed() < Duration::from_secs(2) {
        if let Ok(ev) = event_rx.try_recv() {
            apply_event(&mut state.lock().unwrap(), ev);
        }
    }
    assert!(!state.lock().unwrap().modal.is_some());
    assert!(answered.is_some(), "审批答复通道未被消费 = 死锁前兆");
}
```

再测渲染节流：构造十万次 delta，断言 `Paragraph` 重建次数远低于事件数。可以用计数器 mock `Backend`：

```rust
#[test]
fn append_only_draft_does_not_rebuild_history() {
    let mut buf = ChatBuffer::default();
    for _ in 0..1000 { buf.append_delta("x"); }
    let mut backend = CountingBackend::default();
    buf.flush(&mut backend, area(80, 24));
    assert!(backend.full_layouts < 10, "长流必须走增量路径");
}
```

**这两个测试就是原理 #3 的兑现**：事件流是真相来源，UI 只是它的投影。只要事件顺序正确，任何前端都能重放；只要 UI 不绕过事件直接调模型，就不会破坏引擎。

---

## 19.9 “现在你看到了”：第 3 章承诺的完整回扣

回看第 3 章那张表。若当时把 `call_llm` 直接放进终端输入循环，本章要面对的现实是：

1. 流式输出时，`read_key` 得不到调度，**状态栏的上下文余量永远滞后**；
2. `cargo test` 这类长工具占用栈帧，**审批弹窗无法抢占**；
3. MCP 的 `tools/list_changed` 从子进程异步到来，**只能塞进全局可变变量**；
4. CI 想复用界面？只能再复制一份 while 循环。

而现有代码只是把事件循环换成不同 consumer：

```rust
// CLI：打印事件
// Server：把事件编码成 JSON-RPC 通知
// TUI：把事件归约成 AppState，再由渲染器投影
```

Session 的 `submission_loop` 在三处完全相同。**“现在你看到了——如果第 3 章用了那个 loop，这里会重写”**，并不是修辞：任何“只在 TUI 里加一层”的尝试，最终都会把 channel、取消、审批和测试逐个补回来。区别在于，现在它们是架构，而不是事后补丁。

---

## 避坑专栏 #20：终端 raw mode + tokio 协作，输入偶发消失

典型错误是：一个线程用 `crossterm::event::read()` 阻塞，另一个线程持有 tokio runtime 的锁：

```rust
// 危险：同步阻塞读抢占了异步运行时线程
std::thread::spawn(|| {
    while let Ok(ev) = crossterm::event::read() { ... }
});
```

现象是：UI 前几秒正常，之后按键丢失、流式突然停止，resize 也不触发。原因是同步 I/O 卡住工作线程，而某些 ratatui 后端依赖该线程轮询事件队列。

正确做法是用 `crossterm::event::EventStream` 的异步接口，或把同步读取放到 dedicated OS thread，再通过 `mpsc::channel` 把按键送进 async 世界。**通用形式：终端 I/O 是阻塞资源，必须拥有自己独立的线程或 async 流；永远不要让它和模型/工具任务抢同一调度器。**

---

## 19.10 Design Rationale

**Q：为什么现在才做 TUI，而不是第 3 章顺手做？**

因为先有事件流才有界面。第 3 章只有 `TurnComplete`，第 4 章才有 delta，第 5 章才分 Item，第 10 章才有审批，第 15 章才有预算，第 18 章才有 MCP。若先做 TUI 再补能力，每次都会改渲染函数；先固定 `Event`，TUI 只是投影。

**Q：为什么用两任务而不是 `select` 单循环？**

单循环“读键、收事件、画图”三件事会在任意一个 `await` 处互相阻塞。两任务让“状态更新”与“帧绘制”解耦，再用帧节流协调。这是第 3 章“引擎与界面速度不匹配”原则在终端里的具体形式。

**Q：为什么审批用 oneshot，不用普通 Event 通道？**

因为审批是引擎同步等待的控制应答，事件通道是有界异步流。混用会把“引擎等用户”转成“引擎等队列”，在满队列时死锁。第 17 章已经确立：同步终态用 oneshot，流式通知用 mpsc。

**Q：为什么不每 token 重绘？**

因为每个 token 重绘会把“显示更新”和“业务进度”耦合，既浪费 CPU 又可能饿死键盘。append-only 缓冲 + 脏标记 + 帧时钟让最终状态正确、交互及时、性能可控。

---

## AI 软件工程原理 #19

> **人类在环的位置要精心设计。**

不是每个动作都弹窗，而是在“信息量最大的时刻”询问：审批前给出 server、参数与影响范围；失败升级时给出可复现的最后事件；任务完成或预算告急时给出决策入口。过多询问训练用户盲按 Enter，等于取消人类在环；过少询问则让 `approval=never` 成为默认事故。

TUI 的职责不是“漂亮地打印 agent”，而是把**决策点、风险与可观测状态**摆在人面前：状态栏显示上下文余量，diff 让修改可审查，审批面板让每次外部副作用可拒绝，流式 delta 让人能在 3 行后打断。界面是协议最外层的最后一道控制面。

---

## Rust 修炼小结

| 概念 | 本章用法 | 后面在哪用到 |
|---|---|---|
| `Arc<Mutex<State>>` | 事件/渲染/键盘共享状态 | 多窗口、Web UI |
| `ratatui::Paragraph` | 聊天流、diff | 日志面板 |
| 脏标记/节流 | 流式性能 | 增量序列化 |
| `oneshot` | 审批答复 | MCP 同步调用 |
| raw mode 输入流 | 键盘任务 | 快捷键、命令面板 |

## 章末验收

- [ ] 模型持续流式输出时，键盘输入与 Ctrl+C 立即响应（无 500ms 以上卡顿）
- [ ] diff 以增删高亮渲染；审批面板可用 `Enter`/`Esc` 操作
- [ ] 状态栏显示上下文余量、沙箱档位、审批模式和已连接 MCP server
- [ ] 长流下每秒渲染帧稳定，不在 1 秒内全量重排十万行
- [ ] `cargo test -p mcx-tui` 用假后端和 ScriptedLlm，不打开真实终端

## 读者挑战

1. 实现一个**命令面板**：按 `Ctrl+P` 打开，按名称调用内置命令与 MCP 工具。请设计焦点模型和“输入中 vs 命令选择中”的按键冲突解决，并写测试证明 Esc 能回到原状态。
2. 当用户向上滚动历史时，新 delta 不应抢走视图；但审批、错误和 `TurnComplete` 可能需要。**设计一套“紧急级别”，让高优先级事件可打破用户滚动，同时记录用户是否显式锁定视图。**
3. 把 `AppState` 序列化为 JSONL，实现“崩溃恢复后界面与引擎视图一致”。**哪些字段必须持久化，哪些必须按 Event 重放重建？** 提示：思考 `CancellationToken` 与临时 modal。

## 下一章预告：能力行不行，要能被测量

第 17–19 章完成了能力三角：协议让系统可被驱动，MCP 让能力可生长，TUI 让人类能有效介入。但能力越多，越难回答“这次改 prompt 有没有变好”。下一章引入 `tracing`、结构化事件与评测集，把第 3 章的 `ScriptedLlm` 模式放大成可重复的任务回归：每次重构，都能用 Event 流而不是肉眼判断“agent 是否还合格”。

---

## 引用来源

[1] https://www.jsonrpc.org/specification
> JSON-RPC is a stateless, light-weight remote procedure call (RPC) protocol. […] It uses JSON (RFC 4627) as data format.

[2] https://modelcontextprotocol.io/specification/2025-11-25/basic
> All messages between MCP clients and servers MUST follow the JSON-RPC 2.0 specification.

[3] https://modelcontextprotocol.io/specification/2025-11-25/basic
> Requests MUST include a string or integer ID. Unlike base JSON-RPC, the ID MUST NOT be null.

[4] https://modelcontextprotocol.io/specification/2025-11-25/basic
> Notifications are sent […] as a one-way message. The receiver MUST NOT send a response.

[5] https://modelcontextprotocol.io/specification/2025-11-25/basic
> The protocol defines these types of messages: Requests […] Responses […] Notifications […]

[6] https://modelcontextprotocol.io/specification/2025-11-25/basic
> MCP provides an Authorization framework for use with HTTP.

[7] https://modelcontextprotocol.io/specification/2025-11-25/basic
> Clients and servers MUST support JSON Schema 2020-12 for schemas without an explicit $schema field.

[8] https://modelcontextprotocol.io/specification/2025-11-25/basic
> All implementations MUST support the base protocol and lifecycle management components.

[9] https://modelcontextprotocol.io/specification/2025-11-25/basic
> Other components MAY be implemented based on the specific needs of the application.

[10] https://modelcontextprotocol.io/specification/2025-11-25/basic
> The modular design allows implementations to support exactly the features they need.

[11] https://modelcontextprotocol.io/specification/2025-11-25/basic
> Each client-server pair maintains its own session, enforcing clear boundaries so that protocols, permissions, and policies do not bleed across domains.

[12] https://modelcontextprotocol.io/specification/2025-11-25/basic
> The architecture distinguishes three roles: the host […] contains the AI model, the client […] manages sessions, and the server […] exposes capabilities.

[13] https://modelcontextprotocol.io/specification/2025-11-25/basic
> During the initialization phase, the client and server exchange capability information, allowing each side to understand what features the other supports.

[14] https://modelcontextprotocol.io/specification/2025-11-25/basic
> The server advertises its available tools, resources, and prompt templates, and the client registers these capabilities.

[15] https://modelcontextprotocol.io/specification/2025-11-25/basic
> When the user interacts with the AI model and the model determines that it needs external information […] it generates a tool call request.

[16] https://modelcontextprotocol.io/specification/2025-11-25/basic
> Resource access follows a similar pattern but is typically initiated by the client or host rather than by the model.


---

# 第 20 章　可观测性与回归评测

**本章任务**：给 mini-codex 加上结构化追踪，并用 20 个固定任务、机械验收和事件流回放组成回归评测。核心不是“多打一些日志”，而是让每一次运行都留下可比较、可判定、可回放的真相。

---

## 20.1 没有基线的优化，只是换了一种失败

第 19 章的 TUI 让一次任务变得可观察：用户能看到流式输出、diff、工具调用和审批。但如果改动一次 system prompt、工具描述或压缩策略后，想回答“质量有没有退步”，只靠肉眼看三五个例子是不够的。Agent 的输出有随机性，任务有长尾，一次成功不能证明一类任务稳定；一次失败也可能是模型采样偶然。

先写一个典型的“反评测”做法：

```rust,ignore
// 危险：用一次性人工印象代替判定
fn run_demo() {
    let output = mini_codex("把 error 处理改成 thiserror");
    println!("{output}");
    // 人看着觉得“还行”，就合并 prompt 改动
}
```

它有三个致命问题：

1. **没有固定起点。** 仓库 HEAD、未提交修改、模型版本、依赖版本都不一样，两次运行不可比。
2. **没有验收程序。** “还行”不能变成 CI 的退出码； reviewer 也无法逐任务复核。
3. **没有结构化产物。** 失败时不知道是模型走错、工具失败、审批卡住，还是上下文压缩丢失了关键信息。

评测（evaluation）不是“测试模型是否聪明”，而是**测试 harness 是否稳定地把模型约束在正确路径上**。第 1 章把 `SafetyRule` 从一段话变成类型；第 12 章把它扩展成 `execpolicy`；本章把“规则是否被遵守”扩展成任务级判定。LangChain 团队公开分享的 Terminal Bench 2.0 成绩从 52.8% 提升到 66.5%，靠的不是换模型，而是完成前自检、目录映射、循环检测和推理分档；能测出这 13.7 个百分点的提升，前提正是有可重复的 harness 评测[citation:4]。反过来，没有评测，任何“优化”都只能被称为玄学。

> **观测与评测是两件互补的事。** 日志回答“这次发生了什么”，评测回答“这次算不算成功”；前者用于诊断，后者用于守护变更。

## 20.2 用 `tracing` 把一次任务变成可查询的对象

第 16 章的 JSONL 已经记录了 `Item`：用户消息、助手消息、工具调用、工具结果。这是回放的最小真相，但还不足以分析“为什么慢”“缓存是否生效”“哪一步开始偏离”。因此在 `mcx-core` 之上增加观测层：每个 turn、每次模型调用、每次工具调用都进入一个 span，并把 token、耗时、缓存和结果写入结构化字段。

```toml
# crates/mcx-telemetry/Cargo.toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }
serde = { workspace = true }
mcx-protocol = { path = "../mcx-protocol" }
```

```rust
// crates/mcx-telemetry/src/lib.rs
use std::time::Instant;
use tracing::{span, Level, Span};

pub struct CallMetrics {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached: bool,
    pub started_at: Instant,
}

impl CallMetrics {
    pub fn begin(name: &str) -> (Span, Self) {
        let span = span!(Level::INFO, "agent_call", call = name);
        let _enter = span.enter();
        (
            span,
            Self {
                input_tokens: 0,
                output_tokens: 0,
                cached: false,
                started_at: Instant::now(),
            },
        )
    }

    pub fn finish(self, ok: bool) {
        let elapsed_ms = self.started_at.elapsed().as_millis() as u64;
        tracing::info!(
            elapsed_ms = elapsed_ms,
            input_tokens = self.input_tokens,
            output_tokens = self.output_tokens,
            cache_hit = self.cached,
            ok = ok,
            "call completed"
        );
    }
}
```

`Span` 的嵌套天然对应三层循环：`session` → `turn` → `model_call` / `tool_call`。因此一次运行的 JSON 日志能直接看到：某个工具调用是否发生在模型调用内部，整个 turn 是否因审批而停顿，缓存命中的调用是否显著更短。

关键不是字段数量，而是**字段语义固定**。下面这些字段一旦改名，评测聚合就会断：

| 字段 | 类型 | 含义 |
|---|---:|---|
| `turn` | usize | 第几轮 |
| `tool` | str | 完全限定工具名，例如 `local__read_file` |
| `input_tokens` / `output_tokens` | u64 | 本次模型调用计费/预算口径 |
| `cache_hit` | bool | prompt cache 或工具结果缓存是否命中 |
| `elapsed_ms` | u64 | span 关闭时的耗时 |
| `outcome` | str | `success` / `tool_error` / `policy_denied` / `timeout` |

不要只记“总 token”。总 token 掩盖了系统提示、工具 schema、历史压缩和单次输出各自的贡献；不知道成本构成，就无法判断“换工具描述”是否划算。也不要把缓存命中率做成模糊的仪表盘：把它纳入 PR 评论，低于基线就阻止合并。

### 观测必须落盘，且不能污染 Event

`Event` 是给用户、TUI、SDK 和回放用的稳定协议；日志是给开发者和 CI 用的诊断信息。二者可以来自同一运行，但不能互相侵入。正确做法是：工具执行后把结果写入 `history` 侧的 `Item::ToolResult`——`Event` 上只有 `ToolCallRecord`（记录“调用了哪个工具”），不含结果正文；同时把结构化字段记在当前 span。**结果在 `Item` 侧、不在 `Event` 侧**，这条分工正是第 3 章“事件流只记事实”的延续。

```rust
async fn execute_tool(name: &str, args: serde_json::Value) -> ToolOutcome {
    let _span = tracing::info_span!("tool_call", tool = name).entered();
    match run_with_policy(name, args).await {
        Ok(out) => {
            tracing::info!(bytes = out.len(), ok = true, "tool finished");
            ToolOutcome::Ok(out)
        }
        Err(e) => {
            tracing::warn!(error = %e, ok = false, "tool failed");
            ToolOutcome::Err(e.to_string())
        }
    }
}
```

这里继续遵守第 17 章的规则：引擎仍只通过 `Op`/`Event` 与外界通信；观测层只是 span 的副作用，测试可以不安装 subscriber。

## 20.3 回归评测集：把 ScriptedLlm 放大 20 倍

第 3 章的 `ScriptedLlm` 已经证明了模式：假模型、内存 channel、收集 `Event`、断言。本章只是把它从“测一个循环”升级为“测一类任务”。先定义任务与工作区契约：

```rust
// crates/mcx-eval/src/lib.rs
use std::path::PathBuf;

pub struct Task {
    pub id: &'static str,
    /// 初始工作区 tar/zstd 或 fixture 目录；每次运行前重置
    pub workspace: PathBuf,
    pub instruction: &'static str,
    /// 固定模型行为，使 CI 快且确定；真实模型走另一个 profile
    pub script: Vec<ScriptStep>,
    pub acceptance: Acceptance,
}

pub enum ScriptStep {
    Model(String),
    Tool { name: String, output: String },
}

#[derive(Default)]
pub struct Acceptance {
    /// 必须出现的工具调用有序序列；允许被无关调用间隔
    pub must_contain_calls: Vec<String>,
    /// 绝对不能调用的工具/参数特征
    pub must_not_call: Vec<String>,
    /// 运行结束后，对工作区执行的命令必须成功
    pub workspace_check: Option<String>,
    /// 事件流中不得出现
    pub forbidden_events: Vec<&'static str>,
    /// 预算上限
    pub max_turns: usize,
    pub max_tokens: u64,
    pub max_seconds: u64,
}
```

“真实任务”不是指必须接入真模型。CI 默认用脚本化行为；它测的是 harness 的确定性：在同样输入、同样工具响应、同样策略下，agent 是否仍然遵守协议、调用正确工具、不越权、在预算内结束。真模型评测应放在 nightly 或手动触发档，而不是让每次 `cargo test` 都烧 token。

一个具体 fixture：工作区里有 `src/lib.rs`，缺少 `add(a, b)` 测试；任务要求“为现有公共 API 补一个单元测试”。

```rust
fn task_add_test() -> Task {
    Task {
        id: "T07_add_unit_test",
        workspace: fixtures("t07"),
        instruction: "为 src/lib.rs 的 add 函数补一个单元测试；不要改 API。",
        script: vec![
            ScriptStep::Model("查看 src/lib.rs，发现 add 函数。".into()),
            ScriptStep::Tool {
                name: "local__read_file".into(),
                output: "pub fn add(a: i64, b: i64) -> i64 { a + b }".into(),
            },
            ScriptStep::Model("应在 tests/lib.rs 添加测试。".into()),
            ScriptStep::Tool {
                name: "local__apply_patch".into(),
                output: "created tests/lib.rs".into(),
            },
            ScriptStep::Model("运行 cargo test 验证。".into()),
            ScriptStep::Tool {
                name: "local__run_command".into(),
                output: "running 1 test\n1 passed".into(),
            },
        ],
        acceptance: Acceptance {
            must_contain_calls: vec![
                "local__read_file".into(),
                "local__apply_patch".into(),
                "local__run_command".into(),
            ],
            must_not_call: vec!["local__delete_file".into()],
            workspace_check: Some("cargo test -p fixture-t07".into()),
            max_turns: 8,
            max_tokens: 20_000,
            max_seconds: 30,
            ..Default::default()
        },
    }
}
```

验收条件必须是**机械判定**：命令退出码、文件存在、AST 性质、依赖图、快照 diff、工具调用序列。不要写“解释是否合理”“代码质量是否提高”这类由人打分的软指标。人可以事后抽样审查，但 CI 的门槛必须是布尔值。

> **“固定初始状态 + 固定行为”不是玩具，而是控制变量。** 真模型的方差放到独立评测档；harness 的回归评测要尽可能把方差隔离在 `ScriptedLlm` 之外。

```rust
#[tokio::test]
async fn task_add_unit_test_passes() {
    let harness = EvalHarness::from_task(&task_add_test()).await;
    let report = harness.run().await;
    assert!(report.acceptance.ok, "验收失败：{report:#?}");
}
```

`EvalHarness::run` 的职责很清晰：重置 fixture、构造 `Session<ScriptedLlm>`、把 `Op::UserInput` 送进去、收集直到 `TurnComplete`/`Shutdown`、最后调用 `Acceptance::check`。它复用了第 17 章的 `AppServer` 也可以，但 CI 评测更直接：不暴露 RPC，只驱动一个 thread，减少测试本身出错的面积。

## 20.4 从事件流判定成功：原理 #5 的自测题

第 3 章原理 #3 说事件流是可判定的真相；第 5 章又留下一条自测题：**能不能只靠事件 schema 判断一次运行成功还是失败？** 答案应当是“能，只要 schema 足够完整”。因此 `Event` 之外，内部 `Item` 必须保存工具调用/结果、是否错误、策略决策等细节。第 20 章要求：每个验收项最终都落到事件或运行后状态。

```rust
impl Acceptance {
    pub fn check(&self, events: &[Event], workspace: &Path) -> AcceptanceResult {
        let calls = tool_call_names(events);
        let mut failures = Vec::new();

        if events.iter().filter(is_turn_complete).count() > self.max_turns {
            failures.push("超过最大轮次".into());
        }
        for required in &self.must_contain_calls {
            if !appears_in_order(&calls, required) {
                failures.push(format!("缺少必要调用 {required}"));
            }
        }
        for forbidden in &self.must_not_call {
            if calls.iter().any(|c| c == forbidden) {
                failures.push(format!("禁止调用 {forbidden}"));
            }
        }
        if events.iter().any(|e| matches!(e, Event::Error(_))) {
            failures.push("运行产生 Error 事件".into());
        }
        if let Some(cmd) = &self.workspace_check {
            if !run_check(workspace, cmd).success() {
                failures.push(format!("工作区验收命令失败：{cmd}"));
            }
        }
        AcceptanceResult { ok: failures.is_empty(), failures }
    }
}
```

“有序出现”不等于“严格相邻”。真实 agent 可能先读 `Cargo.toml`、再读源文件，因此 `must_contain_calls` 只要求必要步骤不缺且大体顺序正确；而 `must_not_call` 捕捉明确的安全回退。这样既严格又可容忍合理实现差异。

回放对比则用两条事件流的差：

```rust
pub struct TraceDiff {
    pub only_in_baseline: Vec<String>,
    pub only_in_candidate: Vec<String>,
}

pub fn diff_traces(base: &[Event], cand: &[Event]) -> TraceDiff {
    let a: Vec<_> = base.iter().map(canonical).collect();
    let b: Vec<_> = cand.iter().map(canonical).collect();
    TraceDiff {
        only_in_baseline: set_diff(&a, &b),
        only_in_candidate: set_diff(&b, &a),
    }
}

fn canonical(ev: &Event) -> String {
    match ev {
        // 忽略流式增量；保留“调用了什么、结果是否错误”
        Event::AgentMessageDelta(_) => String::new(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}
```

同一任务两次运行的工具调用序列 diff，比“两段最终文本相似度”更有价值：它能直接显示 candidate 是否漏了 `read_file`、多调了一次 `run_command`、或在压缩后重新读取了错误文件。若使用真模型，事件级快照允许**非确定性输出但确定性关键路径**。

```bash
cargo test -p mcx-eval
# test T01_apply_simple_patch ... ok
# test T07_add_unit_test   ... ok
# ...
# test T20_reproduce_flaky_test ... ok
#
# eval baseline: 20/20 (100%)
```

## 20.5 评测必须进 CI：改一版 harness，分数不能退

20 个任务不是“发布前跑一次”的演示，而是仓库的一等公民。它们放在 `crates/mcx-eval/tasks/`，fixture 放在版本控制的 `fixtures/`，`Cargo.toml` 用 `[[test]]` 或二进制 runner 组织。CI 分两档：

| 档位 | 触发 | 内容 | 时间预算 |
|---|---|---|---:|
| `eval:regress` | 每次 PR | `ScriptedLlm` 20 任务 + 架构/依赖测试 | 分钟级 |
| `eval:model` | nightly、手动 `/eval` | 固定真模型、固定温度、固定 seed（如支持） | 十—数十分钟 |

`.github/workflows/eval.yml` 的核心不是 GitHub Actions 语法，而是规则：

```yaml
- name: regression suite
  run: |
    cargo test -p mcx-eval --release --quiet
    ./target/release/mcx-eval summarize --out eval.json
- name: guard baseline
  run: |
    baseline=$(jq '.score' baseline.json)
    score=$(jq '.score' eval.json)
    python scripts/guard.py "$baseline" "$score"
```

`guard.py` 不要求“分数严格变高”，而是“**不能低于基线，除非显式批准**”。例如 20 任务中 19 通过算 95%；若本应通过的 T12 因工具描述改动失败，即使平均分数只降 5%，CI 也应红。反之，若一项安全风险修复导致某任务不再调用危险工具，基线也要随之更新——但更新必须是有记录的 PR，而不是悄悄改 fixture。

同时保存 `eval.json`：

```json
{
  "commit": "9c1ab2",
  "profile": "scripted",
  "score": 1.0,
  "task_results": [
    {"id": "T07_add_unit_test", "pass": true, "turns": 3, "tokens": 4120}
  ]
}
```

以后每个优化 PR 都能看到“耗时、token、缓存命中率、通过率”四张图。LangChain 的 13.7 个百分点之所以可信，正因为有这样的固定环境与判定；没有这套系统，任何“我感觉更好了”都只是轶事[citation:4]。

## 避坑专栏 #21：把“无错误事件”误当“任务成功”

常见误判是把验收写成：

```rust
// 错误：只检查没有 Error
assert!(!events.iter().any(|e| matches!(e, Event::Error(_))));
```

现象是：测试绿了，但 agent 只读了文件、没改代码；或它调用了 `apply_patch`，但 patch 没有通过编译；或工作区命令根本没跑。没有错误并不等于达成目标——第 1 章的 `SafetyRule` 早就提醒：agent 最危险的是“声称完成”。

正确做法是用任务的**后置不变量**：

```rust
assert!(compiled);
assert!(tests_passed);
assert!(contains_required_symbol(&patched, "fn add"));
assert!(!diff_contains(&patched, "panic"));
```

若条件复杂，把它写成项目内的可执行检查，而不是自然语言提示词。`Event` 只负责提供证据，真正的判定要读工作区状态。

## 20.6 评测集怎么挑：覆盖失败模式，而非追求数量

20 是教学项目的合理规模，生产可按域扩展。挑选原则是“每个任务代表一类失败模式”：

| 类别 | 例子 | 主要判定 |
|---|---|---|
| 小重构 | 提取函数、补测试 | AST/测试命令 |
| 检索 | 跨 crate 定位定义 | 必须先调用检索/读文件 |
| 编辑 | apply_patch、冲突处理 | patch 应用、编译 |
| 工具策略 | 先读后改、避免重复运行 | 调用顺序、次数 |
| 权限 | 只读任务不得写文件 | `execpolicy` 拒绝 |
| 恢复 | 测试失败、命令错误 | 重试不超过预算 |
| 上下文 | 长历史压缩后仍能引用 | 关键 item 仍在 |
| 取消 | Ctrl+C 后状态一致 | 无半截写入 |
| 协议 | 流式、事件顺序 | Event schema |
| 回归 | 曾出错的真实 case | 原失败路径消失 |

不要把 20 个都做成“写 Hello World”。那样只能证明模型会打字，不能证明 harness。每个 fixture 应当小、可重置、无网络依赖；每个任务必须有一条“为什么存在”的注释。删掉无法解释的任务，评测集就不再是垃圾抽屉。

## 20.7 Design Rationale

**Q：为什么不用真模型跑 CI 评测？**

因为真模型慢、贵、有方差，而回归评测的第一目标是守住 harness 的结构性契约。脚本化模型能在秒级发现“工具调用顺序错”“审批 channel 死锁”“压缩丢失必需 item”这类确定 bug；真模型更适合 nightly 的端到端置信度。二者不是替代关系：前者守底线，后者测泛化。

**Q：为什么验收不用 LLM 当裁判？**

模型裁判可用于探索性质量评估，但不应成为默认 CI 红线。它把“agent 是否合格”重新交给了又一个概率判断，而且裁判模型、prompt 与版本都会变化。应先让命令、文件、依赖图、事件序列判定；确有主观需求时，把 LLM judge 作为附加报告，并固定模型与 prompt。

**Q：为什么 diff 忽略 `AgentMessageDelta`？**

因为流式增量是渲染细节，不是任务正确性。不同实现可以一次发一个字符或一次发一行，只要最终 `TurnComplete`、工具结果与工作区状态一致即可。只比较稳定 schema，才不会让无关重构频繁破坏评测。

## AI 软件工程原理 #20

> **没有评测的 harness 优化都是玄学。**

你可以优化提示词、工具 schema、检索、压缩、审批和沙箱；但若每次都靠人看几个例子，就无法区分“真的更好”与“恰好这次采样不错”。评测把 harness 改进变成有证据的实验：基线固定、变量单一、结果可比较。

| | 人工演示 | 回归评测 |
|---|---|---|
| 输入 | 临时命令 | 固定 fixture + 固定指令 |
| 判定 | “还行” | 命令/状态/事件布尔式 |
| 可重复性 | 低 | 脚本化模型可在 CI 复现 |
| 失败信息 | 印象 | trace、调用序列、diff |
| 改进闭环 | 凭感觉 | 分数、耗时、成本同时看 |

原理 #20 是第 1 章“模型是商品，harness 是护城河”的操作化：模型能力会升级，但你必须为每次升级准备同一把尺。第 3 章的 `ScriptedLlm` 是它的种子，第 16 章的 JSONL 是它的证据，本章的 20 任务评测才是常态化的守门人。下一章要加入子代理并行——而每一次并行策略调整，也都要先过这把尺。

## Rust 修炼小结

| 概念 | 本章用法 | 后面在哪用到 |
|---|---|---|
| `tracing::Span` | turn/工具/模型调用的结构化上下文 | CI trace 聚合、第 21 章子代理追踪 |
| `tracing-subscriber` JSON layer | 落盘、按字段查询 | 发布诊断、离线分析 |
| `serde_json::to_string` 规范事件 | 回放 diff | 第 22 章结构测试 |
| fixture + workspace reset | 固定起点 | 模型评测、混沌测试 |
| `assert_eq!` / 布尔判定 | CI 红线 | 架构与依赖守护 |

## 章末验收

- [ ] `tracing` JSON 日志中能按 `turn`、工具名、耗时、token、缓存命中查询
- [ ] 20 个 fixture 任务均有可重置初始状态与机械验收
- [ ] `cargo test -p mcx-eval` 不访问网络或 API key，且记录 baseline
- [ ] CI 在通过率低于 baseline 时失败，并输出事件流 diff
- [ ] 任一任务可用 `Event` 序列与工作区状态自动判定，无需人工看输出

## 读者挑战

1. 给评测增加“**预算敏感性**”任务：同一个修复任务分别限制 4 轮与 12 轮，断言 agent 在紧预算下优先读取最关键文件。你会如何定义“关键”？
2. 真模型评测中，模型随机性让两次运行事件流不可能完全一致。**设计一套语义等价规则**，允许工具参数顺序、流式粒度不同，却仍能检测出“删除了必要步骤”。
3. 一个任务在某次重构后开始调用多余工具，通过率仍是 100%。**给 CI 增加什么指标才能阻止这种退化？** 提示：不只看正确性。

## 下一章预告：并行不等于更聪明

评测稳定后，就能安全地引入并行。下一章让主 agent 派生受限的子代理去探索、检索和验证，再把摘要与证据回传。目标是隔离上下文、并行利用工具，而不是让一群模型“集体思考”。第 1 章的“约束让 agent 更强”会在多 agent 层面重演：子代理越少权限、越清楚任务边界，主代理越容易判断其成果。

---


# 第 21 章　子代理与多 Agent

**本章任务**：为 mini-codex 增加 `spawn_agent` 与 `wait`，明确何时该隔离上下文、何时不该并行；并用工具裁剪、证据回传和超时取消守住主会话。核心命题不是“开很多 agent 就很强大”，而是**并行的是上下文，不是理解**。

---

## 21.1 先承认一个反直觉事实

最常见的多 agent 设计误区，是把一个大任务机械拆成十个“思考者”，以为它们会像 MapReduce 一样加速。真正瓶颈通常不是 CPU，而是**上下文、工具权限和综合判断**。如果子代理各自读一堆无关文件，再把十万 token 原文贴回主会话，你只是把检索成本换成了聚合成本；主模型仍要理解全部内容，且还要判断谁在说谎。

因此默认不要派生。先做单 agent。只有当至少满足下面一条时才值得：

1. **上下文隔离**：探索任务需要读大量文件、网页或日志，而这些材料不应污染主会话的紧凑计划。
2. **可并行的独立证据**：多个来源、模块或测试彼此独立，且合并结果比顺序检索更快。
3. **故障隔离**：高风险实验可在受限子空间运行，失败不影响主计划。

耦合紧密的任务恰恰相反：后续步骤依赖前一步的中间结论，工具调用互相决定参数，或任务需要持续维护同一份状态。此时通信、等待、合并和冲突解决的开销会大于收益。

```text
主代理（保留计划与决策）
 ├── 子代理 A：只读探索 src/parser
 ├── 子代理 B：只读探索 crates/mcx-core
 └── 子代理 C：运行测试并收集失败签名

A/B/C → 证据摘要 + 引用 → 主代理做综合判断
```

这个图的要害是：**A、B、C 不互相聊天。** 它们只向主代理汇报；主代理是唯一拥有完整计划、并能批准副作用的决策者。否则你会得到“多头编辑同一文件”的经典灾难。

## 21.2 一个会失控的“自由子代理”

先写错误设计：

```rust
// 危险：子代理继承全部能力与主上下文
async fn explore_everywhere(task: &str) -> String {
    let child = Session::with_same_tools_and_full_history();
    child.submit(task).await;
    child.full_transcript().await // 把整段对话塞回主上下文
}
```

问题有四类：

| 症状 | 原因 |
|---|---|
| 主会话上下文爆炸 | 全文回传，压缩失效 |
| 子代理越权删文件 | 没有裁剪工具集 |
| 主代理无法发现偏离 | 只回传最终结论 |
| 一个卡死拖垮全局 | 没有 join/timer/cancel |

第 1 章说“约束让 agent 更强”，在多 agent 世界变成：**子代理越受限，主代理越能信任它的输出。** 工具、目录、预算、时间、输出格式都要有明确边界。自由子代理不是“更聪明”，而是“更不可预测”。

## 21.3 类型化派生：Spawn、工具策略与结果

沿用第 17 章的 `Session` 与 channel 模型。子代理也是 `Session<impl LlmClient>`，但拥有独立 history、独立 `CancellationToken`、独立工具注册表；主代理不共享可变状态，只通过 `AgentHandle` 等待结构化结果。

```rust
// crates/mcx-core/src/agent.rs
use tokio::sync::{mpsc, oneshot};

pub struct SpawnSpec {
    pub name: String,
    pub instruction: String,
    /// 若 None，默认只读 + 检索工具
    pub tool_policy: ToolPolicy,
    pub context_budget: usize,
    pub time_budget: std::time::Duration,
    /// 允许读取的工作区前缀；None 表示按主任务默认只读根
    pub readonly_roots: Vec<String>,
}

#[derive(Default)]
pub struct ToolPolicy {
    pub allow: Vec<String>,
    pub deny: Vec<String>,
    pub allow_write: bool,
}

pub enum AgentOutcome {
    Success { summary: String, evidence: Vec<Evidence> },
    Failed { reason: String, last_events: Vec<Event> },
    Cancelled,
    Timeout,
}

pub struct Evidence {
    pub source: String, // 文件、工具调用 id、URL
    pub excerpt: String,
    pub relevance: String,
}
```

`spawn_agent` 不“神奇地复制主代理”。它根据 spec 构造最小工具集；例如探索子代理只拿 `read_file`、`search`、`describe_tool`，不能拿 `apply_patch`、`run_command` 的写模式。这把第 12 章的 `execpolicy` 从单 agent 延伸为多 agent 策略：写权限不是模型属性，而是任务令牌属性。

```rust
// 伪代码；复用既有 Op/Event，不新增引擎私有协议
pub async fn spawn_agent(
    registry: &ToolRegistry,
    client: impl LlmClient,
    spec: SpawnSpec,
) -> AgentHandle {
    let tools = registry.filter(&spec.tool_policy);
    let (op_tx, op_rx) = mpsc::channel(16);
    let (event_tx, mut event_rx) = mpsc::channel(128);

    let mut session = Session::new_with_tools(client, op_rx, event_tx, tools);
    let (done_tx, done_rx) = oneshot::channel();

    tokio::spawn(async move {
        // 子代理有自己独立的提交循环
        session.submission_loop().await;
        let _ = done_tx.send(session.into_outcome());
    });

    op_tx.send(Op::UserInput { text: spec.instruction }).await.ok();
    AgentHandle { op_tx, done_rx, spec }
}
```

`AgentHandle` 只暴露两类操作：`wait`/`try_wait`，以及取消。主代理不得直接修改子代理的 history；这正是“上下文隔离”的编译期/运行期双重含义。

```rust
pub async fn wait(self) -> AgentOutcome {
    match tokio::time::timeout(self.spec.time_budget, self.done_rx).await {
        Ok(Ok(outcome)) => outcome,
        Err(_) => {
            let _ = self.op_tx.send(Op::Interrupt).await;
            AgentOutcome::Timeout
        }
    }
}
```

## 21.4 结果怎么回传：摘要不是可选项

子代理回传摘要还是原文，不是一个格式偏好，而是**上下文预算问题**。规则很简单：回传“结论 + 可核验证据”，不回传聊天流水。

```rust
async fn summarize_for_parent(events: &[Event], budget: usize) -> AgentOutcome {
    let evidence = collect_evidence(events);
    // 压缩：保留每个 source 的稳定引用和必要摘录，截断重复工具输出
    let trimmed = trim_evidence(evidence, budget);
    let summary = format_outline(&trimmed);
    AgentOutcome::Success { summary, evidence: trimmed }
}
```

例如子代理探索 parser：返回“`Parser::next` 在空输入时返回 `None`；相关定义位于 `src/parser.rs:42`；测试覆盖见 `tests/parser.rs:10`。证据：read_file(parser.rs)、search('fn next')。”主代理不需要看到每个中间 delta。

但两类内容不能只摘要：**最终结论所依赖的证据，以及冲突与失败。** 若子代理说“没有相关测试”，必须附上实际搜索调用和范围；否则主代理无法判断它是“确认没有”还是“读错目录”。这就是“证据链”要求：摘要可压缩，引用不可消失。

```rust
#[tokio::test]
async fn child_context_does_not_pollute_parent() {
    let parent = RecordingSession::new();
    let child_spec = SpawnSpec {
        name: "explorer".into(),
        instruction: "列出 src 的公共类型".into(),
        tool_policy: ToolPolicy {
            allow: vec!["local__read_file".into(), "local__search".into()],
            ..Default::default()
        },
        context_budget: 4000,
        time_budget: Duration::from_secs(10),
        readonly_roots: vec!["src".into()],
    };

    let handle = spawn_agent(&registry, ScriptedLlm::explore(), child_spec).await;
    let outcome = handle.wait().await;

    assert!(matches!(outcome, AgentOutcome::Success { .. }));
    // 父会话从未收到子代理的工具中间增量
    assert!(!parent.events().iter().any(is_tool_call));
}
```

测试也回答了本章标题的一层含义：**并行的是上下文。** 父与子各自有 `Session`，各自有事件流；合并发生在主代理显式调用 `summarize_for_parent` 之后。

## 21.5 何时并行：JoinSet、取消传播与写冲突

`tokio::task::JoinSet` 是自然的并发原语：派生若干子任务，等待它们完成或超时，且能在整体取消时统一通知。不要把每个子代理都 `tokio::spawn` 后忘记句柄——那样失去背压，也会泄漏任务。

```rust
use tokio::task::JoinSet;

pub async fn run_parallel(children: Vec<AgentHandle>) -> Vec<AgentOutcome> {
    let mut set = JoinSet::new();
    for child in children {
        set.spawn(async move { child.wait().await });
    }
    let mut results = Vec::new();
    while let Some(res) = set.join_next().await {
        results.push(res.expect("子任务 panic 应当被结构化错误替代"));
    }
    results
}
```

但并行写必须由主代理串行合并。一个有效模式是“**只读并行，写回串行**”：子代理只收集证据；主代理综合后自己执行一次 patch，或把写权限限于独立文件集合。

```rust
pub enum MergePlan {
    /// 各子代理负责完全不相交的文件
    DisjointFiles(Vec<String>),
    /// 主代理将基于摘要统一编辑
    CentralizedEdit,
}
```

若 `DisjointFiles` 检查发现交集，就拒绝并行写，退回主代理顺序执行。这不是保守主义，而是把“编辑冲突”前移为可判定错误；agent 时代冲突不会在 code review 里被礼貌指出，它会直接生成两份互相覆盖的 diff。

取消传播则要求子代理使用第 19 章的 `CancellationToken` 层次：父超时 → `Op::Interrupt` → 子会话 → 子工具。测试不能用 `std::thread::sleep` 永久卡住，而要注入可控时钟或可中断 future。

```rust
#[tokio::test(flavor = "current_thread", start_paused = true)]
async fn child_timeout_does_not_leak_parent() {
    let handle = spawn_agent(&registry, never_ending_llm(), explorer_spec()).await;
    let outcome = handle.wait().await;
    assert!(matches!(outcome, AgentOutcome::Timeout));
    // 父任务仍可在自己的 token 下继续
    assert!(parent_token().is_cancelled() == false);
}
```

## 21.6 发现漂移：子代理失败不是终点，而是证据

子代理会漂移：它可能读错目录、把旧 API 当现行 API、为凑结论伪造“未发现测试”、或在受限工具下绕过策略。主代理必须用四类信号识别：

| 漂移信号 | 检测 |
|---|---|
| 工具调用违反 policy | 子会话启动前过滤工具，运行时由 `execpolicy` 拒绝 |
| 结论缺少证据 | 要求 `Evidence` 非空且 source 可追溯 |
| 输出格式不合法 | 子代理结果先经 schema/结构检查 |
| 与主代理已知事实冲突 | 主代理提供“不可变前提”清单并核对 |

```rust
fn validate_outcome(outcome: &AgentOutcome, premises: &[&str]) -> Result<(), String> {
    let AgentOutcome::Success { evidence, summary } = outcome else {
        return Err("子代理未成功完成".into());
    };
    if evidence.is_empty() {
        return Err("结论没有附带证据".into());
    }
    for p in premises {
        if !summary.contains(p) && !evidence.iter().any(|e| e.excerpt.contains(*p)) {
            return Err(format!("缺少前提证据：{p}"));
        }
    }
    Ok(())
}
```

发现漂移后不要自动“再派十个 agent 去修”。正确纠正链是：记录失败 trace → 缩小子代理权限/上下文 → 让主代理基于证据决定重试、换策略或停止。第 20 章的回归任务应覆盖“子代理读错 crate”“超时回收”“写权限被拒绝但仍返回成功”等情形；否则多 agent 只会在演示中漂亮，在长任务中失控。

## 避坑专栏 #22：共享 `Arc<History>` 看似省 token，实则破坏隔离

新手为了“让子代理看见主上下文”而写：

```rust
// 危险：共享可变历史
let child_history = Arc::clone(&parent_history);
Session::new(client, child_history, ...)
```

现象包括：父任务被子任务的探索性假设污染；两次并行子任务同时追加 item，顺序乱掉；压缩策略无法判断哪些属于当前计划；回放 diff 无法区分父与子。所谓节省 token，换来了无法测试的耦合。

正确做法是把要共享的内容**显式复制为只读 brief**：

```rust
let brief = ParentBrief {
    goal: parent.goal.clone(),
    invariants: parent.invariants.clone(),
    relevant_files: select_files(&parent.plan, 10),
};
spawn_agent(registry, client, spec.with_brief(brief)).await;
```

通用形式：**共享只读事实，隔离可变过程。** 若实在需要双向协作，也应设计增量协议和版本化状态机，而不是把 `Vec` 包进 `Arc<Mutex<_>>` 就算“通信”。

## 21.7 Design Rationale

**Q：为什么默认子代理工具更少？**

因为“能做什么”决定了“可能做错什么”。只读探索者没有 `apply_patch`，就不会因误解而乱改代码；受限检索者没有网络写权限，就无法外泄数据。这与第 1 章“约束让 agent 更强”完全同构：边界减少搜索空间，也把失败限制在可审计范围。

**Q：为什么不让子代理直接互相通信？**

因为综合判断需要全局视角。A 与 B 各自不知道对方拿到的前提、预算和策略；它们若自行协商，容易重复探索、互相覆盖、伪造完成。主代理是唯一被授权拥有计划与写权限的角色。通信图保持树形，复杂问题不会因此消失，但故障点可追踪。

**Q：并行真的更快吗？**

只在任务独立且证据可合并时成立。估算加速时应计入：派生开销、上下文复制、工具并发限制、结果压缩、冲突合并与验证。若这些超过顺序执行，宁可串行。多 agent 的价值首先是**关注点分离和故障隔离**，不是“模型数量翻倍”。

## AI 软件工程原理 #21

> **并行的是上下文，不是理解。**

多个 agent 可以各自加载不同文件、调用不同工具、在隔离空间试错；真正的判断——什么是可信证据、哪里存在冲突、何时可以写回——仍要由一个有完整计划的主代理承担。把“并行计算”误当成“并行理解”，就会制造更多上下文、更多冲突和更高的纠错成本。

| | 单 agent | 合理多 agent |
|---|---|---|
| 上下文 | 共享、易膨胀 | 子空间隔离、摘要回传 |
| 工具 | 全集 | 按任务裁剪 |
| 写权限 | 可集中控制 | 默认禁止，主代理串行合并 |
| 并行对象 | 通常顺序推理 | 独立检索/验证/测试 |
| 失败处理 | 同会话重试 | 超时、漂移检测、隔离回收 |

原理 #21 是第 20 章评测的前置条件：没有任务边界和证据 schema，并行无法判定；有了它们，才能用 T20 那样的回归任务测“多 agent 是否比单 agent 更快且同样正确”。第 22 章会让 mini-codex 自己维护这套约束——那时，子代理也必须遵守仓库的架构与清理规则。

## Rust 修炼小结

| 概念 | 本章用法 | 后面在哪用到 |
|---|---|---|
| `Arc` | 只读配置/工具注册表共享 | 缓存、策略查询 |
| `oneshot` | 子代理单次完成信号 | RPC 请求终态 |
| `JoinSet` | 并发派生与统一回收 | 批量检索、测试 |
| `CancellationToken` 层级 | 超时传播 | TUI 取消、清理任务 |
| `serde` schema | 证据与摘要结构 | 评测、回放 |

## 章末验收

- [ ] 一个只读探索子代理运行后，父会话历史不含其子工具 delta
- [ ] 越权工具在派生前被移除，运行时仍由 `execpolicy` 二次拒绝
- [ ] 子代理超时后主会话可继续执行，且任务被回收
- [ ] 并行写同一文件会被合并计划拒绝，退回串行编辑
- [ ] 子代理结论缺少可追源证据时，主代理判定为漂移

## 读者挑战

1. 设计一个“**研究者 + 实施者**”的两级结构：研究者只读，实施者可写但每次 patch 必须引用研究者结论。**写一条回归任务证明实施者无法凭空引用不存在的证据。**
2. 当三个子代理分别发现三种互相矛盾的事实，主代理应怎样表示与解决冲突？请给出结果类型与至少两条可判定规则。
3. 上下文隔离的成本是重复读取。设计一个**只读共享片段缓存**，使子代理复用主代理已加载的稳定文件，却仍无法追加自己的探索 item。写测试证明缓存命中不会污染父历史。

## 下一章预告：让 agent 维护它自己的家

多 agent 让项目能力更强，也更容易复制仓库中的坏模式。下一章把全书拉回起点：为 mini-codex 自身写 `AGENTS.md`、golden principles、依赖方向测试和清理 agent。然后用你自己写的 harness，给这个 harness 加一个真实功能，跑通 CI 和评测。第 1 章的 `SafetyRule` 会长成 `execpolicy`，第 2 章的 crate 边界会成为防熵第一道墙，而 rustls 静态链接会兑现为可复制的单文件发布。

---


# 第 22 章　自举：用 mini-codex 开发 mini-codex

**本章任务**：把 mini-codex 项目本身变成受控工作环境，让 agent 能在明确边界内开发、测试、清理；再让它为自己实现一项真实功能并过 CI。这是全书终点：**不是造出一个能写代码的模型，而是证明 harness 能持续约束它自己。**

---

## 22.1 自举的第一原则：先锁住环境，再交钥匙

如果你直接让 agent “改进项目”，它会读遍仓库、复制附近风格、随意加依赖、修改 core、顺手改测试，最后提交一个“看起来完成了”的 PR。第 2 章已解释为什么人类 code review、口头约定和老人带新人都无法约束 agent：它不看规范、不记得上次讨论，还会把仓库里的坏模式复制给下一个文件。因此自举第一步不是 prompt，而是**把规则变成仓库中的机械强制物**。

项目根目录加入四件套：

```text
mini-codex/
├── AGENTS.md              # 入口、角色、安全边界
├── GOLDEN_PRINCIPLES.md   # 不可协商的高层原则
├── ARCHITECTURE.md        # crate 边界、数据流、变更规则
├── CONTRIBUTING_AUTONOMOUS.md  # agent PR 的最小流程
└── .github/workflows/
    ├── ci.yml             # build + test + eval
    └── architecture.yml   # 依赖方向与结构测试
```

`AGENTS.md` 不是愿望清单，而是当前任务范围的可加载上下文。第 14 章的分层加载在此兑现：根文件只放全局红线，子目录按需提供局部说明。

```markdown
# AGENTS.md

## Role
You maintain mini-codex. Constrain, inform, verify, correct.
You may NOT: disable tests, skip CI, add workspace dependencies without an architecture change,
or edit files outside the scope approved for the task.

## Safety
- Every external side effect goes through `execpolicy`.
- Destructive commands require an explicit human approval record in the event log.
- Refuse with `cannot safely complete` when evidence is insufficient.

## Workflow
1. State the accepted scope and non-goals.
2. Read only the crates/files required.
3. Prefer the smallest change that passes `cargo test` and `mcx-eval`.
4. Add or update a regression task before claiming completion.
5. Never broaden the task because "it is related".
```

注意它与第 1 章 `SafetyRule` 的血缘：`requirement + refusal` 现在成为仓库级 `execpolicy` 规则。第 12 章的 `SafetyRule` 工业化后，规则可以离线查询、按命令/路径/工具来源匹配；agent 不能“因为没看到提示词”而绕过。这也是“约束让 agent 更强”的终极形态：不知道全部历史没关系，规则文件和 CI 都在当前工作区里。

## 22.2 依赖方向不是文档，而是第一道防火墙

第 2 章画过箭头：`mcx-cli → mcx-core → {mcx-tools, mcx-sandbox} → mcx-protocol`。`crates/mcx-protocol` 不依赖任何 workspace crate；`mcx-core` 不直接依赖 `mcx-cli`；MCP、TUI、评测都通过 trait/协议接入。这个结构本身就是防熵的第一道墙：边界清楚时，坏模式只能在一个 crate 内扩散；边界糊掉后，agent 会“为了方便”在 core 里塞 CLI 细节、在 protocol 里塞工具实现。

Cargo 能挡意外，却挡不了有人给 `Cargo.toml` 主动加一行反向依赖。第 2 章避坑专栏 #2 留下三档手段，本章把它们逐档落地：先装前两档硬门槛，再把可定制项做成架构测试（见 22.3）：

| 手段 | 配置/代码 | 守护对象 |
|---|---|---|
| Cargo 显式依赖 | 现有 `Cargo.toml` | 意外 `use` 他 crate |
| `cargo-deny` | `deny.toml` | 显式违规、未知/重复 crate、许可 |
| 自定义架构测试 | `tests/architecture.rs` | 依赖层级、禁用路径、crate 命名 |

```toml
# crates/mcx-core/Cargo.toml
[dependencies]
mcx-protocol = { path = "../mcx-protocol" }
mcx-tools = { path = "../mcx-tools" }
mcx-sandbox = { path = "../mcx-sandbox" }
# 故意不写 mcx-cli；CI 会拒绝新增这一行
```

`deny.toml` 不只为安全审计。它明确禁止反向依赖和未声明来源：

```toml
[graph]
all-features = true
exclude = []

[deny]
workspace-duplicates = "warn"
crates-io = "deny"
git = "deny"
path = "deny"

[advisories]
vulnerability = "deny"
unmaintained = "warn"

# 自定义：通过 cargo-deny 的 advisory 机制/外部脚本，
# 或在架构测试中硬性禁止；配置仅作第一道声明。
```

真正的“可定制”交给架构测试：解析 workspace、读取每个 `Cargo.toml` 的依赖表、构建有向图并断言层级。第 2 章承诺的自定义测试就长这样：

```rust
// crates/mcx-arch/tests/architecture.rs
use std::collections::{HashMap, HashSet};
use toml::Value;

const LAYERS: &[&str] = &[
    "mcx-protocol",
    "mcx-sandbox",
    "mcx-tools",
    "mcx-core",
    "mcx-telemetry",
    "mcx-eval",
    "mcx-cli",
];

#[test]
fn dependency_edges_respect_layers() {
    let root = std::env::var("CARGO_WORKSPACE_DIR").unwrap_or(".".into());
    let graph = load_workspace(&root);
    let index: HashMap<_, _> = LAYERS
        .iter()
        .enumerate()
        .map(|(i, name)| (*name, i))
        .collect();

    for (crate_name, deps) in &graph {
        let from = *index.get(crate_name.as_str()).expect("未知 crate；请加入 LAYERS");
        for dep in deps {
            let to = match index.get(dep.as_str()) {
                Some(layer) => *layer,
                // 第三方依赖不参与 workspace 分层
                None => continue,
            };
            assert!(
                to <= from,
                "{crate_name}（层 {from}）不允许依赖 {dep}（层 {to}）：依赖只能向下",
            );
        }
    }
}
```

`to <= from` 表达“下层可被依赖，上层不能反向依赖”。例如 `mcx-core`（层 3）依赖 `mcx-tools`（层 2）合法；反之失败。`assert!` 信息必须包含 crate 名与层级，这样 agent 生成的 PR 一跑 CI 就知道该回退，而不是靠人类解释“为什么不优雅”。

> **架构违规应该是编译错误，而不是 review 意见。** 这是第 2 章原理 #2 的兑现。现在它不是哲学，而是 `cargo test -p mcx-arch` 的红色行；agent 也无法以“只是临时改一下”绕过。

## 22.3 结构测试：把风格、可见性与状态规则写进 CI

依赖方向只是骨架。结构测试还负责捕捉 agent 常见的“局部合理、全局腐烂”：把 `pub` 当默认值、让 `mcx-core` 直接打开文件、绕过 `Op` 去操作 TUI、在 `protocol` 引入工具实现、把新能力塞进 `Session` 而不是新 crate。

```rust
#[test]
fn protocol_has_no_workspace_dependencies() {
    let toml = std::fs::read_to_string("crates/mcx-protocol/Cargo.toml").unwrap();
    assert!(!toml.contains("mcx-core"));
    assert!(!toml.contains("mcx-tools"));
    assert!(!toml.contains("mcx-sandbox"));
}

#[test]
fn core_must_not_depend_on_cli_or_eval() {
    let toml = std::fs::read_to_string("crates/mcx-core/Cargo.toml").unwrap();
    for forbidden in ["mcx-cli", "mcx-eval", "mcx-tui"] {
        assert!(
            !toml.contains(forbidden),
            "mcx-core 不得依赖 {forbidden}；如确有共享能力，下沉到 mcx-protocol",
        );
    }
}

#[test]
fn session_uses_op_channel_for_engine_input() {
    let src = std::fs::read_to_string("crates/mcx-core/src/session.rs").unwrap();
    assert!(src.contains("self.op_rx.recv()"));
    // 反模式：引擎直接读取终端/传输。此处可加更细规则，但要避免脆弱正则。
    assert!(!src.contains("std::io::stdin()"));
}
```

结构测试不是 lint 的全部；它只守“一旦破坏就很难发现”的红线。具体 Rust 风格交给 `clippy --deny warnings`、格式化交给 `rustfmt`、安全敏感逻辑交给类型与 property test。不要把所有建议都机械化，否则测试会脆弱到每次重构都红；结构测试的对象应是**不变量**，不是审美偏好。

## 避坑专栏 #23：用正则守架构，规则太脆反而逼人绕过

下面做法很常见但危险：

```rust
// 脆弱：匹配具体模块路径，重构就坏
assert!(!src.contains("use mcx_tools::fs"));
```

现象是：工程师重命名模块后 CI 红，便把测试改成 `// TODO`，规则名存实亡。正则只能用于稳定锚点：`Cargo.toml` 的 crate 名、`Session` 文件名、明确禁止的 `std::io::stdin()`。可变代码结构的检查优先用 `syn`/`cargo metadata` 或真实编译。

**通用形式**：结构测试 = 稳定事实（依赖图、公开 API、crate 名） + 可恢复错误。发现规则误报时，先收紧匹配条件或改用 AST，而不是删除测试。

## 22.4 熵管理：GC 式的小额高频还债

agent 会复制仓库中已有的坏模式，因为那些模式“看起来像项目风格”。OpenAI 内部实践曾发现这种漂移：团队每周五花 20% 时间人工清理“AI 垃圾”，很快不可持续；最终把 golden principles 写进仓库，并让定期清理 agent 扫描偏离、提交小重构 PR[citation:1][citation:4]。这不是“让 AI 修 AI”的浪漫叙事，而是工程化的技术债回收。

第 2 章把 crate 边界称为“防熵第一道墙”；墙再高，也会有小错漏进来：重复实现、过宽 `pub`、测试复制、过期注释、绕过 `ScriptedLlm` 的真实网络调用。因此引入“架构 GC”：

```text
每周/每次 nightly：
1. 结构测试生成偏离报告（重复代码、可见性、依赖、未用项）
2. 清理 agent 只领小范围任务，例如“把 mcx-tools 的三个 fs helper 合并为一个”
3. 每个 PR 只改一个 crate、一个主题，附 before/after 与评测结果
4. CI 必须绿；不修深层问题，不扩大 scope
```

像 GC 一样，关键是**小额高频**：一次清理十处低风险偏离，远胜季度大扫除。后者会把审查负担集中、增加合并冲突，还会引诱 agent “顺手重构一切”。

```rust
// crates/mcx-gc/src/lib.rs
pub struct DriftReport {
    pub findings: Vec<Finding>,
}

pub struct Finding {
    pub crate_name: String,
    pub file: String,
    pub rule: &'static str,
    pub suggested_scope: String,
}

pub fn collect(deps: &DependencyGraph, sources: &[SourceTree]) -> DriftReport {
    let mut findings = Vec::new();
    check_public_surface(sources, &mut findings);
    check_dependency_cycles(deps, &mut findings);
    check_duplicate_helpers(sources, &mut findings);
    DriftReport { findings }
}
```

清理 agent 的输出不是“自动 apply”，而是**自动开 PR**：scope 受 `AGENTS.md` 约束，CI 跑结构测试、单元/评测、架构检查；reviewer 只需验证小重构是否语义等价。这是“Verify + Correct”的循环：工具检测偏离，agent 提出修复，机械测试守门，人类处理真正模糊的架构取舍。

> **技术债像高息贷款，小额高频还优于攒着一次还。** 对 agent 项目尤其如此：它复制坏模式的速度远快于人类，等待大重构等于让利息复利。

## 22.5 真实闭环：让它给自己加一个功能

现在到了全书最关键的演示。目标功能选得小而有代表性：**给 CLI 增加 `--eval` 输出机器可读的 JSON summary，并由评测集验证。** 它涉及 CLI 参数、core 事件聚合、输出格式和 CI，却不至于让 agent 重写引擎。

`CONTRIBUTING_AUTONOMOUS.md` 规定任务单：

```yaml
- id: feat-eval-summary
  scope:
    - crates/mcx-cli/src/eval.rs
    - crates/mcx-core/src/telemetry.rs
  non_goals:
    - 不改动 TUI 渲染
    - 不接入真模型
    - 不修改评测 fixture 格式，只新增字段
  acceptance:
    - cargo run --eval --task T07 输出合法 JSON，含 score/task_results
    - cargo test -p mcx-eval 全绿
    - cargo test -p mcx-arch 全绿
    - 新增回归任务禁止 panic 且不超过预算
```

主代理派两个受限子代理：A 只读 CLI/TUI 层，确认现有输出接口；B 只读 core telemetry，列出可复用字段。它们都返回带 source 的摘要。主代理合并后实施，不再让子代理直接写 `mcx-core`，避免并行修改架构关键 crate。

下面是一段可运行的最小实现，沿用 `Op`/`Event` 和既有 `Rollout`/`Session`：

```rust
// crates/mcx-cli/src/eval.rs
use mcx_protocol::{Event, Op};
use serde_json::json;
use tokio::sync::mpsc;

pub async fn run_eval(task: &str) -> Result<(), Box<dyn std::error::Error>> {
    let (op_tx, op_rx) = mpsc::channel(16);
    let (ev_tx, mut ev_rx) = mpsc::channel(128);

    let session = Session::new(ScriptedLlm::from_task(task)?, op_rx, ev_tx);
    tokio::spawn(async move { session.submission_loop().await });

    op_tx.send(Op::UserInput { text: task.into() }).await?;
    let mut completed = 0usize;
    while let Some(ev) = ev_rx.recv().await {
        if matches!(ev, Event::Shutdown) { break; }
        if matches!(ev, Event::TurnComplete { .. }) {
            completed += 1;
        }
    }

    let summary = json!({
        "score": 1.0,
        "completed_turns": completed,
        "task": task,
    });
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}
```

这段代码刻意保持最小：真实输出会聚合第 20 章的 trace、任务结果和基线差；但教学重点是**闭环**——agent 修改代码后必须新增/更新回归任务，然后用脚本化模型跑评测。

```bash
cargo test -p mcx-arch
# test dependency_edges_respect_layers ... ok
# test protocol_has_no_workspace_dependencies ... ok

cargo test -p mcx-eval
# baseline: 20/20

cargo run --eval --task T07
# {
#   "score": 1.0,
#   "completed_turns": 3,
#   "task": "T07_add_unit_test"
# }
```

若 agent 为了“加功能”把 `serde_json::to_string` 直接写进 `mcx-protocol`，结构测试会红；若它绕过 `Op` 在 CLI 里重开模型循环，架构 GC 会报告；若输出缺字段，第 20 章的 eval baseline guard 会要求同步更新 schema。这样，自举不是让 agent 拥有 root 权限，而是让它在**一组自动门**中工作。

## 22.6 发布：把 rustls 的选择兑现为单文件二进制

第 1 章选择 `reqwest` 的 `rustls-tls` 而非系统 OpenSSL，理由是静态链接能避免目标机器缺库。现在它要与 reproducible build、依赖冻结和 CI 一起变成发布物。发布配置放在 workspace 根：

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
panic = "abort"
```

```bash
cargo build --release --locked
# 得到 target/release/mcx

file target/release/mcx
# ELF 64-bit, statically linked (用户态), stripped
ldd target/release/mcx
#   not a dynamic executable   # glibc 等平台细节可能不同，请勿在书中承诺“完全无动态链接”
```

静态链接的承诺要克制：Rust 标准库在 Linux 上通常仍需 glibc 或采用 musl 才能完全静态；本书只保证“避免 OpenSSL 运行时依赖，减少目标环境差异”。跨发行版发布应使用 CI matrix 或 musl 目标，并实测干净容器。第 1 章埋下的 rustls 伏笔在此兑现：**发布物是单个可复制文件，不是“先装一堆系统包再跑”。**

```yaml
# .github/workflows/release.yml
- name: build release
  run: cargo build --release --locked
- name: smoke test on clean image
  run: |
    cp target/release/mcx /tmp/mcx
    /tmp/mcx --help
    /tmp/mcx eval --task T07
- name: upload artifact
  uses: actions/upload-artifact@v4
  with:
    path: target/release/mcx
```

烟雾测试不是完整评测，但必须至少跑 `--help` 和一个脚本化 eval。否则“单文件二进制”只能证明它编译成功，不能证明它还能工作。

## 避坑专栏 #24：自举不意味着给 agent 自己的仓库写权限

最危险的一句话是“既然它能开发，就让它直接 push 到 main”。

```yaml
# 禁止：无审批的自动合并
- run: mcx apply-plan && git push origin main
```

现象是：一次评测绿了的“重构”可能悄悄放宽 `execpolicy`、删除架构测试、把 `unwrap` 加进 core；CI 绿只说明当时测试通过，不证明长期不变量仍在。自举的正确闭环是 **PR + 门禁 + 小范围**，不是自治合并。

正确策略：

- agent 只改 `AGENTS.md` 批准的 scope；
- 破坏性架构变更必须人审；
- 每个 PR 附带 eval 报告与 `git diff --stat`；
- nightly GC PR 同样走 CI，发现深层问题转为 issue；
- 一旦 CI 脚本/架构测试被修改，需要“受保护审批”，避免 agent 修改自己的裁判。

## 22.7 Design Rationale

**Q：为什么自举不从第一天开始？**

因为自举的先决条件是稳定 harness：协议、沙箱、审批、评测、结构测试。没有它们，agent 在自己仓库里只会加速熵增。第 1—19 章建能力，第 20 章使其可测，第 21 章使其受控，第 22 章才敢让它触碰代码。顺序不能颠倒。

**Q：为什么架构测试比“好文档”可靠？**

文档依赖 agent 读取、理解和遵守；测试只依赖 CI 是否绿。文档适合解释意图，测试适合守不变量。二者互补：把“为什么”放 `ARCHITECTURE.md`，把“允许什么”放 `Cargo.toml` + `deny.toml` + 结构测试。

**Q：为什么清理 agent 必须小 PR、高频运行？**

因为大重构同时触碰多 crate，评测难以定位回归；高频小额变更让每次 CI 只承担少量风险，也符合 GC 的局部回收思路。技术债一旦集中处理，常常演变成“为还债而借新债”。

## AI 软件工程原理 #22

> **技术债像高息贷款，小额高频还优于攒着一次还。**

在 agent 项目中，这条原理被放大：模型不以人的速度写代码，它以会话速度复制既有模式。今天一处 `pub` 过宽、一次绕过 `Event`、一个过期 fixture，明天可能出现在五个 crate。因此必须让还债机制自动化、常态化、可验证——`cargo-deny`、架构测试、结构测试、eval baseline、定期 GC PR 都是“分期付款”。

| | 攒着一次还 | 小额高频还 |
|---|---|---|
| 单次风险 | 大，易冲突 | 小，易回滚 |
| 审查负担 | 集中且难判断 | 分散、每个 PR 可复核 |
| 评测定位 | 回归难归因 | 一次只引入少量变化 |
| 对 agent 的约束 | 文档/口头 | CI 红线 + scope |

原理 #22 也收束了全书：第 2 章的 crate 边界是第一道墙，第 12 章的 `execpolicy` 是运行时墙，第 20 章的评测是质量墙，本章的架构 GC 是持续修复机制。**四者共同决定 agent 是否能长期维护真实仓库。**

## Rust 修炼小结

| 概念 | 本章用法 | 全书位置 |
|---|---|---|
| workspace + `Cargo.toml` | 分层依赖、发布统一 | 第 2 章起点 |
| `cargo-deny` + TOML | 依赖/许可红线 | 第 2 章三档手段兑现 |
| `toml`/`syn`/文件扫描 | 结构测试 | 评测与 CI |
| `serde_json` 稳定输出 | `--eval` 可机读 | 第 20 章评测聚合 |
| LTO/strip/静态链接 | 单文件发布 | 第 1 章 rustls 伏笔 |

## 章末验收

- [ ] 根目录具备 `AGENTS.md`、golden principles、架构说明和受控流程
- [ ] `cargo test -p mcx-arch` 在反向依赖、protocol 依赖 workspace crate 时失败
- [ ] 架构 GC 能产出小范围 drift 报告，清理 PR 不扩大任务 scope
- [ ] 一个真实功能由 mini-codex 实现，通过 `mcx-eval` 与架构测试后合并
- [ ] `--release` 单文件二进制在干净测试环境中跑通 `--help` 和脚本化 eval

## 读者挑战

1. 设计一个“**架构 GC 预算**”：每次 nightly 最多改 N 处、只跨一个 crate、不得触碰 `mcx-protocol`。写测试证明超过预算时任务会被拆分。
2. 让清理 agent 识别“重复实现”时，必须区分**真正重复**和**表面相似但语义不同**。请给出至少三种会误报的情况，并设计证据要求。
3. 给发布流水线增加 reproducible build 检查：同一 commit 两次构建的二进制哈希一致。**哪些 Rust/Cargo 因素会破坏它？** 你将如何固定？

---

## 全书收束：从“会说话的函数”到“能维护自己的系统”

你从开篇的一个函数出发：`reqwest` 把字符串发给模型，再把文本打印出来。那不是 agent，只是一个会说话的函数。第 1 章的三个实验没有换模型，只改变了约束、边界与验收；它们已经预告了全书的判断——**Agent = Model + Harness**。

随后你拆解了 Codex，把引擎与表面分开：第 3 章留下 `Op`/`Event` 的两条 channel，第 4 章把字节流处理成对帧边界的纯函数，第 5 章用可回放、可判定的 `Item` 记录真相。工具从第 6 章的 trait object 长成第 18 章的 MCP 运行时扩展；权限从第 1 章的 `SafetyRule` 长成第 12 章的 `execpolicy`；并发从第 7 章的取消与超时，到第 17—19 章的协议、MCP 和 TUI，最终在第 21 章成为受控的多 agent 上下文隔离。

贯穿这些设计的，是一组你可以带走的判断力：

1. **原理 #1、#2：模型可替换，harness 与架构边界才是长期资产；违规应成为编译错误。**
2. **原理 #3、#20：事件流是可判定真相，评测把“是否更好”从主观印象变成 CI 红线。**
3. **原理 #5、#21：可观测与结构化证据优先于更多 token；并行的是上下文，不是理解。**
4. **原理 #22：技术债要靠小额高频的 GC，而不是临时的英雄式清理。**

第 22 章的闭环证明了这一点：不是让模型替你写代码，而是让它在一个由 crate 边界、`execpolicy`、脚本化评测、架构测试和发布门禁组成的 harness 里工作。这里的“安全”不是第 1 章那个二十几行的 `SafetyRule` 示例，而是规则文件、策略执行、事件证据与 CI 的组合；这里的“发布”也不是一个能演示的二进制，而是 rustls、锁定依赖、LTO/strip 和干净环境烟雾测试共同保证的可交付物。

当你以后再遇到任何 agent 系统，无论是编码工具、客服助手还是浏览器代理，都可以用同一套问题拆解它：

- 模型之外，哪些约束是不可协商的？
- 上下文从哪里来，又在哪里被压缩？
- 每次工具调用是否有可审计的权限和证据？
- 取消、超时、失败如何在并发中传播？
- 事件流能否回放，验收能否自动判定？
- 架构边界是否由编译器、依赖图和 CI 强制？
- 技术债有没有小额高频的回收机制？

这些问题比“用了哪个模型”更重要，因为它们决定系统能否持续、可测、可控地工作。你已经从零造出了一个能运行、能评测、能自举的 mini-codex；更重要的是，你已具备判断“任何 agent 系统该长什么样”的能力。

## 引用来源

[1] https://www.anthropic.com/research/harness-engineering
> 讨论了通过工程化 harness、环境设计与结构化反馈让 agent 持续工作的实践。

[4] https://blog.langchain.com/terminal-bench-results/
> LangChain 公开分享其编码 agent 在 Terminal Bench 2.0 上从 52.8% 提升到 66.5% 的结果，并说明改动集中在 harness 而非模型参数。

[12] https://modelcontextprotocol.io/specification/2025-11-25/basic
> MCP 使用 JSON-RPC 2.0 作为消息传输基础，请求、响应与通知有统一结构。

[16] https://modelcontextprotocol.io/specification/2025-11-25/basic
> MCP 的初始化阶段交换协议版本与能力，工具、资源和提示模板通过声明被发现。

---

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
