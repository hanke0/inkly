//! Summarizer presets: Qwen3.5, DeepSeek-R1-Distill-Qwen, Gemma 4 (Unsloth GGUF), and LFM2.5-Thinking.

use std::fmt;
use std::str::FromStr;

/// Supported GGUF presets (Unsloth Q4_K_M where noted; LFM from LiquidAI).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ModelFamily {
    #[default]
    Qwen35_0_8B,
    Qwen35_2B,
    Qwen35_4B,
    Qwen35_9B,
    Qwen35_27B,
    Qwen35_35B,
    Qwen35_122B,
    /// Unsloth GGUF; base `deepseek-ai/DeepSeek-R1-Distill-Qwen-1.5B`.
    DeepSeekR1_1_5B,
    /// Unsloth GGUF; base `deepseek-ai/DeepSeek-R1-Distill-Qwen-7B`.
    DeepSeekR1_7B,
    /// Unsloth GGUF; base `deepseek-ai/DeepSeek-R1-Distill-Qwen-14B`.
    DeepSeekR1_14B,
    /// Unsloth GGUF; base `deepseek-ai/DeepSeek-R1-Distill-Qwen-32B`.
    DeepSeekR1_32B,
    /// `unsloth/gemma-4-26B-A4B-it-GGUF` (MoE; UD-Q4_K_M).
    Gemma4_26B,
    /// `unsloth/gemma-4-31B-it-GGUF` (Q4_K_M).
    Gemma4_31B,
    /// `unsloth/gemma-4-E4B-it-GGUF` (Q4_K_M).
    #[allow(non_camel_case_types)]
    Gemma4_E4B,
    /// `unsloth/gemma-4-E2B-it-GGUF` (Q4_K_M).
    #[allow(non_camel_case_types)]
    Gemma4_E2B,
    Lfm25_1_2B,
}

/// Sampling hyperparameters aligned with each model family’s typical HF / vendor defaults for chat.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RecommendedSampling {
    pub temperature: f32,
    pub top_p: Option<f32>,
    pub top_k: Option<i32>,
}

impl ModelFamily {
    pub(crate) fn recommended_sampling(self) -> RecommendedSampling {
        match self {
            ModelFamily::Qwen35_0_8B
            | ModelFamily::Qwen35_2B
            | ModelFamily::Qwen35_4B
            | ModelFamily::Qwen35_9B
            | ModelFamily::Qwen35_27B
            | ModelFamily::Qwen35_35B
            | ModelFamily::Qwen35_122B => RecommendedSampling {
                // Deterministic decoding for concise, grounded summaries (Qwen chat cards use greedy for eval-style tasks).
                temperature: 0.0,
                top_p: None,
                top_k: None,
            },
            ModelFamily::DeepSeekR1_1_5B
            | ModelFamily::DeepSeekR1_7B
            | ModelFamily::DeepSeekR1_14B
            | ModelFamily::DeepSeekR1_32B => RecommendedSampling {
                // Matches `generation_config` on `deepseek-ai/DeepSeek-R1-Distill-Qwen-*` (temperature / top_p).
                temperature: 0.6,
                top_p: Some(0.95),
                top_k: None,
            },
            ModelFamily::Gemma4_26B
            | ModelFamily::Gemma4_31B
            | ModelFamily::Gemma4_E4B
            | ModelFamily::Gemma4_E2B => RecommendedSampling {
                // Grounded summaries; Gemma IT cards often use higher T for chat — greedy keeps index text stable.
                temperature: 0.0,
                top_p: None,
                top_k: None,
            },
            ModelFamily::Lfm25_1_2B => RecommendedSampling {
                // Light nucleus sampling; stable summaries while allowing the thinking-trained head room.
                temperature: 0.35,
                top_p: Some(0.9),
                top_k: Some(40),
            },
        }
    }

    pub const QWEN_ALL: &[ModelFamily] = &[
        ModelFamily::Qwen35_0_8B,
        ModelFamily::Qwen35_2B,
        ModelFamily::Qwen35_4B,
        ModelFamily::Qwen35_9B,
        ModelFamily::Qwen35_27B,
        ModelFamily::Qwen35_35B,
        ModelFamily::Qwen35_122B,
    ];

