use redmine_cli::config::{Config, ConfigSource, Format};
use redmine_cli::error::Error;

fn cfg(mutable: bool) -> Config {
    Config {
        url: "http://x".into(),
        api_key: "k".into(),
        default_project: None,
        format: Format::Tab,
        mutable,
        source: ConfigSource { url_from: "test", key_from: "test" },
    }
}

#[test]
fn ensure_mutable_blocks_when_false() {
    let err = cfg(false).ensure_mutable().unwrap_err();
    assert!(matches!(err, Error::Blocked));
    assert_eq!(err.to_string(), "mutable operations are blocked. requires human intervention");
}

#[test]
fn ensure_mutable_allows_when_true() {
    cfg(true).ensure_mutable().unwrap();
}

#[test]
fn blocked_error_exit_code_is_6() {
    let err = cfg(false).ensure_mutable().unwrap_err();
    let code = match err {
        Error::Blocked => 6,
        _ => panic!("expected Blocked"),
    };
    assert_eq!(code, 6);
}
