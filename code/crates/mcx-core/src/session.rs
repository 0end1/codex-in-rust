//! Session：核心引擎。三层循环：
//!   submission_loop（会话层）→ run_turn（回合层）→ tool loop（工具层，第 6 章起）。
//! 骨架对了后面加什么都推不倒重来：工具、流式、取消、会话存储都是在这三层里做文章。

#[cfg(test)]
mod fake_tool;

use crate::llm::LlmClient;
use crate::tools::Registry;
use mcx_protocol::{Event, Item, Message, Op, Role};
use mcx_tools::ToolOutput;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

pub struct Session<C: LlmClient> {
    client: C,
    history: Vec<Message>,
    tools: Registry,
    op_rx: mpsc::Receiver<Op>,
    event_tx: mpsc::Sender<Event>,
    cancel: CancellationToken,
    turn: usize,
    /// 一个回合内最多允许连续调用多少次工具（防模型钻牛角尖）
    max_tool_rounds: usize,
}

impl<C: LlmClient> Session<C> {
    pub fn new(client: C, op_rx: mpsc::Receiver<Op>, event_tx: mpsc::Sender<Event>) -> Self {
        Self {
            client,
            history: Vec::new(),
            tools: Registry::new(),
            op_rx,
            event_tx,
            cancel: CancellationToken::new(),
            turn: 0,
            max_tool_rounds: 25,
        }
    }

    pub fn new_with_tools(
        client: C,
        op_rx: mpsc::Receiver<Op>,
        event_tx: mpsc::Sender<Event>,
        tools: Registry,
        max_tool_rounds: usize,
    ) -> Self {
        Self {
            client,
            history: Vec::new(),
            tools,
            op_rx,
            event_tx,
            cancel: CancellationToken::new(),
            turn: 0,
            max_tool_rounds,
        }
    }

    /// 第一层：会话主循环。整个会话生命期只跑这一个函数。
    pub async fn submission_loop(&mut self) {
        loop {
            // 收到 None 说明所有 Op sender 都被 drop 了 —— 界面全关了，该退了
            let Some(op) = self.op_rx.recv().await else {
                break;
            };

            match op {
                Op::UserInput { text } => self.run_turn(text).await,
                Op::Interrupt => self.cancel.cancel(),
                Op::Shutdown => {
                    self.emit(Event::Shutdown).await;
                    break;
                }
            }
        }
    }

    /// 第二层 + 第三层：处理一次用户输入。
    /// 回合内可能要多轮模型调用（模型要工具 → 执行 → 结果回填 → 再问模型）。
    async fn run_turn(&mut self, text: String) {
        self.turn += 1;
        let turn = self.turn;
        self.emit(Event::TurnBegin { turn }).await;

        self.history.push(Message { role: Role::User, content: text });

        let mut call_seq = 0usize;

        // 第三层：tool loop。最多 self.max_tool_rounds 轮。
        for round in 0..self.max_tool_rounds {
            // 起一个转发任务：模型来的每个增量都立刻送向界面。
            let (delta_tx, mut delta_rx) = mpsc::channel::<String>(64);
            let ev_tx = self.event_tx.clone();
            let forward = tokio::spawn(async move {
                while let Some(delta) = delta_rx.recv().await {
                    let _ = ev_tx.send(Event::AgentMessageDelta(delta)).await;
                }
            });

            let result = self.client.complete(&self.history, &delta_tx).await;

            // 让转发任务把缓冲里剩下的增量发完，再继续——保证事件有序。
            drop(delta_tx);
            let _ = forward.await;

            let reply = match result {
                Ok(r) => r,
                Err(e) => {
                    // 关键：出错不终止会话。用户应该还能继续说话。
                    self.emit(Event::Error(e.to_string())).await;
                    break;
                }
            };
            self.history.push(Message { role: Role::Assistant, content: reply.clone() });

            // 模型这轮还想调工具吗？
            let calls = parse_tool_calls(&reply, call_seq);
            call_seq += calls.len();
            if calls.is_empty() {
                break;
            }
            let is_last_round = round + 1 >= self.max_tool_rounds;

            for call in &calls {
                // 先通知界面：要执行一次工具调用了
                self.emit(Event::ToolCallRecord {
                    turn,
                    call_id: call.call_id.clone(),
                    name: call.name.clone(),
                })
                .await;

                let output = self.execute_call(call).await;
                // 工具结果回填 history：下一轮模型就能「看见自己刚才干了什么」
                self.history.push(item_to_message(&Item::ToolResult {
                    call_id: call.call_id.clone(),
                    output: output.output,
                    is_error: output.is_error,
                }));
            }

            if is_last_round {
                self.history.push(Message {
                    role: Role::Assistant,
                    content: "工具循环达到上限，本轮提前停止。".to_string(),
                });
                break;
            }
        }

        let text = last_text(&self.history);
        self.emit(Event::TurnComplete { turn, text }).await;
    }

