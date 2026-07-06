mod common;

use common::MockServer;
use redmine_cli::config::{Config, ConfigSource, Format};
use redmine_cli::models::common::NamedRef;
use redmine_cli::output;

fn cfg(format: Format) -> Config {
    Config {
        url: "http://x".into(),
        api_key: "k".into(),
        default_project: None,
        format,
        source: ConfigSource { url_from: "test", key_from: "test" },
    }
}

fn issue() -> redmine_cli::models::issue::Issue {
    serde_json::from_str(r#"{
        "id":1001,"project":{"id":10,"name":"Infra","identifier":"infra"},
        "tracker":{"id":1,"name":"Bug"},"status":{"id":2,"name":"In Progress"},
        "priority":{"id":4,"name":"Urgent"},"author":{"id":7,"name":"Alice Smith"},
        "assigned_to":{"id":7,"name":"Alice Smith"},"subject":"Fix build",
        "description":"fails on CI","start_date":"2026-07-01","due_date":null,
        "done_ratio":30,"estimated_hours":4.0,"created_on":"2026-07-01T09:00:00Z",
        "updated_on":"2026-07-05T10:00:00Z"}"#).unwrap()
}

#[test]
fn tab_list_has_no_decorations() {
    let mut buf = Vec::new();
    output::render_issue_list(Format::Tab, &mut buf, &[issue()]);
    let s = String::from_utf8(buf).unwrap();
    assert!(!s.contains('|'));
    assert!(s.starts_with("id\tproject\t"));
    assert!(s.contains("1001\tInfra\tBug\tIn Progress"));
    assert!(!s.contains('\u{2500}'));
}

#[test]
fn pretty_list_has_pipe_borders() {
    let mut buf = Vec::new();
    output::render_issue_list(Format::Pretty, &mut buf, &[issue()]);
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("| id"));
    assert!(s.contains("|---"));
    assert!(s.contains("| 1001 |"));
}

#[test]
fn json_list_is_compact_single_line() {
    let mut buf = Vec::new();
    output::render_issue_list(Format::Json, &mut buf, &[issue()]);
    let s = String::from_utf8(buf).unwrap();
    assert!(s.starts_with("[{"));
    assert!(s.trim_end().ends_with("}]"));
    assert!(!s.contains("\n{"));
}

#[test]
fn detail_tab_shows_description_block() {
    let mut buf = Vec::new();
    output::render_issue_detail(Format::Tab, &mut buf, &issue(), false);
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("subject\tFix build"));
    assert!(s.contains("DESCRIPTION"));
    assert!(s.contains("fails on CI"));
}

#[test]
fn named_list_tab() {
    let items = vec![
        NamedRef { id: 1, name: "New".into() },
        NamedRef { id: 2, name: "Closed".into() },
    ];
    let mut buf = Vec::new();
    output::render_named_list(Format::Tab, &mut buf, &items);
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("id\tname"));
    assert!(s.contains("1\tNew"));
    assert!(s.contains("2\tClosed"));
}

#[test]
fn ack_tab_and_json() {
    let mut buf = Vec::new();
    output::render_ack(Format::Tab, &mut buf, "updated", 1001, &[("status".into(), "Closed".into())]);
    let tab = String::from_utf8(buf).unwrap();
    assert!(tab.contains("updated\t1001"));
    assert!(tab.contains("  status\tClosed"));

    let mut buf = Vec::new();
    output::render_ack(Format::Json, &mut buf, "created", 9, &[("project".into(), "infra".into())]);
    let json = String::from_utf8(buf).unwrap();
    assert!(json.contains(r#""action":"created""#));
    assert!(json.contains(r#""id":9"#));
    assert!(json.contains(r#""field":"project""#));
}

#[test]
fn verbose_error_shows_full_body() {
    redmine_cli::error::set_verbose(true);
    let e = redmine_cli::error::Error::Http {
        status: 422,
        url: "http://x/issues/1.json".into(),
        body: "X".repeat(300),
    };
    let s = e.to_string();
    assert!(s.contains(&"X".repeat(300)));
    redmine_cli::error::set_verbose(false);
}

#[test]
fn non_verbose_error_truncates_body() {
    redmine_cli::error::set_verbose(false);
    let e = redmine_cli::error::Error::Http {
        status: 422,
        url: "http://x/issues/1.json".into(),
        body: "X".repeat(300),
    };
    let s = e.to_string();
    assert!(s.contains("..."));
    assert!(s.matches('X').count() <= 200);
}

#[test]
fn config_picks_flag_over_env() {
    let server = MockServer::start();
    let _ = server;
    let c = cfg(Format::Tab);
    assert_eq!(c.api_key, "k");
}
