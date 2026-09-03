//! FakeTool：测试用的假工具，按预设队列依次返回结果。
//! 它和 ScriptedLlm 是同一套哲学——永远即时、永远稳定、永远免费。

use std::collections::VecDeque;
use std::sync::Mutex;

use async_trait::async_trait;
use mcx_tools::{Tool, ToolOutput};
use serde_json::{json, Value};

pub struct FakeTool {
    name: String,
    replies: Mutex<VecDeque<String>>,
}

impl FakeTool {
    pub fn new(name: &str, replies: Vec<String>) -> Self {
        Self { name: name.to_string(), replies: Mutex::new(replies.into_iter().collect()) }
    }
}

#[async_trait]
impl Tool for FakeTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn schema(&self) -> Value {
        json!({ "name": self.name })
    }

    async fn call(&self, _args_json: &str) -> Result<ToolOutput, mcx_tools::ToolError> {
        let out =
            self.replies.lock().unwrap().pop_front().unwrap_or_else(|| "fake-out".to_string());
        Ok(ToolOutput { output: out, is_error: false })
    }
}
