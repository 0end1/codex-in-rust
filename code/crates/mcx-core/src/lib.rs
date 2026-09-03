//! mcx-core —— 核心引擎：agent 循环（submission / turn / tool loop）、会话与工具分发。
//!
//! 依赖方向：`mcx-core → mcx-protocol / mcx-tools / mcx-sandbox`，永不反向。
//! 引擎不知道界面是什么：只从 `Op` 收指令、往 `Event` 抛结果。

pub mod llm;
pub mod rollout;
pub mod session;
pub mod sse;
pub mod tools;

pub use llm::{LlmClient, LlmError, OpenAiClient, ScriptedLlm};
pub use rollout::{Record, Rollout, RolloutError};
pub use session::Session;
pub use tools::Registry;
