mod common;

#[test]
fn test_attachments_download_success() {
    let server = common::MockServer::start();
    server
        .mock()
        .get("/attachments/download/10")
        .status(200)
        .body("test file content")
        .mount();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_redmine"))
        .args(["--url", &server.url, "--key", "test-key", "issue", "attachments", "download", "10"])
        .current_dir("/tmp")
        .output()
        .expect("failed to execute");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("downloaded") || stdout.contains("saved"));
    
    let _ = std::fs::remove_file("/tmp/10");
}

#[test]
fn test_attachments_download_with_custom_output() {
    let server = common::MockServer::start();
    server
        .mock()
        .get("/attachments/download/20")
        .status(200)
        .body("custom file content")
        .mount();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_redmine"))
        .args([
            "--url", &server.url,
            "--key", "test-key",
            "issue", "attachments", "download", "20",
            "--output", "/tmp/custom_file.txt"
        ])
        .output()
        .expect("failed to execute");

    assert!(output.status.success());
    assert!(std::path::Path::new("/tmp/custom_file.txt").exists());
    
    let content = std::fs::read_to_string("/tmp/custom_file.txt").unwrap();
    assert_eq!(content, "custom file content");
    
    let _ = std::fs::remove_file("/tmp/custom_file.txt");
}

#[test]
fn test_attachments_download_not_found() {
    let server = common::MockServer::start();
    server
        .mock()
        .get("/attachments/download/999")
        .status(404)
        .body(r#"{"error": "Attachment not found"}"#)
        .mount();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_redmine"))
        .args([
            "--url", &server.url,
            "--key", "test-key",
            "issue", "attachments", "download", "999"
        ])
        .output()
        .expect("failed to execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error") || stderr.contains("not found"));
}

#[test]
fn test_attachments_download_forbidden() {
    let server = common::MockServer::start();
    server
        .mock()
        .get("/attachments/download/30")
        .status(403)
        .body(r#"{"error": "Forbidden"}"#)
        .mount();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_redmine"))
        .args([
            "--url", &server.url,
            "--key", "test-key",
            "issue", "attachments", "download", "30"
        ])
        .output()
        .expect("failed to execute");

    assert!(!output.status.success());
    let exit_code = output.status.code().unwrap_or(0);
    assert!(exit_code == 11 || exit_code == 2);
}

#[test]
fn test_attachments_download_network_error() {
    let server = common::MockServer::start();
    server
        .mock()
        .get("/attachments/download/40")
        .status(500)
        .body(r#"{"error": "Internal server error"}"#)
        .mount();

    let output = std::process::Command::new(env!("CARGO_BIN_EXE_redmine"))
        .args([
            "--url", &server.url,
            "--key", "test-key",
            "issue", "attachments", "download", "40"
        ])
        .output()
        .expect("failed to execute");

    assert!(!output.status.success());
}