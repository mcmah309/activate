use std::{collections::HashMap, env, fs, path::Path, sync::Once};

use assert_cmd::cargo::CargoError;
use predicates::prelude::predicate;

static INIT: Once = Once::new();

pub fn initialize() {
    INIT.call_once(|| {
        let test_dir = Path::new("tests");
        assert!(
            env::set_current_dir(&test_dir).is_ok(),
            "Failed to change directory"
        );
    });
    // deactivate
    let assert = assert_cmd::Command::cargo_bin("activate")
        .unwrap()
        .arg("-r")
        .assert();
    assert.success().stdout(predicate::str::contains(""));
}

#[test]
fn links_switching_between() -> Result<(), CargoError> {
    initialize();

    let assert = assert_cmd::Command::cargo_bin("activate")?
        .arg("test")
        .assert();
    assert.success().stdout(predicate::str::contains(""));

    assert_eq!(
        fs::read_to_string(Path::new("another/test_file1")).unwrap(),
        "test_file1"
    );
    assert_eq!(
        fs::read_to_string(Path::new("test_file2_new_name")).unwrap(),
        "test_file2"
    );
    assert_eq!(
        fs::read_to_string(Path::new("test_file3")).unwrap(),
        "test_file3"
    );
    assert!(!Path::new("dev_file1").exists());
    assert!(!Path::new("prod_dir").exists());

    let assert = assert_cmd::Command::cargo_bin("activate")?
        .arg("dev")
        .assert();
    assert.success().stdout(predicate::str::contains(""));

    assert_eq!(
        fs::read_to_string(Path::new("dev_file1")).unwrap(),
        "dev_file1"
    );
    assert!(!Path::new("another/test_file1").exists());
    assert!(!Path::new("test_file2_new_name").exists());
    assert!(!Path::new("test_file3").exists());
    assert!(!Path::new("prod_dir").exists());

    let assert = assert_cmd::Command::cargo_bin("activate")?
        .arg("prod")
        .assert();
    assert.success().stdout(predicate::str::contains(""));

    assert!(!Path::new("dev_file1").exists());
    assert!(!Path::new("another/test_file1").exists());
    assert!(!Path::new("test_file2_new_name").exists());
    assert!(!Path::new("test_file3").exists());
    assert!(Path::new("prod_dir").exists());
    assert_eq!(
        fs::read_to_string(Path::new("prod_dir/prod_file1")).unwrap(),
        "prod_file1"
    );

    // deactivate
    let assert = assert_cmd::Command::cargo_bin("activate")?.assert();
    assert.success().stdout(predicate::str::contains(""));

    assert!(!Path::new("dev_file1").exists());
    assert!(!Path::new("another/test_file1").exists());
    assert!(!Path::new("test_file2_new_name").exists());
    assert!(!Path::new("test_file3").exists());
    assert!(!Path::new("prod_dir").exists());

    Ok(())
}

#[test]
fn env_switching_between() -> Result<(), CargoError> {
    initialize();

    let assert = assert_cmd::Command::cargo_bin("activate")?
        .arg("test")
        .assert();
    assert.success().stdout(predicate::str::contains(""));
    let env_file = Path::new(".activate/state/env.json");
    assert!(env_file.exists());
    let env: HashMap<String, String> =
        serde_json::from_str(&fs::read_to_string(env_file).unwrap()).unwrap();
    assert_eq!(env.get("PYTHONPATH").unwrap(), "src");
    assert_eq!(env.get("DJANGO_SETTINGS_MODULE").unwrap(), "settings");
    assert!(env.get("XDG_CONFIG_HOME").is_none());
    assert!(env.get("XDG_DATA_HOME").is_none());
    assert!(env.get("XDG_CACHE_HOME").is_none());

    let assert = assert_cmd::Command::cargo_bin("activate")?
        .arg("dev")
        .assert();
    assert.success().stdout(predicate::str::contains(""));
    assert!(env_file.exists());
    let env: HashMap<String, String> =
        serde_json::from_str(&fs::read_to_string(env_file).unwrap()).unwrap();
    assert!(env.get("PYTHONPATH").is_none());
    assert!(env.get("DJANGO_SETTINGS_MODULE").is_none());
    assert_eq!(env.get("XDG_CONFIG_HOME").unwrap(), "config");
    assert_eq!(env.get("XDG_DATA_HOME").unwrap(), "data");
    assert_eq!(env.get("XDG_CACHE_HOME").unwrap(), "cache");

    let assert = assert_cmd::Command::cargo_bin("activate")?
        .arg("prod")
        .assert();
    assert.success().stdout(predicate::str::contains(""));
    assert!(!env_file.exists());

    Ok(())
}

