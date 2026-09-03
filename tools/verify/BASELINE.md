# REAL-SUSPECT 发布前技术核查基线（36 项）

> 作用：把 `classify_suspects.py` 判出的 36 项 REAL-SUSPECT 逐项人工定案，
> 回答"发布前是否还有未处理的内容/代码错误"，并为后续 `code/` 最终版抽取归位提供宿主地图。
> 基线文件：`book/scratch/book-draft.md`（即拆分 seed，`md[n-m]` 为其行号）　|　工具链：rustc/cargo 1.98.0　|　核对日期：2026-09-03

## 一、总体结论

1. **36 项全部为"写作风格预期形态"，无新增真错误。** 与《代码验证报告》口径一致：
   本书是"章节递进、聚焦改动处"，绝大多数代码块是省略了前后文的片段，逐块独立编译不是合理目标。
2. 已有内容层修复均已落在同一书稿上，与本表不重叠、不冲突：
   - 逐块语法层真错误 1 处（第 15 章快照测试 `]}` 收尾）→ 已修；
   - 跨章符号一致性 7 处（`LlmError::Cancelled`、`ToolError::InvalidArgs`、`PatchError::EmptyPath`、
     `HookError` 全补、`Event::Compacted{kept_turns}`、死锁图口径、第 10 章反例注记）→ 已修；
   - 内容勘误 A-1–A-9 / B-1–B-7 / C-1–C-5 共 21 项 → 已全部落稿。
3. **类别分布**：`H` 方法宿主 13 项、`D` 依赖上下文 19 项、`P` 教学占位 4 项（合计 36）。
   判定依据：rustc 主错 + 块首行 + 就近正文抽查（方法片段正文多以"在 xx 里加入/补全"指路）。

## 二、判定类别字典

| 码 | 名称 | 含义 | code/ 归位时的处理 |
|---|---|---|---|
| `H` | host-method | 方法片段，缺所属 `impl` 外壳（错误多为 `self` parameter is only allowed in associated functions） | 归入对应类型 `impl`；宿主名以方法名/就近正文为准 |
| `D` | dep-context | 需外部 crate/derive/`use` 上下文（serde、thiserror、async_trait、tokio、serde_json、json!） | 落入 crate 后随真实 `use`/依赖自动消解，应转 PASS |
| `P` | placeholder | 正文演示占位（`...` 省略、单字段/单变体展示、反例展示） | 保持示意注释，或抽取时补成完整形态 |

## 三、36 项逐项定案表

