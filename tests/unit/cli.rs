use super::*;
#[test]
fn default_uses_platform_config_directory() {
    assert_eq!(
        parse([]).unwrap().unwrap(),
        Options {
            database_path: dirs::config_dir().unwrap().join("argv-todo/db.sql"),
            database_override: false,
        }
    );
}
#[test]
fn accepts_override_and_rejects_missing_or_unknown_arguments() {
    assert_eq!(
        parse(["--db".into(), "/tmp/custom.sql".into()]).unwrap(),
        Some(Options {
            database_path: PathBuf::from("/tmp/custom.sql"),
            database_override: true,
        })
    );
    assert!(parse(["--db".into()]).is_err());
    assert!(parse(["--unknown".into()]).is_err());
    assert!(parse(["--db".into(), "".into()]).is_err());
    assert_eq!(parse(["--help".into()]).unwrap(), None);
    assert_eq!(parse(["--version".into()]).unwrap(), None);
}
