//! User-visible instructions for multilingual, length-bounded summaries.

/// Builds the user message body (instructions + document). Chat framing is applied via [`super::ModelFamily::format_chat_prompt`].
pub fn build_user_message(article: &str) -> String {
    format!(
        "\
Please summarize the article, keeping it concise and aligned with its core themes. Ensure the 
summary is between 150-300 words. Do not include any personal opinions or unrelated details.
**Output the document summary directly; strictly no reasoning, <think> tags, or introductory remarks.**

Article:
{}

Output:
**Use the Article's original language**
",
        article
    )
}
