//! 增量 SSE（Server-Sent Events）解析器。
//!
//! 核心纪律：**在帧边界确定之前，那些字节不是文本，只是字节。**
//! 帧的分界是 `\n\n` / `\r\n\r\n`，那是 ASCII 字节，不可能与 UTF-8 多字节序列混淆，
//! 因此按字节找分隔符、按帧处理，中文（三字节 UTF-8）被网络从中间劈开也不是问题。

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SseEvent {
    /// 一帧数据（通常是 JSON）
    Data(String),
    /// 收到 `[DONE]`，流结束
    Done,
}

#[derive(Debug, thiserror::Error)]
pub enum SseError {
    #[error("帧内容不是合法 UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("缓冲区超过上限 {0} 字节，可能是对端没有正确分帧")]
    BufferOverflow(usize),
}

pub struct SseParser {
    buffer: Vec<u8>,
    max_buffer: usize,
}

impl Default for SseParser {
    fn default() -> Self {
        Self::new()
    }
}

impl SseParser {
    pub fn new() -> Self {
        Self { buffer: Vec::new(), max_buffer: 8 * 1024 * 1024 }
    }

    /// 喂进任意大小的一块字节，返回这一块里能解析出的所有完整事件。
    pub fn feed(&mut self, chunk: &[u8]) -> Result<Vec<SseEvent>, SseError> {
        self.buffer.extend_from_slice(chunk);

        let mut events = Vec::new();
        while let Some((idx, sep_len)) = find_frame_end(&self.buffer) {
            // 把 buffer 拆成 [帧内容][分隔符][剩余]
            let mut tail = self.buffer.split_off(idx); // buffer=帧内容, tail 从分隔符开始
            tail.drain(..sep_len); // 丢掉帧界分隔符（2 或 4 字节）
            let frame = std::mem::replace(&mut self.buffer, tail);

            if let Some(ev) = Self::parse_frame(&frame)? {
                events.push(ev);
            }
        }

        if self.buffer.len() > self.max_buffer {
            return Err(SseError::BufferOverflow(self.max_buffer));
        }
        Ok(events)
    }

    /// 流结束时调用，处理最后一帧（有些服务器末尾不发分隔符）。
    pub fn finish(&mut self) -> Result<Option<SseEvent>, SseError> {
        if self.buffer.is_empty() {
            return Ok(None);
        }
        let frame = std::mem::take(&mut self.buffer);
        Self::parse_frame(&frame)
    }

    fn parse_frame(frame: &[u8]) -> Result<Option<SseEvent>, SseError> {
        let text = String::from_utf8(frame.to_vec())?;

        let mut data = String::new();
        for line in text.split('\n') {
            let line = line.strip_suffix('\r').unwrap_or(line); // 处理 CRLF
            if line.is_empty() || line.starts_with(':') {
                continue; // 空行、注释（心跳）
            }
            if let Some(rest) = line.strip_prefix("data:") {
                let rest = rest.strip_prefix(' ').unwrap_or(rest); // 容忍有无空格
                if !data.is_empty() {
                    data.push('\n'); // 多行 data 用 \n 连接
                }
                data.push_str(rest);
            }
        }

        if data.is_empty() {
            return Ok(None);
        }
        if data.trim() == "[DONE]" {
            return Ok(Some(SseEvent::Done));
        }
        Ok(Some(SseEvent::Data(data)))
    }
}

/// 找帧界分隔符，返回（起点, 长度）。SSE 规范允许 `\r\n` 行尾，
/// 所以帧界可能是 `\n\n`（LF，长 2）或 `\r\n\r\n`（CRLF，长 4）。
/// CRLF 必须显式处理：`\r\n\r\n` 的字节是 `0D 0A 0D 0A`，并不包含连续两个 `\n`。
fn find_frame_end(buf: &[u8]) -> Option<(usize, usize)> {
    let lf = buf.windows(2).position(|w| w == b"\n\n").map(|i| (i, 2));
    let crlf = buf.windows(4).position(|w| w == b"\r\n\r\n").map(|i| (i, 4));
    match (lf, crlf) {
        (Some(a), Some(b)) => Some(if a.0 <= b.0 { a } else { b }),
        (a, b) => a.or(b),
    }
}

