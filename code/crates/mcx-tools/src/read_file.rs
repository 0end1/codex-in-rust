//! read_file 工具：让模型能读文件（只能读，不能写——最小权限）。
//!
//! 第 6 章先做参数解析的壳：路径参数解析好了、调用协议通了，
//! 但真实的文件系统读取留到第 9 章（那里有沙箱、路径归一化、大小上限一起上）。
//! 当前是「桩」：只返回 `(read {path})`，让整条 tool loop 能端到端跑通。

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{Tool, ToolError, ToolOutput};

#[derive(Debug, Deserialize)]
pub struct ReadArgs {
    /// 相对 cwd 的文件路径
    pub path: String,
}

pub struct ReadFileTool {
    /// 允许读取的根目录。任何路径都必须归一到它的下面（第 9 章实现）
    cwd: Arc<PathBuf>,
}

impl ReadFileTool {
    pub fn new(cwd: Arc<PathBuf>) -> Self {
        Self { cwd }
    }
}

#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn schema(&self) -> Value {
        json!({
            "name": "read_file",
            "description": "读取文本文件内容。路径相对于工作目录。",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "要读取的文件路径（相对 cwd）"
                    }
                },
                "required": ["path"]
            }
        })
    }

    async fn call(&self, args_json: &str) -> Result<ToolOutput, ToolError> {
        // 参数解析立刻做掉——错误要当场报，让模型看到并改正
        let args: ReadArgs = serde_json::from_str(args_json)?;
        // 桩实现用不到 cwd；标注“读过”避免 dead_code 误报（第 9 章起真正使用）
        let _ = &self.cwd;

        // TODO(ch09): 真实读取。在那之前，模型拿到的是「占位输出」，
        // 但它证明了：参数 → 调用 → 结果回填 history 的整条链路是通的。
        Ok(ToolOutput { output: format!("(read {})", args.path), is_error: false })
    }
}
