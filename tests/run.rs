mod common;

#[allow(deprecated)]
use assert_cmd::cargo::cargo_bin;
use assert_cmd::prelude::*;
use predicates::str::contains;

use std::{path::Path, process::Command};

fn check_ok(root: &Path) {
    Command::new(cargo_bin!())
        .current_dir(&root)
        .arg("run")
        .arg("-d")
        .arg("1")
        .assert()
        .stdout(contains("1"))
        .stdout(contains("2"));
}

fn check_not_ok(root: &Path, err: &str) {
    Command::new(cargo_bin!())
        .current_dir(&root)
        .arg("run")
        .arg("-d")
        .arg("1")
        .assert()
        .stderr(contains(err));
}

#[test]
fn base_case_supports_one_main_file() {
    let (_dir, root) =
        common::create_folder_custom("2025", &["day_01_sol"], &[("main.rs", "main.rs")]);
    check_ok(&root);
}

#[test]
fn supports_other_than_rust_with_language_file() {
    let (_dir, root) =
        common::create_folder_custom("2025", &["day_01_sol"], &[("main.py", "main.py")]);
    check_ok(&root);
}

#[test]
fn cant_run_unsupported_lang() {
    let (_dir, root) =
        common::create_folder_custom("2025", &["day_01_sol"], &[("main.lisp", "main.lisp")]);
    check_not_ok(&root, "Unsupported");
}

#[test]
fn test_multiple_files_without_flag() {
    let (_dir, root) = common::create_folder_custom(
        "2025",
        &["day_01_sol"],
        &[("main.rs", "main.rs"), ("main.py", "main.py")],
    );

    check_not_ok(&root, "Too many");
}

#[test]
fn test_ok_with_multiple_files_without_flag_but_only_one_supported_lang() {
    let (_dir, root) = common::create_folder_custom(
        "2025",
        &["day_01_sol"],
        &[("main.rs", "main.rs"), ("main.lisp", "main.lisp")],
    );

    check_ok(&root);
}

#[test]
fn test_ok_multiple_files_with_flag() {
    let (_dir, root) = common::create_folder_custom(
        "2025",
        &["day_01_sol"],
        &[("main.rs", "main.rs"), ("main.py", "main.py")],
    );

    Command::new(cargo_bin!())
        .current_dir(&root)
        .arg("run")
        .arg("-d")
        .arg("1")
        .arg("--runner")
        .arg("py")
        .assert()
        .stdout(contains("1"))
        .stdout(contains("2"));
}
