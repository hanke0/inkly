//! Summarizer presets: Qwen3.5, DeepSeek-R1-Distill-Qwen, Gemma 4 (Unsloth GGUF), and LFM2.5-Thinking.
//!
//! Chat framing matches Hugging Face `tokenizer_config.json` / `chat_template` token strings; the same
//! literals are captured in repo `tmp/qwen3.5.json`, `tmp/deepseek-r1.json`, `tmp/gemma4.json`, and
//! `tmp/lfm2.5.json` for reference (this crate does not read those files at runtime).

use std::fmt;
use std::str::FromStr;

/// Default system instruction when callers do not supply their own (used by [`ModelFamily::format_summary_prompt`]).
pub const SYSTEM_PROMPT: &str = "You are a summary assistant. Given an article, detect its main language. Output a short summary of the core knowledge (key facts, ideas, findings) in that same language. Keep it brief (3-5 sentences or 3-6 bullet points). Be neutral and factual. Skip fluff, opinions, examples. Output only the summary.";

/// Supported GGUF presets (Unsloth Q4_K_M where noted; LFM from LiquidAI).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Model {
    // qwen3.5 family
    #[default]
    Qwen35_0_8B,
    Qwen35_2B,
    Qwen35_4B,
    Qwen35_9B,
    Qwen35_27B,
    Qwen35_35B,

    // deepseek family
    DeepSeekR1_1_5B,
    DeepSeekR1_7B,
    DeepSeekR1_14B,
    DeepSeekR1_32B,

    // gemma4 family
    #[allow(non_camel_case_types)]
    Gemma4_2B,
    #[allow(non_camel_case_types)]
    Gemma4_4B,
    Gemma4_26B,
    Gemma4_31B,

    // lfm2.5 family
    Lfm25_1_2B,
}

/// Chat template line shared by Hugging Face presets in the same model family (prompt framing, sampling, postprocess).
#[derive(Clone, Copy, Debug)]
enum Family {
    Qwen35,
    DeepSeekR1,
    Gemma4,
    Lfm25,
}

/// One row per [`ModelFamily`] variant: Hugging Face repo / GGUF file, CLI id (`Display` / [`FromStr`]), and template line.
macro_rules! model_preset_dispatch {
    (
        $(
            $variant:ident => $family:ident {
                repo: $repo:literal,
                file: $file:literal,
                display: $display:literal,
                parse: [ $($parse:literal),+ $(,)? ],
            }
        )*
    ) => {
        impl Model {
            /// Every supported preset, in stable order (for CLI listing and tests).
            pub const ALL: &[Model] = &[$( Model::$variant, )*];

            const fn family(self) -> Family {
                match self {
                    $( Model::$variant => Family::$family, )*
                }
            }

            pub fn gguf_repo(self) -> &'static str {
                match self {
                    $( Model::$variant => $repo, )*
                }
            }

            pub fn gguf_filename(self) -> &'static str {
                match self {
                    $( Model::$variant => $file, )*
                }
            }
        }

        impl fmt::Display for Model {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                match self {
                    $( Model::$variant => f.write_str($display), )*
                }
            }
        }

        impl FromStr for Model {
            type Err = String;

            /// Parses the canonical id from [`fmt::Display`], ASCII case-insensitive.
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.trim().to_ascii_lowercase().as_str() {
                    $( $( $parse => Ok(Model::$variant), )+ )*
                    other => {
                        let list = Model::ALL
                            .iter()
                            .map(|m| m.to_string())
                            .collect::<Vec<_>>()
                            .join(", ");
                        Err(format!(
                            "unknown model `{other}`; supported models (ids are ASCII case-insensitive): {list}"
                        ))
                    }
                }
            }
        }
    };
}

