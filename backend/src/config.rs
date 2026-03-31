use std::path::PathBuf;

use inkly_summarize::ModelSize;

#[derive(Clone, Debug)]
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
    /// Which Qwen3.5 parameter size to load. Default: 0.8B.
    /// Configured via `SUMMARIZE_MODEL` env var (e.g. `0.8b`, `2b`, `4b`, `9b`, `27b`, `35b`, `122b`).
    pub summarize_model: ModelSize,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

        let data_dir = std::env::var("DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./data"));

        let username = std::env::var("USERNAME")
            .map_err(|_| "Missing USERNAME".to_string())?;

        let password = std::env::var("PASSWORD")
            .map_err(|_| "Missing PASSWORD".to_string())?;

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

        let summarize_model: ModelSize = match std::env::var("SUMMARIZE_MODEL") {
            Ok(raw) => raw.parse().map_err(|e: String| e)?,
            Err(_) => ModelSize::default(),
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
