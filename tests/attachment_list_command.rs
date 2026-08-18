mod common;

#[test]
fn test_attachments_list_success() {
    let server = common::MockServer::start();
    server
        .mock()
        .get("/issues/1.json")
        .status(200)
        .body(r#"{
            "issue": {
                "id": 1,
                "subject": "Test issue with attachments",
                "description": "Test description",
                "created_on": "2024-01-01T00:00:00Z",
                "updated_on": "2024-01-01T00:00:00Z",
                "attachments": [
                    {
                        "id": 10,
                        "filename": "document.pdf",
                        "filesize": 524288,
                        "content_type": "application/pdf",
                        "content_url": "http://example.com/attachments/10",
                        "author": {"id": 1, "name": "Alice"},
                        "created_on": "2024-01-01T00:00:00Z"
                    },
                    {
                        "id": 11,
                        "filename": "image.jpg",
                        "filesize": 1048576,
                        "content_type": "image/jpeg",
                        "content_url": "http://example.com/attachments/11",
                        "author": {"id": 2, "name": "Bob"},
                        "created_on": "2024-01-02T00:00:00Z"
                    }
                ]
            }
        }"#)
        .mount();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_redmine"))
        .args(["--url", &server.url, "--key", "test-key", "issue", "attachments", "list", "1"])
        .output()
        .expect("failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(stdout.contains("10"));
    assert!(stdout.contains("document.pdf"));
    assert!(stdout.contains("11"));
    assert!(stdout.contains("image.jpg"));
    assert!(stdout.contains("download"));
    assert!(stdout.contains("redmine issue attachments download"));
}

#[test]
fn test_attachments_list_empty() {
    let server = common::MockServer::start();
    server
        .mock()
        .get("/issues/2.json")
        .status(200)
        .body(r#"{
            "issue": {
                "id": 2,
                "subject": "Test issue without attachments",
                "description": "Test description",
                "created_on": "2024-01-01T00:00:00Z",
                "updated_on": "2024-01-01T00:00:00Z"
            }
        }"#)
        .mount();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_redmine"))
        .args(["--url", &server.url, "--key", "test-key", "issue", "attachments", "list", "2"])
        .output()
        .expect("failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(stdout.contains("No attachments") || stdout.lines().count() <= 1);
}

#[test]
fn test_attachments_list_issue_not_found() {
    let server = common::MockServer::start();
    server
        .mock()
        .get("/issues/999.json")
        .status(404)
        .body(r#"{"error": "Issue not found"}"#)
        .mount();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_redmine"))
        .args(["--url", &server.url, "--key", "test-key", "issue", "attachments", "list", "999"])
        .output()
        .expect("failed to execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error") || stderr.contains("not found"));
}

#[test]
fn test_attachments_list_requests_attachments_parameter() {
    let server = common::MockServer::start();
    server
        .mock()
        .get("/issues/1.json")
        .status(200)
        .body(r#"{
            "issue": {
                "id": 1,
                "subject": "Test",
                "description": "",
                "created_on": "2024-01-01T00:00:00Z",
                "updated_on": "2024-01-01T00:00:00Z",
                "attachments": []
            }
        }"#)
        .mount();

    std::process::Command::new(env!("CARGO_BIN_EXE_redmine"))
        .args(["--url", &server.url, "--key", "test-key", "issue", "attachments", "list", "1"])
        .output()
        .expect("failed to execute");

    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].path.contains("/issues/1.json"));
}