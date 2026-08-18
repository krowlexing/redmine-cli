use redmine_cli::models::issue::Issue;

#[test]
fn test_issue_deserializes_with_attachments() {
    let json = r#"{
        "issue": {
            "id": 1234,
            "subject": "Test issue",
            "description": "Test description",
            "created_on": "2024-01-01T00:00:00Z",
            "updated_on": "2024-01-01T00:00:00Z",
            "attachments": [
                {
                    "id": 1,
                    "filename": "test.txt",
                    "filesize": 1024,
                    "content_type": "text/plain",
                    "content_url": "http://example.com/attachments/1",
                    "author": {"id": 1, "name": "Admin"},
                    "created_on": "2024-01-01T00:00:00Z",
                    "description": "Test file"
                },
                {
                    "id": 2,
                    "filename": "image.png",
                    "filesize": 2048,
                    "content_type": "image/png",
                    "content_url": "http://example.com/attachments/2",
                    "author": {"id": 2, "name": "User"},
                    "created_on": "2024-01-02T00:00:00Z"
                }
            ]
        }
    }"#;

    let wrapper: serde_json::Value = serde_json::from_str(json).unwrap();
    let issue: Issue = serde_json::from_value(wrapper["issue"].clone()).unwrap();

    assert_eq!(issue.id, 1234);
    assert_eq!(issue.subject, "Test issue");
    assert!(issue.attachments.is_some());
    let attachments = issue.attachments.as_ref().unwrap();
    assert_eq!(attachments.len(), 2);
    assert_eq!(attachments[0].id, 1);
    assert_eq!(attachments[0].filename, "test.txt");
    assert_eq!(attachments[0].filesize, 1024);
    assert_eq!(attachments[1].id, 2);
    assert_eq!(attachments[1].filename, "image.png");
    assert_eq!(attachments[1].filesize, 2048);
}

#[test]
fn test_issue_deserializes_without_attachments() {
    let json = r#"{
        "issue": {
            "id": 1234,
            "subject": "Test issue",
            "description": "Test description",
            "created_on": "2024-01-01T00:00:00Z",
            "updated_on": "2024-01-01T00:00:00Z"
        }
    }"#;

    let wrapper: serde_json::Value = serde_json::from_str(json).unwrap();
    let issue: Issue = serde_json::from_value(wrapper["issue"].clone()).unwrap();

    assert_eq!(issue.id, 1234);
    assert!(issue.attachments.is_none());
}

#[test]
fn test_attachment_optional_fields() {
    let json = r#"{
        "id": 1,
        "filename": "test.txt",
        "filesize": 1024,
        "content_type": "text/plain",
        "content_url": "http://example.com/attachments/1",
        "author": {"id": 1, "name": "Admin"},
        "created_on": "2024-01-01T00:00:00Z"
    }"#;

    let attachment: redmine_cli::models::issue::Attachment = serde_json::from_str(json).unwrap();
    assert_eq!(attachment.id, 1);
    assert_eq!(attachment.filename, "test.txt");
    assert!(attachment.description.is_none());
}