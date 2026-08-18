mod common;

#[test]
fn test_help_shows_attachments_command() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_redmine"))
        .args(["issue", "--help"])
        .output()
        .expect("failed to execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("attachments"));
}

#[test]
fn test_attachments_help_shows_list_and_download() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_redmine"))
        .args(["issue", "attachments", "--help"])
        .output()
        .expect("failed to execute");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("list"));
    assert!(stdout.contains("download"));
}

#[test]
fn test_attachments_list_requires_issue_id() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_redmine"))
        .args(["issue", "attachments", "list"])
        .output()
        .expect("failed to execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("required") || stderr.contains("error"));
}

#[test]
fn test_attachments_download_requires_attachment_id() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_redmine"))
        .args(["issue", "attachments", "download"])
        .output()
        .expect("failed to execute");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("required") || stderr.contains("error"));
}