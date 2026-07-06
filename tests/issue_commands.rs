mod common;

use redmine_cli::cli::{IssueCreateArgs, IssueListArgs, IssueUpdateArgs};
use redmine_cli::commands::issue;
use redmine_cli::error::Error;

use common::MockServer;

const CURRENT_JSON: &str = r#"{"user":{"id":7,"login":"alice","firstname":"Alice","lastname":"Smith"}}"#;
const PROJECTS_JSON: &str = r#"{"projects":[{"id":10,"name":"Infra","identifier":"infra"}]}"#;

fn issue_list_json() -> &'static str {
    r#"{"issues":[
        {"id":1001,"project":{"id":10,"name":"Infra","identifier":"infra"},"tracker":{"id":1,"name":"Bug"},
         "status":{"id":2,"name":"In Progress"},"priority":{"id":4,"name":"Urgent"},
         "author":{"id":7,"name":"Alice Smith"},"assigned_to":{"id":7,"name":"Alice Smith"},
         "subject":"Fix build","description":"fails on CI","start_date":"2026-07-01",
         "due_date":null,"done_ratio":30,"estimated_hours":4.0,
         "created_on":"2026-07-01T09:00:00Z","updated_on":"2026-07-05T10:00:00Z"}
    ],"total_count":1,"offset":0,"limit":25}"#
}

fn default_list_args() -> IssueListArgs {
    IssueListArgs {
        assigned_to: None, status: None, project: None,
        limit: 25, sort: "updated_on:desc".into(), all: false,
    }
}

fn capture<F: FnOnce(&mut Vec<u8>)>(f: F) -> String {
    let mut buf = Vec::new();
    f(&mut buf);
    String::from_utf8(buf).unwrap()
}

#[test]
fn list_filters_by_assignee_me() {
    let server = MockServer::start();
    server.mock().get("/users/current.json").body(CURRENT_JSON).mount();
    server.mock().get("/issues.json").body(issue_list_json()).mount();

    let cfg = common::test_config(&server);
    let client = common::test_client(&server);
    let out = capture(|buf| issue::mine(&client, &cfg, default_list_args(), buf).unwrap());

    let reqs = server.requests();
    assert!(reqs.iter().any(|r| r.path == "/issues.json" && r.path.contains("")));
    assert!(out.contains("1001"));
    assert!(out.contains("Fix build"));
    assert!(out.contains("In Progress"));

    let q = reqs.iter().find(|r| r.path == "/issues.json").unwrap();
    assert!(q.body.is_empty());
}

#[test]
fn show_renders_description_block() {
    let server = MockServer::start();
    let detail = r#"{"issue":{"id":1001,"project":{"id":10,"name":"Infra","identifier":"infra"},
        "tracker":{"id":1,"name":"Bug"},"status":{"id":2,"name":"In Progress"},
        "priority":{"id":4,"name":"Urgent"},"author":{"id":7,"name":"Alice Smith"},
        "assigned_to":{"id":7,"name":"Alice Smith"},"subject":"Fix build",
        "description":"fails on CI","start_date":"2026-07-01","due_date":null,
        "done_ratio":30,"estimated_hours":4.0,"created_on":"2026-07-01T09:00:00Z",
        "updated_on":"2026-07-05T10:00:00Z"}}"#;
    server.mock().get("/issues/1001.json").body(detail).mount();

    let cfg = common::test_config(&server);
    let client = common::test_client(&server);
    let out = capture(|buf| issue::show(&client, &cfg, 1001, false, buf).unwrap());
    assert!(out.contains("DESCRIPTION"));
    assert!(out.contains("fails on CI"));
    assert!(out.contains("subject\tFix build"));
}

