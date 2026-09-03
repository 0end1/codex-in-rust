//! 第 1 章《Agent = Model + Harness》的随书代码（单包形态）。
//!
//! 能力：
//! 1. 用 Responses API 让模型“说第一句话”；
//! 2. 把「约束」从一段文本变成 Rust 类型 `SafetyRule`（1.3 节），
//!    用 `apply_rules` 折叠进系统提示，全程只输出代码可测试的纯函数。
//!
//! 运行：`cargo run -- "你是一个 Rust 专家。"`（需 `OPENAI_API_KEY`）。

use serde_json::{json, Value};

/// 一条必须施加给模型的硬约束。
struct SafetyRule {
    /// 对人类和模型都可读的要求描述
    requirement: &'static str,
    /// 无法满足时，模型必须输出这句话，而不是硬着头皮做
    refusal: &'static str,
}

/// 把一组规则折叠进基础指令，产出最终的系统提示。
fn apply_rules(base: &str, rules: &[SafetyRule]) -> String {
    let mut prompt = String::from(base);
    for rule in rules {
        prompt.push_str(&format!(
            "\n- {}。若无法满足，只输出「{}」，不要编造。",
            rule.requirement, rule.refusal
        ));
    }
    prompt
}

/// 第 1.3 节的默认规则组：对应正文“实验三”那一组约束。
fn default_rules() -> Vec<SafetyRule> {
    vec![
        SafetyRule {
            requirement: "用 chrono crate，不引入其他依赖",
            refusal: "无法安全改进",
        },
        SafetyRule {
            requirement: "解析失败返回 Err，不得 panic",
            refusal: "无法安全改进",
        },
        SafetyRule {
            requirement: "只输出代码，不输出解释",
            refusal: "无法安全改进",
        },
    ]
}

/// 从 Responses API 的返回里捞出助手的回复文本。
fn extract_text(response: &Value) -> Option<String> {
    let output = response.get("output")?.as_array()?;
    for item in output {
        if item.get("type")?.as_str()? != "message" {
            continue;
        }
        for part in item.get("content")?.as_array()? {
            if part.get("type")?.as_str()? == "output_text" {
                return part.get("text")?.as_str().map(str::to_string);
            }
        }
    }
    None
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("OPENAI_API_KEY")?;

    // 第二个命令行参数是我们的“系统指令”入口。
    let base = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "你是一个 Rust 专家。".to_string());
    let prompt = apply_rules(&base, &default_rules());

    let body = json!({
        "model": "gpt-4o-mini",
        "instructions": prompt,
        "input": "帮我写一个函数，把两个日期之间相差的天数算出来。"
    });

    let response: Value = reqwest::Client::new()
        .post("https://api.openai.com/v1/responses")
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await?
        .json()
        .await?;

    match extract_text(&response) {
        Some(text) => println!("{text}"),
        None => println!(
            "没解析出文本：\n{}",
            serde_json::to_string_pretty(&response)?
        ),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_rules_appends_each_rule() {
        let rules = default_rules();
        let out = apply_rules("你是一个 Rust 专家。", &rules);
        for r in &rules {
            assert!(out.contains(r.requirement));
            assert!(out.contains(r.refusal));
        }
    }

    #[test]
    fn apply_rules_keeps_base_first() {
        let out = apply_rules("base", &default_rules());
        assert!(out.starts_with("base"));
    }
}
