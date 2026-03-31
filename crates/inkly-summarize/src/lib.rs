//! Local article summarization with `llama.cpp` via the `llama_cpp` Rust bindings.
//!
//! Weights are loaded as GGUF from Hugging Face (or a local path).
//! Set [`SummarizerConfig::hf_hub_cache_dir`] (e.g. `{DATA_DIR}/huggingface/hub`) so downloads and
//! subsequent loads use that cache instead of the global default (`~/.cache/huggingface/hub`).
//!
//! **Performance note:** run in release mode for practical throughput.

mod error;
mod prompt;

pub use error::SummarizeError;

use hf_hub::{
    api::sync::{Api, ApiBuilder},
    Repo, RepoType,
};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tracing::instrument;

use crate::prompt::{build_user_message, format_chat_prompt};

/// Hard cap on the number of tokens generated after prefill.
///
/// This is intentionally not configurable via API/CLI to keep decode latency bounded.
pub const INTERNAL_MAX_NEW_TOKENS: usize = 1024;

/// Configuration for model files, decoding, and safety limits.
#[derive(Debug, Clone)]
pub struct SummarizerConfig {
    /// Repo that hosts the GGUF file when `gguf_path` is `None`.
    pub gguf_repo: String,
    pub gguf_revision: String,
    pub gguf_filename: String,
    pub gguf_path: Option<PathBuf>,
    /// When set, Hugging Face downloads use this directory as the **hub** cache root
    /// (same layout as `~/.cache/huggingface/hub`: `snapshots/`, `blobs/`, …).
    /// Typical inkly path: `{DATA_DIR}/huggingface/hub`.
    pub hf_hub_cache_dir: Option<PathBuf>,
    /// When true (default), offload supported layers to GPU when backend is enabled.
    pub prefer_gpu: bool,
    /// Hard cap on document characters (Unicode scalar count) fed to the model.
    pub max_article_chars: usize,
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
            gguf_repo: "unsloth/Qwen3-0.6B-GGUF".to_string(),
            gguf_revision: "main".to_string(),
            gguf_filename: "Qwen3-0.6B-Q4_K_M.gguf".to_string(),
            gguf_path: None,
            hf_hub_cache_dir: None,
            prefer_gpu: true,
            max_article_chars: 3_072,
            temperature: 0.0,
            top_p: None,
            top_k: None,
            repeat_penalty: 1.0,
            repeat_last_n: 64,
            seed: 42,
        }
    }
}

pub struct Summarizer {
    backend: LlamaBackend,
    model: LlamaModel,
    config: SummarizerConfig,
}

/// Timing breakdown for one [`Summarizer::summarize_benchmark`] run.
#[derive(Debug, Clone)]
pub struct SummarizeBenchmark {
    /// Tokens in `token_ids` passed to the first forward (prompt).
    pub prompt_tokens: usize,
    /// Total predicted tokens including the first token from prefill.
    pub generated_tokens: usize,
    /// Tokens after the first prefill step (decode loop only).
    pub decode_phase_tokens: usize,
    pub prefill: Duration,
    pub decode: Duration,
}

impl SummarizeBenchmark {
    /// Decode-phase tokens per second (excludes prefill forward; typical “generation speed”).
    pub fn decode_tokens_per_sec(&self) -> f64 {
        let s = self.decode.as_secs_f64();
        if s <= 0.0 {
            return 0.0;
        }
        self.decode_phase_tokens as f64 / s
    }

    /// All `generated_tokens` over prefill + decode wall time.
    pub fn overall_tokens_per_sec(&self) -> f64 {
        let total = self.prefill + self.decode;
        let s = total.as_secs_f64();
        if s <= 0.0 {
            return 0.0;
        }
        self.generated_tokens as f64 / s
    }
}

