mod common;

#[test]
fn test_attachment_list_tab_format() {
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
                "attachments": [
                    {
                        "id": 10,
                        "filename": "test.txt",
                        "filesize": 1024,
                        "content_type": "text/plain",
                        "content_url": "http://example.com/attachments/10",
                        "author": {"id": 1, "name": "Alice"},
                        "created_on": "2024-01-01T00:00:00Z"
                    },
                    {
                        "id": 11,
                        "filename": "image.png",
                        "filesize": 2048,
                        "content_type": "image/png",
                        "content_url": "http://example.com/attachments/11",
                        "author": {"id": 2, "name": "Bob"},
                        "created_on": "2024-01-02T00:00:00Z"
                    }
                ]
            }
        }"#)
        .mount();

    let client = common::test_client(&server);
    let mut output = Vec::new();

    #[derive(serde::Deserialize)]
    struct IssueWrapper {
        pub issue: redmine_cli::models::issue::Issue,
    }
    
    let wrapper: IssueWrapper = client.get("/issues/1.json", &[]).unwrap();
    let attachments = wrapper.issue.attachments.unwrap();

    redmine_cli::output::render_attachment_list(
        redmine_cli::config::Format::Tab,
        &mut output,
        &attachments,
    );

    let output_str = String::from_utf8_lossy(&output);
    let lines: Vec<&str> = output_str.lines().collect();
    
    assert!(lines.len() >= 3);
    assert!(lines[0].contains("id"));
    assert!(lines[0].contains("filename"));
    assert!(lines[1].contains("10"));
    assert!(lines[1].contains("test.txt"));
    assert!(lines[2].contains("11"));
    assert!(lines[2].contains("image.png"));
}

#[test]
fn test_attachment_list_pretty_format() {
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
                "attachments": [
                    {
                        "id": 10,
                        "filename": "test.txt",
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

    let client = common::test_client(&server);
    let mut output = Vec::new();

    #[derive(serde::Deserialize)]
    struct IssueWrapper {
        pub issue: redmine_cli::models::issue::Issue,
    }
    
    let wrapper: IssueWrapper = client.get("/issues/1.json", &[]).unwrap();
    let attachments = wrapper.issue.attachments.unwrap();

    redmine_cli::output::render_attachment_list(
        redmine_cli::config::Format::Pretty,
        &mut output,
        &attachments,
    );

    let output_str = String::from_utf8_lossy(&output);
    
    assert!(output_str.contains("|"));
    assert!(output_str.contains("---"));
    assert!(output_str.contains("10"));
    assert!(output_str.contains("test.txt"));
    assert!(output_str.contains("KB"));
}

#[test]
fn test_attachment_list_json_format() {
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
                "attachments": [
                    {
                        "id": 10,
                        "filename": "test.txt",
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

    let client = common::test_client(&server);
    let mut output = Vec::new();

    #[derive(serde::Deserialize)]
    struct IssueWrapper {
        pub issue: redmine_cli::models::issue::Issue,
    }
    
    let wrapper: IssueWrapper = client.get("/issues/1.json", &[]).unwrap();
    let attachments = wrapper.issue.attachments.unwrap();

    redmine_cli::output::render_attachment_list(
        redmine_cli::config::Format::Json,
        &mut output,
        &attachments,
    );

    let output_str = String::from_utf8_lossy(&output);
    
    let json_line = output_str.lines().next().unwrap_or("");
    let json: serde_json::Value = serde_json::from_str(json_line).unwrap();
    assert!(json.is_array());
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"].as_str(), Some("10"));
    assert_eq!(arr[0]["filename"].as_str(), Some("test.txt"));
}

#[test]
fn test_attachment_list_shows_download_footer() {
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
                "attachments": [
                    {
                        "id": 10,
                        "filename": "test.txt",
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

    let client = common::test_client(&server);
    let mut output = Vec::new();

    #[derive(serde::Deserialize)]
    struct IssueWrapper {
        pub issue: redmine_cli::models::issue::Issue,
    }
    
    let wrapper: IssueWrapper = client.get("/issues/1.json", &[]).unwrap();
    let attachments = wrapper.issue.attachments.unwrap();

    redmine_cli::output::render_attachment_list(
        redmine_cli::config::Format::Tab,
        &mut output,
        &attachments,
    );

    let output_str = String::from_utf8_lossy(&output);
    
    assert!(output_str.contains("download"));
    assert!(output_str.contains("redmine issue attachments download"));
}