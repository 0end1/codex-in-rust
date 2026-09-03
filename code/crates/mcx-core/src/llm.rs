//! 模型客户端抽象：引擎不该知道模型是 OpenAI 还是本地 Ollama。
//!
//! 第 4 章起 `complete` 多了一个 `delta_tx` 参数：流式增量边走边报，
//! 不再等一整段文本攒齐。引擎不吐「等好久的空白期」——每个 delta 都实时流向界面。

use crate::sse::{extract_delta, SseEvent, SseParser};
use async_trait::async_trait;
use futures_util::StreamExt;
use mcx_protocol::Message;
use serde_json::{json, Value};
use tokio::sync::mpsc;

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("网络错误: {0}")]
    Network(#[from] reqwest::Error),
    #[error("响应格式无法解析: {0}")]
    Malformed(String),
    #[error("请求被取消（收到取消信号，见第 17 章）")]
    Cancelled,
    #[error("缺少 OPENAI_API_KEY 环境变量")]
    MissingApiKey,
    #[error("SSE 流解析失败: {0}")]
    Sse(#[from] crate::sse::SseError),
}

#[async_trait]
pub trait LlmClient: Send + Sync {
    /// 发一轮完整请求。增量文本边收到边经 `delta_tx` 送出；返回值是拼接好的完整回复。
    async fn complete(
        &self,
        messages: &[Message],
        delta_tx: &mpsc::Sender<String>,
    ) -> Result<String, LlmError>;
}

/// 真实客户端：Responses API 的流式接口（`stream: true`）。
pub struct OpenAiClient {
    http: reqwest::Client,
    api_key: String,
    model: String,
}

impl OpenAiClient {
    /// 从环境变量构造：`OPENAI_API_KEY` 必填，`MCX_MODEL` 可选（默认 gpt-4o-mini）。
    pub fn from_env() -> Result<Self, LlmError> {
        let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| LlmError::MissingApiKey)?;
        let model = std::env::var("MCX_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
        Ok(Self { http: reqwest::Client::new(), api_key, model })
    }

    fn build_body(&self, messages: &[Message]) -> Value {
        let input: Vec<_> =
            messages.iter().map(|m| json!({ "role": m.role, "content": m.content })).collect();
        json!({ "model": self.model, "input": input, "stream": true })
    }
}

#[async_trait]
impl LlmClient for OpenAiClient {
    async fn complete(
        &self,
        messages: &[Message],
        delta_tx: &mpsc::Sender<String>,
    ) -> Result<String, LlmError> {
        // `bytes_stream()` 把网络响应切成任意大小的字节块，然后全部丢给 SseParser。
        // 块在哪儿切都无所谓：SseParser 按帧界重组。
        let stream = self
            .http
            .post("https://api.openai.com/v1/responses")
            .bearer_auth(&self.api_key)
            .json(&self.build_body(messages))
            .send()
            .await?
            .bytes_stream();

        let mut parser = SseParser::new();
        let mut full = String::new();

        let mut stream = Box::pin(stream);
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            for ev in parser.feed(&chunk)? {
                match ev {
                    SseEvent::Data(payload) => {
                        if let Some(delta) = extract_delta(&payload) {
                            full.push_str(&delta);
                            // 实时转发给界面；界面已关则忽略
                            let _ = delta_tx.send(delta).await;
                        }
                    }
                    SseEvent::Done => return Ok(full),
                }
            }
        }

        // 流结束但没见到 [DONE] 哨兵：按协议不完整，宁可报错也不吞
        if let Some(SseEvent::Data(payload)) = parser.finish()? {
            if let Some(delta) = extract_delta(&payload) {
                full.push_str(&delta);
                let _ = delta_tx.send(delta).await;
            }
        }
        if full.is_empty() {
            return Err(LlmError::Malformed("流结束但没有收到任何文本增量".into()));
        }
        Ok(full)
    }
}

/// 假模型：按预设队列依次回复。测试不依赖网络、不花钱、永远稳定。
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
    async fn complete(
        &self,
        _messages: &[Message],
        _delta_tx: &mpsc::Sender<String>,
    ) -> Result<String, LlmError> {
        Ok(self.replies.lock().unwrap().pop_front().unwrap_or_default())
    }
}