    pub fn gguf_repo(self) -> &'static str {
        match self {
            ModelFamily::Qwen35_0_8B => "unsloth/Qwen3.5-0.8B-GGUF",
            ModelFamily::Qwen35_2B => "unsloth/Qwen3.5-2B-GGUF",
            ModelFamily::Qwen35_4B => "unsloth/Qwen3.5-4B-GGUF",
            ModelFamily::Qwen35_9B => "unsloth/Qwen3.5-9B-GGUF",
            ModelFamily::Qwen35_27B => "unsloth/Qwen3.5-27B-GGUF",
            ModelFamily::Qwen35_35B => "unsloth/Qwen3.5-35B-A3B-GGUF",
            ModelFamily::Qwen35_122B => "unsloth/Qwen3.5-122B-A10B-GGUF",
            ModelFamily::DeepSeekR1_1_5B => "unsloth/DeepSeek-R1-Distill-Qwen-1.5B-GGUF",
            ModelFamily::DeepSeekR1_7B => "unsloth/DeepSeek-R1-Distill-Qwen-7B-GGUF",
            ModelFamily::DeepSeekR1_14B => "unsloth/DeepSeek-R1-Distill-Qwen-14B-GGUF",
            ModelFamily::DeepSeekR1_32B => "unsloth/DeepSeek-R1-Distill-Qwen-32B-GGUF",
            ModelFamily::Gemma4_26B => "unsloth/gemma-4-26B-A4B-it-GGUF",
            ModelFamily::Gemma4_31B => "unsloth/gemma-4-31B-it-GGUF",
            ModelFamily::Gemma4_E4B => "unsloth/gemma-4-E4B-it-GGUF",
            ModelFamily::Gemma4_E2B => "unsloth/gemma-4-E2B-it-GGUF",
            ModelFamily::Lfm25_1_2B => "LiquidAI/LFM2.5-1.2B-Thinking-GGUF",
        }
    }

    pub fn gguf_filename(self) -> &'static str {
        match self {
            ModelFamily::Qwen35_0_8B => "Qwen3.5-0.8B-Q4_K_M.gguf",
            ModelFamily::Qwen35_2B => "Qwen3.5-2B-Q4_K_M.gguf",
            ModelFamily::Qwen35_4B => "Qwen3.5-4B-Q4_K_M.gguf",
            ModelFamily::Qwen35_9B => "Qwen3.5-9B-Q4_K_M.gguf",
            ModelFamily::Qwen35_27B => "Qwen3.5-27B-Q4_K_M.gguf",
            ModelFamily::Qwen35_35B => "Qwen3.5-35B-A3B-Q4_K_M.gguf",
            ModelFamily::Qwen35_122B => "Qwen3.5-122B-A10B-Q4_K_M.gguf",
            ModelFamily::DeepSeekR1_1_5B => "DeepSeek-R1-Distill-Qwen-1.5B-Q4_K_M.gguf",
            ModelFamily::DeepSeekR1_7B => "DeepSeek-R1-Distill-Qwen-7B-Q4_K_M.gguf",
            ModelFamily::DeepSeekR1_14B => "DeepSeek-R1-Distill-Qwen-14B-Q4_K_M.gguf",
            ModelFamily::DeepSeekR1_32B => "DeepSeek-R1-Distill-Qwen-32B-Q4_K_M.gguf",
            ModelFamily::Gemma4_26B => "gemma-4-26B-A4B-it-UD-Q4_K_M.gguf",
            ModelFamily::Gemma4_31B => "gemma-4-31B-it-Q4_K_M.gguf",
            ModelFamily::Gemma4_E4B => "gemma-4-E4B-it-Q4_K_M.gguf",
            ModelFamily::Gemma4_E2B => "gemma-4-E2B-it-Q4_K_M.gguf",
            ModelFamily::Lfm25_1_2B => "LFM2.5-1.2B-Thinking-Q4_K_M.gguf",
        }
    }

    /// Wrap `user` turn + assistant header per model chat template (Qwen: `/no_think`; LFM: HF `chat_template.jinja`).
    pub fn format_chat_prompt(self, user_message: &str) -> String {
        let im_start = concat!("<|", "im_start", "|>");
        match self {
            ModelFamily::Qwen35_0_8B
            | ModelFamily::Qwen35_2B
            | ModelFamily::Qwen35_4B
            | ModelFamily::Qwen35_9B
            | ModelFamily::Qwen35_27B
            | ModelFamily::Qwen35_35B
            | ModelFamily::Qwen35_122B => {
                let im_end = concat!("<|", "im_end", "|>");
                format!("{im_start}user\n{user_message}/no_think{im_end}\n{im_start}assistant\n")
            }
            ModelFamily::DeepSeekR1_1_5B
            | ModelFamily::DeepSeekR1_7B
            | ModelFamily::DeepSeekR1_14B
            | ModelFamily::DeepSeekR1_32B => {
                // Matches `deepseek-ai-DeepSeek-R1-Distill-Qwen-*.jinja` in llama.cpp (HF `chat_template`).
                let bos = concat!("<|", "redacted_begin_of_sentence", "|>");
                let u = concat!("<|", "redacted_User", "|>");
                let a = concat!("<|", "redacted_Assistant", "|>");
                format!("{bos}{u}{user_message}{a} \n")
            }
            ModelFamily::Gemma4_26B
            | ModelFamily::Gemma4_31B
            | ModelFamily::Gemma4_E4B
            | ModelFamily::Gemma4_E2B => {
                // `models/templates/gemma4.jinja` in llama.cpp (HF `chat_template`); `bos_token` matches Gemma 4 tokenizer.
                let bos = concat!("<|", "bos", "|>");
                let turn_user = concat!("<|", "turn>user\n");
                let turn_model = concat!("<|", "turn>model\n");
                let ch_thought = concat!("<|", "channel>thought\n ");
                let u = user_message.trim();
                format!("{bos}{turn_user}{u} \n{turn_model}{ch_thought}")
            }
            ModelFamily::Lfm25_1_2B => {
                let bos = concat!("<|", "startoftext", "|>");
                let im_end = concat!("<|", "redacted_im_end", "|>");
                format!("{bos}{im_start}user\n{user_message}{im_end}\n{im_start}assistant\n")
            }
        }
    }

    /// Strip model-specific reasoning / CoT spans from decoded text.
    pub fn postprocess_output(self, text: &str) -> String {
        match self {
            ModelFamily::Qwen35_0_8B
            | ModelFamily::Qwen35_2B
            | ModelFamily::Qwen35_4B
            | ModelFamily::Qwen35_9B
            | ModelFamily::Qwen35_27B
            | ModelFamily::Qwen35_35B
            | ModelFamily::Qwen35_122B
            | ModelFamily::DeepSeekR1_1_5B
            | ModelFamily::DeepSeekR1_7B
            | ModelFamily::DeepSeekR1_14B
            | ModelFamily::DeepSeekR1_32B => crate::postprocess::qwen35_output(text),
            ModelFamily::Gemma4_26B
            | ModelFamily::Gemma4_31B
            | ModelFamily::Gemma4_E4B
            | ModelFamily::Gemma4_E2B => crate::postprocess::gemma4_output(text),
            ModelFamily::Lfm25_1_2B => crate::postprocess::lfm25_output(text),
        }
    }

    /// After each decoded piece, remove known end-of-generation fragments and return whether to stop decoding.
    pub(crate) fn strip_stream_delimiters_and_should_stop(self, text: &mut String) -> bool {
        let im_end = concat!("<|", "im_end", "|>");
        let im_end_alt = concat!("<|", "redacted_im_end", "|>");
        let ds_eos = concat!("<|", "redacted_end_of_sentence", "|>");
        let mut stop = false;
        if text.contains(im_end) || text.contains(im_end_alt) || text.contains("</s>") {
            *text = text
                .replace(im_end, "")
                .replace(im_end_alt, "")
                .replace("</s>", "");
            stop = true;
        }
        if matches!(
            self,
            ModelFamily::DeepSeekR1_1_5B
                | ModelFamily::DeepSeekR1_7B
                | ModelFamily::DeepSeekR1_14B
                | ModelFamily::DeepSeekR1_32B
        ) && text.contains(ds_eos)
        {
            *text = text.replace(ds_eos, "");
            stop = true;
        }
        if matches!(
            self,
            ModelFamily::Gemma4_26B
                | ModelFamily::Gemma4_31B
                | ModelFamily::Gemma4_E4B
                | ModelFamily::Gemma4_E2B
        ) {
            let eos = concat!("<|", "eos", "|>");
            let eot = concat!("<|", "eot", "|>");
            let turn = concat!("<|", "turn>");
            if text.contains(eos) {
                *text = text.replace(eos, "");
                stop = true;
            }
            if text.contains(eot) {
                *text = text.replace(eot, "");
                stop = true;
            }
            if let Some(i) = text.find(turn) {
                text.truncate(i);
                stop = true;
            }
        }
        stop
    }
}

