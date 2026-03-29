//! Local article summarization with **Qwen3.5-2B** via [Candle](https://github.com/huggingface/candle).
//!
//! Weights are loaded as **GGUF** (same stack as Candle’s `quantized-qwen3` example), because the
//! upstream `Qwen/Qwen3.5-2B` safetensors build uses a hybrid text stack that does not match
//! Candle’s dense `qwen3` loader. Default GGUF: `unsloth/Qwen3.5-2B-GGUF` + `Qwen/Qwen3.5-2B`
//! tokenizer.
//!
//! Set [`SummarizerConfig::hf_hub_cache_dir`] (e.g. `{DATA_DIR}/huggingface/hub`) so downloads and
//! subsequent loads use that cache instead of the global default (`~/.cache/huggingface/hub`).

mod device;
mod error;
mod prompt;
mod token_output_stream;

pub use error::SummarizeError;

use candle_core::quantized::gguf_file;
use candle_core::Tensor;
use candle_transformers::generation::{LogitsProcessor, Sampling};
use candle_transformers::models::quantized_qwen3::ModelWeights as Qwen3Gguf;
use hf_hub::{
    api::sync::{Api, ApiBuilder},
    Repo, RepoType,
};
use std::path::PathBuf;
use tokenizers::Tokenizer;
use tracing::instrument;

use crate::device::pick_device;
use crate::prompt::{build_user_message, format_chat_prompt};
use crate::token_output_stream::TokenOutputStream;

/// Configuration for model files, decoding, and safety limits.
#[derive(Debug, Clone)]
pub struct SummarizerConfig {
    /// Hugging Face model repo used to download `tokenizer.json` when `tokenizer_path` is `None`.
    pub tokenizer_repo: String,
    /// Repo that hosts the GGUF file when `gguf_path` is `None`.
    pub gguf_repo: String,
    pub gguf_revision: String,
    pub gguf_filename: String,
    pub gguf_path: Option<PathBuf>,
    pub tokenizer_path: Option<PathBuf>,
    /// When set, Hugging Face downloads use this directory as the **hub** cache root
    /// (same layout as `~/.cache/huggingface/hub`: `snapshots/`, `blobs/`, …).
    /// Typical inkly path: `{DATA_DIR}/huggingface/hub`.
    pub hf_hub_cache_dir: Option<PathBuf>,
    /// When true (default), prefer CUDA (Linux/Windows) or Metal (macOS) if available; otherwise use CPU.
    pub prefer_gpu: bool,
    /// Hard cap on document characters (Unicode scalar count) fed to the model.
    pub max_article_chars: usize,
    pub max_new_tokens: usize,
    pub temperature: f64,
    pub top_p: Option<f64>,
    pub top_k: Option<usize>,
    pub repeat_penalty: f32,
    pub repeat_last_n: usize,
    pub seed: u64,
}

impl Default for SummarizerConfig {
    fn default() -> Self {
        Self {
            tokenizer_repo: "Qwen/Qwen3-1.7B".to_string(),
            gguf_repo: "unsloth/Qwen3-1.7B-GGUF".to_string(),
            gguf_revision: "main".to_string(),
            gguf_filename: "Qwen3-1.7B-Q4_K_M.gguf".to_string(),
            gguf_path: None,
            tokenizer_path: None,
            hf_hub_cache_dir: None,
            prefer_gpu: true,
            max_article_chars: 12_000,
            max_new_tokens: 512,
            temperature: 0.35,
            top_p: Some(0.9),
            top_k: None,
            repeat_penalty: 1.12,
            repeat_last_n: 64,
            seed: 42,
        }
    }
}

pub struct Summarizer {
    model: Qwen3Gguf,
    tokenizer: Tokenizer,
    device: candle_core::Device,
    config: SummarizerConfig,
}

