use std::fmt;
use std::path::PathBuf;

use inkly_summarize::Model;

#[derive(Clone)]
pub struct Config {
    pub host: String,
    pub data_dir: PathBuf,
    pub username: String,
    pub password: String,
    pub max_body_bytes: usize,
    /// When true, allow any origin (set `CORS_ORIGINS=*`). Otherwise use `cors_origins`.
    pub cors_permissive: bool,
    pub cors_origins: Vec<String>,
    /// When true, load the local LLM and populate the `summary` field on index routes. Default off.
    pub summarize_enabled: bool,
    /// Which GGUF preset to load. Default: Qwen3.5 0.8B.
    /// `SUMMARIZE_MODEL`: canonical id (see `inkly_summarize::ModelFamily` `Display`), e.g. `qwen3.5:0.8b`, `deepseek-r1:7b`, `gemma4:26b`, `gemmae2b`, `lfm2.5:1.2b`.
    pub summarize_model: Model,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("host", &self.host)
            .field("data_dir", &self.data_dir)
            .field("username", &self.username)
            .field("password", &"[REDACTED]")
            .field("max_body_bytes", &self.max_body_bytes)
            .field("cors_permissive", &self.cors_permissive)
            .field("cors_origins", &self.cors_origins)
            .field("summarize_enabled", &self.summarize_enabled)
            .field("summarize_model", &self.summarize_model)
            .finish()
    }
}

pub fn data_dir() -> PathBuf {
    std::env::var("DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./data"))
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

        let data_dir = data_dir();

        let username = std::env::var("USERNAME").map_err(|_| "Missing USERNAME".to_string())?;

        let password = std::env::var("PASSWORD").map_err(|_| "Missing PASSWORD".to_string())?;

        let max_body_bytes = std::env::var("MAX_BODY_BYTES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1024 * 1024); // 1MiB

        let summarize_enabled = std::env::var("SUMMARIZE_ENABLED")
            .ok()
            .map(|v| {
                matches!(
                    v.trim().to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(false);

        let (cors_permissive, cors_origins) = match std::env::var("CORS_ORIGINS") {
            Ok(raw) if raw.trim() == "*" => (true, Vec::new()),
            Ok(raw) => {
                let origins: Vec<String> = raw
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if origins.is_empty() {
                    return Err(
                        "CORS_ORIGINS must be * or a non-empty comma-separated origin list"
                            .to_string(),
                    );
                }
                (false, origins)
            }
            Err(_) => (
                false,
                vec![
                    "http://127.0.0.1:5173".to_string(),
                    "http://localhost:5173".to_string(),
                ],
            ),
        };

        let summarize_model: Model = match std::env::var("SUMMARIZE_MODEL") {
            Ok(raw) => raw.parse().map_err(|e: String| e)?,
            Err(_) => Model::default(),
        };

        Ok(Self {
            host,
            data_dir,
            username,
            password,
            max_body_bytes,
            cors_permissive,
            cors_origins,
            summarize_enabled,
            summarize_model,
        })
    }
}