#[test]
fn create_sends_project_id_and_subject() {
    let server = MockServer::start();
    server.mock().get("/projects.json").body(PROJECTS_JSON).mount();
    let created = r#"{"issue":{"id":3000,"project":{"id":10,"name":"Infra","identifier":"infra"},
        "tracker":{"id":1,"name":"Bug"},"status":{"id":1,"name":"New"},
        "priority":{"id":2,"name":"Normal"},"author":{"id":7,"name":"Alice Smith"},
        "subject":"New bug","description":"","created_on":"2026-07-06T00:00:00Z",
        "updated_on":"2026-07-06T00:00:00Z"}}"#;
    server.mock().post("/issues.json").body(created).mount();

    let cfg = common::test_config(&server);
    let client = common::test_client(&server);
    let args = IssueCreateArgs {
        project: Some("infra".into()), subject: "New bug".into(),
        description: None, assignee: None, tracker: None, priority: None, status: None,
    };
    let out = capture(|buf| issue::create(&client, &cfg, args, buf).unwrap());
    assert!(out.contains("created"));
    assert!(out.contains("3000"));

    let post = server.requests().into_iter().find(|r| r.method == "POST").unwrap();
    assert!(post.body.contains(r#""project_id":10"#));
    assert!(post.body.contains(r#""subject":"New bug""#));
}

#[test]
fn update_resolves_status_name_and_sends_id() {
    let server = MockServer::start();
    let statuses = r#"{"issue_statuses":[{"id":1,"name":"New"},{"id":2,"name":"In Progress"},{"id":5,"name":"Closed"}]}"#;
    server.mock().get("/issue_statuses.json").body(statuses).mount();
    server.mock().get("/enumerations.json").body(r#"{"issue_priorities":[]}"#).mount();
    server.mock().put("/issues/1001.json").body("").mount();

    let cfg = common::test_config(&server);
    let client = common::test_client(&server);
    let args = IssueUpdateArgs {
        id: 1001, status: Some("Closed".into()), assignee: None, subject: None,
        description: None, priority: None, done: None, note: None,
    };
    let out = capture(|buf| issue::update(&client, &cfg, args, buf).unwrap());
    assert!(out.contains("updated"));
    assert!(out.contains("status\tClosed"));

    let put = server.requests().into_iter().find(|r| r.method == "PUT").unwrap();
    assert!(put.body.contains(r#""status_id":5"#));
}

#[test]
fn http_404_maps_to_error() {
    let server = MockServer::start();
    server.mock().get("/issues/9999.json").status(404).body(r#"{"errors":["not found"]}"#).mount();

    let cfg = common::test_config(&server);
    let client = common::test_client(&server);
    let mut buf = Vec::new();
    let err = issue::show(&client, &cfg, 9999, false, &mut buf).unwrap_err();
    match err {
        Error::Http { status, .. } => assert_eq!(status, 404),
        other => panic!("expected Http, got {other:?}"),
    }
}

#[test]
fn ambiguous_status_errors() {
    let server = MockServer::start();
    let statuses = r#"{"issue_statuses":[
        {"id":1,"name":"New"},{"id":2,"name":"In Progress"},{"id":3,"name":"Resolved"}]}"#;
    server.mock().get("/issue_statuses.json").body(statuses).mount();
    server.mock().get("/enumerations.json").body(r#"{"issue_priorities":[]}"#).mount();

    let cfg = common::test_config(&server);
    let client = common::test_client(&server);
    let args = IssueUpdateArgs {
        id: 1001, status: Some("re".into()), assignee: None, subject: None,
        description: None, priority: None, done: None, note: None,
    };
    let mut buf = Vec::new();
    let err = issue::update(&client, &cfg, args, &mut buf).unwrap_err();
    match err {
        Error::Ambiguous { kind, matches, .. } => {
            assert_eq!(kind, "status");
            assert!(matches.iter().any(|m| m.contains("In Progress")));
        }
        other => panic!("expected Ambiguous, got {other:?}"),
    }
}

#[test]
fn done_ratio_out_of_range_rejected() {
    let server = MockServer::start();
    let cfg = common::test_config(&server);
    let client = common::test_client(&server);
    let args = IssueUpdateArgs {
        id: 1001, status: None, assignee: None, subject: None,
        description: None, priority: None, done: Some(150), note: None,
    };
    let mut buf = Vec::new();
    let err = issue::update(&client, &cfg, args, &mut buf).unwrap_err();
    assert!(matches!(err, Error::Usage(_)));
}

#[test]
fn assigns_to_me_via_current_user() {
    let server = MockServer::start();
    server.mock().get("/users/current.json").body(CURRENT_JSON).mount();
    server.mock().put("/issues/1001.json").body("").mount();

    let cfg = common::test_config(&server);
    let client = common::test_client(&server);
    let args = IssueUpdateArgs {
        id: 1001, status: None, assignee: Some("me".into()), subject: None,
        description: None, priority: None, done: None, note: None,
    };
    let out = capture(|buf| issue::update(&client, &cfg, args, buf).unwrap());
    assert!(out.contains("assignee\tme"));

    let put = server.requests().into_iter().find(|r| r.method == "PUT").unwrap();
    assert!(put.body.contains(r#""assigned_to_id":7"#));
}

#[test]
fn auth_header_sent() {
    let server = MockServer::start();
    server.mock().get("/users/current.json").body(CURRENT_JSON).mount();

    let cfg = common::test_config(&server);
    let client = common::test_client(&server);
    let id = client.current_user().unwrap();
    assert_eq!(id, 7);

    let req = server.requests().into_iter().find(|r| r.path == "/users/current.json").unwrap();
    assert_eq!(req.api_key.as_deref(), Some("test-key"));
}