model_preset_dispatch! {
    Qwen35_0_8B => Qwen35 {
        repo: "unsloth/Qwen3.5-0.8B-GGUF",
        file: "Qwen3.5-0.8B-Q4_K_M.gguf",
        display: "qwen3.5:0.8b",
        parse: ["qwen3.5:0.8b"],
    }
    Qwen35_2B => Qwen35 {
        repo: "unsloth/Qwen3.5-2B-GGUF",
        file: "Qwen3.5-2B-Q4_K_M.gguf",
        display: "qwen3.5:2b",
        parse: ["qwen3.5:2b"],
    }
    Qwen35_4B => Qwen35 {
        repo: "unsloth/Qwen3.5-4B-GGUF",
        file: "Qwen3.5-4B-Q4_K_M.gguf",
        display: "qwen3.5:4b",
        parse: ["qwen3.5:4b"],
    }
    Qwen35_9B => Qwen35 {
        repo: "unsloth/Qwen3.5-9B-GGUF",
        file: "Qwen3.5-9B-Q4_K_M.gguf",
        display: "qwen3.5:9b",
        parse: ["qwen3.5:9b"],
    }
    Qwen35_27B => Qwen35 {
        repo: "unsloth/Qwen3.5-27B-GGUF",
        file: "Qwen3.5-27B-Q4_K_M.gguf",
        display: "qwen3.5:27b",
        parse: ["qwen3.5:27b"],
    }
    Qwen35_35B => Qwen35 {
        repo: "unsloth/Qwen3.5-35B-A3B-GGUF",
        file: "Qwen3.5-35B-A3B-Q4_K_M.gguf",
        display: "qwen3.5:35b",
        parse: ["qwen3.5:35b"],
    }
    DeepSeekR1_1_5B => DeepSeekR1 {
        repo: "unsloth/DeepSeek-R1-Distill-Qwen-1.5B-GGUF",
        file: "DeepSeek-R1-Distill-Qwen-1.5B-Q4_K_M.gguf",
        display: "deepseek-r1:1.5b",
        parse: ["deepseek-r1:1.5b"],
    }
    DeepSeekR1_7B => DeepSeekR1 {
        repo: "unsloth/DeepSeek-R1-Distill-Qwen-7B-GGUF",
        file: "DeepSeek-R1-Distill-Qwen-7B-Q4_K_M.gguf",
        display: "deepseek-r1:7b",
        parse: ["deepseek-r1:7b"],
    }
    DeepSeekR1_14B => DeepSeekR1 {
        repo: "unsloth/DeepSeek-R1-Distill-Qwen-14B-GGUF",
        file: "DeepSeek-R1-Distill-Qwen-14B-Q4_K_M.gguf",
        display: "deepseek-r1:14b",
        parse: ["deepseek-r1:14b"],
    }
    DeepSeekR1_32B => DeepSeekR1 {
        repo: "unsloth/DeepSeek-R1-Distill-Qwen-32B-GGUF",
        file: "DeepSeek-R1-Distill-Qwen-32B-Q4_K_M.gguf",
        display: "deepseek-r1:32b",
        parse: ["deepseek-r1:32b"],
    }
    Gemma4_2B => Gemma4 {
        repo: "unsloth/gemma-4-E2B-it-GGUF",
        file: "gemma-4-E2B-it-Q4_K_M.gguf",
        display: "gemma4:2b",
        parse: ["gemma4:2b"],
    }
    Gemma4_4B => Gemma4 {
        repo: "unsloth/gemma-4-E4B-it-GGUF",
        file: "gemma-4-E4B-it-Q4_K_M.gguf",
        display: "gemma4:4b",
        parse: ["gemma4:4b"],
    }
    Gemma4_26B => Gemma4 {
        repo: "unsloth/gemma-4-26B-A4B-it-GGUF",
        file: "gemma-4-26B-A4B-it-UD-Q4_K_M.gguf",
        display: "gemma4:26b",
        parse: ["gemma4:26b"],
    }
    Gemma4_31B => Gemma4 {
        repo: "unsloth/gemma-4-31B-it-GGUF",
        file: "gemma-4-31B-it-Q4_K_M.gguf",
        display: "gemma4:31b",
        parse: ["gemma4:31b"],
    }
    Lfm25_1_2B => Lfm25 {
        repo: "LiquidAI/LFM2.5-1.2B-Instruct-GGUF",
        file: "LFM2.5-1.2B-Instruct-Q4_K_M.gguf",
        display: "lfm2.5:1.2b",
        parse: ["lfm2.5:1.2b"],
    }
}

/// Sampling hyperparameters aligned with each model family’s typical HF / vendor defaults for chat.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RecommendedSampling {
    pub temperature: f32,
    pub top_p: Option<f32>,
    pub top_k: Option<i32>,
}

impl Model {
    pub(crate) fn recommended_sampling(self) -> RecommendedSampling {
        use Family::*;
        match self.family() {
            Qwen35 => RecommendedSampling {
                // Deterministic decoding for concise, grounded summaries (Qwen chat cards use greedy for eval-style tasks).
                temperature: 0.0,
                top_p: None,
                top_k: None,
            },
            DeepSeekR1 => RecommendedSampling {
                // Matches `generation_config` on `deepseek-ai/DeepSeek-R1-Distill-Qwen-*` (temperature / top_p).
                temperature: 0.6,
                top_p: Some(0.95),
                top_k: None,
            },
            Gemma4 => RecommendedSampling {
                // Grounded summaries; Gemma IT cards often use higher T for chat — greedy keeps index text stable.
                temperature: 0.0,
                top_p: None,
                top_k: None,
            },
            Lfm25 => RecommendedSampling {
                // Light nucleus sampling; stable summaries while allowing the thinking-trained head room.
                temperature: 0.35,
                top_p: Some(0.9),
                top_k: Some(40),
            },
        }
    }

    pub fn format_summary_prompt(self, article: &str) -> String {
        let user = format!(
            "summary article:\n{article}\n**Respond in the same language as the provided article**"
        );
        self.format_prompt(SYSTEM_PROMPT, &user)
    }