| # | idx | 章 | md 行 | 块首行（截断） | rustc 主错 | 类别 | 说明 |
|---|---|---|---|---|---|---|---|
| 1 | 114 | 12 | 6071–6080 | `#[derive(Debug, thiserror::Error)]` | crate `thiserror` 未找到 | D | HookError 展示块（跨章修复新增相关），缺 use/依赖 |
| 2 | 131 | 14 | 6890–6899 | `pub fn render(&self) -> String {` | `self` 仅限关联函数 | H | render 方法片段，正文指路就近 impl |
| 3 | 144 | 15 | 7351–7357 | `#[derive(..., Serialize)]` | derive `Serialize` 未找到 | D | 缺 serde 上下文 |
| 4 | 148 | 16 | 7555–7570 | `#[derive(..., Serialize, Deserialize)]` | derive `Serialize` 未找到 | D | 缺 serde 上下文 |
| 5 | 160 | 17 | 8124–8150 | `async fn handle(&mut self, msg: ClientMessage)` | `self` 仅限关联函数 | H | server handle 片段 |
| 6 | 162 | 17 | 8235–8249 | `async fn run_turn(&mut self, text: String) {` | unexpected token `...` | P | 伪码演示，含 `...` 省略占位 |
| 7 | 164 | 17 | 8265–8275 | `pub async fn shutdown(self) {` | `self` 仅限关联函数 | H | shutdown 片段 |
| 8 | 175 | 18 | 8576–8583 | `#[async_trait]` | attribute `async_trait` 未找到 | D | 缺 async_trait 上下文 |
| 9 | 176 | 18 | 8588–8601 | `#[async_trait]` | crate `serde_json` 未找到 | D | 缺依赖上下文 |
| 10 | 180 | 18 | 8668–8686 | `pub async fn select_for_prompt(&self, …)` | `self` 仅限关联函数 | H | 提示选择片段 |
| 11 | 181 | 18 | 8691–8696 | `async fn invoke(&self, args: Value, …)` | `self` 仅限关联函数 | H | 工具 invoke 片段 |
| 12 | 187 | 18 | 8811–8823 | `struct ScriptedTransport {` | non-item in item list | P | 测试辅助 impl 内含 `...` 占位 |
| 13 | 188 | 18 | 8826–8850 | `#[tokio::test]` | crate `tokio` 未找到 | D | 缺 tokio 上下文 |
| 14 | 189 | 18 | 8855–8867 | `#[tokio::test]` | crate `tokio` 未找到 | D | 缺 tokio 上下文 |
| 15 | 223 | 21 | 10092–10101 | `pub async fn wait(self) -> AgentOutcome {` | `self` 仅限关联函数 | H | session wait 片段 |
| 16 | 009 | 3 | 1386–1427 | `use serde::{Deserialize, Serialize};` | unresolved import `serde` | D | 缺 crate 依赖声明（正文字块起点） |
| 17 | 012 | 3 | 1558–1574 | `use async_trait::async_trait;` | unresolved import `async_trait` | D | 缺依赖上下文 |
| 18 | 013 | 3 | 1581–1610 | `pub struct OpenAiClient {` | crate `serde_json` 未找到 | D | struct 片段，缺 use |
| 19 | 015 | 3 | 1683–1699 | `pub struct ScriptedLlm {` | attribute `async_trait` 未找到 | D | 缺依赖上下文 |
| 20 | 018 | 3 | 1786–1789 | `async fn emit(&self, ev: Event) {` | `self` 仅限关联函数 | H | emit 方法片段 |
| 21 | 020 | 4 | 1964–2052 | `#[derive(Debug, Clone, PartialEq, Eq)]` | crate `thiserror` 未找到 | D | 错误枚举派生块，缺依赖 |
| 22 | 021 | 4 | 2067–2082 | `#[derive(serde::Deserialize)]` | crate `serde` 未找到 | D | 缺依赖上下文 |
| 23 | 022 | 4 | 2097–2106 | `#[async_trait]` | attribute 未找到 | D | 缺依赖上下文 |
| 24 | 023 | 4 | 2111–2152 | `async fn complete(&self, messages, delta_tx)` | `self` 仅限关联函数 | H | Llm::complete 片段（流式签名） |
| 25 | 031 | 5 | 2499–2511 | `#[derive(..., Serialize, Deserialize, …)]` | derive `Serialize` 未找到 | D | 缺 serde 上下文 |
| 26 | 034 | 5 | 2646–2648 | `#[serde(default)]` | visibility `pub` not followed by item | P | 规则 2 单字段展示片段 |
| 27 | 035 | 5 | 2655–2661 | `pub enum Item {` | attribute `serde` 未找到 | D | 规则 3 deprecated variant 演示 |
| 28 | 038 | 5 | 2785–2791 | `#[derive(Deserialize)]` | derive `Deserialize` 未找到 | D | 缺 serde 上下文 |
| 29 | 040 | 5 | 2807–2810 | `#[serde(other)]` | expected one of `!` or `::`, found `,` | P | 坑 3 单变体展示（"只能单元变体"） |
| 30 | 049 | 6 | 3251–3255 | `async fn call(&self, args: &str)` | `self` 仅限关联函数 | H | Tool trait 实现片段 |
| 31 | 053 | 6 | 3337–3355 | `#[test]` | attribute `async_trait` 未找到 | D | 验收测试+`FakeTool` 片段，缺 use |
| 32 | 056 | 7 | 3496–3514 | `fn schema(&self) -> Value {` | `self` 仅限关联函数 | H | schema 方法片段 |
| 33 | 069 | 8 | 4161–4176 | `#[derive(Debug, thiserror::Error)]` | crate `thiserror` 未找到 | D | PatchError 派生块 |
| 34 | 074 | 9 | 4424–4452 | `async fn call(&self, arguments: &str)` | `self` 仅限关联函数 | H | 工具 call 片段 |
| 35 | 076 | 9 | 4527–4554 | `pub struct ViewImageTool {` | `self` 仅限关联函数 | H | struct+方法片段缺 impl 承接 |
| 36 | 077 | 9 | 4567–4571 | `async fn call(&self, args: &str)` | `self` 仅限关联函数 | H | 工具 call 片段 |

## 四、到 code/ 归位的宿主建议

- `H`（13 项）：抽取时并入对应类型 `impl`。宿主线索：
  ch3 `emit`→事件源聚合；ch4 `complete`→`Llm` 实现者；ch6/ch9 工具 `call`→`Tool` trait 各实现；
  ch7 `schema`→shell 工具；ch14 `render`→渲染器；ch17 `handle`/`run_turn`/`shutdown`→server `Session`；
  ch18 `select_for_prompt`/`invoke`→MCP 工具宿主；ch21 `wait`→session agent。
- `D`（19 项）：随 crate 真实依赖（serde/thiserror/async_trait/tokio/serde_json）落地后应自动消解，作为回归基线。
- `P`（4 项）：抽取时或保留为注释示意，或补全（`run_turn`/`ScriptedTransport` 已有完整版形态）。

## 五、复跑命令（任何书稿改动后重出本表）

```bash
cd mini-codex
python3 tools/verify/syntax_check.py book/scratch/book-draft.md     # 生成 _verify_out/syntax_check.json
python3 tools/verify/classify_suspects.py book/scratch/book-draft.md # EXPECTED-FRAGMENT / REAL-SUSPECT
```
对照：REAL-SUSPECT 应为 36 项且逐项类别码不变；出现新增项即书稿新引入，需先于发布处理。
