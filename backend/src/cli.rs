use std::path::PathBuf;

use clap::{Parser, Subcommand};
use inkly_summarize::{Summarizer, SummarizerConfig, INTERNAL_MAX_NEW_TOKENS};

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
    /// Start the HTTP API server (also the default when no subcommand is given).
    Serve,
    /// Load the summarizer and print token timing (prefill vs decode throughput).
    SummaryBench {
        /// Article body to summarize (default: built-in English sample).
        #[arg(long)]
        text: Option<String>,
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

pub fn data_dir() -> PathBuf {
    std::env::var("DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./data"))
}

pub fn run_summary_bench(
    text: Option<String>,
    max_article_chars: Option<usize>,
    runs: u32,
    cpu: bool,
    hf_cache: Option<PathBuf>,
) -> Result<(), String> {
    let data_root = data_dir();
    let cache = hf_cache.unwrap_or_else(|| data_root.join("huggingface").join("hub"));
    std::fs::create_dir_all(&cache).map_err(|e| format!("create hf cache dir: {e}"))?;

    let mut cfg = SummarizerConfig {
        hf_hub_cache_dir: Some(cache),
        prefer_gpu: !cpu,
        ..SummarizerConfig::default()
    };
    if let Some(n) = max_article_chars {
        cfg.max_article_chars = n.max(256);
    }

    eprintln!(
        "Bench config: max_article_chars={} max_new_tokens={}",
        cfg.max_article_chars,
        INTERNAL_MAX_NEW_TOKENS
    );
    eprintln!("Loading summarizer (first run may download weights)...");
    let mut summarizer = Summarizer::load(cfg).map_err(|e| e.to_string())?;

    let article = text.unwrap_or_else(|| DEFAULT_BENCH_TEXT.to_string());
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

        println!(
            "run {}: prompt_tokens={} generated_tokens={} decode_phase_tokens={} \
             prefill_ms={:.1} decode_ms={:.1} decode_tok/s={:.2} overall_tok/s={:.2}",
            i + 1,
            b.prompt_tokens,
            b.generated_tokens,
            b.decode_phase_tokens,
            b.prefill.as_secs_f64() * 1_000.0,
            b.decode.as_secs_f64() * 1_000.0,
            b.decode_tokens_per_sec(),
            b.overall_tokens_per_sec(),
        );

        if i == 0 {
            let preview: String = summary.chars().take(160).collect();
            println!("  summary_preview: {preview}");
        }
    }

    let n = f64::from(runs);
    let prompt_tok_avg = (prompt_tokens_sum as f64 / n).round() as u64;
    println!("--- average over {runs} run(s) ---");
    println!(
        "  prompt_tokens≈{prompt_tok_avg} prefill_ms={:.1} decode_ms={:.1} decode_tok/s={:.2} overall_tok/s={:.2}",
        prefill_ms / n,
        decode_ms / n,
        decode_tps / n,
        overall_tps / n,
    );

    Ok(())
}
