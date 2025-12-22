#[allow(deprecated)]
use assert_cmd::cargo::cargo_bin;
use assert_cmd::prelude::*;
use predicates::str::contains;
use tempfile::TempDir;

use std::{io::Write, path::PathBuf, process::Command};

fn create_empty_root_dir() -> (TempDir, PathBuf) {
    let temp = tempfile::Builder::new().tempdir().unwrap();
    let mut path = temp.path().to_path_buf();
    path.push("2025");

    std::fs::create_dir(&path).unwrap();

    (temp, path)
}

#[test]
fn can_set_token() {
    let (_dir, root) = create_empty_root_dir();

    Command::new(cargo_bin!())
        .current_dir(&root)
        .arg("token")
        .arg("-s")
        .arg("foobar")
        .assert();

    let mut file = root.clone();
    file.push(".env");

    let s = std::fs::read_to_string(file).unwrap();

    assert_eq!(s, "AOC_TOKEN=foobar");
}

#[test]
fn can_get_token() {
    let (_dir, root) = create_empty_root_dir();

    let mut file = root.clone();
    file.push(".env");

    let mut f = std::fs::File::create(&file).unwrap();
    f.write_fmt(format_args!("AOC_TOKEN=foobaz")).unwrap();

    Command::new(cargo_bin!())
        .current_dir(&root)
        .arg("token")
        .arg("-g")
        .assert()
        .stdout(contains("foobaz"));
}
