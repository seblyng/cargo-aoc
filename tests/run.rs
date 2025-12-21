#[allow(deprecated)]
use assert_cmd::cargo::cargo_bin;
use assert_cmd::prelude::*;
use predicates::str::contains;
use tempfile::TempDir;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn create_main_file(day_root: &Path, name: &str, from: &str) {
    let template_code = "./tests/files";
    let template_code = Path::new(template_code).join(from);

    let day = day_root.join(name);
    std::fs::File::create(&day).unwrap();
    std::fs::copy(template_code, day).unwrap();
}

fn create_input_file(day_root: &Path) {
    let template_input = "./tests/files/dummy_input";
    let input = day_root.join("input");
    std::fs::File::create(&input).unwrap();
    std::fs::copy(template_input, input).unwrap();
}

fn create_custom_day(day: &str, path: &Path, main: &str, from: &str) {
    let mut path = path.to_path_buf();
    path.push(day);
    _ = std::fs::create_dir(&path);
    create_main_file(&path, main, from);
    create_input_file(&path);
}

fn create_folder_custom(year: &str, days: &[&str], mains: &[(&str, &str)]) -> (TempDir, PathBuf) {
    let temp = tempfile::Builder::new().tempdir().unwrap();
    let mut path = temp.path().to_path_buf();
    path.push(year);

    std::fs::create_dir(&path).unwrap();

    let lang = "./tests/files/.languages.toml";
    let lang_path = path.join(".languages.toml");

    std::fs::File::create(&lang_path).unwrap();
    std::fs::copy(lang, lang_path).unwrap();

    for day in days {
        for (main, from) in mains {
            create_custom_day(day, &path, main, from);
        }
    }

    (temp, path)
}

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
    let (_dir, root) = create_folder_custom("2025", &["day_01_sol"], &[("main.rs", "main.rs")]);
    check_ok(&root);
}

#[test]
fn supports_other_than_rust_with_language_file() {
    let (_dir, root) = create_folder_custom("2025", &["day_01_sol"], &[("main.py", "main.py")]);
    check_ok(&root);
}

#[test]
fn cant_run_unsupported_lang() {
    let (_dir, root) = create_folder_custom("2025", &["day_01_sol"], &[("main.lisp", "main.lisp")]);
    check_not_ok(&root, "Unsupported");
}

#[test]
fn test_multiple_files_without_flag() {
    let (_dir, root) = create_folder_custom(
        "2025",
        &["day_01_sol"],
        &[("main.rs", "main.rs"), ("main.py", "main.py")],
    );

    check_not_ok(&root, "Too many");
}

#[test]
fn test_ok_with_multiple_files_without_flag_but_only_one_supported_lang() {
    let (_dir, root) = create_folder_custom(
        "2025",
        &["day_01_sol"],
        &[("main.rs", "main.rs"), ("main.lisp", "main.lisp")],
    );

    check_ok(&root);
}

#[test]
fn test_ok_multiple_files_with_flag() {
    let (_dir, root) = create_folder_custom(
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
