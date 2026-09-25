use std::process::Command;

mod common;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_redmine"))
}

fn issue_json(parent: Option<&str>, children: Option<&str>) -> String {
    let mut s = String::from(
        r#"{"issue":{"id":100,"project":{"id":1,"name":"infra"},"tracker":{"id":1,"name":"Feature"},
        "status":{"id":1,"name":"New"},"priority":{"id":2,"name":"Normal"},
        "author":{"id":1,"name":"Alice"},"subject":"Parent task","description":"",
        "created_on":"2026-01-01T00:00:00Z","updated_on":"2026-01-01T00:00:00Z""#,
    );
    if let Some(p) = parent {
        s.push_str(&format!(",\"parent\":{p}"));
    }
    if let Some(c) = children {
        s.push_str(&format!(",\"children\":{c}"));
    }
    s.push_str("}}");
    s
}

fn run_show(server: &common::MockServer, body: String, extra: &[&str]) -> (String, String, Option<i32>) {
    server.mock().get("/issues/100.json").status(200).body(body).mount();
    let mut cmd = bin();
    cmd.args(["--url", &server.url, "--key", "test-key", "issue", "show", "100"]);
    cmd.args(extra);
    let out = cmd.output().unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code(),
    )
}

#[test]
fn show_requests_children_in_include() {
    let server = common::MockServer::start();
    let body = issue_json(None, None);
    let _ = run_show(&server, body, &[]);
    let req = &server.requests()[0];
    assert!(req.query.contains("include="), "query: {}", req.query);
    assert!(req.query.contains("children"), "query: {}", req.query);
    assert!(req.query.contains("attachments"), "query: {}", req.query);
}

#[test]
fn show_with_notes_requests_children_and_journals() {
    let server = common::MockServer::start();
    let body = issue_json(None, None);
    let _ = run_show(&server, body, &["--notes"]);
    let req = &server.requests()[0];
    assert!(req.query.contains("children"), "query: {}", req.query);
    assert!(req.query.contains("journals"), "query: {}", req.query);
}

#[test]
fn show_prints_parent_id_after_project() {
    let server = common::MockServer::start();
    let body = issue_json(Some(r#"{"id":99}"#), None);
    let (stdout, _, code) = run_show(&server, body, &[]);
    assert_eq!(code, Some(0));
    let lines: Vec<&str> = stdout.lines().collect();
    let project = lines.iter().position(|l| l.starts_with("project\t")).unwrap();
    let parent = lines.iter().position(|l| **l == *"parent\t99").unwrap();
    assert_eq!(parent, project + 1, "parent must directly follow project");
}

#[test]
fn show_prints_children_block() {
    let server = common::MockServer::start();
    let children = r#"[{"id":101,"tracker":{"id":1,"name":"Feature"},"subject":"First subtask"},
        {"id":102,"tracker":{"id":2,"name":"Bug"},"subject":"Second subtask"}]"#;
    let body = issue_json(None, Some(children));
    let (stdout, _, code) = run_show(&server, body, &[]);
    assert_eq!(code, Some(0));
    assert!(stdout.contains("\nCHILDREN\n"), "stdout: {stdout}");
    assert!(stdout.contains("101\tFeature\tFirst subtask"));
    assert!(stdout.contains("102\tBug\tSecond subtask"));
    assert!(!stdout.contains("parent\t"));
}

#[test]
fn show_without_parent_or_children_prints_neither() {
    let server = common::MockServer::start();
    let body = issue_json(None, None);
    let (stdout, _, code) = run_show(&server, body, &[]);
    assert_eq!(code, Some(0));
    assert!(!stdout.contains("parent\t"));
    assert!(!stdout.contains("CHILDREN"));
}

#[test]
fn show_json_includes_parent_and_children() {
    let server = common::MockServer::start();
    let children = r#"[{"id":101,"tracker":{"id":1,"name":"Feature"},"subject":"First subtask"}]"#;
    let body = issue_json(Some(r#"{"id":99}"#), Some(children));
    let (stdout, _, code) = run_show(&server, body, &["--format", "json"]);
    assert_eq!(code, Some(0));
    assert!(stdout.contains(r#""parent":{"id":99}"#), "stdout: {stdout}");
    assert!(stdout.contains(r#""id":101"#));
    assert!(stdout.contains("First subtask"));
}
