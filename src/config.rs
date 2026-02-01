// Configuration module
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub input_dir: PathBuf,
    pub output_dir: PathBuf,
    pub server: ServerConfig,
    pub theme: ThemeConfig,
    pub pagination: PaginationConfig,
    pub comments: CommentsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub name: String,
    pub custom_css: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationConfig {
    pub posts_per_page: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentsConfig {
    pub enabled: bool,
    pub system: String,
    pub giscus: Option<GiscusCommentsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GiscusCommentsConfig {
    pub repo: String,
    pub repo_id: String,
    pub category: String,
    pub category_id: String,
    pub mapping: String,
    pub strict: String,
    pub reactions_enabled: String,
    pub emit_metadata: String,
    pub input_position: String,
    pub theme: String,
    pub lang: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            input_dir: PathBuf::from("./md"),
            output_dir: PathBuf::from("./www"),
            server: ServerConfig {
                port: 7878,
                host: "0.0.0.0".to_string(),
            },
            theme: ThemeConfig {
                name: "default".to_string(),
                custom_css: None,
            },
            pagination: PaginationConfig {
                posts_per_page: 15,
            },
            comments: CommentsConfig {
                enabled: false,
                system: "giscus".to_string(),
                giscus: None,
            },
        }
    }
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