#[test]
fn env_eval() -> Result<(), CargoError> {
    initialize();

    let assert = assert_cmd::Command::cargo_bin("activate")?
        .arg("test")
        .arg("-e")
        .assert();
    assert.success().stdout(predicate::eq(
        r#"export DJANGO_SETTINGS_MODULE=settings
export PYTHONPATH=src
"#,
    ));

    let assert = assert_cmd::Command::cargo_bin("activate")?
        .arg("dev")
        .arg("-e")
        .assert();
    assert.success().stdout(predicate::eq(
        r#"unset DJANGO_SETTINGS_MODULE
unset PYTHONPATH
export XDG_CACHE_HOME=cache
export XDG_CONFIG_HOME=config
export XDG_DATA_HOME=data
"#,
    ));

    let assert = assert_cmd::Command::cargo_bin("activate")?
        .arg("prod")
        .arg("-e")
        .assert();
    assert.success().stdout(predicate::eq(
        r#"unset XDG_CACHE_HOME
unset XDG_CONFIG_HOME
unset XDG_DATA_HOME
"#,
    ));

    let assert = assert_cmd::Command::cargo_bin("activate")?
        .arg("dev")
        .arg("-e")
        .assert();
    assert.success().stdout(predicate::eq(
        r#"export XDG_CACHE_HOME=cache
export XDG_CONFIG_HOME=config
export XDG_DATA_HOME=data
"#,
    ));

    // deactivate
    let assert = assert_cmd::Command::cargo_bin("activate")?
        .arg("-e")
        .assert();
    assert.success().stdout(predicate::eq(
        r#"unset XDG_CACHE_HOME
unset XDG_CONFIG_HOME
unset XDG_DATA_HOME
"#,
    ));

    Ok(())
}

#[test]
fn recursive() -> Result<(), CargoError> {
    initialize();

    let assert = assert_cmd::Command::cargo_bin("activate")?
        .arg("test")
        .arg("-e")
        .arg("-r")
        .assert();
    assert.success().stdout(predicate::eq(
        r#"export DJANGO_SETTINGS_MODULE=settings
export PYTHONPATH=src
export TEST_ENV=test
export TEST_ENV2=test2
"#,
    ));

    assert_eq!(
        fs::read_to_string(Path::new("another/test_file1")).unwrap(),
        "test_file1"
    );
    assert_eq!(
        fs::read_to_string(Path::new("test_file2_new_name")).unwrap(),
        "test_file2"
    );
    assert_eq!(
        fs::read_to_string(Path::new("test_file3")).unwrap(),
        "test_file3"
    );
    assert!(!Path::new("dev_file1").exists());
    assert!(!Path::new("prod_dir").exists());

    assert_eq!(
        fs::read_to_string(Path::new("another_active_dir/test_file4")).unwrap(),
        "test_file4"
    );

    Ok(())
}

#[test]
fn dot_env_file() -> Result<(), CargoError> {
    initialize();

    let assert = assert_cmd::Command::cargo_bin("activate")?
        .arg("test")
        .assert();
    assert.success().stdout(predicate::eq(""));

    dotenv::from_path(Path::new(".activate/.env")).unwrap();

    assert_eq!(env::var("DJANGO_SETTINGS_MODULE").unwrap(), "settings");
    assert_eq!(env::var("PYTHONPATH").unwrap(), "src");

    let assert = assert_cmd::Command::cargo_bin("activate")?
        .arg("test")
        .arg("-r")
        .assert();
    assert.success().stdout(predicate::eq(""));

    dotenv::from_path(Path::new(".activate/.env")).unwrap();

    assert_eq!(env::var("DJANGO_SETTINGS_MODULE").unwrap(), "settings");
    assert_eq!(env::var("PYTHONPATH").unwrap(), "src");
    assert_eq!(env::var("TEST_ENV").unwrap(), "test");
    assert_eq!(env::var("TEST_ENV2").unwrap(), "test2");

    Ok(())
}
