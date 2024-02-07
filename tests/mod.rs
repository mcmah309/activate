use std::{env, error::Error, fs, path::Path};

use assert_cmd::{assert::OutputAssertExt, cargo::CargoError};
use predicates::prelude::predicate;

#[test]
fn links_switching_between() -> Result<(), CargoError> {
    let test_dir = Path::new("tests");
    assert!(
        env::set_current_dir(&test_dir).is_ok(),
        "Failed to change directory"
    );

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