/// 从一帧 SSE 数据里取出文本增量；不是文本增量则返回 None。
/// 解析失败就当这个事件不存在，不报错（`.ok()?`）。
pub fn extract_delta(payload: &str) -> Option<String> {
    #[derive(serde::Deserialize)]
    struct DeltaEnvelope {
        #[serde(rename = "type")]
        kind: String,
        delta: Option<String>,
    }

    let env: DeltaEnvelope = serde_json::from_str(payload).ok()?;
    match env.kind.as_str() {
        "response.output_text.delta" => env.delta,
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any_split_point_yields_same_events() {
        // 故意包含：ASCII、中文（三字节 UTF-8）、[DONE]
        let stream: Vec<u8> =
            b"data: {\"x\":1}\n\ndata: \xe4\xbd\xa0\xe5\xa5\xbd\n\ndata: [DONE]\n\n".to_vec();

        let expected =
            vec![SseEvent::Data("{\"x\":1}".into()), SseEvent::Data("你好".into()), SseEvent::Done];

        // 在每一个可能的字节位置把它切成两半，结果必须一致
        for split_at in 0..=stream.len() {
            let mut p = SseParser::new();
            let mut got = p.feed(&stream[..split_at]).unwrap();
            got.extend(p.feed(&stream[split_at..]).unwrap());
            assert_eq!(got, expected, "在字节 {split_at} 处切分时结果不一致");
        }
    }

    #[test]
    fn one_byte_at_a_time() {
        let stream = b"data: hi\n\ndata: [DONE]\n\n";
        let mut p = SseParser::new();
        let mut got = Vec::new();
        for b in stream {
            got.extend(p.feed(&[*b]).unwrap());
        }
        assert_eq!(got, vec![SseEvent::Data("hi".into()), SseEvent::Done]);
    }

    #[test]
    fn handles_crlf_and_multiline_data() {
        // CRLF 帧界 `\r\n\r\n` 长 4 字节且不含 `\n\n`——必须被显式切出，不能指望 finish() 兜底
        let mut p = SseParser::new();
        let got = p.feed(b"data: {\"a\":1}\r\ndata:   \"b\":2\r\n\r\n").unwrap();
        assert_eq!(got, vec![SseEvent::Data("{\"a\":1}\n  \"b\":2".into())]);
    }

    #[test]
    fn crlf_frame_boundary_split_across_chunks() {
        // `\r` 与后面的 `\n\r\n` 分属两个 feed 块时也必须找到帧界；
        // 否则纯 CRLF 事件会一直积压到 finish() 才被处理，CRLF 用例即失败
        let stream = b"data: hi\r\n\r\ndata: [DONE]\r\n\r\n";
        for split in 1..stream.len() {
            let mut p = SseParser::new();
            let mut got = p.feed(&stream[..split]).unwrap();
            got.extend(p.feed(&stream[split..]).unwrap());
            assert_eq!(
                got,
                vec![SseEvent::Data("hi".into()), SseEvent::Done],
                "在字节 {split} 处切分时 CRLF 帧界被破坏"
            );
        }
    }

    #[test]
    fn detects_done_sentinel() {
        let mut p = SseParser::new();
        let got = p.feed(b"data: [DONE]\n\n").unwrap();
        assert_eq!(got, vec![SseEvent::Done]);
    }

    #[test]
    fn oversized_buffer_is_rejected() {
        let mut p = SseParser::new();
        p.max_buffer = 16; // 测试里调小上限
        assert!(matches!(p.feed(&[b'x'; 1024]), Err(SseError::BufferOverflow(_))));
    }

    #[test]
    fn extracts_only_text_delta() {
        assert_eq!(
            extract_delta(r#"{"type":"response.output_text.delta","delta":"你"}"#),
            Some("你".into())
        );
        // 不认识的事件类型 → None（宽容）
        assert_eq!(extract_delta(r#"{"type":"response.reasoning.delta","delta":"想"}"#), None);
        // 畸形 JSON → None（宽容）
        assert_eq!(extract_delta("{not json"), None);
    }
}