impl fmt::Display for ModelFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelFamily::Qwen35_0_8B => write!(f, "qwen3.5:0.8b"),
            ModelFamily::Qwen35_2B => write!(f, "qwen3.5:2b"),
            ModelFamily::Qwen35_4B => write!(f, "qwen3.5:4b"),
            ModelFamily::Qwen35_9B => write!(f, "qwen3.5:9b"),
            ModelFamily::Qwen35_27B => write!(f, "qwen3.5:27b"),
            ModelFamily::Qwen35_35B => write!(f, "qwen3.5:35b"),
            ModelFamily::Qwen35_122B => write!(f, "qwen3.5:122b"),
            ModelFamily::DeepSeekR1_1_5B => f.write_str("deepseek-r1:1.5b"),
            ModelFamily::DeepSeekR1_7B => f.write_str("deepseek-r1:7b"),
            ModelFamily::DeepSeekR1_14B => f.write_str("deepseek-r1:14b"),
            ModelFamily::DeepSeekR1_32B => f.write_str("deepseek-r1:32b"),
            ModelFamily::Gemma4_26B => f.write_str("gemma4:26b"),
            ModelFamily::Gemma4_31B => f.write_str("gemma4:31b"),
            ModelFamily::Gemma4_E4B => f.write_str("gemma:e4b"),
            ModelFamily::Gemma4_E2B => f.write_str("gemmae2b"),
            ModelFamily::Lfm25_1_2B => f.write_str("lfm2.5:1.2b"),
        }
    }
}

