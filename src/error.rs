use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

static VERBOSE: AtomicBool = AtomicBool::new(false);

pub fn set_verbose(on: bool) {
    VERBOSE.store(on, Ordering::Relaxed);
}

pub fn is_verbose() -> bool {
    VERBOSE.load(Ordering::Relaxed)
}

#[derive(Debug)]
pub enum Error {
    Config(String),
    Http { status: u16, url: String, body: String },
    Network(String),
    Decode(String),
    NotFound(String),
    Ambiguous { kind: String, name: String, matches: Vec<String> },
    Usage(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Config(m) => write!(f, "config error: {m}"),
            Error::Http { status, url, body } => {
                if body.is_empty() {
                    return write!(f, "HTTP {status} from {url}");
                }
                if is_verbose() {
                    write!(f, "HTTP {status} from {url}: {body}")
                } else {
                    let limit = 200;
                    let taken: String = body.chars().take(limit).collect();
                    let ellipsis = if body.chars().count() > limit { "..." } else { "" };
                    write!(f, "HTTP {status} from {url}: {taken}{ellipsis}")
                }
            }
            Error::Network(m) => write!(f, "network error: {m}"),
            Error::Decode(m) => write!(f, "decode error: {m}"),
            Error::NotFound(m) => write!(f, "not found: {m}"),
            Error::Ambiguous { kind, name, matches } => {
                write!(f, "ambiguous {kind} '{name}', matches: {}", matches.join(", "))
            }
            Error::Usage(m) => write!(f, "usage error: {m}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        if e.is_status() {
            Error::Http {
                status: e.status().map(|s| s.as_u16()).unwrap_or(0),
                url: e.url().map(|u| u.to_string()).unwrap_or_default(),
                body: String::new(),
            }
        } else {
            Error::Network(e.to_string())
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Decode(e.to_string())
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::Config(format!("invalid config file: {e}"))
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Config(format!("io error: {e}"))
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[allow(dead_code)]
pub fn usage<T>(m: impl Into<String>) -> Result<T> {
    Err(Error::Usage(m.into()))
}
