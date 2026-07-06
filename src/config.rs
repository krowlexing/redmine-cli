use std::env;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

const MUTABLE_ENV: &str = "REDMINE_ALLOW_MUTATIONS";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileConfig {
    pub url: Option<String>,
    pub api_key: Option<String>,
    pub default_project: Option<String>,
    #[serde(default)]
    pub mutable: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub url: String,
    pub api_key: String,
    pub default_project: Option<String>,
    pub format: Format,
    pub mutable: bool,
    pub source: ConfigSource,
}

#[derive(Debug, Clone)]
pub struct ConfigSource {
    pub url_from: &'static str,
    pub key_from: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum, Default)]
#[value(rename_all = "lower")]
pub enum Format {
    #[default]
    Tab,
    Pretty,
    Json,
}

impl Format {
    pub fn as_str(self) -> &'static str {
        match self {
            Format::Tab => "tab",
            Format::Pretty => "pretty",
            Format::Json => "json",
        }
    }
}

impl Config {
    pub fn resolve(
        url_flag: Option<&str>,
        key_flag: Option<&str>,
        project_flag: Option<&str>,
        format_flag: Option<Format>,
    ) -> Result<Config> {
        let file = load_file()?;

        let (url, url_from) = pick(
            url_flag.map(|s| s.to_string()),
            env::var("REDMINE_URL").ok(),
            file.url.clone(),
            "--url",
            "REDMINE_URL",
            "config.url",
        )
        .ok_or_else(|| Error::Config("missing Redmine URL".into()))?;

        let (api_key, key_from) = pick(
            key_flag.map(|s| s.to_string()),
            env::var("REDMINE_API_KEY").ok(),
            file.api_key.clone(),
            "--key",
            "REDMINE_API_KEY",
            "config.api_key",
        )
        .ok_or_else(|| Error::Config("missing Redmine API key".into()))?;

        let default_project = project_flag
            .map(|s| s.to_string())
            .or(env::var("REDMINE_PROJECT").ok())
            .or(file.default_project.clone());

        let format = format_flag
            .or_else(|| env::var("REDMINE_FORMAT").ok().and_then(|s| parse_format(&s)))
            .unwrap_or_default();

        let mutable = resolve_mutable(file.mutable);
        let base = url.trim_end_matches('/').to_string();

        Ok(Config {
            url: base,
            api_key,
            default_project,
            format,
            mutable,
            source: ConfigSource { url_from, key_from },
        })
    }

    pub fn ensure_mutable(&self) -> Result<()> {
        if self.mutable {
            Ok(())
        } else {
            Err(Error::Blocked)
        }
    }
}

fn resolve_mutable(file_val: Option<bool>) -> bool {
    if let Ok(raw) = env::var(MUTABLE_ENV) {
        return truthy(&raw);
    }
    file_val.unwrap_or(false)
}

fn truthy(s: &str) -> bool {
    matches!(s.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on")
}

fn parse_format(s: &str) -> Option<Format> {
    match s.to_ascii_lowercase().as_str() {
        "tab" | "tsv" => Some(Format::Tab),
        "pretty" | "table" | "md" => Some(Format::Pretty),
        "json" => Some(Format::Json),
        _ => None,
    }
}

fn pick(
    flag: Option<String>,
    env_val: Option<String>,
    file_val: Option<String>,
    flag_name: &'static str,
    env_name: &'static str,
    file_name: &'static str,
) -> Option<(String, &'static str)> {
    if let Some(v) = flag {
        return Some((v, flag_name));
    }
    if let Some(v) = env_val {
        return Some((v, env_name));
    }
    file_val.map(move |v| (v, file_name))
}

pub fn config_path() -> Result<PathBuf> {
    let proj = directories::ProjectDirs::from("dev", "redmine", "redmine-cli")
        .ok_or_else(|| Error::Config("cannot determine config directory".into()))?;
    Ok(proj.config_dir().join("config.toml"))
}

fn load_file() -> Result<FileConfig> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(FileConfig::default());
    }
    let raw = fs::read_to_string(&path)?;
    if raw.trim().is_empty() {
        return Ok(FileConfig::default());
    }
    let cfg: FileConfig = toml::from_str(&raw)?;
    Ok(cfg)
}

pub fn save_file(cfg: &FileConfig) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut raw = toml::to_string(cfg).map_err(|e| Error::Config(e.to_string()))?;
    if !raw.is_empty() {
        raw.insert_str(0, "# redmine configuration\n");
    }
    fs::write(&path, raw)?;
    Ok(())
}
