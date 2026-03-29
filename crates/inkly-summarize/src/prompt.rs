//! User-visible instructions for multilingual, length-bounded summaries.

const SUMMARY_INSTRUCTIONS: &str =
    "Same-language one-paragraph summary (~200 words). No meta intro; answer only:\n";

/// Builds the user message body (instructions + document). Chat framing is applied separately.
pub fn build_user_message(article: &str, truncated: bool) -> String {
    let scope_note = if truncated {
        "\n(excerpt; summarize this part only.)\n"
    } else {
        ""
    };

    format!("{SUMMARY_INSTRUCTIONS}{scope_note}\n{article}")
}

/// Qwen3 chat turn: disables “thinking” trace for cleaner summaries (`/no_think`).
pub fn format_chat_prompt(user_message: &str) -> String {
    let im_start = concat!("<|", "im_start", "|>");
    let im_end = concat!("<|", "im_end", "|>");
    format!(
        "{im_start}user\n{user_message} /no_think{im_end}\n{im_start}assistant\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_prompt_contains_no_think_and_roles() {
        let im_start = concat!("<|", "im_start", "|>");
        let u = build_user_message("hello", false);
        let p = format_chat_prompt(&u);
        assert!(p.contains("/no_think"));
        assert!(p.contains(&format!("{im_start}user")));
        assert!(p.contains(&format!("{im_start}assistant")));
    }

    #[test]
    fn truncation_note_present() {
        let u = build_user_message("x", true);
        assert!(u.contains("excerpt"));
    }
}
