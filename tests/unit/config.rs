use super::*;

fn database_path(path: &Path, overridden: bool) -> Result<PathBuf> {
    Ok(Config::load(path, overridden)?.active_database)
}

#[test]
fn accepts_toml_and_rejects_invalid_settings() {
    assert_eq!(parse("").unwrap().database_path, PathBuf::from("db.sql"));
    assert_eq!(parse("").unwrap().task_view, TaskView::Normal);
    let defaults = parse("").unwrap();
    assert_eq!(defaults.default_priority, Priority::Mid);
    assert!(defaults.show_completed && defaults.show_hints);
    assert_eq!(defaults.sort_order, SortOrder::Priority);
    assert_eq!(
        parse("task_view = 'nested'").unwrap().task_view,
        TaskView::Normal
    );
    assert_eq!(
        parse("# local storage\ndatabase_path = 'tasks.sql'")
            .unwrap()
            .database_path,
        PathBuf::from("tasks.sql")
    );
    for content in [
        "database_path = [",
        "database_path = 3",
        "database_path = ''",
        "database_path = '   '",
        "database_path = 'config.toml'",
        "database_path = './config.toml'",
        "databse_path = 'tasks.sql'",
        "task_view = 3",
        "task_view = 'other'",
        "task_view = ''",
        "default_priority = 1",
        "default_priority = 'urgent'",
        "show_completed = 'false'",
        "show_completed = 0",
        "show_hints = 'true'",
        "show_hints = []",
        "sort_order = 'title'",
        "sort_order = false",
    ] {
        assert!(parse(content).is_err(), "Accepted {content}");
    }
    for view in [TaskView::Normal, TaskView::Split] {
        assert_eq!(
            parse(&format!("task_view = '{}'", view.label()))
                .unwrap()
                .task_view,
            view
        );
    }
}

