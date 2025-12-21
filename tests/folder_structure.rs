#[allow(deprecated)]
use assert_cmd::cargo::cargo_bin;
use assert_cmd::prelude::*;
use predicates::str::contains;
use tempfile::TempDir;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn create_main_file(day_root: &Path, name: &str) {
    let template_code = "./tests/files/main.rs";
    let day = day_root.join(name);
    std::fs::File::create(&day).unwrap();
    std::fs::copy(template_code, day).unwrap();
}

fn create_input_file(day_root: &Path, name: &str) {
    let template_input = "./tests/files/dummy_input";
    let input = day_root.join(name);
    std::fs::File::create(&input).unwrap();
    std::fs::copy(template_input, input).unwrap();
}

fn create_custom_day(day: &str, path: &Path, main: &str, input: &str) {
    let mut path = path.to_path_buf();
    path.push(day);
    std::fs::create_dir(&path).unwrap();
    create_main_file(&path, main);
    create_input_file(&path, input);
}

fn create_folder_custom(year: &str, days: &[&str], main: &str, input: &str) -> (TempDir, PathBuf) {
    let temp = tempfile::Builder::new().tempdir().unwrap();
    let mut path = temp.path().to_path_buf();
    path.push(year);

    std::fs::create_dir(&path).unwrap();

    for day in days {
        create_custom_day(day, &path, main, input);
    }

    (temp, path)
}

fn create_folder(year: &str, days: &[&str]) -> (TempDir, PathBuf) {
    create_folder_custom(year, days, "main.rs", "input")
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

#[test]
fn default_folder_structure() {
    let (_dir, root) = create_folder("2025", &["day_01"]);
    check_ok(&root);
}

#[test]
fn year_with_space() {
    let (_dir, root) = create_folder("aoc 2025", &["day_01"]);
    check_ok(&root);
}

#[test]
fn year_with_underscore_and_text_on_both_sides() {
    let (_dir, root) = create_folder("aoc_2025_test", &["day_01"]);
    check_ok(&root);
}

#[test]
fn year_with_other_numbers() {
    let (_dir, root) = create_folder("aoc 3 test 2025 1 2", &["day_01"]);
    check_ok(&root);
}

#[test]
fn day_with_space() {
    let (_dir, root) = create_folder("2025", &["day 1"]);
    check_ok(&root);
}

#[test]
fn day_with_dash() {
    let (_dir, root) = create_folder("2025", &["day-1"]);
    check_ok(&root);
}

#[test]
fn day_with_space_and_padding() {
    let (_dir, root) = create_folder("2025", &["day 01"]);
    check_ok(&root);
}

#[test]
fn day_with_space_and_postfix() {
    let (_dir, root) = create_folder("2025", &["day 1 solution"]);
    check_ok(&root);
}

#[test]
fn day_with_postfix() {
    let (_dir, root) = create_folder("2025", &["day_01_sol"]);
    check_ok(&root);
}

#[test]
fn main_file_with_different_name() {
    let (_dir, root) = create_folder_custom("2025", &["day_01_sol"], "not_main.rs", "input");
    check_ok(&root);
}
