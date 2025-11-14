use clap::Arg;
use regex::Regex;
use std::path::PathBuf;

use chrono::Datelike;
use clap::ArgMatches;
use file::get_root_path;

use self::{
    file::{day_path, get_day_from_path},
    request::AocRequest,
};
use crate::{error::AocError, util::file::ParseFile};

pub mod file;
pub mod request;
#[cfg(feature = "submit")]
pub mod submit;
#[cfg(feature = "tally")]
pub mod tally_util;

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum Task {
    One,
    Two,
}

impl std::fmt::Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Task::One => write!(f, "one"),
            Task::Two => write!(f, "two"),
        }
    }
}

pub fn get_day(matches: &ArgMatches) -> Result<u32, AocError> {
    let day = matches
        .get_one::<String>("day")
        .ok_or(AocError::ArgMatches)?
        .parse::<u32>()?;
    if !(1..=25).contains(&day) {
        Err(AocError::InvalidRunDay)
    } else {
        let source = matches.value_source("day").unwrap();
        if source == clap::parser::ValueSource::DefaultValue
            && let Ok(Some(day)) = get_day_from_path()
        {
            return Ok(day);
        }

        Ok(day)
    }
}

pub fn get_time_symbol() -> String {
    let sym = std::env::var("TASKUNIT").unwrap_or("ms".to_owned());
    if sym == "us" { "μs".to_owned() } else { sym }
}

#[derive(Debug)]
pub struct AocInfo {
    pub title: String,
    pub part1_answer: Option<String>,
    pub part2_answer: Option<String>,
}

pub async fn get_day_title_and_answers(day: u32, year: u32) -> Result<AocInfo, AocError> {
    if let Ok(cache) = read_cache_answers(day).await {
        return Ok(cache);
    }

    let url = format!("https://adventofcode.com/{}/day/{}", year, day);

    let res = AocRequest::new().get(&url).await?;

    let text = res.text().await?;

    let h2 = "<h2>--- ";
    let idx1 = text.find(h2).unwrap() + h2.len();
    let idx2 = text[idx1..].find(" ---</h2>").unwrap();
    let (_, title) = text[idx1..idx1 + idx2].split_once(": ").unwrap();

    let search = "Your puzzle answer was <code>";
    let mut iter = text
        .lines()
        .filter(|&line| line.contains(search))
        .map(|line| {
            let code_end = "</code>";
            let idx = line.find(search).unwrap() + search.len();
            let end = line[idx..].find(code_end).unwrap();

            line[idx..idx + end].to_owned()
        });
    let a1 = iter.next();
    let a2 = iter.next();

    let info = AocInfo {
        title: title.to_owned(),
        part1_answer: a1,
        part2_answer: a2,
    };

    // Ignore possible errors during cache write
    let _ = write_cache_answers(day, &info).await;

    Ok(info)
}

pub fn parse_get_answers(output: &str, parse_file: ParseFile) -> (Option<String>, Option<String>) {
    let strip = strip_ansi_escapes::strip(output);
    let text = std::str::from_utf8(&strip).unwrap();

    let parse = |line: &str, regex: &Regex| {
        regex
            .captures(line)
            .iter()
            .next()
            .map(|cap| cap[1].to_string())
    };

    let mut iter = text.split('\n').filter(|line| !line.is_empty());
    (
        iter.next().and_then(|it| parse(it, &parse_file.task_one)),
        iter.next().and_then(|it| parse(it, &parse_file.task_two)),
    )
}

async fn get_cache_path(day: u32) -> Result<PathBuf, AocError> {
    // Tries to read it from the cache before making a request
    let path = get_root_path()?;
    Ok(day_path(path, day).await?.join(".answers"))
}

pub async fn write_cache_answers(day: u32, info: &AocInfo) -> Result<(), AocError> {
    let path = get_cache_path(day).await?;
    if let (Some(a1), Some(a2)) = (&info.part1_answer, &info.part2_answer) {
        tokio::fs::write(path, format!("{}\n{}\n{}", info.title, a1, a2)).await?;
    }

    Ok(())
}

pub async fn read_cache_answers(day: u32) -> Result<AocInfo, AocError> {
    let path = get_cache_path(day).await?;
    let res = tokio::fs::read_to_string(path).await?;
    let lines = res.lines().collect::<Vec<_>>();
    Ok(AocInfo {
        title: lines[0].to_owned(),
        part1_answer: Some(lines[1].to_owned()),
        part2_answer: Some(lines[2].to_owned()),
    })
}

pub fn get_day_argument() -> Arg {
    let now = chrono::Utc::now();
    let current_year = now.year();
    let current_month = now.month();
    let current_day = now.day();

    if let Ok(Some(day)) = get_day_from_path() {
        return Arg::new("day").short('d').default_value(day.to_string());
    }

    if let Ok(year) = file::get_folder_year()
        && year == current_year
        && current_month == 12
        && current_day <= 13
    {
        // Not sure what the last day will be moving forward? Maybe 13?
        return Arg::new("day")
            .short('d')
            .default_value(current_day.to_string());
    }

    Arg::new("day").short('d').required(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_output() {
        let output = "12345\n67890";

        let (a1, a2) = parse_get_answers(output, ParseFile::default());
        assert_eq!(a1, Some("12345".to_owned()));
        assert_eq!(a2, Some("67890".to_owned()));
    }

    #[test]
    fn test_parse_output_trims_newlines() {
        let output = r#"

12345

67890

"#;

        let (a1, a2) = parse_get_answers(output, ParseFile::default());
        assert_eq!(a1, Some("12345".to_owned()));
        assert_eq!(a2, Some("67890".to_owned()));
    }

    #[test]
    fn test_parse_output_custom_regex() {
        let output = r#"
(3ms)   Task one: 12345
(3ms)   Task two: 67890
"#;

        let parse_file = ParseFile {
            task_one: Regex::new(r"Task one:\s*(\S+)").unwrap(),
            task_two: Regex::new(r"Task two:\s*(\S+)").unwrap(),
        };

        let (a1, a2) = parse_get_answers(output, parse_file);
        assert_eq!(a1, Some("12345".to_owned()));
        assert_eq!(a2, Some("67890".to_owned()));
    }

    #[test]
    fn test_parse_output_with_rust_stderr_output() {
        let output = r#"
 Compiling day_01 v0.1.0 (C:\Source\Advent-of-Code\2024\day_01)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.42s
     Running `target\debug\day_01.exe C:\Source\Advent-of-Code\2024\day_01\input`
(0ms)   Task one: 11
(1ms)   Task two: 22
"#;
        let parse_file = ParseFile {
            task_one: Regex::new(r"Task one:\s*(\S+)").unwrap(),
            task_two: Regex::new(r"Task two:\s*(\S+)").unwrap(),
        };

        let (a1, a2) = parse_get_answers(output, parse_file);
        assert_eq!(a1, Some("11".to_owned()));
        assert_eq!(a2, Some("22".to_owned()));
    }
    #[test]
    fn test_parse_output_wrong_output_but_as_expected() {
        let output = r#"
foo
bar
(0ms)   Task one: 11
(1ms)   Task two: 22
"#;
        let parse_file = ParseFile::default();
        let (a1, a2) = parse_get_answers(output, parse_file);
        assert_eq!(a1, Some("foo".into()));
        assert_eq!(a2, Some("bar".into()));
    }
}
