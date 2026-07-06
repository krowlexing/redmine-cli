use crate::cli::ConfigCommand;
use crate::config::{self, FileConfig};
use crate::error::{Error, Result};

pub fn run(cmd: ConfigCommand) -> Result<()> {
    match cmd {
        ConfigCommand::Set {
            url,
            key,
            default_project,
        } => {
            let mut existing = load_existing()?;
            if let Some(u) = url {
                existing.url = Some(u);
            }
            if let Some(k) = key {
                existing.api_key = Some(k);
            }
            if let Some(p) = default_project {
                existing.default_project = Some(p);
            }
            config::save_file(&existing)?;
            println!("saved\t{}", config::config_path()?.display());
            Ok(())
        }
        ConfigCommand::Show => {
            let cfg = resolve_for_show()?;
            println!("url\t{}", cfg.url);
            println!("api_key\t{}", mask(&cfg.api_key));
            println!("default_project\t{}", cfg.default_project.unwrap_or_default());
            println!("format\t{}", cfg.format.as_str());
            println!("mutable\t{}", cfg.mutable);
            println!("url_source\t{}", cfg.source.url_from);
            println!("api_key_source\t{}", cfg.source.key_from);
            Ok(())
        }
        ConfigCommand::Path => {
            println!("{}", config::config_path()?.display());
            Ok(())
        }
    }
}

fn load_existing() -> Result<FileConfig> {
    let path = config::config_path()?;
    if !path.exists() {
        return Ok(FileConfig::default());
    }
    let raw = std::fs::read_to_string(&path)?;
    if raw.trim().is_empty() {
        return Ok(FileConfig::default());
    }
    Ok(toml::from_str(&raw)?)
}

fn resolve_for_show() -> Result<crate::config::Config> {
    config::Config::resolve(None, None, None, None)
        .map_err(|e| Error::Config(format!("could not resolve config (run 'config set'): {e}")))
}

fn mask(s: &str) -> String {
    if s.len() <= 6 {
        return "***".into();
    }
    let (a, b) = s.split_at(3);
    let tail_len = b.len().min(3);
    let tail = &b[b.len() - tail_len..];
    format!("{a}...{tail} ({})", s.len())
}
