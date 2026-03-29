//! User-visible instructions for multilingual, length-bounded summaries.

const INSTRUCTIONS: &str = r"You are a careful editorial summarizer.

Task: Read the document below and write exactly one concise summary for a reader who has not seen the full text.

Language: Write only in the same language as the document. If the document mixes languages, follow the dominant language. Do not translate.

Length: Target about 200 characters for Chinese, Japanese, or Korean text; for Latin-script languages (e.g. English), target about 35–45 words. Stay within roughly ±20% of that budget.

Content: Capture the main topic, the 2–4 most important facts or claims, and—only if clearly present—the outcome, recommendation, or implication. Skip side anecdotes unless they carry the central idea.

Style: Neutral, informative prose. Do not open with meta phrases like “This article”, “The author”, “The document”, or “In summary”. Prefer one tight paragraph. Use bullet points only if the source itself is a compact list.

Output: Return only the summary. No title, headings, labels, or explanation.";

/// Builds the user message body (instructions + document). Chat framing is applied separately.
pub fn build_user_message(article: &str, truncated: bool) -> String {
    let scope_note = if truncated {
        "\n\nNote: The text below is an excerpt from a longer document; summarize this excerpt faithfully.\n"
    } else {
        ""
    };

    format!("{INSTRUCTIONS}{scope_note}\n--- Document ---\n\n{article}")
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