impl Summarizer {
    /// Download (if needed) and load GGUF weights plus tokenizer.
    #[instrument(skip_all, fields(
        tokenizer_repo = %config.tokenizer_repo,
        gguf_repo = %config.gguf_repo,
        gguf_file = %config.gguf_filename,
    ))]
    pub fn load(config: SummarizerConfig) -> Result<Self, SummarizeError> {
        let device = pick_device(!config.prefer_gpu)?;
        let gguf_path = resolve_gguf_path(&config)?;
        let tokenizer_path = resolve_tokenizer_path(&config)?;

        tracing::info!(gguf_path = %gguf_path.display(), "loading GGUF");
        let mut file = std::fs::File::open(&gguf_path)?;
        let content = gguf_file::Content::read(&mut file)
            .map_err(|e| SummarizeError::GgufLoad {
                path: gguf_path.clone(),
                message: e.to_string(),
            })?;
        let model = Qwen3Gguf::from_gguf(content, &mut file, &device).map_err(|e| {
            SummarizeError::GgufLoad {
                path: gguf_path.clone(),
                message: e.to_string(),
            }
        })?;

        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| SummarizeError::Tokenizer(e.to_string()))?;

        Ok(Self {
            model,
            tokenizer,
            device,
            config,
        })
    }

    /// Summarize `article` in the same language, targeting ~200 CJK characters or ~35–45 English words.
    #[instrument(skip_all, fields(article_len = article.len()))]
    pub fn summarize(&mut self, article: &str) -> Result<String, SummarizeError> {
        if article.trim().is_empty() {
            return Err(SummarizeError::EmptyArticle);
        }

        let (body, truncated) = clamp_chars(article, self.config.max_article_chars);
        let user = build_user_message(&body, truncated);
        let prompt = format_chat_prompt(&user);

        let mut tos = TokenOutputStream::new(self.tokenizer.clone());
        let tokens = tos
            .tokenizer()
            .encode(prompt, true)
            .map_err(|e| SummarizeError::Tokenizer(e.to_string()))?;
        let token_ids = tokens.get_ids();

        let sampling = build_sampling(
            self.config.temperature,
            self.config.top_k,
            self.config.top_p,
        );
        let mut logits_processor =
            LogitsProcessor::from_sampling(self.config.seed, sampling);

        let mut all_tokens: Vec<u32> = Vec::new();

        let mut next_token = {
            let input = Tensor::new(token_ids, &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input, 0)?;
            let logits = logits.squeeze(0)?;
            logits_processor.sample(&logits)?
        };
        all_tokens.push(next_token);

        let eos_id = eos_token_id(tos.tokenizer())?;

        let mut text = String::new();
        if let Some(t) = tos.next_token(next_token)? {
            text.push_str(&t);
        }

        let max_new = self.config.max_new_tokens;
        for index in 0..max_new {
            if next_token == eos_id {
                break;
            }
            let input = Tensor::new(&[next_token], &self.device)?.unsqueeze(0)?;
            let logits = self
                .model
                .forward(&input, token_ids.len().saturating_add(index))?;
            let logits = logits.squeeze(0)?;
            let logits = if self.config.repeat_penalty == 1. {
                logits
            } else {
                let start_at = all_tokens
                    .len()
                    .saturating_sub(self.config.repeat_last_n);
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    self.config.repeat_penalty,
                    &all_tokens[start_at..],
                )?
            };
            next_token = logits_processor.sample(&logits)?;
            all_tokens.push(next_token);
            if let Some(t) = tos.next_token(next_token)? {
                text.push_str(&t);
            }
            if next_token == eos_id {
                break;
            }
        }
        if let Some(rest) = tos.decode_rest().map_err(SummarizeError::Candle)? {
            text.push_str(&rest);
        }

        Ok(text.trim().to_string())
    }
}

fn hub_api(config: &SummarizerConfig) -> Result<Api, SummarizeError> {
    if let Some(dir) = &config.hf_hub_cache_dir {
        return ApiBuilder::new()
            .with_cache_dir(dir.clone())
            .build()
            .map_err(SummarizeError::from);
    }
    Api::new().map_err(SummarizeError::from)
}

fn resolve_gguf_path(config: &SummarizerConfig) -> Result<PathBuf, SummarizeError> {
    if let Some(p) = &config.gguf_path {
        return Ok(p.clone());
    }
    let api = hub_api(config)?;
    let path = api
        .repo(Repo::with_revision(
            config.gguf_repo.clone(),
            RepoType::Model,
            config.gguf_revision.clone(),
        ))
        .get(&config.gguf_filename)?;
    Ok(path)
}

fn resolve_tokenizer_path(config: &SummarizerConfig) -> Result<PathBuf, SummarizeError> {
    if let Some(p) = &config.tokenizer_path {
        return Ok(p.clone());
    }
    let api = hub_api(config)?;
    let api = api.model(config.tokenizer_repo.clone());
    let path = api.get("tokenizer.json")?;
    Ok(path)
}

fn clamp_chars(s: &str, max: usize) -> (String, bool) {
    let n = s.chars().count();
    if n <= max {
        return (s.to_string(), false);
    }
    (s.chars().take(max).collect(), true)
}

fn build_sampling(temperature: f64, top_k: Option<usize>, top_p: Option<f64>) -> Sampling {
    if temperature <= 0. {
        return Sampling::ArgMax;
    }
    match (top_k, top_p) {
        (None, None) => Sampling::All { temperature },
        (Some(k), None) => Sampling::TopK { k, temperature },
        (None, Some(p)) => Sampling::TopP { p, temperature },
        (Some(k), Some(p)) => Sampling::TopKThenTopP { k, p, temperature },
    }
}

fn eos_token_id(tokenizer: &Tokenizer) -> Result<u32, SummarizeError> {
    let im_end = concat!("<|", "im_end", "|>");
    let endoftext = concat!("<|", "endoftext", "|>");
    if let Some(id) = tokenizer.token_to_id(im_end) {
        return Ok(id);
    }
    if let Some(id) = tokenizer.token_to_id(endoftext) {
        return Ok(id);
    }
    Err(SummarizeError::MissingSpecialToken {
        token: "im_end or endoftext".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_chars_truncates() {
        let (s, t) = clamp_chars("abcde", 3);
        assert!(t);
        assert_eq!(s, "abc");
    }
}