    /// System + user turns and the assistant generation header, aligned with each family’s HF chat template
    pub fn format_prompt(self, system: &str, user: &str) -> String {
        use Family::*;
        match self.family() {
            Qwen35 => {
                format!(
                    "<|im_start|>system\n{system}<|im_end|>\n\
                    <|im_start|>user\n{user}\n/nothink<|im_end|>\n\
                    <|im_start|>assistant\n\
                    <think>\n\n</think>\n\n"
                )
            }
            DeepSeekR1 => {
                format!("<｜User｜>\n{system}\n{user}\n/nothink<｜Assistant｜>")
            }
            Gemma4 => {
                // `tmp/gemma4.json` documents `bos_token` as `<bos>`; llama.cpp Gemma 4 template uses `<|bos|>` / `<|turn>`.
                format!(
                    "<start_of_turn>system\n{system}<end_of_turn>\
                    <start_of_turn>user\n{user}\n/nothink<end_of_turn>\
                    <start_of_turn>model\n"
                )
            }
            Lfm25 => {
                format!(
                    "<|startoftext|><|im_start|>system\n{system}<|im_end|>\n\
                    <|im_start|>user\n{user}\n/nothink<|im_end|>\n\
                    <|im_start|>assistant\n"
                )
            }
        }
    }

    pub(crate) fn response_parser(&self) -> ResponseParser {
        ResponseParser::new(self.family())
    }
}

pub(crate) struct ResponseParser {
    think_start_token: &'static str,
    think_end_token: &'static str,
    text: String,
}

impl ResponseParser {
    fn new(family: Family) -> Self {
        use Family::*;
        let (think_start_token, think_end_token) = match family {
            Qwen35 => ("<think>", "</think>"),
            DeepSeekR1 => ("<think>", "</think>"),
            Gemma4 => ("<|channel>", "<channel|>"),
            Lfm25 => ("<think>", "</think>"),
        };
        Self {
            think_end_token,
            think_start_token,
            text: String::new(),
        }
    }

    pub fn feed(&mut self, token: &str) {
        self.text.push_str(token);
    }

    pub fn finally(&mut self) -> String {
        let Some(start) = self.text.find(self.think_start_token) else {
            return self.text.trim().to_string();
        };
        let Some(end) = self.text[start..].find(self.think_end_token) else {
            return self.text.trim().to_string();
        };
        self.text[end+self.think_end_token.len()..].trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_unknown_includes_full_model_list() {
        let err = "not-a-model".parse::<Model>().unwrap_err();
        assert!(err.contains("qwen3.5:0.8b"), "{err}");
        assert!(err.contains("lfm2.5:1.2b"), "{err}");
        assert!(err.contains("not-a-model"), "{err}");
    }

    #[test]
    fn from_str_matches_display_case_insensitive() {
        assert_eq!("LFM2.5:1.2B".parse::<Model>().unwrap(), Model::Lfm25_1_2B);
        assert_eq!("Qwen3.5:2b".parse::<Model>().unwrap(), Model::Qwen35_2B);
    }

    #[test]
    fn display_roundtrips_via_from_str() {
        for &m in Model::ALL {
            let s = m.to_string();
            let parsed: Model = s.parse().expect("roundtrip");
            assert_eq!(parsed, m, "{s}");
        }
    }

    #[test]
    fn response_parser_qwen_plain_text_two_feeds_keeps_output() {
        let mut p = Model::Qwen35_2B.response_parser();
        p.feed("This prefix is long enough.");
        p.feed(" Rest.");
        let text = p.finally();
        assert_eq!(text, "This prefix is long enough. Rest.");
    }

    #[test]
    fn response_parser_qwen_strips_think_block() {
        let mut p = Model::Qwen35_2B.response_parser();
        p.feed("<think>work");
        p.feed("ing</think>Answer.");
        p.feed("");
        let text = p.finally();
        assert_eq!(text, "Answer.");
    }

    #[test]
    fn response_parser_finally_clears_when_stuck_in_expect_think_start() {
        let mut p = Model::Qwen35_2B.response_parser();
        let chunk = "Single long chunk with no think tags.";
        p.feed(chunk);
        let text = p.finally();
        assert_eq!(text, chunk);
    }

    #[test]
    fn response_parser_first_piece_trim_increments_skip_size() {
        let mut p = Model::Qwen35_2B.response_parser();
        p.feed("  padded start. ");
        assert_eq!(p.finally(), "padded start.");
        p.feed(" More text here.");
        assert_eq!(p.finally(), "padded start.  More text here.");
    }

    #[test]
    fn response_parser_gemma_strips_channel_block() {
        let mut p = Model::Gemma4_2B.response_parser();
        p.feed("<|channel>hidden");
        p.feed("<channel|>Visible.");
        p.feed("");
        assert_eq!(p.finally(), "Visible.");
    }

    #[test]
    fn response_parser_deepseek_plain_text_without_think_close_marker() {
        let mut p = Model::DeepSeekR1_7B.response_parser();
        p.feed("Long enough prefix here.");
        p.feed(" Tail.");
        assert_eq!(p.finally(), "Long enough prefix here. Tail.");
    }

    #[test]
    fn response_parser_lfm_matches_qwen_think_tokens() {
        let mut p = Model::Lfm25_1_2B.response_parser();
        p.feed("<think>x");
        p.feed("</think>OK");
        p.feed("");
        let text = p.finally();
        assert_eq!(text, "OK");
    }
}
