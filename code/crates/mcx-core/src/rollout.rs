//! Rollout：把每轮发生的事 append 到 `{thread_id}.jsonl`，永不重写文件。
//!
//! 它是审计与回放的基础：崩溃、评测、复现问题都靠这条不可变的事故日志。
//! 宽容性是这里的头等大事：
//! - 老 schema 的行跳过去（向前兼容，见 5.4）；
//! - 认不出的 Item 落到 `Unknown` 并记警告，但数据还在；
//! - 坏的 JSON 行只跳过，绝不把整个线程文件搞挂。

use mcx_protocol::Item;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

#[derive(Debug, thiserror::Error)]
pub enum RolloutError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON 错误: {0}")]
    Json(#[from] serde_json::Error),
}

/// 一行 JSONL 就是一条 Record：一条 Item + 它是谁、在哪一轮、什么时候发生。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Record {
    /// schema 版本。以后改了 Record 的字段就 bump，读旧行的逻辑靠它做向前兼容
    pub v: u16,
    /// unix 毫秒
    pub ts_ms: i64,
    pub thread_id: String,
    pub turn: usize,
    pub item: Item,
}

pub struct Rollout {
    dir: PathBuf,
}

impl Rollout {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn thread_path(&self, thread_id: &str) -> PathBuf {
        self.dir.join(format!("{thread_id}.jsonl"))
    }

    /// 追加若干条记录到该线程的 JSONL。文件不存在就创建。
    pub async fn append(&self, thread_id: &str, records: &[Record]) -> Result<(), RolloutError> {
        tokio::fs::create_dir_all(&self.dir).await?;
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.thread_path(thread_id))
            .await?;

        for r in records {
            let mut line = serde_json::to_vec(r)?;
            line.push(b'\n');
            file.write_all(&line).await?;
        }
        file.flush().await?;
        Ok(())
    }

    /// 读回某线程的全部记录。宽容：坏行跳过（并警告），好行照常返回。
    pub fn read_all(&self, thread_id: &str) -> Result<Vec<Record>, RolloutError> {
        let path = self.thread_path(thread_id);
        if !path.exists() {
            return Ok(Vec::new());
        }
        let content = std::fs::read_to_string(&path)?;
        let mut out = Vec::new();
        for (line_no, line) in content.lines().enumerate() {
            match serde_json::from_str::<Record>(line) {
                Ok(r) => {
                    if r.item == Item::Unknown {
                        tracing::warn!(
                            thread_id,
                            line = line_no + 1,
                            "Item 落在 Unknown：新版本工具写的数据，当前代码读不懂"
                        );
                    }
                    out.push(r);
                }
                Err(e) => {
                    tracing::warn!(thread_id, line = line_no + 1, %e, "跳过坏行");
                }
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mcx_protocol::{Message, Role};

    /// 每个测试用独立目录，避免 cargo 并行跑测试时互相踩文件。
    /// 原子计数器保证同一进程内绝对不重复——只靠时钟取 nano 在并发下可能撞车。
    fn unique_dir() -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("rollout-test-{}-{n}", std::process::id()))
    }

    fn sample_records(thread_id: &str) -> Vec<Record> {
        vec![
            Record {
                v: 1,
                ts_ms: 1_000,
                thread_id: thread_id.into(),
                turn: 1,
                item: Item::UserInput { text: "帮我读一下 a.rs".into() },
            },
            Record {
                v: 1,
                ts_ms: 2_000,
                thread_id: thread_id.into(),
                turn: 1,
                item: Item::ThreadMessage {
                    message: Message { role: Role::Assistant, content: "好的".into() },
                },
            },
        ]
    }

    #[tokio::test]
    async fn roundtrip_preserves_full_record() {
        let dir = unique_dir();
        let roll = Rollout::new(dir.clone());
        let recs = sample_records("thread-1");

        roll.append("thread-1", &recs).await.unwrap();

        let read_back = roll.read_all("thread-1").unwrap();
        assert_eq!(read_back, recs);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn older_schema_line_is_skipped() {
        let dir = unique_dir();
        let roll = Rollout::new(dir.clone());
        let thread = "thread-old";
        roll.append(thread, &sample_records(thread)).await.unwrap();

        // 追加一行「老 schema」：v 不在，结构全不对——读回时应被跳过
        let mut f = tokio::fs::OpenOptions::new()
            .append(true)
            .open(roll.thread_path(thread))
            .await
            .unwrap();
        f.write_all(b"{\"old\": true, \"whatever\": 1}\n").await.unwrap();
        f.flush().await.unwrap();

        let read_back = roll.read_all(thread).unwrap();
        // 好行原样回来，坏行被跳过，文件没被破坏
        assert_eq!(read_back, sample_records(thread));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn new_item_type_survives_as_unknown_with_warning() {
        let dir = unique_dir();
        let roll = Rollout::new(dir.clone());
        let thread = "thread-new-item";

        // 模拟「未来版本」写入的行：type 是我们不认识的新类型
        let line = r#"{"v":1,"ts_ms":9000,"thread_id":"__x__","turn":2,"item":{"type":"quantum_leap","q":42}}"#;
        let dir2 = dir.clone();
        let tid = thread.to_string();
        tokio::fs::create_dir_all(&dir2).await.unwrap();
        tokio::fs::write(roll.thread_path(&tid), format!("{line}\n")).await.unwrap();

        let read_back = roll.read_all(thread).unwrap();
        assert_eq!(read_back.len(), 1);
        // 认不出的类型没有弄丢数据，只是降级成 Unknown
        assert_eq!(read_back[0].item, Item::Unknown);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn bad_json_line_is_skipped_with_warning() {
        let dir = unique_dir();
        let roll = Rollout::new(dir.clone());
        let thread = "thread-bad";
        roll.append(thread, &sample_records(thread)).await.unwrap();

        // 半截行（比如进程写到一半被杀）
        let mut f = tokio::fs::OpenOptions::new()
            .append(true)
            .open(roll.thread_path(thread))
            .await
            .unwrap();
        f.write_all(b"{\"v\":1,\"ts_ms\":3").await.unwrap();
        f.flush().await.unwrap();

        let read_back = roll.read_all(thread).unwrap();
        assert_eq!(read_back, sample_records(thread));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_all_returns_empty_for_missing_thread() {
        let dir = unique_dir();
        let roll = Rollout::new(dir.clone());
        assert_eq!(roll.read_all("nope").unwrap(), Vec::<Record>::new());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
