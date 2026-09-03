//! Registry：工具注册表（属于引擎侧——引擎统一分发作废了谁、允许了谁）。
//!
//! 关键设计：注册表做两件额外的事——
//! 1. 给每个注册的工具**统一起名**（`tool.name()` 为唯一真相，谁注册谁负责）；
//! 2. 在注册那一刻就生成传给模型的完整 schema 列表（引擎不关心工具实现）。

use std::collections::HashMap;
use std::sync::Arc;

use mcx_tools::Tool;

#[derive(Default)]
pub struct Registry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个工具。同名的后注册者覆盖先注册者。
    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        let name = tool.name().to_owned();
        self.tools.insert(name, Arc::new(tool));
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(name).cloned()
    }

    /// 所有工具的 schema 列表——未来发给模型做 function calling 用（第 8 章起）。
    pub fn schema_json(&self) -> serde_json::Value {
        let schemas: Vec<_> = self.tools.values().map(|t| t.schema()).collect();
        serde_json::Value::Array(schemas)
    }
}
