use serde::de::DeserializeOwned;
use std::path::Path;

use crate::config::Config;
use crate::error::{Error, Result};
use crate::models::project::CurrentUser;

pub struct RedmineClient {
    inner: reqwest::blocking::Client,
    base: String,
    api_key: String,
    verbose: bool,
}

impl RedmineClient {
    #[allow(dead_code)]
    pub fn new(config: &Config) -> Result<Self> {
        Self::with_verbose(config, false)
    }

    pub fn with_verbose(config: &Config, verbose: bool) -> Result<Self> {
        let inner = reqwest::blocking::Client::builder()
            .user_agent(concat!("redmine/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| Error::Network(e.to_string()))?;

        Ok(Self {
            inner,
            base: config.url.clone(),
            api_key: config.api_key.clone(),
            verbose,
        })
    }

    #[allow(dead_code)]
    pub fn base(&self) -> &str {
        &self.base
    }

    #[allow(dead_code)]
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn get<T: DeserializeOwned>(&self, path: &str, query: &[(&str, &str)]) -> Result<T> {
        let url = format!("{}{}", self.base, path);
        let mut req = self
            .inner
            .request(reqwest::Method::GET, &url)
            .header("X-Redmine-API-Key", &self.api_key);
        if !query.is_empty() {
            req = req.query(query);
        }
        let resp = req.send()?;
        self.decode("GET", url, resp)
    }

    pub fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.base, path);
        let resp = self
            .inner
            .request(reqwest::Method::POST, &url)
            .header("X-Redmine-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(body)
            .send()?;
        self.decode("POST", url, resp)
    }

    pub fn put<B: serde::Serialize>(&self, path: &str, body: &B) -> Result<()> {
        let url = format!("{}{}", self.base, path);
        let resp = self
            .inner
            .request(reqwest::Method::PUT, &url)
            .header("X-Redmine-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(body)
            .send()?;
        let status = resp.status();
        if status.is_success() {
            return Ok(());
        }
        let text = resp.text().unwrap_or_default();
        Err(self.http_err("PUT", url, status.as_u16(), text))
    }

    fn decode<T: DeserializeOwned>(
        &self,
        method: &str,
        url: String,
        resp: reqwest::blocking::Response,
    ) -> Result<T> {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        if !status.is_success() {
            return Err(self.http_err(method, url, status.as_u16(), text));
        }
        serde_json::from_str::<T>(&text).map_err(|e| {
            if self.verbose {
                Error::Decode(format!("{}: {}", e, text))
            } else {
                Error::Decode(e.to_string())
            }
        })
    }

    fn http_err(&self, method: &str, url: String, status: u16, body: String) -> Error {
        if self.verbose {
            Error::Http {
                status,
                url: format!("{method} {url}"),
                body,
            }
        } else {
            Error::Http { status, url, body }
        }
    }

    pub fn current_user(&self) -> Result<i64> {
        let cu: CurrentUser = self.get("/users/current.json", &[])?;
        Ok(cu.user.id)
    }

    pub fn download_attachment(&self, id: i64, output_path: &Path) -> Result<()> {
        let url = format!("{}/attachments/download/{}", self.base, id);
        let resp = self
            .inner
            .request(reqwest::Method::GET, &url)
            .header("X-Redmine-API-Key", &self.api_key)
            .send()?;

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().unwrap_or_default();
            return Err(self.http_err("GET", url, status.as_u16(), text));
        }

        let bytes = resp.bytes().map_err(|e| Error::Network(e.to_string()))?;
        let temp_path = output_path.with_extension("tmp");
        std::fs::write(&temp_path, bytes).map_err(|e| Error::Network(format!("failed to write file: {}", e)))?;
        std::fs::rename(&temp_path, output_path).map_err(|e| Error::Network(format!("failed to finalize file: {}", e)))?;
        Ok(())
    }
}
