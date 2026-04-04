//! Model-specific cleanup of decoded summary text.

pub fn qwen35_output(text: &str) -> String {
    let mut out = text.to_string();

    loop {
        let Some(start) = out.find("<redacted_thinking>") else {
            break;
        };
        let Some(end_rel) = out[start..].find("</redacted_thinking>") else {
            break;
        };
        let end = start + end_rel + "</redacted_thinking>".len();
        out.replace_range(start..end, "");
    }

    out.trim().to_string()
}

/// LFM2.5-Thinking emits chain-of-thought inside `<|cot_start|>…<|cot_end|>` before the answer.
pub fn lfm25_output(text: &str) -> String {
    let mut out = text.to_string();

    loop {
        let Some(start) = out.find("<|cot_start|>") else {
            break;
        };
        let Some(end_rel) = out[start..].find("<|cot_end|>") else {
            break;
        };
        let end = start + end_rel + "<|cot_end|>".len();
        out.replace_range(start..end, "");
    }

    // Same markers Qwen may use; harmless if absent.
    qwen35_output(&out)
}

/// Gemma 4 may emit `<|channel>thought` before `<|channel>final` (see `gemma4.jinja`); keep the final segment.
pub fn gemma4_output(text: &str) -> String {
    let t = text.trim();
    // Template uses `<|channel>final` (one `>` after `channel`), not `<|channel|>final`.
    let final_ch = concat!("<|", "channel>final");
    let out = if let Some(i) = t.rfind(final_ch) {
        t[i + final_ch.len()..].trim().to_string()
    } else {
        t.to_string()
    };
    qwen35_output(&out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qwen_strips_redacted_thinking() {
        let got = qwen35_output("<redacted_thinking>x</redacted_thinking>\nhi");
        assert_eq!(got, "hi");
    }

    #[test]
    fn lfm_strips_cot_then_qwen_markers() {
        let got = lfm25_output("<|cot_start|>think<|cot_end|>\nSummary here");
        assert_eq!(got, "Summary here");
    }

    #[test]
    fn gemma4_keeps_final_channel() {
        let final_ch = concat!("<|", "channel>final");
        let got = gemma4_output(&format!("x {final_ch}\nThe summary."));
        assert_eq!(got, "The summary.");
    }
}
