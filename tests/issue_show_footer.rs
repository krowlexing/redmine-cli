mod common;

#[test]
fn test_issue_show_with_attachments_shows_footer() {
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
                    }
                ]
            }
        }"#)
        .mount();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_redmine"))
        .args(["--url", &server.url, "--key", "test-key", "issue", "show", "1"])
        .output()
        .expect("failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(stdout.contains("This issue has 1 attachment"));
    assert!(stdout.contains("redmine issue attachments 1"));
}

#[test]
fn test_issue_show_with_multiple_attachments_shows_footer() {
    let server = common::MockServer::start();
    server
        .mock()
        .get("/issues/2.json")
        .status(200)
        .body(r#"{
            "issue": {
                "id": 2,
                "subject": "Test issue with multiple attachments",
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
        .args(["--url", &server.url, "--key", "test-key", "issue", "show", "2"])
        .output()
        .expect("failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(stdout.contains("This issue has 2 attachments"));
    assert!(stdout.contains("redmine issue attachments 2"));
}

#[test]
fn test_issue_show_without_attachments_no_footer() {
    let server = common::MockServer::start();
    server
        .mock()
        .get("/issues/3.json")
        .status(200)
        .body(r#"{
            "issue": {
                "id": 3,
                "subject": "Test issue without attachments",
                "description": "Test description",
                "created_on": "2024-01-01T00:00:00Z",
                "updated_on": "2024-01-01T00:00:00Z"
            }
        }"#)
        .mount();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_redmine"))
        .args(["--url", &server.url, "--key", "test-key", "issue", "show", "3"])
        .output()
        .expect("failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(!stdout.contains("This issue has"));
    assert!(!stdout.contains("redmine issue attachments"));
}

#[test]
fn test_issue_show_with_notes_and_attachments() {
    let server = common::MockServer::start();
    server
        .mock()
        .get("/issues/4.json")
        .status(200)
        .body(r#"{
            "issue": {
                "id": 4,
                "subject": "Test issue with both",
                "description": "Test description",
                "created_on": "2024-01-01T00:00:00Z",
                "updated_on": "2024-01-01T00:00:00Z",
                "journals": [
                    {
                        "id": 1,
                        "user": {"id": 1, "name": "Alice"},
                        "notes": "First note",
                        "created_on": "2024-01-01T00:00:00Z",
                        "details": []
                    }
                ],
                "attachments": [
                    {
                        "id": 10,
                        "filename": "file.txt",
                        "filesize": 1024,
                        "content_type": "text/plain",
                        "content_url": "http://example.com/attachments/10",
                        "author": {"id": 1, "name": "Alice"},
                        "created_on": "2024-01-01T00:00:00Z"
                    }
                ]
            }
        }"#)
        .mount();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_redmine"))
        .args(["--url", &server.url, "--key", "test-key", "issue", "show", "4", "--notes"])
        .output()
        .expect("failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    assert!(stdout.contains("First note"));
    assert!(stdout.contains("This issue has 1 attachment"));
    assert!(stdout.contains("redmine issue attachments 4"));
}