impl Summarizer {
    /// Download (if needed) and load GGUF weights plus tokenizer.
    #[instrument(skip_all, fields(
        gguf_repo = %config.gguf_repo,
        gguf_file = %config.gguf_filename,
    ))]
    pub fn load(config: SummarizerConfig) -> Result<Self, SummarizeError> {
        let gguf_path = resolve_gguf_path(&config)?;
        let mut backend = LlamaBackend::init().map_err(|e| SummarizeError::Llama(e.to_string()))?;
        backend.void_logs();

        tracing::info!(gguf_path = %gguf_path.display(), "loading GGUF");
        let mut params = LlamaModelParams::default();
        if !config.prefer_gpu {
            params = params.with_n_gpu_layers(0);
        }
        let model = LlamaModel::load_from_file(&backend, &gguf_path, &params).map_err(|e| SummarizeError::GgufLoad {
                path: gguf_path.clone(),
                message: e.to_string(),
            })?;

        Ok(Self {
            backend,
            model,
            config,
        })
    }

    /// Summarize `article` in the same language, targeting ~200 CJK characters or ~35–45 English words.
    #[instrument(skip_all, fields(article_len = article.len()))]
    pub fn summarize(&mut self, article: &str) -> Result<String, SummarizeError> {
        Ok(self.summarize_internal(article, false)?.0)
    }

    /// Same as [`summarize`](Self::summarize) but returns timing for prompt (prefill) vs decode.
    #[instrument(skip_all, fields(article_len = article.len()))]
    pub fn summarize_benchmark(&mut self, article: &str) -> Result<(String, SummarizeBenchmark), SummarizeError> {
        let (text, bench) = self.summarize_internal(article, true)?;
        let bench = bench.ok_or(SummarizeError::Internal)?;
        Ok((text, bench))
    }

    fn summarize_internal(
        &mut self,
        article: &str,
        with_benchmark: bool,
    ) -> Result<(String, Option<SummarizeBenchmark>), SummarizeError> {
        if article.trim().is_empty() {
            return Err(SummarizeError::EmptyArticle);
        }
        let (body, truncated) = clamp_chars(article, self.config.max_article_chars);
        let user = build_user_message(&body, truncated);
        let prompt = format_chat_prompt(&user);
        let prompt_tokens = self
            .model
            .str_to_token(&prompt, AddBos::Never)
            .map_err(|e| SummarizeError::Llama(e.to_string()))?;
        let prompt_token_count = prompt_tokens.len();

        // n_batch must be >= the number of tokens submitted in a single decode call.
        // n_ctx must cover prompt + all generated tokens so the KV cache is large enough.
        let n_batch = (prompt_token_count as u32).max(512);
        let n_ctx = std::num::NonZeroU32::new(
            (prompt_token_count as u32 + INTERNAL_MAX_NEW_TOKENS as u32).max(n_batch),
        );
        let mut ctx_params = LlamaContextParams::default()
            .with_n_batch(n_batch)
            .with_n_ctx(n_ctx);
        if !self.config.prefer_gpu {
            ctx_params = ctx_params.with_offload_kqv(false).with_op_offload(false);
        }
        let mut ctx = self
            .model
            .new_context(&self.backend, ctx_params)
            .map_err(|e| SummarizeError::Llama(e.to_string()))?;

        let prefill_start = Instant::now();
        let mut batch = LlamaBatch::new(prompt_tokens.len().max(1), 1);
        batch
            .add_sequence(&prompt_tokens, 0, false)
            .map_err(|e| SummarizeError::Llama(e.to_string()))?;
        ctx.decode(&mut batch)
            .map_err(|e| SummarizeError::Llama(e.to_string()))?;
        let prefill = prefill_start.elapsed();

        let mut text = String::new();
        let mut generated_tokens = 0usize;
        let decode_start = Instant::now();
        let mut sampler = build_sampler(&self.config);
        sampler.accept_many(prompt_tokens.iter());
        let mut pos = prompt_token_count as u32;
        for _ in 0..INTERNAL_MAX_NEW_TOKENS {
            let token = sampler.sample(&ctx, -1);
            if self.model.is_eog_token(token) {
                break;
            }
            let bytes = self
                .model
                .token_to_piece_bytes(token, 32, true, None)
                .map_err(|e| SummarizeError::Llama(e.to_string()))?;
            let piece = String::from_utf8_lossy(&bytes).into_owned();
            text.push_str(&piece);
            generated_tokens += 1;
            if text.contains("<|im_end|>") || text.contains("</s>") {
                break;
            }
            let mut step = LlamaBatch::new(1, 1);
            step.add(token, pos as i32, &[0], true)
                .map_err(|e| SummarizeError::Llama(e.to_string()))?;
            ctx.decode(&mut step)
                .map_err(|e| SummarizeError::Llama(e.to_string()))?;
            sampler.accept(token);
            pos = pos.saturating_add(1);
        }
        let decode = decode_start.elapsed();
        let decode_phase_tokens = generated_tokens;

        let bench = with_benchmark.then_some(SummarizeBenchmark {
            prompt_tokens: prompt_token_count,
            generated_tokens,
            decode_phase_tokens,
            prefill,
            decode,
        });

        Ok((strip_think_sections(&text), bench))
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

fn clamp_chars(s: &str, max: usize) -> (String, bool) {
    let n = s.chars().count();
    if n <= max {
        return (s.to_string(), false);
    }
    (s.chars().take(max).collect(), true)
}

fn build_sampler(config: &SummarizerConfig) -> LlamaSampler {
    if config.temperature <= 0.0 {
        return LlamaSampler::greedy();
    }

    let mut samplers = vec![LlamaSampler::temp(config.temperature as f32)];
    if let Some(k) = config.top_k {
        samplers.push(LlamaSampler::top_k(k as i32));
    }
    if let Some(p) = config.top_p {
        samplers.push(LlamaSampler::top_p(p as f32, 1));
    }
    samplers.push(LlamaSampler::penalties(
        config.repeat_last_n as i32,
        config.repeat_penalty,
        0.0,
        0.0,
    ));
    samplers.push(LlamaSampler::dist(config.seed as u32));
    LlamaSampler::chain_simple(samplers)
}

fn strip_think_sections(text: &str) -> String {
    let mut out = text.to_string();

    // Some Qwen variants may still emit explicit reasoning blocks despite `/no_think`.
    // Remove all complete `<think>...</think>` spans so persisted summaries stay clean.
    loop {
        let Some(start) = out.find("<think>") else {
            break;
        };
        let Some(end_rel) = out[start..].find("</think>") else {
            break;
        };
        let end = start + end_rel + "</think>".len();
        out.replace_range(start..end, "");
    }

    out.trim().to_string()
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

    #[test]
    fn strip_think_sections_removes_reasoning_block() {
        let got = strip_think_sections("<think>hidden steps</think>\nfinal answer");
        assert_eq!(got, "final answer");
    }

    #[test]
    fn strip_think_sections_keeps_plain_text() {
        let got = strip_think_sections("plain summary");
        assert_eq!(got, "plain summary");
    }
}
