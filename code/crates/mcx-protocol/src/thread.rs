//! 对话的三层结构（第 5 章）：
//!   Thread（对话线程）→ Turn（一轮，从用户输入到模型停）→ Item（这一轮里发生的事）。
//!
//! `Item` 是协议里最大的 serde 枚举：它会被存进 JSONL（Rollout），跨多个 Rust 大版本存活。
//! 未来新增任何变体都不会破坏旧数据——旧的落到 `Unknown`，读旧数据的代码也照常工作。
//! 这正是 `#[serde(tag = "type", other)]` 的设计意图，被 5.4 的向前兼容测试验证。

use serde::{Deserialize, Serialize};

use crate::Message;

/// 一轮里「发生了什么」的一条原子记录。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Item {
    /// 用户输入
    #[serde(rename = "user_input")]
    UserInput { text: String },
    /// 模型说的一句话
    #[serde(rename = "thread_message")]
    ThreadMessage { message: Message },
    /// 模型决定调用某个函数（函数名 + JSON 字符串参数）
    #[serde(rename = "function_call")]
    FunctionCall { name: String, arguments: String },
    /// 引擎即将执行的一个工具调用
    #[serde(rename = "tool_call")]
    ToolCall { call_id: String, name: String, arguments: String },
    /// 工具执行结果
    #[serde(rename = "tool_result")]
    ToolResult { call_id: String, output: String, is_error: bool },
    /// 未来版本新增的类型在这里着陆：旧代码读到它时，容忍地继续跑
    #[serde(other)]
    Unknown,
}

/// 一次「用户发话 → 模型停嘴」的完整回合。
#[derive(Debug, Clone, PartialEq)]
pub struct Turn {
    /// 在本线程里的序号（从 1 开始）
    pub index: usize,
    /// 用户这次说了什么
    pub user_input: Option<String>,
    /// 这一轮发生的事件，按时间顺序
    pub items: Vec<Item>,
}

/// 一个对话线程。
#[derive(Debug, Clone, PartialEq)]
pub struct Thread {
    pub id: String,
    /// 用于向用户展示的标题（可为空，等模型起名）
    pub title: Option<String>,
    pub turns: Vec<Turn>,
}

impl Thread {
    pub fn new(id: String) -> Self {
        Self { id, title: None, turns: Vec::new() }
    }
}
