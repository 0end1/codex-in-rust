//! mcx-protocol —— 依赖图最底层的公共词汇表（不依赖任何 workspace 内 crate）。
//!
//! 第 3 章引入全书最核心的一对抽象：
//! - `Op`（下行）：客户端发给引擎的指令；
//! - `Event`（上行）：引擎报给界面的事件。
//!
//! `Event` 是 `Clone + PartialEq` 而 `Op` 不是——因为 Event 需要被测试和回放，
//! 这个 derive 差异本身就是设计意图的表达（第 20 章评测会 `assert_eq!(events, ...)`）。

use serde::{Deserialize, Serialize};

mod thread;

pub use thread::{Item, Thread, Turn};

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
    /// 引擎决定执行一次工具调用（第 6 章起）
    ToolCallRecord { turn: usize, call_id: String, name: String },
    /// 一轮结束，附完整文本
    TurnComplete { turn: usize, text: String },
    /// 出错了，但会话还能继续
    Error(String),
    /// 引擎已退出
    Shutdown,
}
