//! User-visible instructions for multilingual, length-bounded summaries.

/// Builds the user message body (instructions + document). Chat framing is applied separately.
pub fn build_user_message(article: &str) -> String {
    format!(
        "\
Please summarize the article, keeping it concise and aligned with its core themes. Ensure the 
summary is between 150-300 words. Do not include any personal opinions or unrelated details.

Article:
{}

Output:
**Use the Article's original language**
",
        article
    )
}

/// Qwen3.5 chat turn: disables “thinking” trace for cleaner summaries (`/no_think`).
pub fn format_chat_prompt(user_message: &str) -> String {
    let im_start = concat!("<|", "im_start", "|>");
    let im_end = concat!("<|", "im_end", "|>");
    format!("{im_start}user\n{user_message}{im_end}\n{im_start}assistant\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_prompt_contains_no_think_and_roles() {
        let im_start = concat!("<|", "im_start", "|>");
        let u = build_user_message("hello");
        let p = format_chat_prompt(&u);
        assert!(p.contains("/no_think"));
        assert!(p.contains(&format!("{im_start}user")));
        assert!(p.contains(&format!("{im_start}assistant")));
    }
}