#[test]
fn preferences_save_typed_values_preserve_comments_and_reload_atomically() {
    let folder = std::env::temp_dir().join(format!(
        "argv-todo-preferences-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let mut config = Config::load(&folder.join("db.sql"), false).unwrap();
    fs::write(&config.path, "# header\ndefault_priority = 'mid' # priority\nshow_completed = true # visibility\nshow_hints = true # hints\nsort_order = 'priority' # order\n# footer\n").unwrap();
    config.save_default_priority(Priority::Low).unwrap();
    config.save_show_completed(false).unwrap();
    config.save_show_hints(false).unwrap();
    config.save_sort_order(SortOrder::Manual).unwrap();
    let saved = fs::read_to_string(&config.path).unwrap();
    assert_eq!(
        saved,
        "# header\ndefault_priority = \"low\" # priority\nshow_completed = false # visibility\nshow_hints = false # hints\nsort_order = \"manual\" # order\n# footer\n"
    );
    config.reload().unwrap();
    assert_eq!(config.default_priority, Priority::Low);
    assert!(!config.show_completed && !config.show_hints);
    assert_eq!(config.sort_order, SortOrder::Manual);
    let blocked = config
        .path
        .with_file_name(format!(".config.toml.{}.tmp", std::process::id()));
    fs::write(&blocked, "blocked").unwrap();
    assert!(config.save_default_priority(Priority::High).is_err());
    assert!(config.save_show_completed(true).is_err());
    assert!(config.save_show_hints(true).is_err());
    assert!(config.save_sort_order(SortOrder::Priority).is_err());
    assert_eq!(config.default_priority, Priority::Low);
    assert!(!config.show_completed && !config.show_hints);
    assert_eq!(config.sort_order, SortOrder::Manual);
    assert_eq!(fs::read_to_string(&config.path).unwrap(), saved);
    fs::remove_file(blocked).unwrap();
    fs::write(
        &config.path,
        "default_priority = 'high'\nshow_hints = 'bad'",
    )
    .unwrap();
    assert!(config.reload().is_err());
    assert_eq!(config.default_priority, Priority::Low);
    fs::remove_dir_all(folder).unwrap();
}

#[test]
fn creates_config_preserves_edits_and_resolves_database_paths() {
    let folder = std::env::temp_dir().join(format!(
        "argv-todo config test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let nested = folder.join("nested");
    let default_database = nested.join("db.sql");
    let config = nested.join("config.toml");
    assert_eq!(
        database_path(&default_database, false).unwrap(),
        default_database
    );
    assert_eq!(fs::read_to_string(&config).unwrap(), DEFAULT_CONFIG);
    assert!(!default_database.exists());

    let content = "# keep my comment\r\ndatabase_path = 'task storage/tasks.sql'\r\n";
    fs::write(&config, content).unwrap();
    assert_eq!(
        database_path(&default_database, false).unwrap(),
        nested.join("task storage/tasks.sql")
    );
    let override_database = nested.join("custom.sql");
    assert_eq!(
        database_path(&override_database, true).unwrap(),
        override_database
    );
    assert_eq!(fs::read_to_string(&config).unwrap(), content);

    let absolute = folder.join("absolute.sql");
    // Use literal strings so Windows path separators remain unchanged.
    fs::write(&config, format!("database_path = '{}'", absolute.display())).unwrap();
    assert_eq!(database_path(&default_database, false).unwrap(), absolute);

    fs::write(&config, "database_path = [").unwrap();
    let error = database_path(&default_database, false).unwrap_err();
    assert!(error.to_string().contains(&config.display().to_string()));
    assert!(database_path(&override_database, true).is_err());
    assert!(database_path(&config, true).is_err());
    assert_eq!(fs::read_to_string(&config).unwrap(), "database_path = [");
    fs::remove_dir_all(folder).unwrap();
}

#[test]
fn saves_paths_preserving_comments_and_keeps_current_database_on_failure() {
    let folder = std::env::temp_dir().join(format!(
        "argv-todo-config-save-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let database = folder.join("db.sql");
    let mut config = Config::load(&database, false).unwrap();
    for value in [
        "'db.sql'",
        "\"db.sql\"",
        "'''\ndb.sql'''",
        "\"\"\"\ndb.sql\"\"\"",
    ] {
        fs::write(
            &config.path,
            format!("# header\n\"database_path\" = {value} # inline\n# footer\n"),
        )
        .unwrap();
        let new_path = "storage/界 'quoted' \\tasks.sql";
        config.save_database_path(new_path).unwrap();
        let saved = fs::read_to_string(&config.path).unwrap();
        assert_eq!(
            parse(&saved).unwrap().database_path,
            PathBuf::from(new_path)
        );
        assert!(saved.starts_with("# header\n\"database_path\" = "));
        assert!(saved.ends_with(" # inline\n# footer\n"));
        assert_eq!(config.database_path, new_path);
        assert_eq!(config.active_database, database);
    }
    fs::write(&config.path, "# no settings yet").unwrap();
    config.save_database_path("tasks.sql").unwrap();
    let saved = fs::read_to_string(&config.path).unwrap();
    assert!(saved.starts_with("# no settings yet\n"));
    assert_eq!(
        parse(&saved).unwrap().database_path,
        PathBuf::from("tasks.sql")
    );
    for invalid in ["", " ", "config.toml"] {
        assert!(config.save_database_path(invalid).is_err());
        assert_eq!(fs::read_to_string(&config.path).unwrap(), saved);
        assert_eq!(config.database_path, "tasks.sql");
    }
    fs::write(&config.path, "database_path = [").unwrap();
    assert!(config.save_database_path("new.sql").is_err());
    assert_eq!(
        fs::read_to_string(&config.path).unwrap(),
        "database_path = ["
    );
    assert!(config.reload().is_err());
    fs::remove_file(&config.path).unwrap();
    fs::create_dir(&config.path).unwrap();
    assert!(config.save_database_path("new.sql").is_err());
    assert_eq!(config.database_path, "tasks.sql");
    assert_eq!(config.active_database, database);
    assert_eq!(fs::read_dir(&folder).unwrap().count(), 1);
    fs::remove_dir_all(folder).unwrap();
}

#[test]
fn layout_saves_preserve_comments_database_settings_and_legacy_configs() {
    let folder = std::env::temp_dir().join(format!(
        "argv-todo-config-layout-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let database = folder.join("custom.sql");
    let mut config = Config::load(&database, true).unwrap();
    let legacy = "# my storage\ndatabase_path = 'storage.sql' # keep this\n";
    fs::write(&config.path, legacy).unwrap();
    config.reload().unwrap();
    assert_eq!(config.task_view, TaskView::Normal);
    config.save_task_view(TaskView::Split).unwrap();
    assert_eq!(
        fs::read_to_string(&config.path).unwrap(),
        format!("{legacy}task_view = \"split\"\n")
    );
    fs::write(
        &config.path,
        format!("{legacy}\"task_view\" = 'split' # layout comment\n# footer\n"),
    )
    .unwrap();
    config.save_task_view(TaskView::Normal).unwrap();
    let content = fs::read_to_string(&config.path).unwrap();
    assert!(content.starts_with(legacy));
    assert!(content.ends_with("\"task_view\" = \"normal\" # layout comment\n# footer\n"));
    assert_eq!(config.task_view, TaskView::Normal);
    assert_eq!(config.active_database, database);
    config.save_database_path("updated.sql").unwrap();
    assert_eq!(
        parse(&fs::read_to_string(&config.path).unwrap())
            .unwrap()
            .task_view,
        TaskView::Normal
    );
    assert_eq!(config.task_view, TaskView::Normal);
    fs::write(
        &config.path,
        "database_path = 'bad.sql'\ntask_view = 'invalid'",
    )
    .unwrap();
    assert!(config.reload().is_err());
    assert!(config.save_task_view(TaskView::Split).is_err());
    assert_eq!(config.database_path, "updated.sql");
    assert_eq!(config.task_view, TaskView::Normal);
    fs::remove_dir_all(folder).unwrap();
}
