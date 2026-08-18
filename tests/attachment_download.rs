use std::fs;
use std::path::PathBuf;

mod common;

#[test]
fn test_download_attachment_success() {
    let server = common::MockServer::start();
    server
        .mock()
        .get("/attachments/download/1")
        .status(200)
        .body("test file content")
        .mount();

    let client = common::test_client(&server);
    let output_path = PathBuf::from("/tmp/test_download.txt");

    let result = client.download_attachment(1, &output_path);

    assert!(result.is_ok());
    assert!(output_path.exists());
    let content = fs::read_to_string(&output_path).unwrap();
    assert_eq!(content, "test file content");

    let _ = fs::remove_file(&output_path);
}

#[test]
fn test_download_attachment_not_found() {
    let server = common::MockServer::start();
    server
        .mock()
        .get("/attachments/download/999")
        .status(404)
        .body("{\"error\": \"Attachment not found\"}")
        .mount();

    let client = common::test_client(&server);
    let output_path = PathBuf::from("/tmp/test_download_404.txt");

    let result = client.download_attachment(999, &output_path);

    assert!(result.is_err());
    assert!(!output_path.exists());
}

#[test]
fn test_download_attachment_forbidden() {
    let server = common::MockServer::start();
    server
        .mock()
        .get("/attachments/download/2")
        .status(403)
        .body("{\"error\": \"Forbidden\"}")
        .mount();

    let client = common::test_client(&server);
    let output_path = PathBuf::from("/tmp/test_download_403.txt");

    let result = client.download_attachment(2, &output_path);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), redmine_cli::error::Error::Http { status, .. } if status == 403));
}

#[test]
fn test_download_attachment_sends_api_key() {
    let server = common::MockServer::start();
    server
        .mock()
        .get("/attachments/download/3")
        .status(200)
        .body("content")
        .mount();

    let client = common::test_client(&server);
    let output_path = PathBuf::from("/tmp/test_download_auth.txt");

    let _ = client.download_attachment(3, &output_path);

    let requests = server.requests();
    assert_eq!(requests.len(), 1);
    let request = &requests[0];
    assert_eq!(request.path, "/attachments/download/3");
    assert!(request.api_key.as_ref().is_some_and(|k| k == "test-key"));

    let _ = fs::remove_file(&output_path);
}