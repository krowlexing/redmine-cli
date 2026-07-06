use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use redmine_cli::client::RedmineClient;
use redmine_cli::config::{Config, ConfigSource, Format};

pub struct MockServer {
    pub url: String,
    requests: Arc<Mutex<Vec<RecordedRequest>>>,
    mocks: Arc<Mutex<VecDeque<Mock>>>,
    _handle: thread::JoinHandle<()>,
}

#[derive(Clone)]
pub struct RecordedRequest {
    pub method: String,
    pub path: String,
    pub body: String,
    pub api_key: Option<String>,
}

#[derive(Clone)]
struct Mock {
    method: String,
    path_contains: String,
    status: u16,
    body: String,
}

pub struct MockBuilder<'a> {
    server: &'a MockServer,
    last: Option<Mock>,
}

impl MockServer {
    pub fn start() -> MockServer {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("addr");
        let url = format!("http://{addr}");
        let requests: Arc<Mutex<Vec<RecordedRequest>>> = Arc::new(Mutex::new(Vec::new()));
        let mocks: Arc<Mutex<VecDeque<Mock>>> = Arc::new(Mutex::new(VecDeque::new()));

        let reqs = requests.clone();
        let mks = mocks.clone();
        let handle = thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { break };
                let _ = handle_one(stream, reqs.clone(), mks.clone());
            }
        });

        MockServer { url, requests, mocks, _handle: handle }
    }

    pub fn mock(&self) -> MockBuilder<'_> {
        MockBuilder { server: self, last: None }
    }

    pub fn requests(&self) -> Vec<RecordedRequest> {
        self.requests.lock().unwrap().clone()
    }
}

impl<'a> MockBuilder<'a> {
    pub fn get(mut self, path_contains: &str) -> Self {
        self.last = Some(Mock {
            method: "GET".into(), path_contains: path_contains.into(),
            status: 200, body: String::new(),
        });
        self
    }
    pub fn post(mut self, path_contains: &str) -> Self {
        self.last = Some(Mock {
            method: "POST".into(), path_contains: path_contains.into(),
            status: 201, body: String::new(),
        });
        self
    }
    pub fn put(mut self, path_contains: &str) -> Self {
        self.last = Some(Mock {
            method: "PUT".into(), path_contains: path_contains.into(),
            status: 200, body: String::new(),
        });
        self
    }
    pub fn status(mut self, code: u16) -> Self {
        if let Some(m) = self.last.as_mut() { m.status = code; }
        self
    }
    pub fn body(mut self, b: impl Into<String>) -> Self {
        if let Some(m) = self.last.as_mut() { m.body = b.into(); }
        self
    }
    pub fn mount(self) {
        if let Some(m) = self.last {
            self.server.mocks.lock().unwrap().push_back(m);
        }
    }
}

fn handle_one(
    mut stream: TcpStream,
    requests: Arc<Mutex<Vec<RecordedRequest>>>,
    mocks: Arc<Mutex<VecDeque<Mock>>>,
) -> std::io::Result<()> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];
    loop {
        let n = stream.read(&mut tmp)?;
        if n == 0 { break; }
        buf.extend_from_slice(&tmp[..n]);
        if buf.windows(4).any(|w| w == b"\r\n\r\n") { break; }
    }
    let raw = String::from_utf8_lossy(&buf).to_string();
    let (method, path, api_key, body) = parse_request(&raw);

    requests.lock().unwrap().push(RecordedRequest {
        method: method.clone(), path: path.clone(), body: body.clone(), api_key,
    });

    let (status, body_out) = {
        let mut g = mocks.lock().unwrap();
        let idx = g.iter().position(|m| m.method == method && path.contains(&m.path_contains));
        match idx {
            Some(i) => {
                let m = g.remove(i).unwrap();
                (m.status, m.body)
            }
            None => (500, r#"{"errors":["no mock matched"]}"#.to_string()),
        }
    };

    let status_line = match status {
        200 => "HTTP/1.1 200 OK",
        201 => "HTTP/1.1 201 Created",
        404 => "HTTP/1.1 404 Not Found",
        422 => "HTTP/1.1 422 Unprocessable Entity",
        500 => "HTTP/1.1 500 Internal Server Error",
        _ => "HTTP/1.1 200 OK",
    };
    let resp = format!(
        "{status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body_out.len(),
        body_out
    );
    stream.write_all(resp.as_bytes())?;
    stream.flush()?;
    Ok(())
}

fn parse_request(raw: &str) -> (String, String, Option<String>, String) {
    let mut header_end = 0usize;
    if let Some(idx) = raw.find("\r\n\r\n") { header_end = idx; }
    let head = &raw[..header_end];
    let body = raw.get(header_end + 4..).unwrap_or("").to_string();
    let mut lines = head.split("\r\n");
    let request_line = lines.next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let target = parts.next().unwrap_or("").to_string();
    let path = target.split('?').next().unwrap_or("").to_string();
    let mut api_key = None;
    for line in lines {
        let lower = line.to_ascii_lowercase();
        if let Some(v) = lower.strip_prefix("x-redmine-api-key:") {
            let start = "x-redmine-api-key:".len();
            api_key = Some(line[start..].trim().to_string());
        }
    }
    (method, path, api_key, body)
}

pub fn test_config(server: &MockServer) -> Config {
    Config {
        url: server.url.clone(),
        api_key: "test-key".into(),
        default_project: None,
        format: Format::Tab,
        mutable: true,
        source: ConfigSource { url_from: "test", key_from: "test" },
    }
}

pub fn test_client(server: &MockServer) -> RedmineClient {
    RedmineClient::new(&test_config(server)).expect("client build")
}
