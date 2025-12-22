#[allow(deprecated)]
use tempfile::TempDir;

use std::path::{Path, PathBuf};

pub fn create_main_file(day_root: &Path, name: &str, from: &str) {
    let template_code = "./tests/files";
    let template_code = Path::new(template_code).join(from);

    let day = day_root.join(name);
    std::fs::File::create(&day).unwrap();
    std::fs::copy(template_code, day).unwrap();
}

pub fn create_input_file(day_root: &Path) {
    let template_input = "./tests/files/dummy_input";
    let input = day_root.join("input");
    std::fs::File::create(&input).unwrap();
    std::fs::copy(template_input, input).unwrap();
}

pub fn create_custom_day(day: &str, path: &Path, main: &str, from: &str) {
    let mut path = path.to_path_buf();
    path.push(day);
    _ = std::fs::create_dir(&path);
    create_main_file(&path, main, from);
    create_input_file(&path);
}

pub fn create_folder_custom(
    year: &str,
    days: &[&str],
    mains: &[(&str, &str)],
) -> (TempDir, PathBuf) {
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
