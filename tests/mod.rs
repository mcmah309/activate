use std::{env, error::Error, fs, path::Path};

use assert_cmd::{assert::OutputAssertExt, cargo::CargoError};
use predicates::prelude::predicate;





#[test]
fn switch_between_test() -> Result<(),CargoError> {
    let test_dir = Path::new("tests");
    assert!(env::set_current_dir(&test_dir).is_ok(), "Failed to change directory");

    let assert = assert_cmd::Command::cargo_bin("activate")?
    .arg("test")
    .assert();

    assert.success().stdout(predicate::str::contains(""));

    assert_eq!(fs::read_to_string(Path::new("another/test_file1")).expect("Could not read `test_file1` file."),"test_file1");
    assert_eq!(fs::read_to_string(Path::new("test_file2")).expect("Could not read `test_file2` file."),"test_file2");
    assert_eq!(fs::read_to_string(Path::new("test_file3")).expect("Could not read `test_file3` file."),"test_file3");
    assert!(!Path::new("dev_file1").exists());

    let assert = assert_cmd::Command::cargo_bin("activate")?
    .arg("dev")
    .assert();

    assert.success().stdout(predicate::str::contains(""));

    assert_eq!(fs::read_to_string(Path::new("dev_file1")).expect("Could not read `dev_file1` file."),"dev_file1");
    assert!(!Path::new("another/test_file1").exists());
    assert!(!Path::new("test_file2").exists());
    assert!(!Path::new("test_file3").exists());

    let assert = assert_cmd::Command::cargo_bin("activate")?
    .assert();

    assert.success().stdout(predicate::str::contains(""));

    assert!(!Path::new("dev_file1").exists());
    assert!(!Path::new("another/test_file1").exists());
    assert!(!Path::new("test_file2").exists());
    assert!(!Path::new("test_file3").exists());

    Ok(())
}