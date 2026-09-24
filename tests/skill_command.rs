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
    assert!(text.ends_with('\n'));
    let end = text.find("\n---").unwrap();
    let frontmatter = &text[4..end];
    let name = frontmatter
        .lines()
        .find(|l| l.starts_with("name:"))
        .unwrap()
        .trim_start_matches("name:")
        .trim();
    assert_eq!(name, "redmine");
    assert!(!name.is_empty() && name.len() <= 64);
    assert!(name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'));
    let description = frontmatter
        .lines()
        .find(|l| l.starts_with("description:"))
        .unwrap()
        .trim_start_matches("description:")
        .trim();
    assert!(description.len() > 20, "description must be specific");
    assert!(description.len() <= 1024, "description exceeds agent-skills limit");
    assert!(!description.contains(": "), "colon in description would break frontmatter parsing");
}

#[cfg(unix)]
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
    assert_eq!(stdout, format!("installed\t{}\n", installed.display()));
    let _ = fs::remove_dir_all(&home);
}

#[cfg(unix)]
#[test]
fn skill_install_reports_overwrite_and_restores_content() {
    let home = temp_home("overwrite");
    let installed = home.join(".agents/skills/redmine/SKILL.md");
    bin().args(["skill", "install"]).env("HOME", &home).output().unwrap();
    fs::write(&installed, "user customization").unwrap();
    let out = bin().args(["skill", "install"]).env("HOME", &home).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout, format!("overwritten\t{}\n", installed.display()));
    let disk = fs::read_to_string(repo_skill()).unwrap();
    assert_eq!(fs::read_to_string(&installed).unwrap(), disk);
    let _ = fs::remove_dir_all(&home);
}

#[cfg(unix)]
#[test]
fn skill_install_creates_missing_directories() {
    let home = temp_home("dirs");
    let out = bin().args(["skill", "install"]).env("HOME", &home).output().unwrap();
    assert!(out.status.success());
    assert!(home.join(".agents/skills/redmine/SKILL.md").is_file());
    assert!(!home.join(".agents/skills/redmine/SKILL.md.tmp").exists());
    let _ = fs::remove_dir_all(&home);
}

#[cfg(unix)]
#[test]
fn skill_install_fails_with_config_error_on_unwritable_home() {
    let home = temp_home("readonly");
    let mut perms = fs::metadata(&home).unwrap().permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o555);
    fs::set_permissions(&home, perms).unwrap();
    let out = bin().args(["skill", "install"]).env("HOME", &home).output().unwrap();
    assert_eq!(out.status.code(), Some(5));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("cannot install skill to"), "stderr: {stderr}");
    let mut perms = fs::metadata(&home).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&home, perms).unwrap();
    let _ = fs::remove_dir_all(&home);
}
