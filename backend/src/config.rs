use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub bind_addr: String,
    pub tantivy_dir: PathBuf,
    pub jwt_secret: String,
    pub max_body_bytes: usize,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        let bind_addr = std::env::var("INKLY_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());

        let tantivy_dir = std::env::var("INKLY_TANTIVY_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./data/tantivy"));

        let jwt_secret = std::env::var("INKLY_JWT_SECRET")
            .map_err(|_| "Missing INKLY_JWT_SECRET".to_string())?;

        let max_body_bytes = std::env::var("INKLY_MAX_BODY_BYTES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1024 * 1024); // 1MiB

        Ok(Self {
            bind_addr,
            tantivy_dir,
            jwt_secret,
            max_body_bytes,
        })
    }
}