    /// 执行一次工具调用：统一分发，引擎不关心工具具体是谁。
    async fn execute_call(&self, call: &ToolCall) -> ToolOutput {
        match self.tools.get(&call.name) {
            Some(tool) => match tool.call(&call.arguments).await {
                Ok(out) => out,
                Err(e) => ToolOutput { output: format!("工具错误: {e}"), is_error: true },
            },
            None => ToolOutput { output: format!("未知工具: {}", call.name), is_error: true },
        }
    }

    /// 上报一个事件。发送失败不算错误 —— 界面关了而已。
    async fn emit(&self, ev: Event) {
        let _ = self.event_tx.send(ev).await;
    }
}

/// 一次待执行的工具调用。
#[derive(Debug, Clone)]
struct ToolCall {
    call_id: String,
    name: String,
    arguments: String,
}

/// 从模型回复里抠出工具调用。
///
/// 第 6 章的文本协议：`[TOOL] name(json参数)` 一行一个调用（真实模型的
/// function calling 解析在第 8 章接入，接口不变）。参数用 JSON 字符串表达。
fn parse_tool_calls(reply: &str, seq_base: usize) -> Vec<ToolCall> {
    let mut calls = Vec::new();
    for raw in reply.lines() {
        let line = raw.trim();
        let Some(rest) = line.strip_prefix("[TOOL]") else {
            continue;
        };
        let rest = rest.trim();
        let name = rest.split(['(', '{', ' ']).next().unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        let open = rest.find('{');
        let close = rest.rfind('}');
        let arguments = match (open, close) {
            (Some(o), Some(c)) if c > o => rest[o..=c].to_string(),
            _ => "{}".to_string(),
        };
        calls.push(ToolCall {
            call_id: format!("call_{}", seq_base + calls.len()),
            name: name.to_string(),
            arguments,
        });
    }
    calls
}

/// 把一条 Item 变成可以塞进 history 的 Message（工具结果回填靠它）。
fn item_to_message(item: &Item) -> Message {
    match item {
        Item::ToolResult { call_id, output, is_error } => Message {
            role: Role::User,
            content: format!(
                "[工具结果] {call_id} {}\n{output}",
                if *is_error { "(错误)" } else { "(成功)" },
            ),
        },
        Item::UserInput { text } => Message { role: Role::User, content: text.clone() },
        Item::ThreadMessage { message } => message.clone(),
        other => Message { role: Role::Assistant, content: format!("{other:?}") },
    }
}

