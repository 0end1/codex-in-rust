//! mcx-cli —— 命令行入口。
//!
//! 把三个角色拆成三个独立的 tokio 任务：
//!   任务 A：引擎（Session）—— 不认识 stdin，也不认识 stdout；
//!   任务 B：渲染 —— 只消费 Event，别的什么都不管；
//!   任务 C：输入 —— 读取一行，发一个 Op。
//!
//! 模型思考时你还能敲下一句话——它会被排队，等这轮结束后立即处理。UI 永远不死。

use mcx_core::{OpenAiClient, Session};
use mcx_protocol::{Event, Op};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (op_tx, op_rx) = mpsc::channel(16);
    let (ev_tx, mut ev_rx) = mpsc::channel(128);

    // 任务 A：引擎。
    let mut session = Session::new(OpenAiClient::from_env()?, op_rx, ev_tx);
    tokio::spawn(async move { session.submission_loop().await });

    // 任务 B：渲染。
    tokio::spawn(async move {
        while let Some(ev) = ev_rx.recv().await {
            match ev {
                Event::AgentMessageDelta(s) => {
                    print!("{s}");
                    let _ = std::io::Write::flush(&mut std::io::stdout());
                }
                Event::TurnComplete { .. } => println!(),
                Event::Error(e) => eprintln!("\n[错误] {e}"),
                Event::Shutdown => break,
                _ => {}
            }
        }
    });

    // 任务 C：输入。
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    while let Some(line) = lines.next_line().await? {
        if line.trim() == "/quit" {
            op_tx.send(Op::Shutdown).await?;
            break;
        }
        op_tx.send(Op::UserInput { text: line }).await?;
    }
    Ok(())
}
