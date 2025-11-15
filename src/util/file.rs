use std::{
    env::home_dir,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use chrono::Datelike;
use clap::ArgMatches;
use reqwest::StatusCode;

use super::request::AocRequest;
use crate::{error::AocError, task_config::Config};

static PARSE_FILE: &str = ".parse.toml";
use crate::language::{Common, RunningArgs};

pub fn get_day_from_path() -> Result<Option<u32>, AocError> {
    let get_day = |s: &str| -> Option<u32> {
        let mut num = "".to_string();

        for ch in s.chars() {
            if ch.is_ascii_digit() {
                if ch == '0' && num.is_empty() {
                    continue;
                }
                num.push(ch);
            }
        }
        let num = num.parse::<u32>().ok()?;

        (1..=25).contains(&num).then_some(num)
    };

    let mut cwd = std::env::current_dir()?;

    loop {
        let name = cwd.file_name();
        let name = name
            .ok_or(AocError::InvalidRunDay)?
            .to_str()
            .ok_or(AocError::InvalidRunDay)?;

        if let Some(day) = get_day(name) {
            return Ok(Some(day));
        }
        if !cwd.pop() {
            return Ok(None);
        }
    }
}

pub fn get_root_path() -> Result<std::path::PathBuf, AocError> {
    let mut cwd = std::env::current_dir()?;

    loop {
        let name = cwd.file_name().ok_or_else(std::io::Error::last_os_error)?;

        let Ok(year): Result<i32, _> = name.to_str().unwrap().parse() else {
            if !cwd.pop() {
                return Err(AocError::InvalidYear);
            }
            continue;
        };

        let current_year = chrono::Utc::now().year();

        if (2015..=current_year).contains(&year) {
            return Ok(cwd);
        }
        if !cwd.pop() {
            return Err(AocError::InvalidYear);
        }
    }
}

pub fn get_folder_year() -> Result<i32, AocError> {
    let current_year = chrono::Utc::now().year();
    let valid = 2015..=current_year;

    std::env::current_dir()?
        .ancestors()
        .find_map(|p| {
            p.file_name()
                .and_then(|s| s.to_str())
                .and_then(|s| s.parse::<i32>().ok())
                .and_then(|y| valid.contains(&y).then_some(y))
        })
        .ok_or(AocError::InvalidYear)
}

pub async fn day_path<P: AsRef<Path>>(root: P, day: u32) -> Result<std::path::PathBuf, AocError> {
    use std::{collections::VecDeque, io::*};
    let dir_name = format!("day_{:02}", day);
    let dir_name = OsStr::new(&dir_name);
    let ignore = [OsStr::new("target"), OsStr::new(".git")];

    let mut vec = VecDeque::new();
    vec.push_back(root.as_ref().as_os_str().to_os_string());

    while let Some(path) = vec.pop_front() {
        let mut stream = tokio::fs::read_dir(&path).await?;
        while let Ok(Some(entry)) = stream.next_entry().await {
            let file_name = entry.file_name();
            if ignore.contains(&file_name.as_os_str()) {
                continue;
            }

            if file_name == dir_name {
                let mut buff: PathBuf = path.into();
                buff.push(dir_name);
                return Ok(buff);
            }

            let file_type = entry.file_type().await?;
            if file_type.is_dir() {
                let mut path = Path::new(&path).to_path_buf();
                path.push(entry.file_name());
                let name = path.as_os_str().to_os_string();

                vec.push_back(name);
            }
        }
    }
    let err_text = format!("could not find folder for day_{}", day);
    Err(Error::new(ErrorKind::NotFound, err_text).into())
}

pub async fn download_input_file(day: u32, year: i32, dir: &Path) -> Result<(), AocError> {
    let url = format!("https://adventofcode.com/{}/day/{}/input", year, day);
    let res = AocRequest::new().get(url).await?;

    if res.status() != StatusCode::OK {
        return Err(AocError::DownloadError(format!(
            "Couldn't download input for year: {} and day: {}",
            year, day
        )));
    }

    let bytes = res.bytes().await?;
    tokio::fs::write(dir.join("input"), bytes).await?;
    Ok(())
}

pub fn get_parse_config(root: &Path, day: &Path) -> Config {
    let f = || {
        if let Some(file) = find_file(day, PARSE_FILE) {
            return Some(file);
        }

        let root = root.join(PARSE_FILE);
        if root.exists() {
            return Some(root);
        }

        let config = home_dir()?
            .join(".config")
            .join("cargo-aoc")
            .join(PARSE_FILE);
        if config.exists() {
            return Some(config);
        }

        None
    };

    f().and_then(|path| Config::new(&path).ok())
        .unwrap_or_default()
}

pub fn get_input_file(matches: &ArgMatches) -> &str {
    if matches.get_flag("test") {
        "test"
    } else {
        "input"
    }
}

pub async fn get_running_args(matches: &ArgMatches) -> Result<RunningArgs, AocError> {
    let day = super::get_day(matches)?;
    let root = get_root_path()?;
    let day_path = day_path(&root, day).await?;

    let main = find_file(&day_path, "main").unwrap();

    let input_file = get_input_file(matches);

    let mut input = day_path.clone();
    input.push(input_file);

    let trailing_args = matches
        .get_many::<String>("args")
        .unwrap_or_default()
        .cloned()
        .collect::<Vec<_>>();

    Ok(RunningArgs {
        arguments: trailing_args,
        release: false,
        common: Common {
            file: main,
            day: day as i32,
            day_folder: day_path,
            root_folder: root,
            input_file: input,
        },
    })
}

pub fn find_file(start_dir: &Path, filename: &str) -> Option<PathBuf> {
    let entries = match std::fs::read_dir(start_dir) {
        Ok(entries) => entries,
        Err(_) => return None,
    };

    let mut subdirs = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && path
                .file_name()
                .is_some_and(|f| f.to_str().unwrap().starts_with(filename))
        {
            return Some(path);
        } else if path.is_dir() {
            subdirs.push(path);
        }
    }

    subdirs.sort_by(|a, b| {
        let count_a = std::fs::read_dir(a)
            .map(|r| r.count())
            .unwrap_or(usize::MAX);
        let count_b = std::fs::read_dir(b)
            .map(|r| r.count())
            .unwrap_or(usize::MAX);
        count_a.cmp(&count_b)
    });

    for subdir in subdirs {
        if let Some(found) = find_file(&subdir, filename) {
            return Some(found);
        }
    }

    None
}
