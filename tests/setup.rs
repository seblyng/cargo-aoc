#[allow(deprecated)]
use assert_cmd::cargo::cargo_bin;

use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

fn assert_structure(path: &Path, year: usize) {
    let year_num = year;
    let year = path.join(year.to_string());
    let content = std::fs::read_dir(&year).unwrap();

    let len = if (2015..=2024).contains(&year_num) {
        25
    } else {
        12
    };

    let folders = content
        .flatten()
        .filter(|entry| entry.file_type().unwrap().is_dir())
        .collect::<Vec<_>>();

    let content = std::fs::read_dir(&year).unwrap();
    let files = content
        .flatten()
        .filter(|entry| entry.file_type().unwrap().is_file())
        .collect::<Vec<_>>();

    assert!(folders.len() == len);
    assert!(
        folders
            .into_iter()
            .all(|entry| entry.file_name().to_str().unwrap().starts_with("day_"))
    );
    assert_eq!(files.len(), 1);
    assert!(files[0].file_name().to_str().unwrap() == ".env");
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
    stdin.write_all("foobar\n".as_bytes()).unwrap();

    let out = cmd.wait_with_output().unwrap();

    assert!(out.status.success());
    assert!(std::fs::exists(path.join(year)).unwrap());

    assert_structure(&path, year.parse().unwrap());
}
