#[allow(deprecated)]
use assert_cmd::cargo::cargo_bin;

use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

fn assert_structure<const NUM_DAYS: usize>(path: &Path, year: usize, token: &str) {
    let year = path.join(year.to_string());
    let content = std::fs::read_dir(&year).unwrap();

    let folders = content
        .flatten()
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .collect::<Vec<_>>();

    let content = std::fs::read_dir(&year).unwrap();
    let files = content
        .flatten()
        .filter(|entry| entry.file_type().unwrap().is_file())
        .collect::<Vec<_>>();

    assert_eq!(folders.len(), NUM_DAYS);
    assert!(
        folders
            .into_iter()
            .all(|entry| entry.file_name().to_str().unwrap().starts_with("day_"))
    );
    assert_eq!(files.len(), 1);
    assert!(files[0].file_name().to_str().unwrap() == ".env");

    let file_path = files[0].path();
    let content = std::fs::read_to_string(file_path).unwrap();
    assert_eq!(content, format!("AOC_TOKEN={token}"));
}

#[test]
fn test_setup_creates_folders_with_tokens() {
    let temp = tempfile::Builder::new().tempdir().unwrap();
    let path = temp.path().to_path_buf();

    let year = "2024";

    let mut cmd = Command::new(cargo_bin!())
        .current_dir(&path)
        .arg("setup")
        .arg("-y")
        .arg(year)
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = cmd.stdin.take().unwrap();

    let token = "foobar";
    stdin.write_all(format!("{token}\n").as_bytes()).unwrap();

    let out = cmd.wait_with_output().unwrap();

    assert!(out.status.success());
    assert!(std::fs::exists(path.join(year)).unwrap());

    assert_structure::<25>(&path, year.parse().unwrap(), token);
}

#[test]
fn test_setup_year_2025() {
    let temp = tempfile::Builder::new().tempdir().unwrap();
    let path = temp.path().to_path_buf();

    let year = "2025";

    let mut cmd = Command::new(cargo_bin!())
        .current_dir(&path)
        .arg("setup")
        .arg("-y")
        .arg(year)
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = cmd.stdin.take().unwrap();

    let token = "foobar";
    stdin.write_all(format!("{token}\n").as_bytes()).unwrap();

    let out = cmd.wait_with_output().unwrap();

    assert!(out.status.success());
    assert!(std::fs::exists(path.join(year)).unwrap());

    assert_structure::<12>(&path, year.parse().unwrap(), token);
}