impl FromStr for ModelFamily {
    type Err = String;

    /// Parses the canonical id from [`fmt::Display`], ASCII case-insensitive.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const HINT: &str = "use the id from `Display`, e.g. qwen3.5:0.8b, deepseek-r1:7b, gemma4:26b, gemmae2b, lfm2.5:1.2b";
        match s.trim().to_ascii_lowercase().as_str() {
            "qwen3.5:0.8b" => Ok(ModelFamily::Qwen35_0_8B),
            "qwen3.5:2b" => Ok(ModelFamily::Qwen35_2B),
            "qwen3.5:4b" => Ok(ModelFamily::Qwen35_4B),
            "qwen3.5:9b" => Ok(ModelFamily::Qwen35_9B),
            "qwen3.5:27b" => Ok(ModelFamily::Qwen35_27B),
            "qwen3.5:35b" => Ok(ModelFamily::Qwen35_35B),
            "qwen3.5:122b" => Ok(ModelFamily::Qwen35_122B),
            "deepseek-r1:1.5b" => Ok(ModelFamily::DeepSeekR1_1_5B),
            "deepseek-r1:7b" => Ok(ModelFamily::DeepSeekR1_7B),
            "deepseek-r1:14b" => Ok(ModelFamily::DeepSeekR1_14B),
            "deepseek-r1:32b" => Ok(ModelFamily::DeepSeekR1_32B),
            "gemma4:26b" => Ok(ModelFamily::Gemma4_26B),
            "gemma4:31b" => Ok(ModelFamily::Gemma4_31B),
            "gemma:e4b" => Ok(ModelFamily::Gemma4_E4B),
            "gemmae2b" => Ok(ModelFamily::Gemma4_E2B),
            "lfm2.5:1.2b" => Ok(ModelFamily::Lfm25_1_2B),
            other => Err(format!("unknown model `{other}`; {HINT}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::build_user_message;

    #[test]
    fn from_str_matches_display_case_insensitive() {
        assert_eq!(
            "LFM2.5:1.2B".parse::<ModelFamily>().unwrap(),
            ModelFamily::Lfm25_1_2B
        );
        assert_eq!(
            "Qwen3.5:2b".parse::<ModelFamily>().unwrap(),
            ModelFamily::Qwen35_2B
        );
    }

    #[test]
    fn display_roundtrips_via_from_str() {
        let all = [
            ModelFamily::Qwen35_0_8B,
            ModelFamily::Qwen35_2B,
            ModelFamily::Qwen35_4B,
            ModelFamily::Qwen35_9B,
            ModelFamily::Qwen35_27B,
            ModelFamily::Qwen35_35B,
            ModelFamily::Qwen35_122B,
            ModelFamily::DeepSeekR1_1_5B,
            ModelFamily::DeepSeekR1_7B,
            ModelFamily::DeepSeekR1_14B,
            ModelFamily::DeepSeekR1_32B,
            ModelFamily::Gemma4_26B,
            ModelFamily::Gemma4_31B,
            ModelFamily::Gemma4_E4B,
            ModelFamily::Gemma4_E2B,
            ModelFamily::Lfm25_1_2B,
        ];
        for m in all {
            let s = m.to_string();
            let parsed: ModelFamily = s.parse().expect("roundtrip");
            assert_eq!(parsed, m, "{s}");
        }
    }

    #[test]
    fn deepseek_chat_prompt_matches_llama_cpp_template() {
        let bos = concat!("<|", "redacted_begin_of_sentence", "|>");
        let u = concat!("<|", "redacted_User", "|>");
        let a = concat!("<|", "redacted_Assistant", "|>");
        let body = build_user_message("hi");
        let p = ModelFamily::DeepSeekR1_7B.format_chat_prompt(&body);
        assert!(p.starts_with(bos));
        assert!(p.contains(u));
        assert!(p.contains(&body));
        assert!(p.ends_with(&format!("{a} \n")));
    }

    #[test]
    fn qwen_chat_prompt_contains_no_think_and_roles() {
        let im_start = concat!("<|", "im_start", "|>");
        let u = build_user_message("hello");
        let p = ModelFamily::Qwen35_0_8B.format_chat_prompt(&u);
        assert!(p.contains("/no_think"));
        assert!(p.contains(&format!("{im_start}user")));
        assert!(p.contains(&format!("{im_start}assistant")));
    }

    #[test]
    fn lfm_chat_prompt_matches_hf_template() {
        let bos = concat!("<|", "startoftext", "|>");
        let im_start = concat!("<|", "im_start", "|>");
        let im_end = concat!("<|", "redacted_im_end", "|>");
        let u = build_user_message("hello");
        let p = ModelFamily::Lfm25_1_2B.format_chat_prompt(&u);
        assert!(p.starts_with(bos));
        assert!(p.contains(&format!("{im_start}user\n{u}{im_end}")));
        assert!(p.ends_with(&format!("{im_start}assistant\n")));
        assert!(!p.contains("/no_think"));
    }

    #[test]
    fn gemma4_chat_prompt_matches_llama_cpp_template() {
        let bos = concat!("<|", "bos", "|>");
        let turn_user = concat!("<|", "turn>user\n");
        let turn_model = concat!("<|", "turn>model\n");
        let suffix = concat!("<|", "channel>thought\n ");
        let u = build_user_message("hello");
        let p = ModelFamily::Gemma4_E2B.format_chat_prompt(&u);
        assert!(p.starts_with(bos));
        assert!(p.contains(turn_user));
        assert!(p.contains(turn_model));
        assert!(p.ends_with(suffix));
        assert!(p.contains(u.trim()));
    }
}
