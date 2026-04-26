use std::path::PathBuf;

use clap::{Parser, Subcommand};
use inkly_summarize::{INTERNAL_MAX_NEW_TOKENS, Model, Summarizer, SummarizerConfig};

use crate::config;

const DEFAULT_BENCH_TEXT: &str = "\
Large language models are used for summarization, question answering, and many other text tasks. \
They are trained on broad corpora and fine-tuned for helpful, safe responses. \
Inference speed depends on hardware, batch size, and model size.";

#[derive(Parser, Debug)]
#[command(name = "inkly", version, about = "Inkly backend server and CLI tools")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Print this binary's version (from Cargo package metadata).
    Version,
    /// Start the HTTP API server (also the default when no subcommand is given).
    Serve,
    /// Print supported summarizer model ids (same values as `summary-bench --model`).
    Models,
    /// Upgrade on-disk document search storage to the current `data_version` (stop the server first).
    ///
    /// Rebuilds the documents root into the current SQLite + FTS5 layout (using the `simple`
    /// tokenizer for Chinese / pinyin search) and rewrites `version.data`, preserving
    /// `auto_increment` and all indexed documents. Legacy Tantivy trees (`index/`) are read
    /// read-only during migration and left untouched under the `*.old.<rfc3339>.backup` directory.
    Migrate {
        /// Directory containing `index/` and `version.data` (default: `$DATA_DIR/documents`, same as the server).
        #[arg(long)]
        documents_root: Option<PathBuf>,
        /// Build the new index here before swapping into `documents-root` (default: sibling `basename.migrate.<rfc3339>`).
        /// Must be empty if it already exists; must not be inside `documents-root` (or vice versa). Prefer the same filesystem as the live data for `rename`.
        #[arg(long)]
        staging_dir: Option<PathBuf>,
    },
    /// Load the summarizer and print token timing (prefill vs decode throughput).
    SummaryBench {
        /// Path to a file whose contents will be used as the article (default: built-in English sample).
        #[arg(long)]
        file: Option<PathBuf>,
        /// Summarizer preset (canonical id, same as `Display`: e.g. qwen3.5:0.8b, deepseek-r1:7b).
        #[arg(long, default_value = "qwen3.5:0.8b")]
        model: Model,
        /// Cap article length (Unicode chars); prefill cost scales roughly with the square of token count on CPU.
        #[arg(long)]
        max_article_chars: Option<usize>,
        /// Number of timed runs (default 1).
        #[arg(long, default_value_t = 1)]
        runs: u32,
        /// Prefer CPU only (no CUDA / Metal).
        #[arg(long)]
        cpu: bool,
        /// Hugging Face hub cache directory (default: `DATA_DIR/huggingface/hub`).
        #[arg(long)]
        hf_cache: Option<PathBuf>,
    },
}

pub fn run_summary_bench(
    file: Option<PathBuf>,
    model: Model,
    max_article_chars: Option<usize>,
    runs: u32,
    cpu: bool,
    hf_cache: Option<PathBuf>,
) -> Result<(), String> {
    let data_root = config::data_dir();
    let cache = hf_cache.unwrap_or_else(|| data_root.join("huggingface").join("hub"));
    std::fs::create_dir_all(&cache).map_err(|e| format!("create hf cache dir: {e}"))?;

    let mut cfg = SummarizerConfig {
        hf_hub_cache_dir: Some(cache),
        prefer_gpu: !cpu,
        ..SummarizerConfig::with_model(model)
    };
    if let Some(n) = max_article_chars {
        cfg.max_article_chars = n.max(256);
    }

    eprintln!(
        "Bench config: model={model} max_article_chars={} max_new_tokens={}",
        cfg.max_article_chars, INTERNAL_MAX_NEW_TOKENS
    );
    eprintln!("Loading summarizer (first run may download weights)...");
    let mut summarizer = Summarizer::load(cfg).map_err(|e| e.to_string())?;

    let article = match file {
        Some(p) => std::fs::read_to_string(&p).map_err(|e| format!("read {}: {e}", p.display()))?,
        None => DEFAULT_BENCH_TEXT.to_string(),
    };
    let runs = runs.max(1);

    let mut prefill_ms = 0.0f64;
    let mut decode_ms = 0.0f64;
    let mut decode_tps = 0.0f64;
    let mut overall_tps = 0.0f64;
    let mut prompt_tokens_sum = 0u64;

    for i in 0..runs {
        let (summary, b) = summarizer
            .summarize_benchmark(&article)
            .map_err(|e| e.to_string())?;

        prefill_ms += b.prefill.as_secs_f64() * 1_000.0;
        decode_ms += b.decode.as_secs_f64() * 1_000.0;
        decode_tps += b.decode_tokens_per_sec();
        overall_tps += b.overall_tokens_per_sec();
        prompt_tokens_sum += b.prompt_tokens as u64;
        if i == 0 {
            println!("  summary_preview: {summary}");
        }
        println!(
            "run {}: prompt_tokens={} generated_tokens={} decode_phase_tokens={} \
             prefill_ms={:.1} decode_ms={:.1} overall_ms={:.1} \
             decode_tokens/s={:.2} overall_tokens/s={:.2} \
             generated_text_size={} output_text_size={} think_text_size={}",
            i + 1,
            b.prompt_tokens,
            b.generated_tokens,
            b.decode_phase_tokens,
            b.prefill.as_secs_f64() * 1_000.0,
            b.decode.as_secs_f64() * 1_000.0,
            b.prefill.as_secs_f64() * 1_000.0 + b.decode.as_secs_f64() * 1_000.0,
            b.decode_tokens_per_sec(),
            b.overall_tokens_per_sec(),
            b.generated_text_size,
            b.output_text_size,
            b.generated_text_size - b.output_text_size,
        );
    }

    let n = f64::from(runs);
    let prompt_tok_avg = (prompt_tokens_sum as f64 / n).round() as u64;
    println!("--- average over {runs} run(s) ---");
    println!(
        "  prompt_tokens≈{prompt_tok_avg} prefill_ms={:.1} decode_ms={:.1} decode_tokens/s={:.2} overall_tokens/s={:.2}",
        prefill_ms / n,
        decode_ms / n,
        decode_tps / n,
        overall_tps / n,
    );

    Ok(())
}

pub fn run_list_models() {
    for m in Model::ALL {
        println!("{m}");
    }
}

pub fn run_print_version() {
    println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}
