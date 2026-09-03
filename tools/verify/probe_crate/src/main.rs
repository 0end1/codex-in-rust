// 自动生成：由《用Rust造一个Codex》修订稿抽取的错误/策略枚举的真实编译验证。
#![allow(dead_code, unused)]
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::HashMap;
use std::path::PathBuf;

// 占位：仅用于让 LlmError::Network(#[from] X) 无需外部 crate 也能编译
// （真实 reqwest::Error 实现了 Debug + Display + std::error::Error）
mod reqwest_stub {
    #[derive(Debug)]
    pub struct Error;
    impl std::fmt::Display for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { Ok(()) }
    }
    impl std::error::Error for Error {}
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

#[derive(Debug, thiserror::Error)]
pub enum HookError {
    #[error("hook 执行超时: {command}")]
    Timeout { command: String },
    #[error("hook 被用户取消")]
    Cancelled,
    #[error("hook 底层进程错误: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone)]
pub enum HookVerdict {
    Continue,
    Block(String), // 仅 PreToolUse 有效
}

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

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("网络错误: {0}")]
    Network(#[from] reqwest_stub::Error),
    #[error("响应格式无法解析: {0}")]
    Malformed(String),
    #[error("请求被取消（收到取消信号，见第 17 章）")]
    Cancelled,
}

// LlmError 的 Network 字段在正文引用 reqwest::Error；本 probe 用同名 stub 顶替以离线编译
fn main() {}