/// 取 history 里最后一句助手的话——它就是这一轮「完整回答」的结束语。
fn last_text(history: &[Message]) -> String {
    history
        .iter()
        .rev()
        .find(|m| m.role == Role::Assistant)
        .map(|m| m.content.clone())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::ScriptedLlm;
    use crate::session::fake_tool::FakeTool;
    use mcx_tools::ReadFileTool;
    use std::path::PathBuf;
    use std::sync::Arc;

    /// 两个 turn 按顺序被处理（不联网、不花钱、30ms 内跑完）。
    #[tokio::test]
    async fn two_turns_are_processed_in_order() {
        let (op_tx, op_rx) = mpsc::channel(8);
        let (ev_tx, mut ev_rx) = mpsc::channel(64);
        let mut session =
            Session::new(ScriptedLlm::new(vec!["A".into(), "B".into()]), op_rx, ev_tx);

        let handle = tokio::spawn(async move {
            session.submission_loop().await;
            session
        });

        op_tx.send(Op::UserInput { text: "1".into() }).await.unwrap();
        op_tx.send(Op::UserInput { text: "2".into() }).await.unwrap();
        op_tx.send(Op::Shutdown).await.unwrap();

        let mut events = Vec::new();
        while let Some(ev) = ev_rx.recv().await {
            let done = matches!(ev, Event::Shutdown);
            events.push(ev);
            if done {
                break;
            }
        }
        let session = handle.await.unwrap();

        let replies: Vec<_> = events
            .iter()
            .filter_map(|e| match e {
                Event::TurnComplete { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(replies, vec!["A".to_string(), "B".to_string()]);

        // 两轮都进了 history，并且顺序是对的
        let assistant: Vec<_> = session
            .history
            .iter()
            .filter(|m| m.role == Role::Assistant)
            .map(|m| m.content.as_str())
            .collect();
        assert_eq!(assistant, vec!["A", "B"]);
    }

    /// tool loop：模型要一次 read_file → 引擎执行 → 结果回填 → 模型给结论。
    #[tokio::test]
    async fn tool_loop_executes_call_and_feeds_result_back() {
        let mut registry = Registry::new();
        registry.register(ReadFileTool::new(Arc::new(PathBuf::from("/tmp"))));

        let (op_tx, op_rx) = mpsc::channel(8);
        let (ev_tx, _ev_rx) = mpsc::channel(64);
        let mut session = Session::new_with_tools(
            ScriptedLlm::new(vec![
                r#"[TOOL] read_file({"path":"a.rs"})"#.into(),
                "读完了，a.rs 很小。".into(),
            ]),
            op_rx,
            ev_tx,
            registry,
            25,
        );

        let handle = tokio::spawn(async move {
            session.submission_loop().await;
            session
        });

        op_tx.send(Op::UserInput { text: "读 a.rs".into() }).await.unwrap();
        op_tx.send(Op::Shutdown).await.unwrap();

        let session = handle.await.unwrap();

        // 工具结果回到了 history（模型第二轮的完整输入里带着它）
        let all: String =
            session.history.iter().map(|m| m.content.clone()).collect::<Vec<_>>().join("\n");
        assert!(all.contains("(read a.rs)"), "工具结果没回填:\n{all}");
        assert!(all.contains("读完了"), "第二轮结论缺失:\n{all}");
    }

    /// 换一套工具集，引擎循环一行不改——这就是面向接口编程的红利。
    #[tokio::test]
    async fn adding_a_tool_requires_no_change_to_agent_loop() {
        let mut registry = Registry::new();
        registry.register(FakeTool::new("alpha", vec![]));
        registry.register(FakeTool::new("beta", vec![]));
        assert_eq!(registry.schema_json().as_array().unwrap().len(), 2);

        let (op_tx, op_rx) = mpsc::channel(8);
        let (ev_tx, _ev_rx) = mpsc::channel(64);
        let mut session = Session::new_with_tools(
            ScriptedLlm::new(vec!["你好，我能帮忙。".into()]),
            op_rx,
            ev_tx,
            registry,
            25,
        );

        let handle = tokio::spawn(async move {
            session.submission_loop().await;
            session
        });

        op_tx.send(Op::UserInput { text: "打招呼".into() }).await.unwrap();
        op_tx.send(Op::Shutdown).await.unwrap();

        let session = handle.await.unwrap();
        assert!(last_text(&session.history).contains("你好"));
    }
}
