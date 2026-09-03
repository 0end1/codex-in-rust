//! mcx-tools —— 工具实现层。
//!
//! 引擎不关心具体工具是 read_file 还是 shell：它只认识 `Tool` trait。
//! 工具是模型与真实世界之间的一层薄壳，每个工具 = 一把受控的钥匙。

pub mod read_file;

pub use read_file::ReadFileTool;

use async_trait::async_trait;

/// 一次工具调用的产物。
#[derive(Debug, Clone, PartialEq)]
pub struct ToolOutput {
    /// 执行结果文本
    pub output: String,
    /// 是否算失败（失败也是输出，也要给模型看——模型需要知道自己错了）
    pub is_error: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("参数解析失败: {0}")]
    Args(#[from] serde_json::Error),
    #[error("执行失败: {0}")]
    Exec(String),
}

/// 一个工具的完整接口：三样东西，缺一不可。
#[async_trait]
pub trait Tool: Send + Sync {
    /// 唯一名字，引擎/模型都用它指认工具
    fn name(&self) -> &str;
    /// 告诉模型「我能干什么」的 JSON schema
    fn schema(&self) -> serde_json::Value;
    /// 实际执行。传进来的是模型写的 JSON 字符串参数
    async fn call(&self, args_json: &str) -> Result<ToolOutput, ToolError>;
}
