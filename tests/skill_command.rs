use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_redmine"))
}

fn repo_skill() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("skills/redmine/SKILL.md")
}

fn temp_home(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("redmine-skill-{}-{}", tag, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn show_output() -> String {
    let out = bin().args(["skill", "show"]).output().unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn skill_show_matches_repo_file() {
    let disk = fs::read_to_string(repo_skill()).unwrap();
    assert_eq!(show_output(), disk);
}

#[test]
fn skill_frontmatter_is_valid() {
    let text = show_output();
    assert!(text.starts_with("---\n"));
    let end = text.find("\n---").unwrap();
    let frontmatter = &text[4..end];
    let name = frontmatter
        .lines()
        .find(|l| l.starts_with("name:"))
        .unwrap()
        .trim_start_matches("name:")
        .trim();
    assert_eq!(name, "redmine");
    let description = frontmatter
        .lines()
        .find(|l| l.starts_with("description:"))
        .unwrap()
        .trim_start_matches("description:")
        .trim();
    assert!(description.len() > 20, "description must be specific");
}

#[test]
fn skill_install_writes_file_under_home() {
    let home = temp_home("install");
    let out = bin().args(["skill", "install"]).env("HOME", &home).output().unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let installed = home.join(".agents/skills/redmine/SKILL.md");
    let disk = fs::read_to_string(repo_skill()).unwrap();
    assert_eq!(fs::read_to_string(&installed).unwrap(), disk);
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("installed"), "stdout: {stdout}");
    let _ = fs::remove_dir_all(&home);
}

#[test]
fn skill_install_is_idempotent() {
    let home = temp_home("idempotent");
    for _ in 0..2 {
        let out = bin().args(["skill", "install"]).env("HOME", &home).output().unwrap();
        assert!(out.status.success());
    }
    let installed = home.join(".agents/skills/redmine/SKILL.md");
    let disk = fs::read_to_string(repo_skill()).unwrap();
    assert_eq!(fs::read_to_string(&installed).unwrap(), disk);
    let _ = fs::remove_dir_all(&home);
}

#[test]
fn skill_install_creates_missing_directories() {
    let home = temp_home("dirs");
    let out = bin().args(["skill", "install"]).env("HOME", &home).output().unwrap();
    assert!(out.status.success());
    assert!(home.join(".agents/skills/redmine/SKILL.md").is_file());
    let _ = fs::remove_dir_all(&home);
}
