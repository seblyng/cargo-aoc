#[allow(deprecated)]
use assert_cmd::cargo::cargo_bin;
use assert_cmd::prelude::*;
use predicates::str::contains;
use tempfile::TempDir;

use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn create_folder(year: &str, day: &str) -> (TempDir, PathBuf) {
    common::create_folder_custom(year, &[day], &[("main.rs", "main.rs")])
}

mod common;

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
    let (_dir, root) = create_folder("2025", "day_01");
    check_ok(&root);
}

#[test]
fn year_with_space() {
    let (_dir, root) = create_folder("aoc 2025", "day_01");
    check_ok(&root);
}

#[test]
fn year_with_underscore_and_text_on_both_sides() {
    let (_dir, root) = create_folder("aoc_2025_test", "day_01");
    check_ok(&root);
}

#[test]
fn year_with_other_numbers() {
    let (_dir, root) = create_folder("aoc 3 test 2025 1 2", "day_01");
    check_ok(&root);
}

#[test]
fn day_with_space() {
    let (_dir, root) = create_folder("2025", "day 1");
    check_ok(&root);
}

#[test]
fn day_with_dash() {
    let (_dir, root) = create_folder("2025", "day-1");
    check_ok(&root);
}

#[test]
fn day_with_space_and_padding() {
    let (_dir, root) = create_folder("2025", "day 01");
    check_ok(&root);
}

#[test]
fn day_with_space_and_postfix() {
    let (_dir, root) = create_folder("2025", "day 1 solution");
    check_ok(&root);
}

#[test]
fn day_with_postfix() {
    let (_dir, root) = create_folder("2025", "day_01_sol");
    check_ok(&root);
}

#[test]
fn main_file_with_different_name() {
    let (_dir, root) =
        common::create_folder_custom("2025", &["day_01_sol"], &[("not_main.rs", "main.rs")]);
    check_ok(&root);
}
